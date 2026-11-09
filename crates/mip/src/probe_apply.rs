//! Probing `4d3a1d`: the pass loop and the write-back after the engine (docs/m2_red_probe.md §2.2, §2.8):
//! aggregations (`564ce0`), bounds (rounded for integers, `3df916` when fixed), rows made `=`, coefficient lifting
//! with new nodes (`3bea24`), implications into the conflict graph and the two-pass decision.
use crate::driver::Timer;
use crate::probe::{Args, Ctx, Stop, TimerView};
use crate::probe_engine::EngineEnd;
use crate::state::{Node, State, NIL};
use crate::Params;

impl State {
    /// `3bea24(v, env, st, row, col, &node)`: a node from the free list (a chunk of max(3000, m) when empty), put at
    /// the front of its row and column lists; counts and nnz +1. No work, no marks.
    pub fn new_node(&mut self, v: f64, row: usize, col: usize) -> u32 {
        if self.free == NIL {
            let n = 3000.max(self.m);
            let at = self.nd.len();
            for k in 0..n {
                let nc = if k + 1 < n { (at + k + 1) as u32 } else { NIL };
                self.nd.push(Node { val: 0.0, row: 0, col: -1, nr: NIL, nc });
            }
            self.free = at as u32;
            self.nfree += n as i64; // +0x158 stays: the chunk goes on the +0x1b8 chain
        }
        let at = self.free;
        let q = &mut self.nd[at as usize];
        self.free = q.nc;
        self.nfree -= 1;
        *q = Node { val: v, row: row as i32, col: col as i32, nr: self.row_head[row], nc: self.col_head[col] };
        self.col_head[col] = at;
        self.col_cnt[col] += 1;
        self.nnz += 1;
        self.row_head[row] = at;
        self.row_cnt[row] += 1;
        at
    }

    /// `4d3a1d(model, 0, st, env, T, mode, a7, a8, a9, a10)` on one thread; `t` is the reduction timer before
    /// `timed` starts it. Returns a status (3 infeasible with `bad_col`).
    #[allow(clippy::too_many_arguments)]
    pub fn probe(&mut self, p: &Params, mode: i32, a7: i32, a8: i32, a9: i32, a10: i32, t: Timer) -> i32 {
        let mut t = t;
        t.start(self.work);
        let tv = TimerView { acc: t.acc(), running: t.running(), limit: t.limit };
        self.probe_run(p, &Args { mode, a7, a8, a9, a10 }, &tv)
    }

    /// `4d3a1d` with its (running) timer as `tv`: the passes. Returns a status.
    pub fn probe_run(&mut self, p: &Params, a: &Args, tv: &TimerView) -> i32 {
        let (mut second, mut no_more) = (false, false);
        loop {
            // pass start: `3ba5ac` (callback, no state change), `5b9aad(G, 0, NULL)`
            self.cg.merge(self.n, self.cols_removed, &self.col_cnt, &mut self.work, self.work_mult);
            match self.probe_pass(p, a, !second, tv, &mut no_more) {
                Ok(true) => second = true,
                Ok(false) => return 0,
                Err(rc) => return rc,
            }
        }
    }

    /// One pass; Ok(true) = run another (the second pass).
    fn probe_pass(&mut self, p: &Params, a: &Args, first: bool, tv: &TimerView, no_more: &mut bool) -> Result<bool, i32> {
        let (mut c, mut cand) = match self.probe_build(p, a, first, tv) {
            Ok(x) => x,
            Err(Stop::Done) => return Ok(false),
            Err(Stop::Status(rc, j)) => {
                if j >= 0 {
                    self.bad_col = j;
                }
                return Err(rc);
            }
        };
        let mut w = self.work;
        let end = self.probe_engine(p, &mut c, &mut cand, &mut w);
        self.work = w;
        if end == EngineEnd::Budget {
            // `3ba9de` inside the engine: PRELIMIT −1 past the budget, the engine returns 0 and 4d3a1d goes on
            self.budget_check();
        }
        if c.stop || tv.over(self.work) {
            *no_more = true;
        }
        let (n, m, mu) = (self.n, self.m, self.work_mult);
        // §2.8.1 aggregations (columns probing fixed are left out)
        if c.nagg != 0 {
            for j in 0..n {
                if c.ub[j] - c.lb[j] < 1e-10 {
                    c.aggto[j] = -1;
                }
            }
            self.work += (n as f64 + n as f64) * mu;
            let (rc, _) = self.substitute(&mut c.aggto, &mut c.aggmul, &mut c.aggoff, true);
            if rc != 0 {
                return Err(rc);
            }
        }
        // §2.8.2 bounds; integers rounded inward by IntFeasTol, then fixed columns removed (activities at the end)
        if c.nchg > 0 {
            for j in 0..n {
                if self.col_cnt[j] < 0 {
                    continue;
                }
                if c.lb[j] > self.lb[j] + 1e-06 {
                    self.lb[j] = if self.vtype[j] != b'C' { (c.lb[j] - p.intfeastol).ceil() } else { c.lb[j] };
                    self.tracker.mark_col(j as i32);
                    self.bounds_tightened += 1;
                }
                if c.ub[j] < self.ub[j] - 1e-06 {
                    self.ub[j] = if self.vtype[j] != b'C' { (c.ub[j] + p.intfeastol).floor() } else { c.ub[j] };
                    self.tracker.mark_col(j as i32);
                    self.bounds_tightened += 1;
                }
                if self.lb[j] > self.ub[j] + 1e-06 {
                    self.bad_col = j as i32;
                    return Err(3);
                }
                if self.lb[j] > self.ub[j] - 1e-10 {
                    let rc = self.fix_near(j, p.feastol);
                    if rc != 0 {
                        return Err(rc);
                    }
                }
            }
            self.work += n as f64 * 5.0 * mu;
            self.activity(None);
        }
        // §2.8.3 rows probing proved tight
        if c.neq > 0 {
            for i in 0..m {
                let k = c.rowmap[i];
                if self.sense[i] != b'=' && k >= 0 && c.sense[k as usize] == b'=' {
                    self.sense[i] = b'=';
                    self.ineq_to_eq += 1;
                    self.tracker.mark_row(i as i32);
                }
            }
        }
        // §2.8.4 coefficient lifting from the redundant-row records
        if !c.rlit.is_empty() {
            self.probe_lift(p, a, &c)?;
        }
        // §2.8.5 implications, final merge, §2.8.6 second pass
        let nadd = match self.probe_export(p, &c, &cand) {
            Some(x) => x,
            None => return Ok(false),
        };
        let tot = self.cg.merge(self.n, self.cols_removed, &self.col_cnt, &mut self.work, self.work_mult);
        Ok(!(tot - nadd <= cand.len() as i64 / 4 || *no_more || !first))
    }

    /// §2.8.4: per probe row (`<`, live), the literals whose fixing leaves the row slack v; score each (and the
    /// cliques holding them); add the best literal's or clique's slack to the row's coefficients (new nodes only
    /// while under the fill limit). Counts +0x3fc.
    /// shortcut: the score with an LP solution (st+0x3c8) and the SOS block of a MIP context are not ported; both are
    /// absent in presolve.
    fn probe_lift(&mut self, p: &Params, a: &Args, c: &Ctx) -> Result<(), i32> {
        let (n, mu) = (self.n, self.work_mult);
        let fill_max: i64 = if a.a9 == 0 {
            0
        } else {
            let t = 2_000_000_000i64 - c.nnz_p as i32 as i64;
            if a.mode < 3 { t.min(c.nnz_p) } else { t }
        };
        let mut lv = vec![0.0f64; 2 * n];
        let mut lits: Vec<usize> = Vec::new();
        let mut node_of = vec![NIL; n];
        let (mut nfill, mut nmod, mut ndel) = (0i64, 0i32, 0i32);
        let mut cl: Option<crate::clique::Cliques> = None;
        let mut clsc: Vec<f64> = Vec::new();
        let mut w0 = self.work;
        let mut rc = 0;
        'rows: for k in 0..c.nrow0 {
            let i = c.prow[k] as usize;
            if 1e7 < self.work - w0 {
                self.budget_check();
                if self.prelimit < 0 {
                    break;
                }
                w0 = self.work;
            }
            if self.row_cnt[i] < 1 || self.sense[i] == b'=' {
                continue;
            }
            // 1) literals
            lits.clear();
            let mut cnt = 0;
            let mut e = c.rhead[k];
            while e != -1 {
                let (q, v) = (c.rlit[e as usize], c.rslack[e as usize]);
                let (l, col) = if q < 0 { ((-1 - q) as usize, (-1 - q) as usize) } else { (n + q as usize, q as usize) };
                if self.col_cnt[col] > 0 {
                    if lv[l] == 0.0 {
                        lits.push(l);
                    }
                    lv[l] = if lv[l] <= v { v } else { lv[l] };
                }
                cnt += 1;
                e = c.rnext[e as usize];
            }
            self.work += cnt as f64 * 5.0 * mu;
            self.work += (1.0 + 1.0) * mu;
            if lits.is_empty() {
                continue;
            }
            // 2) the row's nodes by column
            let mut walked = 0;
            let mut q = self.row_head[i];
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 {
                    node_of[nd.col as usize] = q;
                }
                walked += 1;
                q = nd.nr;
            }
            self.work += (walked as f64 + walked as f64) * mu;
            // 3) clique table, once per pass (`451bed` on all rows, binaries only)
            if cl.is_none() {
                match self.clique_table(p, None, true) {
                    Ok(t) => {
                        clsc = vec![0.0; t.len()];
                        cl = Some(t);
                    }
                    Err(e) => {
                        rc = e;
                        break 'rows;
                    }
                }
            }
            let tab = cl.as_ref().unwrap();
            // 4) scores
            let (mut best, mut pick) = (0.0f64, -1i64);
            let mut cll: Vec<usize> = Vec::new();
            for &l in &lits {
                let col = if l < n { l } else { l - n };
                let v = if (nfill) < fill_max {
                    lv[l]
                } else if node_of[col] == NIL {
                    lv[l] = 0.0;
                    0.0
                } else {
                    lv[l]
                };
                let mut s = 0.0;
                if v != 0.0 {
                    let cc = if l < n { c.cnt0[l] } else { c.cnt1[l - n] } as f64;
                    s = ((0.1 * v) / (v + 100.0) + 1.0) * (cc / (100.0 + cc));
                }
                if s == 0.0 {
                    lv[l] = 0.0;
                    continue;
                }
                if best < s {
                    best = s;
                    pick = l as i64;
                }
                if !tab.is_empty() {
                    let b = tab.colstart[col];
                    let e = tab.colstart[col + 1].min(b + 1000);
                    for x in b..e {
                        let q = tab.cidx[x as usize];
                        // the original compares the sign of l (never negative) with q's: only q >= 0 counts
                        if q >= 0 {
                            if clsc[q as usize] == 0.0 {
                                cll.push(q as usize);
                            }
                            clsc[q as usize] += s;
                        }
                    }
                    self.work += (e - b) as f64 * 3.0 * mu;
                }
            }
            self.work += lits.len() as f64 * 5.0 * mu;
            for &q in &cll {
                let s = clsc[q];
                clsc[q] = 0.0;
                if best < s {
                    best = s;
                    pick = (q + 2 * n) as i64;
                }
            }
            self.work += (cll.len() as f64 + cll.len() as f64) * mu;
            // 5) apply
            if pick >= 0 {
                let mem: Vec<usize> = if (pick as usize) < 2 * n {
                    vec![pick as usize]
                } else {
                    let cq = pick as usize - 2 * n;
                    tab.lits[tab.start[cq] as usize..tab.start[cq + 1] as usize]
                        .iter()
                        .map(|&x| if x >= 0 { x as usize } else { n + (-1 - x) as usize })
                        .collect()
                };
                for &l in &mem {
                    let col = if l < n { l } else { l - n };
                    let v = lv[l];
                    if v == 0.0 {
                        continue;
                    }
                    self.tracker.mark_col(col as i32);
                    let mut nd = node_of[col];
                    if nd == NIL {
                        nfill += 1;
                        nd = self.new_node(0.0, i, col);
                        node_of[col] = nd;
                    } else {
                        nmod += 1;
                    }
                    let at = nd as usize;
                    let val = if l < n {
                        v + self.nd[at].val
                    } else {
                        let r = self.nd[at].row as usize;
                        self.nd[at].val -= v;
                        self.rhs[r] -= v;
                        self.nd[at].val
                    };
                    self.nd[at].val = val;
                    if val.abs() < 1e-10 {
                        nmod -= 1;
                        let (r, cc) = (self.nd[at].row as usize, self.nd[at].col as usize);
                        node_of[cc] = NIL;
                        ndel += 1;
                        self.row_cnt[r] -= 1;
                        self.col_cnt[cc] -= 1;
                        self.nd[at].col = -1;
                    }
                }
                self.work += mem.len() as f64 * 3.0 * mu;
                self.activity(Some(&[i as i32]));
                self.tracker.mark_row(i as i32);
            }
            // 6) clean up
            for &l in &lits {
                lv[l] = 0.0;
            }
            self.work += (lits.len() as f64 + lits.len() as f64) * mu;
            let mut walked = 0;
            let mut q = self.row_head[i];
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 {
                    node_of[nd.col as usize] = NIL;
                }
                walked += 1;
                q = nd.nr;
            }
            self.work += (walked as f64 + walked as f64) * mu;
        }
        self.coefs_changed += nmod + nfill as i32 + ndel;
        if rc != 0 { Err(rc) } else { Ok(()) }
    }
}
