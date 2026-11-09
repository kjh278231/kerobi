//! `451bed`: the clique table built from rows (docs/m2/m2_red_clique.md §2). `45885d` (here, §3) calls it on the `53a487`
//! rows with `bin_only` false; probing (`4d3a1d`) calls it on all rows with `bin_only` true.
use crate::state::{State, INF, NIL};
use crate::Params;

/// Output of `451bed`. A literal `l >= 0` is x_l, `l < 0` the complement of x_{-1-l}; a literal is "on" when its
/// column leaves the bound that gives the row its minimum activity. At most one literal of a clique is on.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cliques {
    /// clique c is `lits[start[c]..start[c + 1]]` (`start` has one more entry than there are cliques)
    pub start: Vec<i32>,
    pub lits: Vec<i32>,
    /// per clique: b'=' exactly one literal is on, b'<' at most one
    pub sense: Vec<u8>,
    /// per clique: 1 when it has fewer than two non-binary literals (`out_flag`)
    pub flag: Vec<i32>,
    /// `cidx[colstart[j]..colstart[j + 1]]`: the cliques holding column j in increasing order, c for x_j and -1-c for
    /// its complement (`colstart` has n + 1 entries)
    pub colstart: Vec<i32>,
    pub cidx: Vec<i32>,
    /// rows not wholly one clique (written to state+0x390), whether each gave a clique, and their longest live count
    pub rec_rows: Vec<i32>,
    pub rec_flag: Vec<u8>,
    pub maxlen: i32,
}

impl Cliques {
    pub fn len(&self) -> usize {
        self.sense.len()
    }
    pub fn is_empty(&self) -> bool {
        self.sense.is_empty()
    }
}

impl State {
    /// `451bed(env, st, nlist, list, bin_only, ...)`: `list` None = rows 0..m. With no clique everything stays empty
    /// (Gurobi then leaves the row outputs at 0). Err = status (3: column `bad_col` has lb > ub + feastol).
    /// shortcut: the SOS1 cliques (+0x1f8 > 0, `a85f71`) are not ported (linear models only), nor the 2e9 caps on the
    /// literal and clique buffers.
    pub fn clique_table(&mut self, p: &Params, list: Option<&[i32]>, bin_only: bool) -> Result<Cliques, i32> {
        let (ft, mu) = (p.feastol, self.work_mult);
        let all: Vec<i32>;
        let rows = match list {
            Some(l) => l,
            None => {
                all = (0..self.m as i32).collect();
                &all
            }
        };
        let mut t = Cliques::default();
        for &r in rows {
            let r = r as usize;
            if self.row_cnt[r] <= 0 {
                continue;
            }
            let mut so = self.sense[r];
            let (mut s, mut nb, mut produced) = (0i32, 0, 0u8);
            let mark = t.lits.len();
            if self.inf_min[r] == 0 && self.inf_max[r] == 0 {
                let mut ent = 0;
                let mut q = self.row_head[r];
                while q != NIL {
                    let node = self.nd[q as usize];
                    q = node.nr;
                    ent += 1;
                    if node.col < 0 {
                        continue;
                    }
                    let j = node.col as usize;
                    let w = self.ub[j] - self.lb[j];
                    if w <= 1e-10 {
                        if self.ub[j] + ft < self.lb[j] {
                            self.bad_col = j as i32;
                            return Err(3);
                        }
                        let mut mx = 0.0f64;
                        if w > 1e-13 {
                            let mut c = self.col_head[j];
                            while c != NIL {
                                let e = self.nd[c as usize];
                                if e.col >= 0 && mx <= e.val.abs() {
                                    mx = e.val.abs();
                                }
                                c = e.nc;
                            }
                        }
                        if w * mx < ft {
                            let v = if 1e-10 >= self.ub[j] && !(self.lb[j] < -1e-10) {
                                0.0
                            } else if self.obj[j] > 0.0 {
                                self.lb[j]
                            } else {
                                self.ub[j]
                            };
                            let rc = self.fix_col(j, v, true);
                            if rc != 0 {
                                return Err(rc);
                            }
                            continue;
                        }
                    }
                    if bin_only && self.vtype[j] != b'B' {
                        so = b'<';
                        continue;
                    }
                    let mut a = node.val;
                    let aa = a.abs();
                    if aa < 1e-4 {
                        so = b'<';
                        continue;
                    }
                    let d = w * aa + self.act_min[r]; // read only when s >= 0
                    if s < 0 || !(d >= -1e-10) {
                        // max-activity mode, `=` rows only
                        if s > 0 || self.sense[r] != b'=' || 1e-10 < self.act_max[r] - w * aa {
                            so = b'<';
                            continue;
                        }
                        a = -a;
                        s -= 1;
                    } else {
                        s += 1;
                        if so == b'=' && d > 1e-10 {
                            so = b'<';
                        }
                    }
                    if self.vtype[j] != b'B' {
                        nb += 1;
                    }
                    t.lits.push(if 0.0 <= a { j as i32 } else { -1 - j as i32 });
                }
                self.work += (ent as f64 * 8.0) * mu;
            }
            let rc = self.row_cnt[r];
            if !(s < 2 && -2 < s) {
                t.start.push(mark as i32);
                t.sense.push(if s == rc { so } else { b'<' });
                t.flag.push((nb < 2) as i32);
                produced = 1;
            } else {
                t.lits.truncate(mark);
            }
            if s != rc && -s != rc {
                t.rec_rows.push(r as i32);
                t.rec_flag.push(produced);
                t.maxlen = t.maxlen.max(rc);
            }
        }
        self.work += (8.0 * rows.len() as f64) * mu;
        if !bin_only && 1 <= self.nbin && self.nbin <= self.n as i32 - 2 {
            self.implied_pairs(&mut t);
        }
        if t.sense.is_empty() {
            return Ok(Cliques::default());
        }
        self.index_cliques(&mut t);
        Ok(t)
    }

    /// `451bed` L1026-1400, `bin_only` false: for each binary b, the non-binary literals of the rows b alone fills
    /// when on (`l0`) and when off (`l1`), at most 16 each, each row used once; every (l0, l1) pair cannot be on
    /// together and becomes a `<` clique (at most 16 per binary).
    fn implied_pairs(&mut self, t: &mut Cliques) {
        let (n, mu) = (self.n, self.work_mult);
        let mut used_row = vec![false; self.m];
        let mut listed = vec![false; n];
        for b in 0..n {
            if self.vtype[b] != b'B' {
                continue;
            }
            let (mut l0, mut l1): (Vec<i32>, Vec<i32>) = (Vec::new(), Vec::new());
            let mut seen = 0;
            let mut q = self.col_head[b];
            while q != NIL {
                if 16 <= l0.len() && 16 <= l1.len() {
                    break;
                }
                let node = self.nd[q as usize];
                q = node.nc;
                seen += 1;
                let r = node.row as usize;
                if node.col < 0 || used_row[r] {
                    continue;
                }
                let ab = node.val;
                let dir = if self.inf_min[r] == 0 && ab.abs() + self.act_min[r] >= -1e-10 {
                    1.0
                } else if self.sense[r] == b'=' && self.inf_max[r] == 0 && 1e-10 >= ab.abs() + self.act_max[r] {
                    -1.0 // asm 0x40b9f6: |a| + (max act - rhs), not max act - rhs - |a|
                } else {
                    continue;
                };
                used_row[r] = true;
                let l = if ab * dir > 0.0 { &mut l0 } else { &mut l1 };
                let mut it = 0;
                let mut e = self.row_head[r];
                while e != NIL && l.len() < 16 {
                    let x = self.nd[e as usize];
                    e = x.nr;
                    it += 1;
                    if x.col >= 0 && self.vtype[x.col as usize] != b'B' && !listed[x.col as usize] {
                        listed[x.col as usize] = true;
                        l.push(if dir * x.val > 0.0 { x.col } else { -1 - x.col });
                    }
                }
                self.work += (it as f64 * 3.0) * mu;
            }
            self.work += (seen as f64 * 7.0) * mu;
            for l in [&l0, &l1] {
                for &x in l {
                    listed[var(x)] = false;
                }
                self.work += (l.len() as f64 + l.len() as f64) * mu;
            }
            let (mut lo, mut hi) = (l0.len(), l1.len());
            if lo == 0 || hi == 0 {
                continue;
            }
            if 16 < lo * hi {
                if hi < lo {
                    lo = 16 / hi;
                } else {
                    hi = 16 / lo;
                }
            }
            for &x in &l0[..lo] {
                for &y in &l1[..hi] {
                    t.start.push(t.lits.len() as i32);
                    t.sense.push(b'<');
                    t.flag.push(1);
                    t.lits.extend([x, y]);
                }
                self.work += (hi as f64 * 6.0) * mu;
            }
            self.work += lo as f64 * mu;
        }
        self.work += n as f64 * mu;
    }

    /// `451bed` end (L1215-1340): close `start` and build the column index (counting sort, cliques in order).
    fn index_cliques(&mut self, t: &mut Cliques) {
        let (n, mu) = (self.n, self.work_mult);
        t.start.push(t.lits.len() as i32);
        self.work += t.lits.len() as f64 * mu;
        let mut cs = vec![0i32; n + 2];
        for &l in &t.lits {
            cs[var(l) + 2] += 1;
        }
        if n >= 1 {
            for j in 0..n {
                cs[j + 2] += cs[j + 1];
            }
            self.work += n as f64 * mu;
        }
        t.cidx = vec![0; t.lits.len()];
        for c in 0..t.sense.len() {
            let (a, b) = (t.start[c] as usize, t.start[c + 1] as usize);
            for &l in &t.lits[a..b] {
                let v = var(l);
                t.cidx[cs[v + 1] as usize] = if l < 0 { -1 - c as i32 } else { c as i32 };
                cs[v + 1] += 1;
            }
            self.work += ((b - a) as f64 * mu) * 3.0;
        }
        self.work += t.sense.len() as f64 * mu;
        cs.truncate(n + 1);
        t.colstart = cs;
    }
}

/// env+0x2498 GURO_PAR_ZERO at its default
const ZERO: f64 = 1e-13;

fn var(l: i32) -> usize {
    (if l < 0 { -1 - l } else { l }) as usize
}

/// `45885d` scratch: new bounds of the columns marked so far (bit 1 lb, 2 ub, in mark order) and the coefficient
/// scatter (state+0x3a8 / +0x3b0 in the original).
struct Marks {
    newlb: Vec<f64>,
    newub: Vec<f64>,
    flag: Vec<u8>,
    order: Vec<usize>,
}

impl Marks {
    fn mark(&mut self, v: usize, bit: u8) {
        if self.flag[v] == 0 {
            self.order.push(v);
        }
        self.flag[v] |= bit;
    }
}

impl State {
    /// `45885d(st, env, timer)` (docs/m2/m2_red_clique.md §3): cover each row by cliques of `451bed`'s table, take the
    /// clique-aware activity range, then tighten bounds, drop redundant `<` rows and strengthen coefficients.
    /// Returns 0 or a status (3: `bad_row` / `bad_col` infeasible).
    /// shortcut: the reduction timer (`adf2be`) is not passed in; its budget is never reached on the 38 models.
    pub fn clique(&mut self, p: &Params) -> i32 {
        let (ft, ift, mu) = (p.feastol, self.intfeastol, self.work_mult);
        let tt = 100.0 * ft;
        let mut w0 = self.work;
        let list = self.adjacent_list(false);
        let t = match self.clique_table(p, Some(&list), false) {
            Ok(t) => t,
            Err(rc) => return rc,
        };
        if t.is_empty() {
            return 0;
        }
        let (n, ncl) = (self.n, t.len());
        let mut cnt = vec![0i32; ncl.max(self.m)];
        let mut lst = vec![0usize; ncl.max(self.m)];
        let mut mk = Marks { newlb: vec![0.0; n], newub: vec![0.0; n], flag: vec![0; n], order: Vec::new() };
        let mut sc = vec![0.0f64; n];
        let (mut ncoef, mut nrem) = (0, 0);
        let mut rows_out: Vec<i32> = Vec::new();
        let mut idx = 0;
        while idx < t.rec_rows.len() {
            if self.work - w0 > 1e7 {
                self.budget_check(); // `3ba5ac`, `3ba9de`
                if self.prelimit < 0 {
                    for &v in &mk.order {
                        mk.flag[v] = 0;
                    }
                    self.work += (mk.order.len() as f64 + mk.order.len() as f64) * mu;
                    return 0;
                }
                w0 = self.work;
            }
            let r = t.rec_rows[idx] as usize;
            let produced = t.rec_flag[idx];
            idx += 1;
            if self.inf_max[r] != 0 && (self.inf_min[r] != 0 || self.sense[r] != b'=') {
                continue;
            }
            // (a) nearly fixed columns, as in 451bed but with w < 1e-10
            let mut k = 0;
            let mut q = self.row_head[r];
            while q != NIL {
                let node = self.nd[q as usize];
                q = node.nr;
                k += 1;
                if node.col < 0 {
                    continue;
                }
                let j = node.col as usize;
                let w = self.ub[j] - self.lb[j];
                if 1e-10 > w {
                    if self.ub[j] + ft < self.lb[j] {
                        self.bad_col = j as i32;
                        return 3;
                    }
                    let mut mx = 0.0f64;
                    if w > 1e-13 {
                        let mut c = self.col_head[j];
                        while c != NIL {
                            let e = self.nd[c as usize];
                            if e.col >= 0 {
                                mx = crate::helpers::maxsd(mx, e.val.abs());
                            }
                            c = e.nc;
                        }
                    }
                    if w * mx < ft {
                        let v = if 1e-10 >= self.ub[j] && !(self.lb[j] < -1e-10) {
                            0.0
                        } else if self.obj[j] > 0.0 {
                            self.lb[j]
                        } else {
                            self.ub[j]
                        };
                        let rc = self.fix_col(j, v, true);
                        if rc != 0 {
                            return rc;
                        }
                    }
                }
            }
            self.work += (k as f64 * 4.0) * mu;
            if !(1 < self.row_cnt[r]) {
                continue;
            }
            // (b) scatter the row and count, per clique, how many of its columns it holds
            let (mut nd, mut walked) = (0usize, 0);
            let mut q = self.row_head[r];
            while q != NIL {
                let node = self.nd[q as usize];
                q = node.nr;
                walked += 1;
                if node.col < 0 {
                    continue;
                }
                let j = node.col as usize;
                sc[j] = node.val;
                let (a, b) = (t.colstart[j] as usize, t.colstart[j + 1] as usize);
                for &c in &t.cidx[a..b] {
                    let c = var(c);
                    if cnt[c] == 0 {
                        lst[nd] = c;
                        nd += 1;
                    }
                    cnt[c] += 1;
                }
                self.work += ((b - a) as f64 * 3.0) * mu;
            }
            self.work += (walked as f64 * 5.0) * mu;
            if nd == 0 {
                self.clear_scatter(r, &mut sc);
                continue;
            }
            // (c) greedy clique cover: parts p hold plit/pco[pst[p]..pst[p + 1]]
            let mut target = self.row_cnt[r];
            let (mut np, mut cur, mut acc) = (0usize, 0usize, 0.0f64);
            let (mut pst, mut plit, mut pco) = (Vec::new(), Vec::new(), Vec::new());
            let (mut psense, mut pflag) = (Vec::new(), Vec::new());
            loop {
                pst.push(plit.len());
                if np >= nd || acc >= 1e8 {
                    break;
                }
                let (mut best, mut bestpos) = (1, usize::MAX);
                let (start, mut scanned) = (cur, 0);
                let (pick, stop);
                loop {
                    let c = lst[cur];
                    if cnt[c] == target {
                        (pick, stop) = (cur, cur);
                        break;
                    }
                    if best < cnt[c] {
                        (best, bestpos) = (cnt[c], cur);
                    }
                    cur += 1;
                    if cur == nd {
                        cur = np;
                    }
                    if cur == start {
                        (pick, stop) = (bestpos, cur);
                        break;
                    }
                    scanned += 1;
                }
                self.work += (scanned as f64 + scanned as f64) * mu;
                if pick == usize::MAX {
                    break;
                }
                acc += scanned as f64;
                lst.swap(pick, np);
                if best < target && stop != pick {
                    target = best;
                }
                cur = if stop == np { stop + 1 } else { stop };
                let c = lst[np];
                let (a, b) = (t.start[c] as usize, t.start[c + 1] as usize);
                let s0 = plit.len();
                for &l in &t.lits[a..b] {
                    let v = var(l);
                    if sc[v] == 0.0 {
                        continue;
                    }
                    plit.push(l);
                    pco.push(sc[v]);
                    sc[v] = 0.0;
                    let (ca, cb) = (t.colstart[v] as usize, t.colstart[v + 1] as usize);
                    for &c3 in &t.cidx[ca..cb] {
                        let c3 = var(c3);
                        if cnt[c3] != 0 {
                            cnt[c3] -= 1;
                        }
                    }
                    self.work += ((cb - ca) as f64 * 2.0) * mu;
                }
                self.work += ((b - a) as f64 * 7.0) * mu;
                psense.push(if plit.len() - s0 == b - a { t.sense[c] } else { b'<' });
                pflag.push(t.flag[c]);
                np += 1;
            }
            self.work += (np as f64 * 4.0) * mu;
            for &c in &lst[..nd] {
                cnt[c] = 0;
            }
            self.work += (nd as f64 + nd as f64) * mu;
            if np == 0 {
                self.clear_scatter(r, &mut sc);
                continue;
            }
            // (d) the columns left alone and each part's range
            let nchg0 = mk.order.len();
            let (mut min_s, mut max_s, mut min_m, mut max_m) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
            let mut walked = 0;
            let mut q = self.row_head[r];
            while q != NIL {
                let node = self.nd[q as usize];
                q = node.nr;
                walked += 1;
                if node.col < 0 || sc[node.col as usize] == 0.0 {
                    continue;
                }
                let j = node.col as usize;
                let a = sc[j];
                sc[j] = 0.0;
                pco.push(a);
                plit.push(j as i32);
                let (u, l) = (self.ub[j], self.lb[j]);
                if a <= 0.0 {
                    if u < INF {
                        min_s += a * u;
                        min_m -= u.abs() * a;
                    }
                    if -INF < l {
                        max_s += l * a;
                        max_m -= l.abs() * a;
                    }
                } else {
                    if u < INF {
                        max_s += a * u;
                        max_m += u.abs() * a;
                    }
                    if -INF < l {
                        min_s += l * a;
                        min_m += l.abs() * a;
                    }
                }
            }
            self.work += (walked as f64 * 7.0) * mu;
            let (mut pmin, mut pmax) = (Vec::with_capacity(np), Vec::with_capacity(np));
            for q in 0..np {
                let (mut dmin, mut dmax, mut bs) = (0.0f64, 0.0f64, 0.0f64);
                for k in pst[q]..pst[q + 1] {
                    let (l, a) = (plit[k], pco[k]);
                    let v = var(l);
                    let (base, d) = if l < 0 { (self.ub[v], self.lb[v] - self.ub[v]) } else { (self.lb[v], self.ub[v] - self.lb[v]) };
                    bs += a * base;
                    let d = d * a;
                    dmin = if dmin <= d { dmin } else { d };
                    dmax = if d <= dmax { dmax } else { d };
                }
                self.work += ((pst[q + 1] - pst[q]) as f64 * 4.0) * mu;
                pmin.push(dmin);
                pmax.push(dmax);
                min_s = min_s + dmin + bs;
                max_s = max_s + bs + dmax;
                min_m = min_m + dmin.abs() + bs.abs();
                max_m = max_m + dmax.abs() + bs.abs();
            }
            self.work += (np as f64 * 3.0) * mu;
            let (okmin, okmax) = (min_m < 1e7, max_m < 1e7);
            let (cmax, cmin) = (max_s - self.rhs[r], min_s - self.rhs[r]);
            if self.inf_min[r] == 0 && tt < cmin {
                self.bad_row = r as i32;
                return 3;
            }
            if self.sense[r] == b'<' {
                if !(okmin || okmax) {
                    continue;
                }
                if !(self.inf_max[r] != 0 || tt <= cmax) {
                    if produced == 0 && cmax < 1e-10 {
                        nrem += 1;
                        let mut walked = 0;
                        let mut q = self.row_head[r];
                        while q != NIL {
                            let at = q as usize;
                            q = self.nd[at].nr;
                            walked += 1;
                            let j = self.nd[at].col;
                            if j >= 0 {
                                self.tracker.mark_col(j);
                                self.col_cnt[j as usize] -= 1;
                                self.nd[at].col = -1;
                            }
                        }
                        self.work += (walked as f64 * 3.0) * mu;
                        self.row_cnt[r] = -2;
                    }
                    continue;
                }
            } else {
                if self.inf_max[r] == 0 && cmax < -tt {
                    self.bad_row = r as i32;
                    return 3;
                }
                if !(okmin || okmax) {
                    continue;
                }
            }
            // (e) bounds
            let (r1, r2) = (tt - cmin, tt + cmax);
            let ncoef0 = ncoef;
            let mut s = -1.0f64;
            while s <= 1.0 {
                let on = (0.0 <= s || self.sense[r] == b'=')
                    && ((0.0 < s && self.inf_min[r] == 0 && okmin) || (s < 0.0 && self.inf_max[r] == 0 && okmax));
                if on {
                    for q in 0..np {
                        let res = if s <= 0.0 { r2 - pmax[q] } else { pmin[q] + r1 };
                        for k in pst[q]..pst[q + 1] {
                            let (sa, l) = (s * pco[k], plit[k]);
                            let v = var(l);
                            if !(1e-06 <= sa.abs()) {
                                continue;
                            }
                            if pflag[q] == 0 && (self.vtype[v] == b'C' || self.ub[v] - self.lb[v] > 1.5) {
                                continue;
                            }
                            let (u, lo) = (self.ub[v], self.lb[v]);
                            if l < 0 {
                                if 0.0 <= sa {
                                    let d = if INF > u { -(u - lo) } else { lo };
                                    if res < sa * d {
                                        mk.mark(v, 1);
                                        mk.newlb[v] = u;
                                    }
                                } else {
                                    let mut x = res / sa + u;
                                    if self.vtype[v] != b'C' {
                                        x = (x - ift).ceil();
                                    }
                                    x = crate::helpers::minsd(x, u);
                                    if lo + 1e-06 < x {
                                        mk.mark(v, 1);
                                        mk.newlb[v] = x;
                                    }
                                }
                            } else if sa <= 0.0 {
                                let d = if lo > -INF { u - lo } else { u };
                                if res < sa * d {
                                    mk.mark(v, 2);
                                    mk.newub[v] = lo;
                                }
                            } else {
                                let mut x = res / sa + lo;
                                if self.vtype[v] != b'C' {
                                    x = (x + ift).floor();
                                }
                                x = crate::helpers::maxsd(x, lo);
                                if x < u - 1e-06 {
                                    mk.mark(v, 2);
                                    mk.newub[v] = x;
                                }
                            }
                        }
                        self.work += ((pst[q + 1] - pst[q]) as f64 * 7.0) * mu;
                    }
                    self.work += (np as f64 + np as f64) * mu;
                    let dd = if s <= 0.0 { cmax } else { -cmin };
                    let mut k = pst[np];
                    while (k as i32) < self.row_cnt[r] {
                        let (sa, j) = (s * pco[k], plit[k] as usize);
                        if sa > 1e-06 {
                            let x = dd / sa + self.lb[j];
                            if mk.flag[j] & 2 == 0 {
                                if x < self.ub[j] - tt {
                                    mk.mark(j, 2);
                                    mk.newub[j] = x;
                                }
                            } else if x < mk.newub[j] - tt {
                                mk.newub[j] = x;
                            }
                        } else if sa < -1e-06 {
                            let x = dd / sa + self.ub[j];
                            if mk.flag[j] & 1 == 0 {
                                if self.lb[j] + tt < x {
                                    mk.mark(j, 1);
                                    mk.newlb[j] = x;
                                }
                            } else if mk.newlb[j] + tt < x {
                                mk.newlb[j] = x;
                            }
                        }
                        k += 1;
                    }
                    self.work += ((k - pst[np]) as f64 * 5.0) * mu;
                }
                s += 2.0;
            }
            // (f) coefficients of `<` rows that got no new bound
            if self.sense[r] != b'=' && self.inf_max[r] == 0 && mk.order.len() == nchg0 && okmax {
                for q in 0..np {
                    let mut dd = cmax - pmax[q];
                    if !(pmin[q] + dd < -tt && dd.abs() < 1e8) {
                        continue;
                    }
                    let mut de = -dd;
                    if de <= tt {
                        de = 0.0;
                    }
                    dd += de;
                    let mut k = pst[q];
                    while k < pst[q + 1] {
                        let v = var(plit[k]);
                        if self.vtype[v] == b'C' || self.ub[v] - self.lb[v] != 1.0 {
                            break;
                        }
                        k += 1;
                    }
                    self.work += ((k - pst[q]) as f64 * 2.0) * mu;
                    if k < pst[q + 1] {
                        continue;
                    }
                    self.rhs[r] -= de;
                    for k in pst[q]..pst[q + 1] {
                        let (a, l) = (pco[k], plit[k]);
                        let v = var(l);
                        if (l >= 0) == (0.0 < a) {
                            if 0.0 < de {
                                if l < 0 {
                                    pco[k] = a + de;
                                    self.rhs[r] = self.ub[v] * de + self.rhs[r];
                                } else {
                                    pco[k] = a - de;
                                    self.rhs[r] -= self.lb[v] * de;
                                }
                                ncoef += 1;
                            }
                        } else if tt < a.abs() - dd {
                            if l < 0 {
                                if self.ub[v] == 0.0 || a < 1e8 {
                                    ncoef += 1;
                                    let x = self.rhs[r] - a.abs() * self.ub[v];
                                    self.rhs[r] = x + self.ub[v] * dd;
                                    pco[k] = dd;
                                }
                            } else if self.lb[v] == 0.0 || -a < 1e8 {
                                ncoef += 1;
                                let x = a.abs() * self.lb[v] + self.rhs[r];
                                self.rhs[r] = x - self.lb[v] * dd;
                                pco[k] = -dd;
                            }
                        }
                    }
                    self.work += ((pst[q + 1] - pst[q]) as f64 * 2.0) * mu;
                }
                self.work += (np as f64 + np as f64) * mu;
                let (end, mut k) = (self.row_cnt[r] as usize, pst[np]);
                let mut last = k;
                while k < end {
                    let j = plit[k] as usize;
                    last = k + 1;
                    if self.vtype[j] != b'C' && self.lb[j] == 0.0 {
                        let a = pco[k];
                        if a > r2 && self.ub[j] == 1.0 {
                            ncoef += 1;
                            let d = a - cmax;
                            pco[k] = a - d;
                            self.rhs[r] -= d;
                        } else if r2 < -a {
                            ncoef += 1;
                            pco[k] = a + -(cmax + a);
                        }
                    }
                    k += 1;
                }
                self.work += ((last - pst[np]) as f64 * 5.0) * mu;
            }
            // (g) write changed coefficients back into the row list
            if ncoef != ncoef0 {
                rows_out.push(r as i32);
                self.tracker.mark_row(r as i32);
                let rc = self.row_cnt[r] as usize;
                for k in 0..rc {
                    sc[var(plit[k])] = pco[k];
                }
                self.work += (rc as f64 * 3.0) * mu;
                let mut walked = 0;
                let mut q = self.row_head[r];
                while q != NIL {
                    let at = q as usize;
                    q = self.nd[at].nr;
                    walked += 1;
                    let j = self.nd[at].col;
                    if j < 0 {
                        continue;
                    }
                    let j = j as usize;
                    let x = sc[j];
                    if x.abs() <= ZERO {
                        self.tracker.mark_col(j as i32);
                        self.nd[at].col = -1;
                        self.col_cnt[j] -= 1;
                        self.row_cnt[r] -= 1;
                    } else {
                        if 1e-10 < (self.nd[at].val - x).abs() {
                            self.tracker.mark_col(j as i32);
                        }
                        self.nd[at].val = x;
                    }
                    sc[j] = 0.0;
                }
                self.work += (walked as f64 * 4.0) * mu;
            }
        }
        // apply the new bounds; activities of every touched row are summed again
        self.work += (idx as f64 * 8.0) * mu;
        let mut seen = vec![false; self.m];
        for &r in &rows_out {
            seen[r as usize] = true;
        }
        self.work += (rows_out.len() as f64 + rows_out.len() as f64) * mu;
        for &j in &mk.order {
            self.tracker.mark_col(j as i32);
            let mut q = self.col_head[j];
            while q != NIL {
                let node = self.nd[q as usize];
                q = node.nc;
                if node.col >= 0 && !seen[node.row as usize] {
                    seen[node.row as usize] = true;
                    rows_out.push(node.row);
                }
            }
            let f = mk.flag[j];
            if f & 1 != 0 {
                if self.vtype[j] != b'C' {
                    mk.newlb[j] = (mk.newlb[j] - ift).ceil();
                }
                self.lb[j] = mk.newlb[j];
            }
            if f & 2 != 0 {
                if self.vtype[j] != b'C' {
                    mk.newub[j] = (mk.newub[j] + ift).floor();
                }
                self.ub[j] = mk.newub[j];
            }
            if self.ub[j] - 1e-10 < self.lb[j] {
                let rc = self.fix_near(j, ft);
                if rc != 0 {
                    return rc;
                }
            }
        }
        self.work += (mk.order.len() as f64 * 8.0) * mu;
        self.coefs_changed += ncoef;
        self.bounds_tightened += mk.order.len() as i32;
        self.rows_removed += nrem;
        if !rows_out.is_empty() {
            self.activity(Some(&rows_out));
        }
        0
    }

    /// Zero the scatter of row `r`'s live columns; W += (nodes·3)·μ.
    fn clear_scatter(&mut self, r: usize, sc: &mut [f64]) {
        let mut walked = 0;
        let mut q = self.row_head[r];
        while q != NIL {
            let node = self.nd[q as usize];
            q = node.nr;
            walked += 1;
            if node.col >= 0 {
                sc[node.col as usize] = 0.0;
            }
        }
        self.work += (walked as f64 * 3.0) * self.work_mult;
    }
}
