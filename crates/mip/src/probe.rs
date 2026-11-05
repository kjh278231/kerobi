//! Probing `4d3a1d` (docs/m2_red_probe.md §2): the probe matrix and context of one pass (§2.3–2.7, with the row sort
//! `396a2d` and the transpose `7886b`) and the implication export to the conflict graph (§2.8.5). The engine `4e48f7`
//! is in probe_engine.rs. These read the state and only add work; writing results back (bounds, fixings,
//! aggregation, lifting) is the caller's (part B).
use crate::conflict;
use crate::state::{State, NIL};
use crate::Params;

/// 4d3a1d's arguments 6-10: mode (max(probe mode, 2)), final call, implication records, lift fill, p10.
#[derive(Clone, Copy, Debug)]
pub struct Args {
    pub mode: i32,
    pub a7: i32,
    pub a8: i32,
    pub a9: i32,
    pub a10: i32,
}

/// The reduction timer 4d3a1d gets (`3cd406` timer 8 or 9): `adf00e` used = acc (+ work while running).
#[derive(Clone, Copy, Debug)]
pub struct TimerView {
    pub acc: f64,
    pub running: bool,
    pub limit: f64,
}

impl TimerView {
    pub fn used(&self, w: f64) -> f64 {
        self.acc + if self.running { w } else { 0.0 }
    }
    /// `adf2be`
    pub fn over(&self, w: f64) -> bool {
        let r = if self.limit < 0.0 { 1.0 } else if 0.0 < self.limit { self.used(w) / self.limit } else { 0.0 };
        1.0 <= r
    }
}

/// The probe context (`calloc(1, 0x228)`, §2.6): the probe matrix and what `4e48f7` reads and writes.
#[derive(Clone, Debug, Default)]
pub struct Ctx {
    /// +0x20 / +0x24 probe rows (no GUB rows: both R), +0x28 / +0x2c columns (no GUB columns: both n)
    pub nr: usize,
    pub nrow0: usize,
    pub nc: usize,
    pub n: usize,
    /// +0x18 μ, +0xd0 IntFeasTol, +0xd8 work budget, +0xe0 budget factor
    pub mu: f64,
    pub itol: f64,
    pub budget: f64,
    pub fac: f64,
    /// +0xe8 stopped on the budget, +0xf0 final call
    pub stop: bool,
    pub p7: bool,
    /// +0xf4 / +0xf8 / +0xfc / +0x140 bounds changed / of binaries / rows made `=` / aggregations
    pub nchg: i32,
    pub nbin: i32,
    pub neq: i32,
    pub nagg: i32,
    /// +0x1b0 record redundant rows (lifting), +0x1e4 collect implications, +0x1e0 / +0x1f0 implication cap limit / cap
    pub rec: bool,
    pub imp: bool,
    pub lim: i32,
    pub cap: i32,
    /// row CSR +0x50 / +0x58 (end of the general part) / +0x60 / +0x68, column CSR +0x38 / +0x40 / +0x48
    pub rbeg: Vec<i32>,
    pub rnb: Vec<i32>,
    pub rcol: Vec<i32>,
    pub rval: Vec<f64>,
    pub cbeg: Vec<i32>,
    pub crow: Vec<i32>,
    pub cval: Vec<f64>,
    /// +0x80 rhs, +0x88 max |a|·(ub−lb) per row, +0x98 types, +0x120 senses
    pub rhs: Vec<f64>,
    pub range: Vec<f64>,
    pub vt: Vec<u8>,
    pub sense: Vec<u8>,
    /// +0x100 / +0x108 bounds (probing tightens them), +0x110 / +0x118 the bounds at the start
    pub lb: Vec<f64>,
    pub ub: Vec<f64>,
    pub lb0: Vec<f64>,
    pub ub0: Vec<f64>,
    /// +0x128 / +0x130 min / max activity − rhs (k infinite terms: ∓(k+1)·2^400)
    pub amin: Vec<f64>,
    pub amax: Vec<f64>,
    /// +0x148 / +0x150 / +0x158 aggregation x_j = mul·x_to + off (to −1 = none)
    pub aggto: Vec<i32>,
    pub aggmul: Vec<f64>,
    pub aggoff: Vec<f64>,
    /// +0x160 / +0x168 columns changed in the x=0 / x=1 scenario of a candidate (both feasible)
    pub cnt0: Vec<i32>,
    pub cnt1: Vec<i32>,
    /// +0x1f8 group bounds (3 per candidate reached, shared), +0x200 implied literals; +0x1e8 groups = gbeg.len()/2
    pub gbeg: Vec<i32>,
    pub imps: Vec<i32>,
    /// +0x1c0..+0x1d8 redundant-row records: head per row (−1), next, literal, slack
    pub rhead: Vec<i32>,
    pub rnext: Vec<i32>,
    pub rlit: Vec<i32>,
    pub rslack: Vec<f64>,
    /// 4d3a1d's local_40: state row of each probe row; lVar16: probe row of each state row (−1)
    pub prow: Vec<i32>,
    pub rowmap: Vec<i32>,
    /// local_250 (entries of the probed columns), local_238 (of the binary candidates)
    pub nnz_p: i64,
    pub nnz_b: i64,
}

/// Why a pass ends before the engine.
#[derive(Debug, PartialEq)]
pub enum Stop {
    /// nothing to probe, or a limit (`local_c0 = 0` exit)
    Done,
    /// status from 4d3a1d (3 infeasible with `bad_col`, 0x2711 memory)
    Status(i32, i32),
}

/// Descending sort of `aabbe3` (`aae2ff` with two payloads): ranges of 100 or more get a three-way partition around
/// a median-of-three pivot (at most 50 levels), the rest a shell sort with gaps n/2, /2.2, 2→1. Payloads move with
/// the keys exactly as in the original, so ties keep its order (crates/lp/src/common.rs `sort_desc`).
pub fn sort_desc<T: Copy>(key: &mut [f64], pay: &mut [T]) {
    quick(0, key.len(), 0, key, pay);
}

fn quick<T: Copy>(mut lo: usize, mut hi: usize, mut depth: i32, key: &mut [f64], pay: &mut [T]) {
    while depth < 50 {
        let len = hi - lo;
        if len < 100 {
            break;
        }
        let (d1, d2, d3) = (key[lo], key[hi - 1], key[lo + len / 2]);
        let mut p = d1;
        if ((d3 < d1) || (d1 < d2)) && ((d1 < d3) || (d2 < d1)) {
            p = d3;
            if ((d1 < d3) || (d3 < d2)) && ((d3 < d1) || (d2 < d3)) {
                p = d2;
            }
        }
        let (mut i, mut g, mut j) = (lo, lo, hi);
        if key[lo] > p {
            while i < hi && key[i] > p {
                i += 1;
            }
            g = i;
        }
        while i < j {
            let x = key[i];
            if x > p {
                key.swap(g, i);
                pay.swap(g, i);
                g += 1;
                i += 1;
            } else if x == p {
                i += 1;
            } else {
                j -= 1;
                let mut y = key[j];
                let mut done = false;
                while y <= p {
                    if y == p {
                        if i < j {
                            key.swap(i, j);
                            pay.swap(i, j);
                            i += 1;
                        }
                        done = true;
                        break;
                    }
                    if j <= i {
                        done = true;
                        break;
                    }
                    j -= 1;
                    y = key[j];
                }
                if !done && i < j {
                    let (xi, gi, ji) = (pay[i], pay[g], pay[j]);
                    key[i] = key[g];
                    pay[i] = gi;
                    key[g] = y;
                    pay[g] = ji;
                    key[j] = x;
                    pay[j] = xi;
                    g += 1;
                    i += 1;
                }
            }
        }
        depth += 1;
        if hi - j < g - lo {
            quick(j, hi, depth, key, pay);
            hi = g;
        } else {
            quick(lo, g, depth, key, pay);
            lo = j;
        }
    }
    let mut gap = (hi - lo) / 2;
    while gap > 0 {
        loop {
            let mut k = lo + gap;
            while k < hi {
                let (x, xp) = (key[k], pay[k]);
                let mut t = k;
                while t >= lo + gap && x > key[t - gap] {
                    key[t] = key[t - gap];
                    pay[t] = pay[t - gap];
                    t -= gap;
                }
                key[t] = x;
                pay[t] = xp;
                k += 1;
            }
            if gap == 2 && lo + 1 < hi {
                gap = 1;
            } else {
                break;
            }
        }
        gap = (gap as f64 / 2.2) as usize;
    }
}

/// `396a2d(len, cols, vals, vt, lb, ub, buf, W)`: one probe row as [general | binary] (moved by swaps), binaries by
/// |a| descending (signs taken off and put back), general entries by |a|·(ub−lb) descending (1e100 for an infinite
/// bound). Its work has no μ.
fn row_sort(cols: &mut [i32], vals: &mut [f64], vt: &[u8], lb: &[f64], ub: &[f64], w: &mut f64) {
    let len = cols.len();
    let mut nb = 0;
    for i in 0..len {
        if vt[cols[i] as usize] != b'B' {
            cols.swap(i, nb);
            vals.swap(i, nb);
            nb += 1;
        }
    }
    *w += len as f64 * 3.0;
    if nb < len {
        let (bc, bv) = (&mut cols[nb..], &mut vals[nb..]);
        for i in 0..bc.len() {
            if bv[i] < 0.0 {
                bc[i] = -1 - bc[i];
                bv[i] = -bv[i];
            }
        }
        *w += (len - nb) as f64 * 2.0;
        sort_desc(bv, bc);
        *w += (len - nb) as f64 * 5.0;
        for i in 0..bc.len() {
            if bc[i] < 0 {
                bc[i] = -1 - bc[i];
                bv[i] = -bv[i];
            }
        }
        *w += (len - nb) as f64 * 2.0;
    }
    if nb > 0 {
        let mut key: Vec<f64> = (0..nb)
            .map(|i| {
                let c = cols[i] as usize;
                if lb[c] <= -1e30 || 1e30 <= ub[c] { 1e100 } else { vals[i].abs() * (ub[c] - lb[c]) }
            })
            .collect();
        *w += nb as f64 * 5.0;
        let mut pay: Vec<(f64, i32)> = (0..nb).map(|i| (vals[i], cols[i])).collect();
        sort_desc(&mut key, &mut pay);
        for (i, &(v, c)) in pay.iter().enumerate() {
            vals[i] = v;
            cols[i] = c;
        }
    }
}

/// `7886b(ncols, nrows, rbeg, rcol, rval, cbeg, crow, cval, W)`: transpose; rows in order, so each column's rows
/// ascend. Its work has no μ.
fn transpose(nc: usize, rbeg: &[i32], rcol: &[i32], rval: &[f64], w: &mut f64) -> (Vec<i32>, Vec<i32>, Vec<f64>) {
    let nr = rbeg.len() - 1;
    let mut cbeg = vec![0i32; nc + 1];
    for r in 0..nr {
        for k in rbeg[r]..rbeg[r + 1] {
            cbeg[rcol[k as usize] as usize + 1] += 1;
        }
        *w += (rbeg[r + 1] - rbeg[r]) as f64 * 2.0;
    }
    if nr >= 1 {
        *w += nr as f64;
    }
    let mut acc = rbeg[nr];
    for c in (1..=nc).rev() {
        acc -= cbeg[c];
        cbeg[c] = acc;
    }
    *w += 0.0;
    let nz = rbeg[nr] as usize;
    let (mut crow, mut cval) = (vec![0i32; nz], vec![0.0f64; nz]);
    for r in 0..nr {
        for k in rbeg[r]..rbeg[r + 1] {
            let c = rcol[k as usize] as usize + 1;
            let p = cbeg[c] as usize;
            crow[p] = r as i32;
            cval[p] = rval[k as usize];
            cbeg[c] += 1;
        }
        *w += (rbeg[r + 1] - rbeg[r]) as f64 * 5.0;
    }
    if nr >= 1 {
        *w += nr as f64 * 3.0;
    }
    (cbeg, crow, cval)
}

impl State {
    /// One probing pass up to the engine (4d3a1d L429–3063, §2.3–2.7): probe rows (every row with entries), column
    /// counts, candidates (binary columns, then on the first pass `<` rows whose every entry reaches the rhs),
    /// the probe matrix with sorted rows, activities, column copy, budget. `t` is 4d3a1d's timer; `w` the work.
    /// Returns the context and the candidates (columns j, rows n + probe row).
    /// shortcut: default paths only — all-row selection (st+0xf8 = 0 and mode 2), no GUB (PREGUBPROBE −1 with mode
    /// 2), no Q/QC/SOS blocks, one thread; the others never run on the 38 models (docs/m2_red_probe.md §2.4, §1.4).
    pub fn probe_build(&mut self, p: &Params, a: &Args, first_pass: bool, t: &TimerView)
                       -> Result<(Box<Ctx>, Vec<i32>), Stop> {
        let (n, m) = (self.n, self.m);
        let mu = self.work_mult;
        assert!(a.mode == 2 && !self.eq_rows, "probe: only the all-row selection of mode 2 is ported");
        if 2e9 < (n + m) as f64 * 4.0 {
            return Err(Stop::Done);
        }
        // §2.3.2 rows with entries, §2.3.3 column counts
        let mut rowmap = vec![-1i32; m];
        let mut prow = Vec::new();
        for i in 0..m {
            if self.row_cnt[i] > 0 {
                rowmap[i] = prow.len() as i32;
                prow.push(i as i32);
            }
        }
        self.work += (m as f64 + m as f64) * mu;
        let colcnt: Vec<i32> = self.col_cnt.iter().map(|&c| c.max(0)).collect();
        self.work += mu * (n as f64 + n as f64);
        // §2.3.4 column candidates
        let (mut nnz_p, mut nnz_b) = (0i64, 0i64);
        let mut cand: Vec<i32> = Vec::new();
        for j in 0..n {
            let c = colcnt[j];
            if c <= 0 {
                continue;
            }
            if self.ub[j] - self.lb[j] < 1e-10 {
                if self.ub[j] + p.feastol < self.lb[j] {
                    return Err(Stop::Status(3, j as i32));
                }
                // `3de18e(lb, INF, env, st, j, 1)`: fixed with a record, inside the loop (its work lands here)
                let rc = self.fix_col(j, self.lb[j], true);
                if rc != 0 {
                    return Err(Stop::Status(rc, -1));
                }
                continue;
            }
            nnz_p += c as i64;
            if self.vtype[j] == b'B' {
                cand.push(j as i32);
                nnz_b += c as i64;
            }
        }
        self.work += n as f64 * 4.0 * mu;
        if cand.is_empty() || 2e9 < nnz_p as f64 + nnz_p as f64 {
            return Err(Stop::Done);
        }
        // the GUB block is off: PREGUBPROBE −1 asks for mode > 2 or st+0xf8
        if 2_000_000_000 < nnz_p {
            return Err(Stop::Done);
        }
        // §2.3.5 row candidates (first pass)
        if first_pass {
            for i in 0..m {
                let k = rowmap[i];
                if k < 0 || self.sense[i] != b'<' || !(2..=100).contains(&self.row_cnt[i]) || self.inf_min[i] != 0 {
                    continue;
                }
                let (mut v, mut ok) = (0, true);
                let mut q = self.row_head[i];
                while q != NIL {
                    let nd = self.nd[q as usize];
                    if nd.col >= 0 && (self.vtype[nd.col as usize] == b'C' || nd.val.abs() + self.act_min[i] < -1e-10) {
                        ok = false;
                        break;
                    }
                    v += 1;
                    q = nd.nr;
                }
                self.work += v as f64 * 4.0 * mu;
                if ok {
                    cand.push(k + n as i32);
                }
            }
        }
        // §2.5 context and probe matrix
        let r = prow.len();
        let mut c = Box::new(Ctx { nr: r, nrow0: r, nc: n, n, mu, itol: p.intfeastol, fac: 1.0, p7: a.a7 != 0, ..Ctx::default() });
        c.rec = a.a8 != 0 && (a.a10 != 0); // env+0x2728 == 0; st+0x3c8 (LP solution) is NULL in presolve
        c.imp = true; // env+0x2728 == 0
        let base = (n + m) as f64 * 1000.0 + 1e7;
        let base = if self.eq_rows { base * 10.0 } else { base };
        c.budget = if (p.presolve == 2 && a.mode == 3) || a.mode >= 4 {
            base * 1000.0
        } else if p.presolve != 2 && a.a7 == 0 && a.mode != 3 {
            base
        } else {
            base * 10.0
        };
        c.rhs = prow.iter().map(|&i| self.rhs[i as usize]).collect();
        c.sense = prow.iter().map(|&i| self.sense[i as usize]).collect();
        self.work += (m as f64 + r as f64 * 5.0) * mu;
        c.lb = self.lb.clone();
        c.ub = self.ub.clone();
        c.vt = self.vtype.clone();
        c.aggto = vec![-1; n];
        c.aggmul = vec![0.0; n];
        c.aggoff = vec![0.0; n];
        c.cnt0 = vec![0; n];
        c.cnt1 = vec![0; n];
        // row CSR in list order, live nodes only
        let mut pos = 0i32;
        for &i in &prow {
            c.rbeg.push(pos);
            let mut nb = pos;
            let mut v = 0;
            let mut q = self.row_head[i as usize];
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 {
                    if c.vt[nd.col as usize] != b'B' {
                        nb += 1;
                    }
                    c.rcol.push(nd.col);
                    c.rval.push(nd.val);
                    pos += 1;
                }
                v += 1;
                q = nd.nr;
            }
            self.work += v as f64 * 5.0 * mu;
            c.rnb.push(nb);
        }
        self.work += r as f64 * 6.0 * mu;
        c.rbeg.push(pos);
        c.lb0 = c.lb.clone();
        c.ub0 = c.ub.clone();
        for k in 0..r {
            let (b, e) = (c.rbeg[k] as usize, c.rbeg[k + 1] as usize);
            row_sort(&mut c.rcol[b..e], &mut c.rval[b..e], &c.vt, &c.lb, &c.ub, &mut self.work);
        }
        self.work += r as f64 * mu;
        // activities over the sorted rows (§2.5)
        let big = 2.5822498780869086e120;
        for k in 0..r {
            let (mut mn, mut mx) = (-c.rhs[k], -c.rhs[k]);
            let (mut imn, mut imx, mut rng) = (0, 0, 0.0f64);
            let (b, e) = (c.rbeg[k] as usize, c.rbeg[k + 1] as usize);
            for q in b..e {
                let (a, j) = (c.rval[q], c.rcol[q] as usize);
                let (l, u) = (c.lb[j], c.ub[j]);
                if a <= 0.0 {
                    if u < 1e30 { mn = gdn(mn, a * u) } else { imn += 1 }
                    if -1e30 < l { mx = gup(mx, a * l) } else { imx += 1 }
                } else {
                    if u < 1e30 { mx = gup(mx, a * u) } else { imx += 1 }
                    if -1e30 < l { mn = gdn(mn, a * l) } else { imn += 1 }
                }
                if rng < 1e100 {
                    if l <= -1e30 || 1e30 <= u {
                        rng = 1e100;
                    } else {
                        let d = a.abs() * (u - l);
                        if rng <= d {
                            rng = d;
                        }
                    }
                }
            }
            self.work += (e - b) as f64 * 4.0 * mu;
            if imn > 0 {
                mn = (-1 - imn) as f64 * big;
            }
            if imx > 0 {
                mx = (imx + 1) as f64 * big;
            }
            c.amin.push(mn);
            c.amax.push(mx);
            c.range.push(if rng < 1e100 { rng } else { 2.5 * big });
        }
        self.work += r as f64 * 9.0 * mu;
        let (cb, cr, cv) = transpose(n, &c.rbeg, &c.rcol, &c.rval, &mut self.work);
        c.cbeg = cb;
        c.crow = cr;
        c.cval = cv;
        // §1.4 one thread: few candidates, a small matrix or a small budget (else Threads decides; 1 here)
        // §2.7 limits, then the budget against the timer
        if self.work - self.work_base > self.budget || self.prelimit < 0 || t.over(self.work) {
            return Err(Stop::Done);
        }
        if t.limit != 0.0 {
            let left = t.limit - t.used(self.work);
            if left <= c.budget {
                c.budget = left;
            }
        }
        c.lim = 2_000_000_000;
        let cap = nnz_b as f64 + 1000.0 + nnz_b as f64;
        c.cap = if 2e9 < cap { 2_000_000_000 } else { (nnz_b * 2 + 1000) as i32 };
        c.prow = prow;
        c.rowmap = rowmap;
        c.nnz_p = nnz_p;
        c.nnz_b = nnz_b;
        Ok((c, cand))
    }

    /// The implication export after the engine (4d3a1d L4699–4930, §2.8.5): for each candidate column the engine
    /// reached that is still a live unfixed binary, queue (¬x_j, e) for the literals e implied false by x_j = 0 and
    /// (x_j, e) for x_j = 1, while the graph has room. Returns the pairs queued (local_870); `None` when `5b86ce`
    /// finds the graph full (4d3a1d then ends the pass without merging).
    pub fn probe_export(&mut self, p: &Params, c: &Ctx, cand: &[i32]) -> Option<i64> {
        let lim = conflict::limit(self.nnz, self.n as i32 - self.cols_removed, self.m as i32 - self.rows_removed,
                                  p.preclique, p.presolve);
        if !c.imp || lim < self.cg.edges() + self.cg.pa.len() as i64 {
            return None;
        }
        let mu = self.work_mult;
        let n = self.n;
        let free = |k: usize| self.col_cnt[k] >= 0 && self.lb[k] < 0.5 && self.ub[k] > 0.5;
        let mut nadd = 0i64;
        let groups = c.gbeg.len().saturating_sub(1) / 2;
        let mut t = 0;
        'all: while t < groups {
            let j = cand[t];
            if (j as usize) < n {
                let j = j as usize;
                if self.col_cnt[j] >= 0 && self.lb[j] <= 0.5 && self.ub[j] >= 0.5 {
                    for (side, lit) in [(0usize, -1 - j as i32), (1, j as i32)] {
                        let (s, e) = (c.gbeg[2 * t + side], c.gbeg[2 * t + side + 1]);
                        let mut q = s;
                        while q < e {
                            let x = c.imps[q as usize];
                            if free(conflict::var(x)) {
                                if self.cg.push(lit, x, false, lim, &mut self.work, mu) {
                                    self.work += (q - s) as f64 * 3.0 * mu;
                                    break 'all;
                                }
                                nadd += 1;
                            }
                            q += 1;
                        }
                        self.work += (e - s) as f64 * 3.0 * mu;
                    }
                }
            }
            t += 1;
        }
        self.work += t as f64 * 4.0 * mu;
        Some(nadd)
    }
}

/// `guarded_add` towards −∞ / +∞ (`rdown` / `rup`)
pub fn gdn(acc: f64, t: f64) -> f64 {
    crate::state::guarded_add(acc, t, false)
}
pub fn gup(acc: f64, t: f64) -> f64 {
    crate::state::guarded_add(acc, t, true)
}

impl State {
    /// Oracle loader (tools/probe_check.py): replace the row lists by the dumped ones (list order, dead nodes with
    /// column −1 included). Column lists are left empty: the probing pass reads rows only.
    pub fn load_rows(&mut self, rbeg: &[i64], rcol: &[i32], rval: &[f64]) {
        let nz = rcol.len();
        self.nd = vec![crate::state::Node { val: 0.0, row: 0, col: -1, nr: NIL, nc: NIL }; nz];
        self.row_head = vec![NIL; self.m];
        self.col_head = vec![NIL; self.n];
        for i in 0..self.m {
            let (b, e) = (rbeg[i] as usize, rbeg[i + 1] as usize);
            for k in b..e {
                self.nd[k] = crate::state::Node { val: rval[k], row: i as i32, col: rcol[k], nr: if k + 1 < e { (k + 1) as u32 } else { NIL }, nc: NIL };
            }
            if b < e {
                self.row_head[i] = b as u32;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe_engine::EngineEnd;
    use rsolver_model::Model;

    #[test]
    fn sort_desc_moves_payload() {
        for n in [0usize, 1, 7, 99, 100, 257] {
            let mut key: Vec<f64> = (0..n).map(|i| ((i * 7919) % 1013) as f64).collect();
            let orig = key.clone();
            let mut pay: Vec<(f64, i32)> = (0..n).map(|i| (i as f64, i as i32)).collect();
            sort_desc(&mut key, &mut pay);
            assert!(key.windows(2).all(|w| w[0] >= w[1]));
            assert!(pay.iter().zip(&key).all(|(&(_, i), &k)| orig[i as usize] == k));
        }
    }

    /// x0 − x1 ≥ 0 and x0 + x1 ≤ 1: trying x1 = 1 forces x0 = 1 and breaks the second row, so x1 = 0.
    #[test]
    fn probing_fixes_a_binary() {
        let md = Model {
            name: "t".into(),
            sense: 1,
            obj: vec![1.0, -1.0],
            lb: vec![0.0; 2],
            ub: vec![1.0; 2],
            vtype: b"BB".to_vec(),
            col_beg: vec![0, 2],
            col_len: vec![2, 2],
            col_ind: vec![0, 1, 0, 1],
            col_val: vec![1.0, 1.0, -1.0, 1.0],
            row_sense: b"><".to_vec(),
            rhs: vec![0.0, 1.0],
            var_names: vec!["x0".into(), "x1".into()],
            constr_names: vec!["r0".into(), "r1".into()],
            ..Model::default()
        };
        let p = Params::default();
        let mut st = State::new(&md, &p);
        st.activity(None);
        let args = Args { mode: 2, a7: 0, a8: 0, a9: 0, a10: 0 };
        let t = TimerView { acc: 0.0, running: true, limit: 0.0 };
        let (mut c, mut cand) = st.probe_build(&p, &args, true, &t).unwrap();
        let mut w = st.work;
        assert_eq!(cand.len(), 2 + 2); // both columns and both rows (r0 is stored as −x0 + x1 ≤ 0)
        assert_eq!(st.probe_engine(&p, &mut c, &mut cand, &mut w), EngineEnd::Done);
        assert_eq!((c.lb[1], c.ub[1]), (0.0, 0.0));
        assert!(c.nchg >= 1);
    }
}
