//! `56c80e` doubleton equation aggregation (docs/m2/m2_red_inner.md §7): `=` rows with two entries (`566cc5`) or
//! whose binary spans the activity range (`569830`) define x_s = m·x_parent + o; the substitutions are chained
//! (path compression) and applied at the end by moving the substituted columns' nodes to their root column, then
//! duplicate columns in the touched rows are merged. No node is created and +0xd8 / free / pending stay as they
//! are (except through `36e6b6(st, 2)`).
use crate::helpers::{maxsd, minsd};
use crate::state::{guarded_add, State, INF, NIL};
use crate::Params;

/// Work buffers of one call: the substitution forest and its bookkeeping.
struct Agg {
    /// parent column + 1 (0 = root), multiplier and offset to the parent (st+0x398 / +0x3a0)
    parent: Vec<i32>,
    mult: Vec<f64>,
    off: Vec<f64>,
    tmp: Vec<usize>,
    /// substituted columns in discovery order (`list`, `nsub`)
    list: Vec<usize>,
    /// `local_f8`: substitutions made (569830 compares it before and after a row)
    nsub2: i32,
    /// `local_a8`: entries killed since the last GC
    dead: i64,
    /// `*status`: 3 = infeasible
    status: i32,
}

/// Tolerances of the call.
struct T {
    ft: f64,
    ift: f64,
    am: f64,
}

impl State {
    /// `56c80e(st, env, 0, &status)`. Returns 0 or a `3de18e` status; `status` = 3 when infeasible (`bad_row` /
    /// `bad_col`).
    pub fn doubleton_agg(&mut self, p: &Params, status: &mut i32) -> i32 {
        let t = T { ft: p.feastol, ift: self.intfeastol, am: p.agg_mult_limit };
        let n = self.n;
        let rows = self.rows_list();
        let mut g = Agg { parent: vec![0; n], mult: vec![0.0; n], off: vec![0.0; n], tmp: Vec::new(), list: Vec::new(),
                          nsub2: 0, dead: 0, status: 0 };
        for (idx, &ii) in rows.iter().enumerate() {
            if idx % 10000 == 0 {
                self.budget_check(); // `3ba5ac`, `3ba9de`
            }
            let r = ii as usize;
            let k = self.row_cnt[r];
            if k > 1 && self.sense[r] == b'=' {
                let ret = if k == 2 {
                    self.agg_pair(&mut g, r, &t)
                } else if self.inf_min[r] < 1 && self.inf_max[r] < 1
                    && (self.act_min[r] + self.act_max[r]).abs() <= 9.999999999999999e-05
                {
                    self.agg_binary(&mut g, r, &t)
                } else {
                    0
                };
                if g.status != 0 || ret != 0 {
                    *status = g.status;
                    return ret;
                }
            }
        }
        self.work += (rows.len() as f64 * 10.0) * self.work_mult;
        if !g.list.is_empty() {
            self.agg_apply(&mut g);
        }
        if 1_000_000 < g.dead {
            self.gc(2);
        }
        *status = g.status;
        0
    }

    /// Root of column `j` in the forest with the composed multiplier and offset (x_j = m·x_root + o), compressing
    /// the path. `phase2`: the work term is (k·4)·μ instead of (4·μ)·k.
    fn chain(&mut self, g: &mut Agg, j: usize, phase2: bool) -> (usize, f64, f64) {
        let mu = self.work_mult;
        g.tmp.clear();
        let mut t = j;
        loop {
            g.tmp.push(t);
            t = (g.parent[t] - 1) as usize;
            if g.parent[t] == 0 {
                break;
            }
        }
        let k = g.tmp.len();
        self.work += (k as f64 + k as f64) * mu;
        let last = g.tmp[k - 1];
        let (mut m, mut o) = (g.mult[last], g.off[last]);
        for q in (0..k - 1).rev() {
            let i = g.tmp[q];
            m *= g.mult[i];
            o = o * g.mult[i] + g.off[i];
            g.mult[i] = m;
            g.parent[i] = t as i32 + 1;
            g.off[i] = o;
        }
        let it = (k - 1) as f64;
        self.work += if phase2 { (it * 4.0) * mu } else { (4.0 * mu) * it };
        (t, m, o)
    }

    /// Kill row `r` after its substitution: nodes die, the row goes (-1), `36e6b6(st, 2)` once more entries died
    /// than +0xd8 counts.
    fn agg_kill_row(&mut self, g: &mut Agg, r: usize, factor: f64) {
        self.kill_row_nodes_agg(r);
        self.work += (self.row_cnt[r] as f64 * factor) * self.work_mult;
        g.dead += self.row_cnt[r] as i64;
        if self.nnz < g.dead {
            self.gc(2);
            g.dead = 0;
        }
        self.row_cnt[r] = -1;
        self.rows_removed += 1;
    }

    fn kill_row_nodes_agg(&mut self, r: usize) {
        let mut q = self.row_head[r];
        while q != NIL {
            let at = q as usize;
            q = self.nd[at].nr;
            let c = self.nd[at].col;
            if c >= 0 {
                self.col_cnt[c as usize] -= 1;
                self.nd[at].col = -1;
            }
        }
    }

    /// Columns whose node in row `r` is big (> 1e8 in magnitude) cannot be substituted; W += (col_cnt·3)·μ.
    fn has_big(&mut self, c: usize) -> bool {
        let mut q = self.col_head[c];
        let mut big = false;
        while q != NIL {
            let nd = self.nd[q as usize];
            if nd.col >= 0 && 1e8 < nd.val.abs() {
                big = true;
                break;
            }
            q = nd.nc;
        }
        self.work += (self.col_cnt[c] as f64 * 3.0) * self.work_mult;
        big
    }

    /// `566cc5`: a two-entry `=` row a1·x1 + a2·x2 = rhs. Picks the column to substitute (continuous before integer,
    /// among integers the smaller |a| when the ratio and rhs are integral, else the one with fewer entries / B kept),
    /// tightens the kept column's bounds from the substituted one's, and records x_s = −a_t/a_s·x_t + rr/a_s
    /// unless the kept column became fixed. The row goes. Returns 0 or a `3de18e` status.
    fn agg_pair(&mut self, g: &mut Agg, r: usize, t: &T) -> i32 {
        let mu = self.work_mult;
        let (am, inv) = (t.am, 1.0 / t.am);
        let rhs = self.rhs[r];
        if rhs != 0.0 && rhs.abs() < 1e-6 {
            return 0;
        }
        let (mut c1, mut c2, mut a1, mut a2) = (-1i64, -1i64, 0.0, 0.0);
        let mut vis = 0;
        let mut q = self.row_head[r];
        while q != NIL {
            let nd = self.nd[q as usize];
            q = nd.nr;
            if nd.col >= 0 {
                let c = nd.col as usize;
                if self.ub[c] - self.lb[c] < 1e-10 {
                    self.work += ((vis as f64 + 10.0) + vis as f64) * mu;
                    return 0;
                }
                if c1 == -1 {
                    a1 = nd.val;
                    c1 = c as i64;
                } else {
                    a2 = nd.val;
                    c2 = c as i64;
                }
            }
            vis += 1;
        }
        self.work += ((10.0 + vis as f64) + vis as f64) * mu;
        let (mut c1, mut c2) = (c1 as usize, c2 as usize);
        let (mut o1, mut o2) = (0.0, 0.0);
        if g.parent[c1] != 0 {
            let (root, m, o) = self.chain(g, c1, false);
            c1 = root;
            o1 = o * a1;
            a1 *= m;
        }
        if g.parent[c2] != 0 {
            let (root, m, o) = self.chain(g, c2, false);
            c2 = root;
            o2 = o * a2;
            a2 *= m;
        }
        let (f1, f2) = (self.col_flag[c1], self.col_flag[c2]); // mode 0: every flag bit forbids
        if f1 != 0 && f2 != 0 {
            return 0;
        }
        let (aa1, aa2) = (a1.abs(), a2.abs());
        if aa1 / aa2 < inv || am < aa1 / aa2 || aa1 < inv || am < aa1 || aa2 < inv || am < aa2 {
            return 0;
        }
        let rr = (rhs - o1) - o2;
        if rr != 0.0 && (rr.abs() < 1e-6 || 1e6 < rr.abs()) {
            return 0;
        }
        let (t1, t2) = (self.vtype[c1], self.vtype[c2]);
        let mut keep = c2;
        if (t1 == b'C') == (t2 == b'C') {
            let mut chosen = false;
            if t1 != b'C' {
                let lo = if aa2 <= aa1 { aa2 } else { aa1 };
                let hi = if aa1 <= aa2 { aa2 } else { aa1 };
                if 1e-10 < hi / lo - (hi / lo + 1e-10).floor() {
                    return 0;
                }
                if 1e-10 < rr / lo - (rr / lo + 1e-10).floor() {
                    return 0;
                }
                if aa1 != aa2 {
                    if aa2 < aa1 {
                        keep = c1;
                    }
                    chosen = true;
                }
            }
            if !chosen {
                if f2 == f1 {
                    if (f1 == 0) == (f2 == 0) {
                        if t1 == t2 {
                            if self.col_cnt[c2] <= self.col_cnt[c1] {
                                keep = c1;
                            }
                        } else if t1 == b'B' {
                            keep = c1;
                        }
                    } else if f2 != 0 {
                        keep = c1;
                    }
                } else if f2 != 0 {
                    keep = c1;
                }
            }
        } else if t1 != b'C' {
            keep = c1;
        }
        let (s, as_, tk, at) = if keep == c1 { (c2, a2, c1, a1) } else { (c1, a1, c2, a2) };
        if self.col_flag[s] != 0 {
            return 0;
        }
        let ratio = (at / as_).abs();
        if 1e-10 < (ratio - 1.0).abs() {
            if self.ub[s] < 1e30 && 1e6 < self.ub[s].abs() {
                return 0;
            }
            if -1e30 < self.lb[s] && 1e6 < self.lb[s].abs() {
                return 0;
            }
        }
        if self.mult_lo[s] * ratio < inv || am < self.mult_hi[s] * ratio || 1e6 < ratio || ratio < 1e-6 {
            return 0;
        }
        if self.has_big(s) {
            return 0;
        }
        let (lt0, ut0) = (self.lb[tk], self.ub[tk]);
        let (mut lbt, mut ubt) = (lt0, ut0);
        let mut done = false;
        if tk == s {
            // both entries resolve to one column: (1 + a_t/a_s)·x = rr/a_s
            let d35 = rr / as_;
            let d45 = 1.0 - (-at / as_);
            if 1e-13 < d35.abs() {
                if d45 != 0.0 && ((d35 / d45).abs() < 1e-6 || 1e6 < (d35 / d45).abs()) {
                    return 0;
                }
                if d45 == 0.0 {
                    if d35.abs() < 1e-6 {
                        return 0;
                    }
                    self.bad_col = tk as i32;
                    g.status = 3;
                    return 0;
                }
                let mut v = d35 / d45;
                if self.vtype[tk] != b'C' {
                    v = (v + t.ift).floor();
                }
                if self.ub[tk] + t.ft < v || v < self.lb[tk] - t.ft {
                    self.bad_col = tk as i32;
                    g.status = 3;
                    return 0;
                }
                // shortcut: not seen on the MIPLIB-38 defaults (decoded only)
                let ret = self.fix_col(tk, v, false);
                if ret != 0 {
                    return ret;
                }
                done = true;
                (lbt, ubt) = (self.lb[tk], self.ub[tk]);
            } else if d45.abs() <= t.ft {
                done = true;
            } else {
                if t.ft < self.lb[tk] || self.ub[tk] < -t.ft {
                    self.bad_col = tk as i32;
                    g.status = 3;
                    return 0;
                }
                if self.lb[tk] != 0.0 || self.ub[tk] != 0.0 {
                    let ret = self.fix_col(tk, 0.0, false);
                    if ret != 0 {
                        return ret;
                    }
                }
                done = true;
                (lbt, ubt) = (self.lb[tk], self.ub[tk]);
            }
        }
        // bounds of the kept column implied by the substituted one's
        let f = -as_ / at;
        let (mut lo, mut hi) = (lbt, ubt);
        if -1e30 < self.lb[s] {
            let v = rr / at + self.lb[s] * f;
            if 0.0 <= f { lo = v } else { hi = v }
        }
        if self.ub[s] < 1e30 {
            let v = rr / at + self.ub[s] * f;
            if 0.0 <= f { hi = v } else { lo = v }
        }
        if self.vtype[tk] != b'C' {
            lo = (lo - t.ift).ceil();
            hi = (hi + t.ift).floor();
        }
        let ff = minsd(f.abs(), 1.0);
        let b16 = (lbt + 1e-10) + ff * 1e-6 < lo;
        if b16 {
            self.bounds_tightened += 1;
            self.lb[tk] = lo;
            ubt = self.ub[tk];
            lbt = lo;
        }
        let b17 = hi < (ubt - 1e-10) - ff * 1e-6;
        if b17 {
            self.bounds_tightened += 1;
            self.ub[tk] = hi;
            lbt = self.lb[tk];
            ubt = hi;
        }
        if lbt - ubt <= t.ift {
            if 0.0 < lbt - ubt {
                if ubt == hi { self.ub[tk] = lbt } else { self.lb[tk] = ubt }
            }
        } else {
            self.bad_row = r as i32;
            g.status = 3;
        }
        if b16 || b17 {
            let (l, u) = (self.lb[tk], self.ub[tk]);
            self.bound_change(tk, lt0, ut0, l, u);
        }
        if g.status == 3 {
            return 0;
        }
        if !(1e-10 < self.ub[tk] - self.lb[tk]) {
            // the kept column is fixed: the substituted one is determined
            let mut v = self.lb[tk] * (-at / as_) + rr / as_;
            let (ls, us) = (self.lb[s], self.ub[s]);
            if self.vtype[s] != b'C' {
                if 1e-6 < v - (v + 1e-6).floor() {
                    self.bad_col = s as i32;
                    g.status = 3;
                    return 0;
                }
                v = (v + 0.5).floor();
            }
            if t.ft < v - us || t.ft < ls - v {
                return 0;
            }
            let hi2 = maxsd(v, ls);
            let lo2 = minsd(v, us);
            let mut ch = false;
            if lo2 > ls + 1e-6 {
                self.bounds_tightened += 1;
                self.lb[s] = lo2;
                ch = true;
                if hi2 < self.ub[s] - 1e-6 {
                    self.bounds_tightened += 1;
                    self.ub[s] = hi2;
                }
            } else if hi2 < us - 1e-6 {
                self.bounds_tightened += 1;
                self.ub[s] = hi2;
                ch = true;
            }
            if ch {
                let (l, u) = (self.lb[s], self.ub[s]);
                self.bound_change(s, ls, us, l, u);
            }
            done = true;
        }
        if self.vtype[tk] != b'B' && self.vtype[s] == b'B' {
            return 0;
        }
        if !done {
            let o = rr / as_;
            let (m1000, p1000) = (t.ft * -1000.0, t.ft * 1000.0);
            for bt in [self.lb[tk], self.ub[tk]] {
                let v = o - (at * bt) / as_;
                let (ls, us) = (self.lb[s], self.ub[s]);
                if v - ls <= m1000 && -1e30 < ls {
                    return 0;
                }
                if p1000 <= v - us && us < 1e30 {
                    return 0;
                }
            }
            g.parent[s] = tk as i32 + 1;
            g.mult[s] = -at / as_;
            g.off[s] = o;
            let m = g.mult[s].abs();
            let x = self.mult_lo[s] * m;
            if x <= self.mult_lo[tk] {
                self.mult_lo[tk] = x;
            }
            let x = self.mult_hi[s] * m;
            if self.mult_hi[tk] <= x {
                self.mult_hi[tk] = x;
            }
            g.list.push(s);
            g.nsub2 += 1;
        }
        self.agg_kill_row(g, r, 2.0);
        0
    }

    /// `569830`: an `=` row (no infinite activity terms, rhs mid-range) where a binary b spans the whole activity
    /// range: every other column sits at one bound when b = 0 and at the other when b = 1, so x_c = cc + w·x_b.
    /// The row goes when all of them were substituted. Returns 0 or a `3de18e` status.
    fn agg_binary(&mut self, g: &mut Agg, r: usize, t: &T) -> i32 {
        let mu = self.work_mult;
        let (mut b, mut ab) = (-1i64, 0.0);
        let (mut mx, mut mn) = (-self.rhs[r], -self.rhs[r]);
        let mut q = self.row_head[r];
        while q != NIL {
            let nd = self.nd[q as usize];
            q = nd.nr;
            if nd.col < 0 {
                continue;
            }
            let c = nd.col as usize;
            let (a, u, l) = (nd.val, self.ub[c], self.lb[c]);
            if 0.0 < a {
                mx += u * a;
                mn += a * l;
            } else {
                mn += u * a;
                mx += a * l;
            }
            if self.vtype[c] == b'B' && l == 0.0 && u == 1.0 && (a.abs() + self.act_min[r]).abs() < 9.999999999999999e-05 {
                ab = a;
                b = c as i64;
            }
        }
        self.work += (self.row_cnt[r] as f64 * 7.0) * mu;
        if b < 0 || 1e-6 < (mx + mn).abs() || 1e-6 < (ab.abs() + mn).abs() {
            return 0;
        }
        let b0 = b as usize;
        let mut b = b0;
        if g.parent[b] != 0 {
            let (root, m, _) = self.chain(g, b, false);
            b = root;
            ab *= m;
        }
        let (f0, cnt0) = (g.nsub2, self.row_cnt[r]);
        let mut ret = 0;
        if self.row_head[r] != NIL {
            let px = mx - ab.abs();
            let nx = -(mn + ab.abs());
            let inv = 1.0 / t.am;
            let mut q = self.row_head[r];
            while q != NIL {
                let nd = self.nd[q as usize];
                q = nd.nr;
                if nd.col < 0 || nd.col as usize == b0 {
                    continue;
                }
                let (mut c, mut ac) = (nd.col as usize, nd.val);
                if g.parent[c] != 0 {
                    let (root, m, _) = self.chain(g, c, false);
                    c = root;
                    ac *= m;
                }
                if self.col_flag[c] != 0 {
                    continue;
                }
                let w = self.ub[c] - self.lb[c];
                if w < 1e-8 {
                    continue;
                }
                let aa = ac.abs();
                if self.vtype[c] == b'C' {
                    if !(nx / aa <= 1e-10 && px / aa <= 1e-10) {
                        continue;
                    }
                } else if 0.5 < nx / aa || 0.5 < px / aa {
                    continue;
                }
                if 1e6 < w || t.am < self.mult_hi[c] * w || self.mult_lo[c] * w < inv {
                    continue;
                }
                if self.has_big(c) {
                    continue;
                }
                let (mut w, mut cc) = (self.ub[c] - self.lb[c], self.lb[c]);
                if 0.0 <= ac * ab {
                    w = -w;
                    cc = self.ub[c];
                }
                if c == b {
                    // shortcut: same-root branch decoded from the note only (never reached on the 38 models)
                    let d33 = 1.0 - w;
                    if 1e-13 < cc.abs() {
                        if d33 == 0.0 {
                            if 1e-6 <= cc.abs() {
                                self.bad_col = b as i32;
                                g.status = 3;
                                return 0;
                            }
                            continue;
                        }
                        let x = cc / d33;
                        if x.abs() < 1e-6 || 1e6 < x.abs() {
                            continue;
                        }
                        let v = if self.vtype[b] != b'C' { (x + t.ift).floor() } else { x };
                        if self.ub[b] + t.ft < v || v < self.lb[b] - t.ft {
                            self.bad_col = b as i32;
                            g.status = 3;
                            return 0;
                        }
                        ret = self.fix_col(b, v, false);
                        if ret != 0 {
                            return ret;
                        }
                    } else if d33.abs() > t.ft {
                        if t.ft < self.lb[b] || self.ub[b] < -t.ft {
                            self.bad_col = b as i32;
                            g.status = 3;
                            return 0;
                        }
                        if self.lb[b] != 0.0 || self.ub[b] != 0.0 {
                            ret = self.fix_col(b, 0.0, false);
                            if ret != 0 {
                                return ret;
                            }
                        }
                    }
                    continue;
                }
                g.parent[c] = b as i32 + 1;
                g.mult[c] = w;
                g.off[c] = cc;
                g.list.push(c);
                let x = self.mult_lo[c] * w.abs();
                self.mult_lo[b] = minsd(self.mult_lo[b], x);
                g.nsub2 += 1;
                let x = self.mult_hi[c] * w.abs();
                self.mult_hi[b] = maxsd(self.mult_hi[b], x);
            }
            self.work += (self.row_cnt[r] as f64 * 8.0) * mu;
        }
        if g.nsub2 - f0 == cnt0 - 1 {
            self.agg_kill_row(g, r, 3.0);
        }
        ret
    }

    /// Plain (unguarded) add or remove of coefficient `a` of column `c` in row `r`'s activities, lower bound first.
    fn agg_contrib(&mut self, r: usize, a: f64, c: usize, add: bool) {
        let (l, u) = (self.lb[c], self.ub[c]);
        let d = if add { 1 } else { -1 };
        let upd = |x: &mut f64, t: f64| *x = if add { t + *x } else { *x - t };
        if a > 0.0 {
            if -INF < l { upd(&mut self.act_min[r], l * a) } else { self.inf_min[r] += d }
            if u < INF { upd(&mut self.act_max[r], a * u) } else { self.inf_max[r] += d }
        } else if a < 0.0 {
            if -INF < l { upd(&mut self.act_max[r], l * a) } else { self.inf_max[r] += d }
            if u < INF { upd(&mut self.act_min[r], a * u) } else { self.inf_min[r] += d }
        }
    }

    /// Bounds of j equal root's scaled by m (then the activities need no update).
    fn agg_equiv(&self, j: usize, k: usize, m: f64) -> bool {
        let (lb, ub) = (&self.lb, &self.ub);
        if 0.0 < m {
            (lb[j] == lb[k] * m || (lb[j] == -1e30 && lb[k] == -1e30)) && (ub[j] == ub[k] * m || (ub[j] == 1e30 && ub[k] == 1e30))
        } else if m < 0.0 {
            (lb[j] == ub[k] * m || (lb[j] == -1e30 && ub[k] == 1e30)) && (ub[j] == lb[k] * m || (ub[j] == 1e30 && lb[k] == -1e30))
        } else {
            false
        }
    }

    /// `3c3abf`: a moved node whose new coefficient is tiny (|new| <= 1e-13) dies, taking the old coefficient's
    /// contribution (old column's bounds, guarded) out of the row.
    pub(crate) fn agg_tiny(&mut self, r: usize, at: usize, j_old: usize, new: f64, old: f64) {
        if 1e-13 < new.abs() || self.nd[at].col < 0 {
            return;
        }
        let (u, l) = (self.ub[j_old], self.lb[j_old]);
        if 0.0 < old {
            if u >= INF { self.inf_max[r] -= 1 } else { self.act_max[r] = guarded_add(self.act_max[r], -(u * old), true) }
            if -INF >= l { self.inf_min[r] -= 1 } else { self.act_min[r] = guarded_add(self.act_min[r], -(old * l), false) }
        } else {
            if u >= INF { self.inf_min[r] -= 1 } else { self.act_min[r] = guarded_add(self.act_min[r], -(u * old), false) }
            if -INF >= l { self.inf_max[r] -= 1 } else { self.act_max[r] = guarded_add(self.act_max[r], -(l * old), true) }
        }
        self.row_cnt[r] -= 1;
        let c = self.nd[at].col as usize;
        self.col_cnt[c] -= 1;
        self.nd[at].col = -1;
    }

    /// Phases 2–4: apply the substitutions in discovery order (nodes move to the front of the root's column list,
    /// activities and rhs follow), record them (`39d9e1`), merge duplicate columns in the touched rows, mark.
    fn agg_apply(&mut self, g: &mut Agg) {
        let mu = self.work_mult;
        let mut roots = Vec::new();
        let mut touched = Vec::new();
        let mut rmark = vec![false; self.m];
        let list = std::mem::take(&mut g.list);
        for &j in &list {
            let (root, m, o) = if g.parent[j] != 0 { self.chain(g, j, true) } else { (j, 1.0, 0.0) };
            if !(1e-8 <= m.abs()) {
                continue;
            }
            if root != j {
                let cnt0 = self.col_cnt[j];
                let mut cur = self.col_head[j];
                self.col_head[j] = NIL;
                let mut tail = NIL;
                while cur != NIL {
                    let at = cur as usize;
                    let nxt = self.nd[at].nc;
                    if self.nd[at].col < 0 {
                        if tail == NIL { self.col_head[j] = cur } else { self.nd[tail as usize].nc = cur }
                        tail = cur;
                        cur = nxt;
                        continue;
                    }
                    let (r, a) = (self.nd[at].row as usize, self.nd[at].val);
                    let eq = self.agg_equiv(j, root, m);
                    if !eq {
                        self.agg_contrib(r, a, j, false);
                    }
                    self.nd[at].col = root as i32;
                    self.nd[at].val = m * a;
                    self.agg_tiny(r, at, j, m * a, a);
                    if !eq && self.nd[at].col >= 0 {
                        self.agg_contrib(r, m * a, root, true);
                    }
                    if o != 0.0 {
                        let d = a * o;
                        self.act_max[r] += d;
                        self.act_min[r] += d;
                        self.rhs[r] -= d;
                    }
                    self.nd[at].nc = self.col_head[root];
                    self.col_head[root] = cur;
                    self.col_cnt[root] += 1;
                    if !rmark[r] {
                        rmark[r] = true;
                        touched.push(r);
                    }
                    self.col_cnt[j] -= 1;
                    cur = nxt;
                }
                self.work += (cnt0 as f64 * 16.0) * mu;
                if tail != NIL {
                    self.nd[tail as usize].nc = NIL;
                }
                self.col_cnt[j] = -3;
                self.cols_removed += 1;
                self.obj[root] = self.obj[j] * m + self.obj[root];
                self.objcon = self.obj[j] * o + self.objcon;
                self.obj[j] = 0.0;
                self.push_agg(j, root, m, o);
            }
            roots.push(root);
        }
        self.work += list.len() as f64 * mu;
        // merge duplicate columns in the touched rows (the earlier node keeps the sum), drop exact zeros
        let mut pos = vec![NIL; self.n];
        for &r in &touched {
            let h = self.row_head[r];
            if h != NIL && self.nd[h as usize].row >= 0 {
                let (mut vis, mut dups) = (0, 0);
                let mut q = h;
                while q != NIL {
                    let at = q as usize;
                    q = self.nd[at].nr;
                    vis += 1;
                    let c = self.nd[at].col;
                    if c < 0 {
                        continue;
                    }
                    let c = c as usize;
                    if pos[c] == NIL {
                        pos[c] = at as u32;
                        continue;
                    }
                    let pn = pos[c] as usize;
                    let (prow, pval, pcol) = (self.nd[pn].row as usize, self.nd[pn].val, self.nd[pn].col as usize);
                    self.agg_contrib(prow, pval, pcol, false);
                    self.agg_contrib(r, self.nd[at].val, c, false);
                    let s = pval + self.nd[at].val;
                    if s.abs() <= 1e-13 {
                        self.nd[pn].val = 0.0;
                    } else {
                        self.nd[pn].val = s;
                        self.agg_contrib(prow, s, pcol, true);
                    }
                    self.col_cnt[c] -= 1;
                    g.dead += 1;
                    dups += 1;
                    let nr = self.nd[at].row as usize;
                    self.row_cnt[nr] -= 1;
                    self.nd[at].col = -1;
                }
                self.work += (dups as f64 * 11.0 + vis as f64 * 3.0) * mu;
                let (mut vis, mut z) = (0, 0);
                let mut q = h;
                while q != NIL {
                    let at = q as usize;
                    q = self.nd[at].nr;
                    vis += 1;
                    let c = self.nd[at].col;
                    if c < 0 {
                        continue;
                    }
                    pos[c as usize] = NIL;
                    if self.nd[at].val == 0.0 {
                        self.col_cnt[c as usize] -= 1;
                        g.dead += 1;
                        z += 1;
                        let nr = self.nd[at].row as usize;
                        self.row_cnt[nr] -= 1;
                        self.nd[at].col = -1;
                    }
                }
                self.work += ((vis as f64 + vis as f64) + z as f64 * 5.0) * mu;
            }
        }
        self.work += (touched.len() as f64 + touched.len() as f64) * mu;
        for &r in &touched {
            self.tracker.mark_row(r as i32); // `5382b5`
        }
        for &c in &roots {
            if 0 < self.col_cnt[c] {
                self.tracker.mark_col(c as i32);
            }
        }
        if !roots.is_empty() {
            self.work += (roots.len() as f64 + roots.len() as f64) * mu;
        }
    }
}
