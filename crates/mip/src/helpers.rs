//! Helpers shared by MIP presolve reductions (docs/m2_red_inner.md §1): tracker marks over the lists, bound changes
//! with incremental activities, column fixing, postsolve record pushes, implied column bounds. Other reductions
//! (clique, probing, ...) call these instead of copying them.
use crate::post::PostRec;
use crate::state::{guarded_add, State, INF, NIL};

/// `minsd(a, b)` / `maxsd(a, b)`: the first operand unless the second is strictly smaller / larger (NaN: second).
pub(crate) fn minsd(a: f64, b: f64) -> f64 {
    if a < b { a } else { b }
}
pub(crate) fn maxsd(a: f64, b: f64) -> f64 {
    if a > b { a } else { b }
}

impl State {
    /// `53802c(tr, st, j)` (linked-list branch): mark the rows of column `j`'s live nodes. W += (μ·5)·nodes walked.
    pub fn mark_rows_of_col(&mut self, j: usize) {
        let mut p = self.col_head[j];
        let mut walked = 0;
        while p != NIL {
            let q = self.nd[p as usize];
            if q.col >= 0 {
                self.tracker.mark_row(q.row);
            }
            walked += 1;
            p = q.nc;
        }
        self.work += self.work_mult * 5.0 * walked as f64;
    }

    /// `537da8(tr, st, i)` (linked-list branch): mark the columns of row `i`'s live nodes. W += (μ·4)·nodes walked.
    pub fn mark_cols_of_row(&mut self, i: usize) {
        let mut p = self.row_head[i];
        let mut walked = 0;
        while p != NIL {
            let q = self.nd[p as usize];
            if q.col >= 0 {
                self.tracker.mark_col(q.col);
            }
            walked += 1;
            p = q.nr;
        }
        self.work += self.work_mult * 4.0 * walked as f64;
    }

    /// One bound's contribution `a·b` to row `r`'s activity: added (`add`) or removed, guarded, or an infinite-count
    /// change when the bound is infinite. `upper`: b is the column's upper bound.
    pub(crate) fn contrib(&mut self, r: usize, a: f64, b: f64, upper: bool, add: bool) {
        let finite = if upper { b < INF } else { -INF < b };
        let max_side = if a <= 0.0 { !upper } else { upper };
        let d = if add { 1 } else { -1 };
        if !finite {
            if max_side { self.inf_max[r] += d } else { self.inf_min[r] += d }
            return;
        }
        let t = if add { a * b } else { -(b * a) };
        if max_side {
            self.act_max[r] = guarded_add(self.act_max[r], t, true);
        } else {
            self.act_min[r] = guarded_add(self.act_min[r], t, false);
        }
    }

    /// `3dd6f1(ol, ou, nl, nu, INF, st, j)`: column `j`'s bounds go from [ol, ou] to [nl, nu]; update its rows'
    /// activities (the caller stores the bounds and marks the column). Per live node: ub out, lb out, ub in, lb in
    /// (only the changed side). W += (μ·6)·nodes walked when anything changed.
    pub fn bound_change(&mut self, j: usize, ol: f64, ou: f64, nl: f64, nu: f64) {
        let (lo_same, up_same) = (nl == ol, nu == ou);
        if lo_same && up_same {
            return;
        }
        let mut p = self.col_head[j];
        let mut walked = 0;
        while p != NIL {
            let q = self.nd[p as usize];
            p = q.nc;
            walked += 1;
            if q.col < 0 {
                continue;
            }
            let (r, a) = (q.row as usize, q.val);
            if !up_same {
                self.contrib(r, a, ou, true, false);
            }
            if !lo_same {
                self.contrib(r, a, ol, false, false);
            }
            if !up_same {
                self.contrib(r, a, nu, true, true);
            }
            if !lo_same {
                self.contrib(r, a, nl, false, true);
            }
        }
        self.work += self.work_mult * 6.0 * walked as f64;
    }

    /// `39ab33(v, env, st, j)`, MIP branch: type 1 record, columns removed +1 (no work).
    pub fn push_fix(&mut self, j: usize, v: f64) {
        self.cols_removed += 1;
        self.post.push(PostRec::Fix { j: j as i32, v });
    }

    /// `39b9e0(c0, a, env, st, mode, j, r, rowhead, cnt)`, MIP branch: x_j = c0 + Σ (−a_k/a)·x_k over row `r`'s live
    /// nodes (column j's own node is already dead), type 2 record; rows and columns removed +1; the multiplier
    /// growth bounds of the other columns follow j's. W += (5·μ)·nodes walked.
    pub fn push_subst_row(&mut self, j: usize, r: usize, c0: f64, a: f64) {
        self.cols_removed += 1;
        self.rows_removed += 1;
        let (mut ind, mut val) = (Vec::new(), Vec::new());
        let mut p = self.row_head[r];
        let mut walked = 0;
        while p != NIL {
            let q = self.nd[p as usize];
            p = q.nr;
            walked += 1;
            if q.col < 0 {
                continue;
            }
            let k = q.col as usize;
            ind.push(q.col);
            val.push((-q.val) / a);
            if k != j {
                let t = (q.val / a).abs();
                self.mult_lo[k] = minsd(self.mult_lo[k], self.mult_lo[j] * t);
                self.mult_hi[k] = maxsd(self.mult_hi[k], self.mult_hi[j] * t);
            }
        }
        self.work += (5.0 * self.work_mult) * walked as f64;
        self.post.push(PostRec::Subst { j: j as i32, c0, ind, val });
    }

    /// `39d9e1(m, o, env, st, j, root)`: x_j = o + m·x_root (type 2 record with one term); for m != 1 the root's
    /// multiplier growth bounds take j's scaled by |m|. No work.
    pub fn push_agg(&mut self, j: usize, root: usize, m: f64, o: f64) {
        if m != 1.0 {
            self.mult_lo[root] = minsd(self.mult_lo[root], self.mult_lo[j] * m.abs());
            self.mult_hi[root] = maxsd(self.mult_hi[root], m.abs() * self.mult_hi[j]);
        }
        self.post.push(PostRec::Subst { j: j as i32, c0: o, ind: vec![root as i32], val: vec![m] });
    }

    /// `3a061e(d1, d2, env, st, i)`: type 9 record (no counters, no work).
    pub fn push_t9(&mut self, i: i32, d1: f64, d2: f64) {
        self.post.push(PostRec::T9 { i, d1, d2 });
    }

    /// `3a07b5(d1, d2, d3, env, st, j, k)`: type 10 record (no counters, no work).
    pub fn push_t10(&mut self, j: i32, k: i32, d1: f64, d2: f64, d3: f64) {
        self.post.push(PostRec::T10 { j, k, d1, d2, d3 });
    }

    /// `3de18e(v, INF, env, st, j, rec)`: fix column `j` at `v`. With `rec` (and no column flags other than 0x08/0x80)
    /// a type 1 record and the column is removed (-2); without, it stays as an empty column fixed at `v` (0, marked).
    /// Rows: rhs −= v·a, live count −1, activities lose the old bounds' contribution and gain v·a; the nodes die
    /// in place. W: `53802c` + (col_cnt·7)·μ. Returns 0.
    /// shortcut: the Q objective (+0x250/+0x260) and PWL objective (flag 0x80) blocks are not ported (linear models).
    pub fn fix_col(&mut self, j: usize, v: f64, rec: bool) -> i32 {
        let (ol, ou) = (self.lb[j], self.ub[j]);
        let rec = rec && self.col_flag[j] & 0xffff_ff77 == 0;
        if rec {
            self.push_fix(j, v);
        }
        self.objcon = self.obj[j] * v + self.objcon;
        self.obj[j] = 0.0;
        self.col_flag[j] &= !0x08;
        let cnt = self.col_cnt[j];
        if cnt >= 1 {
            self.mark_rows_of_col(j);
            let mut p = self.col_head[j];
            while p != NIL {
                let at = p as usize;
                let q = self.nd[at];
                p = q.nc;
                if q.col < 0 {
                    continue;
                }
                let (r, a) = (q.row as usize, q.val);
                let t = v * a;
                self.nd[at].col = -1;
                self.row_cnt[r] -= 1;
                self.rhs[r] -= t;
                self.contrib(r, a, ou, true, false);
                self.contrib(r, a, ol, false, false);
                self.act_max[r] = guarded_add(self.act_max[r], t, true);
                self.act_min[r] = guarded_add(self.act_min[r], t, false);
            }
            self.work += (cnt as f64 * 7.0) * self.work_mult;
            if !rec {
                if cnt > 0 {
                    self.col_cnt[j] = 0;
                    self.tracker.mark_col(j as i32);
                }
            } else {
                self.col_cnt[j] = -2;
            }
        } else if rec {
            self.col_cnt[j] = -2;
        }
        self.lb[j] = v;
        self.ub[j] = v;
        0
    }

    /// `3df916(ft, env, st, j)` (also inline at the head of `3e39a6`): a column whose bounds (nearly) meet is fixed —
    /// at its cost-preferred bound when the range matters for the cost, else at the midpoint or 0 — unless some
    /// coefficient times the range exceeds `ft`. Rows lose it (rhs, live count; activities are left as they are),
    /// type 1 record when the column has no flags but 0x08/0x80. Returns 3 (bad_col) when lb > ub + 2·ft, else 0.
    pub fn fix_near(&mut self, j: usize, ft: f64) -> i32 {
        let d = self.ub[j] - self.lb[j];
        if ft + ft < -d {
            self.bad_col = j as i32;
            return 3;
        }
        if 1e-13 < d {
            let (mut amax, mut walked) = (0.0f64, 0);
            let mut p = self.col_head[j];
            while p != NIL {
                let q = self.nd[p as usize];
                if q.col >= 0 {
                    amax = maxsd(amax, q.val.abs());
                }
                walked += 1;
                p = q.nc;
            }
            self.work += (walked as f64 * 3.0) * self.work_mult;
            if ft < amax * d {
                return 0;
            }
        }
        let c = self.obj[j];
        let v = if ft * 0.01 <= (d * c).abs() {
            if c <= 0.0 { self.ub[j] } else { self.lb[j] }
        } else if 0.0 < self.lb[j] || self.ub[j] < 0.0 {
            (self.ub[j] + self.lb[j]) * 0.5
        } else {
            0.0
        };
        // mask 0x88 (0x888 when state+0x248 is set and lb < 0.5; +0x248 is 0 here)
        if self.col_flag[j] & !0x88 == 0 {
            // shortcut: flag 0x80 (PWL objective) block not ported
            self.push_fix(j, v);
            self.col_flag[j] = 0;
        }
        self.objcon = self.obj[j] * v + self.objcon;
        self.obj[j] = 0.0;
        self.mark_rows_of_col(j);
        let mut p = self.col_head[j];
        let mut walked = 0;
        while p != NIL {
            let q = self.nd[p as usize];
            if q.col >= 0 {
                let r = q.row as usize;
                self.rhs[r] -= v * q.val;
                self.row_cnt[r] -= 1;
                self.nd[p as usize].col = -1;
            }
            walked += 1;
            p = q.nc;
        }
        self.work += (walked as f64 * 6.0) * self.work_mult;
        self.col_cnt[j] = if self.col_flag[j] == 0 { -2 } else { 0 };
        0
    }

    /// Bounds on column `j` (coefficient `a`) implied by row `r`'s activities, folded into (lo, hi) the way
    /// `3dcc78` mode 1 does (`lo` raised, `hi` lowered; |a| <= 1e-9 gives nothing).
    fn row_implies(&self, r: usize, a: f64, l: f64, u: f64, lo: &mut f64, hi: &mut f64) {
        let up = |hi: &mut f64, c: f64| *hi = if c <= *hi { c } else { *hi };
        let dn = |lo: &mut f64, c: f64| *lo = if *lo <= c { c } else { *lo };
        let eq = self.sense[r] == b'=';
        if a > 1e-9 {
            if self.inf_min[r] == 0 {
                up(hi, l - self.act_min[r] / a);
            } else if self.inf_min[r] == 1 && l <= -INF {
                up(hi, -self.act_min[r] / a);
            }
            if eq {
                if self.inf_max[r] == 0 {
                    dn(lo, u - self.act_max[r] / a);
                } else if self.inf_max[r] == 1 && INF <= u {
                    dn(lo, -self.act_max[r] / a);
                }
            }
        } else if a < -1e-9 {
            if self.inf_min[r] == 0 {
                dn(lo, u - self.act_min[r] / a);
            } else if self.inf_min[r] == 1 && INF <= u {
                dn(lo, -self.act_min[r] / a);
            }
            if eq {
                if self.inf_max[r] == 0 {
                    up(hi, l - self.act_max[r] / a);
                } else if self.inf_max[r] == 1 && l <= -INF {
                    up(hi, -self.act_max[r] / a);
                }
            }
        }
    }

    /// `3dcc78(INF, a, st, j, mode, single, r, &lo, &hi)`: bounds of column `j` implied by its rows. Mode bit 1:
    /// row activities, over all rows of j or (`row` = Some((r, a))) one row; integer columns are rounded inward by
    /// IntFeasTol. Mode bit 2 (columns without flags other than 1/2): optimality, from the cost sign and the `<`
    /// rows' max activities (no inference through `=` rows). Returns (lo, hi) with hi >= lo.
    pub fn implied_col_bounds(&mut self, j: usize, mode: i32, row: Option<(usize, f64)>) -> (f64, f64) {
        let (l, u) = (self.lb[j], self.ub[j]);
        let int = self.vtype[j] != b'C';
        let (mut lo, mut hi) = (-INF, INF);
        if mode & 1 != 0 {
            match row {
                Some((r, a)) => {
                    self.row_implies(r, a, l, u, &mut lo, &mut hi);
                    self.work += self.work_mult * 3.0;
                }
                None => {
                    let mut p = self.col_head[j];
                    let mut walked = 0;
                    while p != NIL {
                        let q = self.nd[p as usize];
                        if q.col >= 0 {
                            self.row_implies(q.row as usize, q.val, l, u, &mut lo, &mut hi);
                        }
                        walked += 1;
                        p = q.nc;
                    }
                    self.work += self.work_mult * 7.0 * walked as f64;
                }
            }
            if int {
                lo = (lo - self.intfeastol).ceil();
                hi = (hi + self.intfeastol).floor();
            }
        }
        if mode & 2 != 0 && self.col_flag[j] & 0xffff_fffc == 0 {
            let c = self.obj[j];
            let mut u2 = if -1e-10 < c { l } else { INF };
            let mut l2 = if c < 1e-10 { u } else { -INF };
            let mut p = self.col_head[j];
            let mut walked = 0;
            while p != NIL {
                let q = self.nd[p as usize];
                if q.col >= 0 {
                    let r = q.row as usize;
                    if self.sense[r] == b'=' {
                        u2 = INF;
                        l2 = -INF;
                        break;
                    }
                    let a = q.val;
                    if a < -1e-13 {
                        if self.inf_max[r] == 0 {
                            let mut t = l - self.act_max[r] / a;
                            if int {
                                t = (t - 1e-13).ceil();
                            }
                            u2 = if u2 <= t { t } else { u2 };
                        } else {
                            u2 = INF;
                        }
                    } else if a > 1e-13 {
                        if self.inf_max[r] == 0 {
                            let mut t = u - self.act_max[r] / a;
                            if int {
                                t = (t + 1e-13).floor();
                            }
                            l2 = if t <= l2 { t } else { l2 };
                        } else {
                            l2 = -INF;
                        }
                    }
                }
                walked += 1;
                p = q.nc;
            }
            self.work += self.work_mult * 4.0 * walked as f64;
            if self.col_flag[j] & 3 != 0 {
                u2 = if u2 <= 0.0 { 0.0 } else { u2 };
                l2 = if 0.0 <= l2 { 0.0 } else { l2 };
            }
            let (mut lc, mut hc) = (l2, u2);
            if u2 < l2 {
                if 0.0 < u2 || l2 < 0.0 {
                    let x = if l2.abs() <= u2.abs() { l2 } else { u2 };
                    (lc, hc) = (x, x);
                } else {
                    (lc, hc) = (0.0, 0.0);
                }
            }
            lo = if lo <= lc { lc } else { lo };
            hi = if hc <= hi { hc } else { hi };
        }
        (lo, if hi <= lo { lo } else { hi })
    }
}
