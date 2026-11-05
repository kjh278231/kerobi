//! LP presolve reductions. Each function names its original; bodies follow the original's loop order, tests and
//! floating-point operation order, since the presolved model must match byte for byte.
use super::sort::sort_by_key;
use super::state::{guarded_add, PostRec, State};

impl State {
    /// `51a144`: in a row with two or more singleton columns (columns that appear nowhere else), sort them by
    /// `-c/a` and fix, at the bound that minimises their row activity, the leading ones whose ranges the row
    /// can absorb: their dual price makes them sit at that bound in some optimal solution. `=` rows are looked
    /// at in both directions (`-row`, then `row`), `<` rows only as is. Needs DualReductions and a basis.
    pub fn dual_singleton_fix(&mut self, dual_reductions: bool, infbound: f64) {
        if !dual_reductions {
            return;
        }
        let rows = self.rows_list();
        for &r in &rows {
            let r = r as usize;
            if self.row_cnt[r] < 0 {
                continue;
            }
            let mut dir = -1.0f64;
            while dir <= 1.0 {
                if self.sense[r] == b'=' || dir == 1.0 {
                    self.dual_singleton_fix_row(r, dir, infbound);
                }
                dir += 2.0;
            }
        }
    }

    fn dual_singleton_fix_row(&mut self, r: usize, dir: f64, infbound: f64) {
        let b = self.row_beg[r] as usize;
        let end = b + self.row_len[r] as usize;
        let mut resid = dir * self.rhs[r];
        let mut nsing = 0;
        let mut sum = 0.0f64;
        let is_single = |s: &State, j: i32| j >= 0 && s.col_cnt[j as usize] == 1 && s.col_flag[j as usize] == 0;
        for k in b..end {
            let j = self.row_ind[k];
            if j < 0 || self.col_cnt[j as usize] < 0 {
                continue;
            }
            let ju = j as usize;
            let a = dir * self.row_val[k];
            if is_single(self, j) {
                nsing += 1;
                sum += if a < 0.0 { self.lb[ju] * a } else { self.ub[ju] * a };
            } else if a <= 0.0 {
                resid -= self.ub[ju] * a;
            } else {
                resid -= self.lb[ju] * a;
            }
        }
        if !(nsing > 1 && nsing != self.row_cnt[r] && resid < 1e10 && sum < 1e10 && resid + 1e-6 <= sum) {
            return;
        }
        // state+0x398 (coefficient by column), +0x390 (columns), +0x3b0 (sort keys)
        let mut coef = Vec::new();
        let mut cols = Vec::new();
        let mut keys = Vec::new();
        for k in b..end {
            let j = self.row_ind[k];
            let a = dir * self.row_val[k];
            if is_single(self, j) {
                coef.push(a);
                cols.push(j);
                keys.push(-self.obj[j as usize] / a);
            }
        }
        // `coef` is indexed by column in the original; columns are unique within a row, so position works.
        let by_col: Vec<(i32, f64)> = cols.iter().copied().zip(coef.iter().copied()).collect();
        sort_by_key(&mut keys, &mut cols);
        let coef_of = |j: i32| by_col.iter().find(|&&(c, _)| c == j).unwrap().1;
        let mut nfix = 0;
        while nfix < cols.len() {
            let j = cols[nfix] as usize;
            let w = ((self.ub[j] - self.lb[j]) * coef_of(cols[nfix])).abs();
            if 1e10 < w {
                break;
            }
            sum -= w;
            if sum < resid - 1e-10 {
                break;
            }
            nfix += 1;
        }
        for &j in &cols[..nfix] {
            let ju = j as usize;
            let v = if coef_of(j) <= 0.0 { self.ub[ju] } else { self.lb[ju] };
            self.fix_at(ju, v, infbound, true);
        }
    }
}

/// Status codes reductions return (same numbers as the original): 3 infeasible, 5 unbounded.
pub const INFEASIBLE: i32 = 3;
pub const UNBOUNDED: i32 = 5;

/// Tolerances `4fe113` reads from the environment.
#[derive(Clone, Copy, Debug)]
pub struct ColTols {
    pub infbound: f64,
    pub feastol: f64,
    pub opttol: f64,
    /// env+0x2718 NumericFocus
    pub numeric_focus: i32,
    /// env+0x26c0 GURO_PAR_AGGMULTLIMIT
    pub agg_mult_limit: f64,
    pub dual_reductions: bool,
    /// state+0x1e0 PreCrush
    pub precrush: i32,
}

impl State {
    /// `4fe113`: column pass over the columns changed at this level. Near-fixed columns are fixed (`4f6740`);
    /// with DualReductions, empty columns go to their cheaper bound, singleton columns that are free (or
    /// implied free) are substituted out with their row, singletons in `=` rows with an implied bound turn the
    /// row into an inequality, and the rest get the dual-fixing test (`4fd703`). Returns 0, 3 or 5.
    pub fn singleton_cols(&mut self, t: &ColTols) -> i32 {
        let fix_tol = if t.numeric_focus == 1 {
            1.0000000000000001e-11
        } else if t.numeric_focus < 2 {
            1e-10
        } else {
            1e-12
        };
        let inf = t.infbound;
        let cols = self.cols_list();
        for &jj in &cols {
            let j = jj as usize;
            let mut c = self.col_cnt[j];
            if c < 0 {
                continue;
            }
            let (l, u) = (self.lb[j], self.ub[j]);
            if t.feastol + u < l {
                self.bad_col = jj;
                return INFEASIBLE;
            }
            if u - l <= fix_tol {
                let rc = self.fix_near(j, t.feastol);
                if rc != 0 {
                    return rc;
                }
                continue;
            }
            if !t.dual_reductions {
                continue;
            }
            if self.col_flag[j] != 0 {
                let rc = self.dual_fix(j, inf, t.opttol, t.precrush);
                if rc != 0 {
                    return rc;
                }
                continue;
            }
            if c == 0 {
                let cost = self.obj[j];
                let v;
                if cost <= t.opttol {
                    if -t.opttol <= cost {
                        let mut w = self.lb[j];
                        if w <= -inf {
                            w = self.ub[j];
                            if inf <= w {
                                w = 0.0;
                            }
                        }
                        v = w;
                    } else {
                        v = self.ub[j];
                        if inf <= v {
                            self.bad_col = jj;
                            return UNBOUNDED;
                        }
                    }
                } else {
                    v = self.lb[j];
                    if v <= -inf {
                        self.bad_col = jj;
                        return UNBOUNDED;
                    }
                }
                self.push_fix(j, v);
                self.objcon += v * self.obj[j];
                self.obj[j] = 0.0;
                self.col_cnt[j] = -1;
                continue;
            }
            let mut rowish = 0; // iVar23: 1 when the singleton's row is an inequality
            if c == 1 {
                match self.singleton_col(j, t, &mut rowish) {
                    Ok(cnt) => c = cnt,
                    Err(rc) => return rc,
                }
            }
            if 1 < c + rowish && t.dual_reductions {
                let rc = self.dual_fix(j, inf, t.opttol, t.precrush);
                if rc != 0 {
                    return rc;
                }
            }
        }
        0
    }

    /// The `col_cnt == 1` branch of `4fe113`. Returns the column's live count afterwards (negative once it is
    /// gone) for the dual-fixing test, or a status.
    fn singleton_col(&mut self, j: usize, t: &ColTols, rowish: &mut i32) -> Result<i32, i32> {
        let inf = t.infbound;
        let cost = self.obj[j];
        let (l, u) = (self.lb[j], self.ub[j]);
        let b = self.col_beg[j] as usize;
        let end = b + self.col_len[j] as usize;
        // first live entry with a nonzero value (r and the slot are the last visited if none)
        let (mut r, mut slot) = (-1i32, b);
        let a;
        let mut last = 0.0f64;
        let mut k = b;
        while k < end {
            r = self.col_ind[k];
            slot = k;
            if r >= 0 && self.row_cnt[r as usize] >= 0 {
                last = self.col_val[k];
                if last != 0.0 || last.is_nan() {
                    break;
                }
            }
            k += 1;
        }
        a = last;
        self.work += self.col_len[j] as f64 * 3.0;
        if !(1e-9 < a.abs() && (1e-6 <= a.abs() * (u - l) || cost.abs() * (u - l) <= 1e-6)) {
            return Ok(self.col_cnt[j]);
        }
        let ru = r as usize;
        let eq = self.sense[ru] == b'=';
        if !eq {
            *rowish = 1;
            if (1e-9 < cost && 0.0 < a) || (cost < -1e-9 && a < 0.0) {
                return Ok(self.col_cnt[j]);
            }
        }
        let free = l <= -inf && inf <= u;
        let (mut lo_i, mut hi_i) = (0.0, 0.0);
        let implied_free = !free && {
            let (lo, hi) = self.implied_bounds(j, inf);
            lo_i = lo;
            hi_i = hi;
            (l <= -inf || l - 1e-9 <= lo) && (inf <= u || hi <= u + 1e-9)
        };
        if free || implied_free {
            return self.substitute_free_singleton(j, ru, slot, a, cost, t).map(|_| self.col_cnt[j]);
        }
        if !eq {
            return Ok(self.col_cnt[j]);
        }
        // `=` row, column not free: if one bound is implied, the column is the row's slack
        let d = cost.abs();
        if !(d < 1e-10 || (1e-6 <= a.abs() && d <= 1e6 && self.row_cnt[ru] < 1)) {
            return Ok(self.col_cnt[j]);
        }
        let mode = if inf <= u {
            1
        } else if l <= -inf {
            2
        } else if hi_i < u + 1e-9 {
            1
        } else if lo_i <= l - 1e-9 {
            return Ok(self.col_cnt[j]);
        } else {
            2
        };
        self.col_ind[slot] = -1;
        let old = self.col_cnt[j];
        self.col_cnt[j] = -1;
        let c0 = self.rhs[ru] / a;
        self.push_subst(j, ru, c0, a, mode, self.row_cnt[ru] - 1, old - 1);
        self.col_cnt[j] = old;
        if 1e-10 <= d {
            self.objcon += c0 * cost;
            self.obj[j] = 0.0;
            let rb = self.row_beg[ru] as usize;
            for k in rb..rb + self.row_len[ru] as usize {
                let q = self.row_ind[k];
                if q >= 0 && self.col_cnt[q as usize] >= 0 && q as usize != j {
                    self.obj[q as usize] -= self.row_val[k] * (cost / a);
                }
            }
            self.work += self.row_len[ru] as f64 * 4.0;
        }
        self.col_ind[slot] = r;
        self.rows_removed -= 1;
        if (mode == 1 && a < 0.0) || (mode == 2 && 0.0 < a) {
            self.negate_row(ru);
        }
        self.tracker.mark_row(r);
        self.mark_cols_of_row(ru);
        self.sense[ru] = b'<';
        let v = if mode == 1 { l } else { u };
        self.fix_at(j, v, inf, false);
        self.col_cnt[j] = -3 - r;
        Ok(self.col_cnt[j])
    }

    /// Free (or implied free) column singleton of `4fe113`: substitute it out with its row, unless the
    /// multipliers it would spread exceed AGGMULTLIMIT.
    fn substitute_free_singleton(&mut self, j: usize, r: usize, slot: usize, a: f64, cost: f64, t: &ColTols) -> Result<(), i32> {
        let lim = t.agg_mult_limit;
        let (mut lo, mut hi) = (1.0f64, 1.0f64);
        let rb = self.row_beg[r] as usize;
        let rend = rb + self.row_len[r] as usize;
        let mut k = rb;
        while k < rend {
            let q = self.row_ind[k];
            if q >= 0 && self.col_cnt[q as usize] >= 0 && q as usize != j {
                let q = q as usize;
                let ratio = (self.row_val[k] / a).abs();
                let small = self.mult_lo[j] * ratio;
                let m = if small <= self.mult_lo[q] { small } else { self.mult_lo[q] };
                if m < lo {
                    lo = m;
                    if m < 1.0 / lim {
                        break;
                    }
                }
                let big = self.mult_hi[j] * ratio;
                let mh = if self.mult_hi[q] <= big { big } else { self.mult_hi[q] };
                if hi < mh {
                    hi = mh;
                    if lim < mh {
                        break;
                    }
                }
            }
            k += 1;
        }
        self.work += (k - rb) as f64 * 2.0 * self.work_mult;
        if k < rend {
            return Ok(()); // multiplier limit hit: keep the column
        }
        self.col_ind[slot] = -1;
        let old = self.col_cnt[j];
        self.col_cnt[j] = -1;
        let c0 = self.rhs[r] / a;
        self.push_subst(j, r, c0, a, 0, self.row_cnt[r] - 1, old - 1);
        self.objcon += c0 * cost;
        self.obj[j] = 0.0;
        let ratio = cost / a;
        for k in rb..rend {
            let q = self.row_ind[k];
            if q >= 0 && self.col_cnt[q as usize] >= 0 {
                self.tracker.mark_col(q);
                self.col_cnt[q as usize] -= 1;
                self.obj[q as usize] -= self.row_val[k] * ratio;
                self.row_ind[k] = -1;
            }
        }
        self.work += self.row_len[r] as f64 * 6.0;
        self.row_len[r] = -1;
        self.row_cnt[r] = -3;
        self.col_cnt[j] = -3 - r as i32;
        Ok(())
    }

    /// Multiply row `r` by -1 in both CSR copies and swap its activity sides (inline in `4fe113`).
    fn negate_row(&mut self, r: usize) {
        let rb = self.row_beg[r] as usize;
        let len = self.row_len[r] as usize;
        for k in rb..rb + len {
            let q = self.row_ind[k];
            if q < 0 || self.col_cnt[q as usize] < 0 {
                continue;
            }
            let q = q as usize;
            self.row_val[k] = -self.row_val[k];
            let cb = self.col_beg[q] as usize;
            for p in cb..cb + self.col_len[q] as usize {
                let i = self.col_ind[p];
                if i >= 0 && self.row_cnt[i as usize] >= 0 && i as usize == r {
                    self.col_val[p] = -self.col_val[p];
                    break;
                }
            }
            self.work += self.col_len[q] as f64 * 6.0;
        }
        self.work += len as f64 * 5.0;
        self.rhs[r] = -self.rhs[r];
        let (im, ix) = (self.inf_min[r], self.inf_max[r]);
        self.inf_min[r] = ix;
        self.inf_max[r] = im;
        let (am, ax) = (self.act_min[r], self.act_max[r]);
        self.act_min[r] = -ax;
        self.act_max[r] = -am;
    }

    /// `537da8`: mark every column of row `r` (row CSR, removed columns included).
    pub fn mark_cols_of_row(&mut self, r: usize) {
        let rb = self.row_beg[r] as usize;
        let len = self.row_len[r].max(0) as usize;
        for k in rb..rb + len {
            let q = self.row_ind[k];
            if q >= 0 {
                self.tracker.mark_col(q);
            }
        }
        self.work += self.work_mult * 3.0 * len as f64;
    }

    /// `4f4d0a`: type-2 record. Counts the column and the row as removed.
    #[allow(clippy::too_many_arguments)]
    fn push_subst(&mut self, j: usize, r: usize, c0: f64, a: f64, mode: i32, _nrow: i32, ncol: i32) {
        self.cols_removed += 1;
        self.rows_removed += 1;
        let mut row = Vec::new();
        let rb = self.row_beg[r] as usize;
        for k in rb..rb + self.row_len[r] as usize {
            let q = self.row_ind[k];
            if q >= 0 && self.col_cnt[q as usize] >= 0 {
                row.push((q, -self.row_val[k] / a));
            }
        }
        self.work += self.row_len[r] as f64 * 6.0;
        let mut col = Vec::new();
        if 0 < ncol {
            let cb = self.col_beg[j] as usize;
            for k in cb..cb + self.col_len[j] as usize {
                let i = self.col_ind[k];
                if i >= 0 && i as usize != r && self.row_cnt[i as usize] >= 0 {
                    col.push((i, self.col_val[k]));
                }
            }
            self.work += self.col_len[j] as f64 * 7.0;
        }
        let mode = if mode == 0 { ncol } else { -mode };
        self.post.push(PostRec::Subst { j: j as i32, r: r as i32, c0, a, obj: self.obj[j], mode, row, col });
    }

    /// `4fd703`: dual fixing of column `j` (no `=` rows): if its cost and every coefficient push it the same
    /// way, it goes to that bound (`4fa83a`); toward an infinite bound the column and all its rows go away
    /// with a type-3 record, or the LP is unbounded if the cost strictly wants it. Returns 0, 5.
    pub fn dual_fix(&mut self, j: usize, inf: f64, opttol: f64, precrush: i32) -> i32 {
        let c = self.obj[j];
        let mut down = -1e-10 < c; // all a > 0 and c >= 0: decreasing x helps
        let mut up = c < 1e-10; // all a <= 0 and c <= 0: increasing x helps
        if self.col_flag[j] & !0x40 != 0 {
            // shortcut: special columns (PWL objective etc.) never occur in plain LP; port 3dc399 with them.
            return 0;
        }
        let cb = self.col_beg[j] as usize;
        let len = self.col_len[j] as usize;
        for k in cb..cb + len {
            let i = self.col_ind[k];
            if i < 0 || self.row_cnt[i as usize] < 0 {
                continue;
            }
            if self.sense[i as usize] == b'=' {
                self.work += len as f64 * 4.0;
                return 0;
            }
            let a = self.col_val[k];
            up &= a <= 0.0;
            down &= 0.0 < a;
        }
        self.work += len as f64 * 4.0;
        if !up && !down {
            return 0;
        }
        let (l, u) = (self.lb[j], self.ub[j]);
        let mut act = 0; // 1 drop column and rows, 2 fix at the finite bound
        if up {
            if inf <= u {
                if precrush == 0 {
                    if c < -opttol {
                        self.bad_col = j as i32;
                        return UNBOUNDED;
                    }
                    self.push_dual_fix(j, 1);
                    act = 1;
                }
            } else {
                act = 2;
            }
        } else if l <= -inf {
            if precrush == 0 {
                if opttol < c {
                    self.bad_col = j as i32;
                    return UNBOUNDED;
                }
                self.push_dual_fix(j, 0);
                act = 1;
            }
        } else {
            act = 2;
        }
        if act == 1 {
            for k in cb..cb + self.col_len[j] as usize {
                let i = self.col_ind[k];
                if i < 0 || self.row_cnt[i as usize] < 0 {
                    continue;
                }
                let i = i as usize;
                self.row_cnt[i] = -1;
                let rb = self.row_beg[i] as usize;
                for p in rb..rb + self.row_len[i] as usize {
                    let q = self.row_ind[p];
                    if q >= 0 && self.col_cnt[q as usize] >= 0 {
                        self.tracker.mark_col(q);
                        self.col_cnt[q as usize] -= 1;
                        self.row_ind[p] = -1;
                    }
                }
                self.col_ind[k] = -1;
                self.work += self.row_len[i] as f64 * 4.0;
            }
            self.work += self.col_len[j] as f64 * 4.0;
            self.col_len[j] = -1;
            self.col_cnt[j] = -2;
        } else if act == 2 {
            let v = if up { u } else { l };
            self.fix_at(j, v, inf, true);
        }
        0
    }

    /// `4f39e3`: type-3 record; the column and its live rows count as removed.
    fn push_dual_fix(&mut self, j: usize, dir: i32) {
        let cb = self.col_beg[j] as usize;
        let len = self.col_len[j] as usize;
        self.work += len as f64 * 3.0;
        self.cols_removed += 1;
        self.rows_removed += self.col_cnt[j];
        let (mut rows, mut terms) = (Vec::new(), Vec::new());
        for k in cb..cb + len {
            let i = self.col_ind[k];
            if i < 0 || self.row_cnt[i as usize] < 0 {
                continue;
            }
            let iu = i as usize;
            let a = self.col_val[k];
            rows.push((i, self.rhs[iu] / a));
            let mut tt = Vec::new();
            let rb = self.row_beg[iu] as usize;
            for p in rb..rb + self.row_len[iu] as usize {
                let q = self.row_ind[p];
                if q != j as i32 && q >= 0 && self.col_cnt[q as usize] >= 0 {
                    tt.push((q, -self.row_val[p] / a));
                }
            }
            terms.push(tt);
            self.work += self.row_len[iu] as f64 * 6.0;
        }
        self.work += len as f64 * 7.0;
        self.post.push(PostRec::DualFix { j: j as i32, dir, rows, terms });
    }
}

impl State {
    /// `4ffe11`: singleton rows among the rows marked at this level (raw list, `538353`): the row becomes a
    /// bound on its column (both bounds for `=`, which fixes the column via `4fa83a`), or proves infeasibility.
    /// Bound changes within tolerance of the old bound are ignored; slightly violated ones are clipped.
    pub fn singleton_rows(&mut self, feastol: f64, inf: f64) -> i32 {
        let lv = self.tracker.level;
        let rows: Vec<i32> = self.tracker.rows.list[lv][..self.tracker.rows.a[lv]].to_vec();
        let mut removed = 0;
        for &rr in &rows {
            let r = rr as usize;
            if self.row_cnt[r] != 1 {
                continue;
            }
            let b = self.row_beg[r] as usize;
            let end = b + self.row_len[r] as usize;
            let mut k = b;
            while k < end && (self.row_ind[k] < 0 || self.col_cnt[self.row_ind[k] as usize] < 0) {
                k += 1;
            }
            self.work += self.row_len[r] as f64 * 2.0;
            let a = self.row_val[k];
            let aa = a.abs();
            if aa < 1e-8 {
                continue;
            }
            let rhs = self.rhs[r];
            let mut x = rhs / a;
            let jj = self.row_ind[k];
            let j = jj as usize;
            let code: u32;
            if self.sense[r] == b'=' {
                let (l, u) = (self.lb[j], self.ub[j]);
                if x < l - feastol || feastol + u < x {
                    code = 4;
                } else if x < l - feastol / aa || feastol / aa + u < x {
                    continue;
                } else {
                    code = (l + 1e-10 < x) as u32 + if x < u - 1e-10 { 2 } else { 0 };
                }
            } else if a <= 0.0 {
                if self.lb[j] + 1e-10 < x {
                    let u = self.ub[j];
                    if feastol + u < x {
                        code = 4;
                    } else if (feastol * 10.0) / aa + u < x {
                        continue;
                    } else {
                        code = 1;
                    }
                } else {
                    code = 0;
                }
            } else if x < self.ub[j] - 1e-10 {
                let l = self.lb[j];
                if x < l - feastol {
                    code = 4;
                } else if x < l - (feastol * 10.0) / aa {
                    continue;
                } else {
                    code = 2;
                }
            } else {
                code = 0;
            }
            if code & 1 != 0 && self.ub[j] < x {
                let t = if a <= 0.0 { feastol + rhs } else { rhs - feastol };
                x = if t / a <= self.ub[j] { self.ub[j] } else { t / a };
            }
            if code & 2 != 0 && x < self.lb[j] {
                let t = if a <= 0.0 { rhs - feastol } else { rhs + feastol };
                x = if self.lb[j] <= t / a { self.lb[j] } else { t / a };
            }
            if code == 4 {
                self.row_ind[k] = -1;
                if feastol <= self.act_min[r] || (self.act_max[r] <= -feastol && self.sense[r] == b'=') {
                    self.bad_row = rr;
                    self.bad_col = jj;
                    return INFEASIBLE;
                }
                self.row_ind[k] = jj;
                continue;
            }
            self.row_ind[k] = -1;
            if code != 0 {
                self.post.push(PostRec::SingletonRow { j: jj, r: rr, code: code as i32, a, x });
                let (ol, ou) = (self.lb[j], self.ub[j]);
                let nl = if code & 1 != 0 { if x < ou { x } else { ou } } else { ol };
                let nu = if code & 2 != 0 { if x > nl { x } else { nl } } else { ou };
                self.bound_change(j, ol, ou, nl, nu, inf);
                self.lb[j] = nl;
                self.ub[j] = nu;
            }
            self.tracker.mark_col(jj);
            self.row_cnt[r] = -1;
            self.col_cnt[j] -= 1;
            if code == 3 {
                self.fix_at(j, x, inf, true);
            }
            removed += 1;
        }
        self.rows_removed += removed;
        0
    }
}

impl State {
    /// `4ff158`: forcing rows. If a row's min activity (any row) or max activity (`=` rows) equals its rhs
    /// with no infinite contribution, every column sits at the bound giving that activity: fix them all
    /// (`4fa83a`) and drop the row with a type-8 record. Skipped when some column's range times its
    /// coefficient is below 100 feastol.
    pub fn forcing_rows(&mut self, feastol: f64, inf: f64) -> i32 {
        let rows = self.rows_list();
        let mut removed = 0;
        for &rr in &rows {
            let r = rr as usize;
            let c = self.row_cnt[r];
            if c <= 1 {
                continue;
            }
            let dir = if !(self.inf_min[r] != 0 || self.act_min[r] <= -1e-10 || feastol <= self.act_min[r]) {
                1.0
            } else if self.sense[r] == b'=' && self.inf_max[r] == 0 && self.act_max[r] < 1e-10 && -feastol < self.act_max[r] {
                -1.0
            } else {
                continue;
            };
            let b = self.row_beg[r] as usize;
            let len = self.row_len[r] as usize;
            let tiny = (b..b + len).any(|k| {
                let j = self.row_ind[k];
                j >= 0
                    && self.col_cnt[j as usize] >= 0
                    && ((self.ub[j as usize] - self.lb[j as usize]) * self.row_val[k]).abs() < feastol * 100.0
            });
            self.work += len as f64 * 5.0;
            if tiny {
                continue;
            }
            let mut entries = Vec::new();
            for k in b..b + len {
                let j = self.row_ind[k];
                if j >= 0 && self.col_cnt[j as usize] >= 0 {
                    entries.push((j, self.row_val[k]));
                }
            }
            self.work += len as f64 * 6.0;
            self.post.push(PostRec::Forcing { r: rr, dir, entries });
            self.row_cnt[r] = -1;
            for k in b..b + len {
                let j = self.row_ind[k];
                if j < 0 || self.col_cnt[j as usize] < 0 {
                    continue;
                }
                let j = j as usize;
                let a = self.row_val[k];
                self.row_ind[k] = -1;
                self.col_cnt[j] -= 1;
                let v = if dir * a <= 0.0 { self.ub[j] } else { self.lb[j] };
                self.fix_at(j, v, inf, true);
            }
            removed += 1;
            self.row_cnt[r] = -2;
            self.work += self.row_len[r] as f64 * 6.0;
        }
        self.rows_removed += removed;
        0
    }
}

impl State {
    /// `50cc90`: empty rows (infeasible if violated), rows whose max activity cannot exceed the rhs (and, for
    /// `=`, min activity cannot fall below it) are dropped. With `mode != 0`, coefficients too small to matter
    /// (`cnt*|a|*(u-l) < 0.01 feastol`, `|a| < 0.1 opttol`) move to the rhs at the column's lower bound, and
    /// the changed rows get their activities recomputed. Returns 0 or 3.
    pub fn redundant_rows(&mut self, mode: i32, feastol: f64, opttol: f64, inf: f64) -> i32 {
        let coef_tol = opttol * 0.1;
        let rows = self.rows_list();
        let mut removed = 0;
        let mut changed = Vec::new(); // state+0x3b8
        for &rr in &rows {
            let r = rr as usize;
            let c = self.row_cnt[r];
            if c == 0 {
                let rhs = self.rhs[r];
                if rhs < -feastol || (feastol < rhs && self.sense[r] == b'=') {
                    self.bad_row = rr;
                    self.rows_removed += removed;
                    return INFEASIBLE;
                }
                self.row_cnt[r] = -1;
                removed += 1;
                continue;
            }
            if c < 0 {
                continue;
            }
            let b = self.row_beg[r] as usize;
            let len = self.row_len[r] as usize;
            let needed = self.inf_max[r] != 0
                || feastol <= self.act_max[r]
                || (self.sense[r] == b'=' && (self.inf_min[r] != 0 || self.act_min[r] <= -feastol));
            if !needed {
                removed += 1;
                self.mark_cols_of_row(r);
                for k in b..b + len {
                    let j = self.row_ind[k];
                    if j >= 0 && self.col_cnt[j as usize] >= 0 {
                        self.col_cnt[j as usize] -= 1;
                        self.row_ind[k] = -1;
                    }
                }
                self.work += len as f64 * 4.0;
                self.row_cnt[r] = -2;
                continue;
            }
            if mode == 0 {
                continue;
            }
            let mut cnt = 0;
            for k in b..b + len {
                let j = self.row_ind[k];
                if j < 0 || self.col_cnt[j as usize] < 0 {
                    continue;
                }
                let ju = j as usize;
                let a = self.row_val[k];
                if !(self.row_cnt[r] as f64 * a.abs() * (self.ub[ju] - self.lb[ju]) < feastol * 0.01 && a.abs() < coef_tol) {
                    continue;
                }
                self.tracker.mark_col(j);
                self.row_cnt[r] -= 1;
                self.col_cnt[ju] -= 1;
                self.row_ind[k] = -1;
                self.rhs[r] -= a * self.lb[ju];
                let cb = self.col_beg[ju] as usize;
                for p in cb..cb + self.col_len[ju].max(0) as usize {
                    if self.col_ind[p] == rr {
                        self.col_ind[p] = -1;
                        break;
                    }
                }
                cnt += 1;
            }
            self.work += len as f64 * 9.0;
            if cnt != 0 {
                self.tracker.mark_row(rr);
                changed.push(rr);
                self.coefs_changed += cnt;
            }
        }
        self.rows_removed += removed;
        if !changed.is_empty() {
            self.activity(Some(&changed), inf);
        }
        0
    }
}

impl State {
    /// `50d878`: every empty column goes to the bound its cost prefers (unbounded if that bound is infinite
    /// and the cost is significant), then columns with `u < l + 1e-10` are fixed (`4f6740`). Returns 0, 3, 5.
    pub fn empty_cols(&mut self, opttol: f64, feastol: f64, inf: f64) -> i32 {
        let n = self.n;
        for j in 0..n {
            if self.col_cnt[j] != 0 || self.col_flag[j] != 0 {
                continue;
            }
            let c = self.obj[j];
            let (l, u) = (self.lb[j], self.ub[j]);
            let v = if c <= opttol {
                if -opttol <= c {
                    if c < 0.0 {
                        if u < inf { u } else if -inf < l { l } else { 0.0 }
                    } else if -inf < l {
                        l
                    } else if u < inf {
                        u
                    } else {
                        0.0
                    }
                } else {
                    if inf <= u {
                        self.bad_col = j as i32;
                        return UNBOUNDED;
                    }
                    u
                }
            } else {
                if l <= -inf {
                    self.bad_col = j as i32;
                    return UNBOUNDED;
                }
                l
            };
            self.push_fix(j, v);
            self.col_cnt[j] = -1;
            self.objcon += v * c;
            self.obj[j] = 0.0;
        }
        self.work += n as f64 * 8.0;
        for j in 0..n {
            if 0 < self.col_cnt[j] && self.ub[j] < self.lb[j] + 1e-10 {
                let rc = self.fix_near(j, feastol);
                if rc != 0 {
                    return rc;
                }
            }
        }
        self.work += n as f64 * 4.0;
        0
    }
}

impl State {
    /// `38e49a`: put every column at the bound its cost prefers (`x`, 0 for a zero cost when 0 is a bound).
    /// If `x` satisfies every row, all columns are fixed there; if it only satisfies the `=` rows and the
    /// inequality rows it might break keep the `=`-row columns at their min-activity bound, those columns are
    /// fixed. Both end with `50cc90(0)`. Gives up on a special column, an infinite preferred bound or a
    /// zero-cost column strictly inside its range.
    pub fn cost_point_fix(&mut self, feastol: f64, opttol: f64, inf: f64) -> i32 {
        let (n, m) = (self.n, self.m);
        if n < 1 {
            return 0;
        }
        let mu = self.work_mult;
        let mut x = vec![0.0f64; n];
        let mut inside = 0;
        let mut j = 0;
        while j < n {
            if self.col_cnt[j] >= 0 {
                if self.col_flag[j] != 0 {
                    break;
                }
                let (c, l, u) = (self.obj[j], self.lb[j], self.ub[j]);
                if 0.0 <= c {
                    if c <= 0.0 {
                        if l <= 0.0 {
                            if 0.0 <= u {
                                if l == 0.0 || u == 0.0 {
                                    x[j] = 0.0;
                                } else {
                                    inside += 1;
                                    x[j] = 1e101;
                                }
                            } else {
                                x[j] = u;
                            }
                        } else {
                            x[j] = l;
                        }
                    } else if l <= -inf {
                        break;
                    } else {
                        x[j] = l;
                    }
                } else if inf <= u {
                    break;
                } else {
                    x[j] = u;
                }
            }
            j += 1;
        }
        self.work += mu * 5.0 * j as f64;
        if j < n || 0 < inside {
            return 0;
        }
        // `=` rows must hold at x; their columns are collected (state+0x390, marks +0x3a8)
        let mut eq_cols: Vec<i32> = Vec::new();
        let mut mark = vec![false; n];
        let mut neq = 0;
        let mut r = 0;
        let mut w = self.work;
        while r < m {
            if self.row_cnt[r] >= 0 && self.sense[r] == b'=' {
                neq += 1;
                let before = eq_cols.len();
                let b = self.row_beg[r] as usize;
                let len = self.row_len[r] as usize;
                let mut k = 0;
                let mut sum = 0.0f64;
                while k < len {
                    let jj = self.row_ind[b + k];
                    if jj >= 0 && self.col_cnt[jj as usize] >= 0 {
                        if !mark[jj as usize] {
                            eq_cols.push(jj);
                            mark[jj as usize] = true;
                        }
                        if let Some(s) = point_term(x[jj as usize], self.row_val[b + k], &mut sum) {
                            sum = s;
                            break;
                        }
                    }
                    k += 1;
                }
                w = k as f64 * 3.0 + self.work + (eq_cols.len() - before) as f64 * 2.0;
                if inf <= sum.abs() {
                    break;
                }
                self.work = w;
                if 1e-10 < (sum - self.rhs[r]).abs() {
                    break;
                }
            }
            r += 1;
        }
        if r == m {
            w = self.work;
        }
        self.work = w + (neq as f64 * 5.0 + r as f64 + r as f64) * mu;
        if r != m {
            self.work += mu * 2.0 * eq_cols.len() as f64;
            return 0;
        }
        // max activity of the inequality rows with the `=`-row columns moved to x (+0x3b8 / +0x398 copies)
        let mut infc = self.inf_max.clone();
        let mut actc = self.act_max.clone();
        let mut seen = vec![false; m];
        let mut rows: Vec<i32> = Vec::new();
        for &jj in &eq_cols {
            let j = jj as usize;
            let cb = self.col_beg[j] as usize;
            let len = self.col_len[j] as usize;
            let mut cnt = 0;
            for p in cb..cb + len {
                let i = self.col_ind[p];
                if i < 0 || self.row_cnt[i as usize] < 0 || self.sense[i as usize] == b'=' {
                    continue;
                }
                let i = i as usize;
                cnt += 1;
                let a = self.col_val[p];
                if !seen[i] {
                    seen[i] = true;
                    rows.push(i as i32);
                }
                let v = x[j];
                let (bnd, finite) = if a <= 0.0 { (self.lb[j], -inf < self.lb[j]) } else { (self.ub[j], self.ub[j] < inf) };
                if v == bnd {
                    continue;
                }
                if finite {
                    actc[i] = guarded_add(actc[i], (v - bnd) * a, true);
                } else {
                    infc[i] -= 1;
                    actc[i] = guarded_add(actc[i], a * v, true);
                }
            }
            self.work += (len as f64 * 3.0 + cnt as f64 * 5.0) * mu;
        }
        self.work += (eq_cols.len() as f64 * 4.0 + rows.len() as f64) * mu;
        let nrows = rows.len();
        rows.retain(|&i| 0 < infc[i as usize] || 1e-10 < actc[i as usize]);
        self.work += mu * 3.0 * nrows as f64;
        // in the rows x might break, the `=`-row columns must sit at their min-activity bound
        let mut t = 0;
        while t < rows.len() {
            let i = rows[t] as usize;
            let b = self.row_beg[i] as usize;
            let e = b + self.row_len[i] as usize;
            let mut k = b;
            while k < e {
                let jj = self.row_ind[k];
                if jj >= 0 && self.col_cnt[jj as usize] >= 0 && mark[jj as usize] {
                    let j = jj as usize;
                    let at = if self.row_val[k] <= 0.0 { self.ub[j] } else { self.lb[j] };
                    if x[j] != at {
                        break;
                    }
                }
                k += 1;
            }
            self.work += mu * 4.0 * (k - b) as f64;
            if k < e {
                break;
            }
            t += 1;
        }
        self.work += mu * 4.0 * t as f64;
        let at_min = t == rows.len();
        // inequality rows at x
        let mut nineq = 0;
        let mut r = 0;
        loop {
            while r < m && (self.row_cnt[r] < 0 || self.sense[r] == b'=') {
                r += 1;
            }
            if r >= m {
                w = self.work;
                break;
            }
            nineq += 1;
            let b = self.row_beg[r] as usize;
            let len = self.row_len[r] as usize;
            let mut k = 0;
            let mut sum = 0.0f64;
            while k < len {
                let jj = self.row_ind[b + k];
                if jj >= 0 && self.col_cnt[jj as usize] >= 0 {
                    if let Some(s) = point_term(x[jj as usize], self.row_val[b + k], &mut sum) {
                        sum = s;
                        break;
                    }
                }
                k += 1;
            }
            w = k as f64 * 3.0 + self.work;
            if inf <= sum.abs() {
                break;
            }
            self.work = w;
            if 1e-10 < sum - self.rhs[r] {
                break;
            }
            r += 1;
        }
        self.work = w + (nineq as f64 * 5.0 + r as f64 + r as f64) * mu;
        self.work += mu * 2.0 * eq_cols.len() as f64;
        if m <= r {
            for j in 0..n {
                if self.col_cnt[j] >= 0 {
                    self.fix_at(j, x[j], inf, true);
                }
            }
            self.work += (n as f64 + n as f64) * mu;
        } else {
            if !at_min || eq_cols.is_empty() {
                return 0;
            }
            for &jj in &eq_cols {
                self.fix_at(jj as usize, x[jj as usize], inf, true);
            }
            // the original counts the last column's index here, not the number of columns
            let last = *eq_cols.last().unwrap() as f64;
            self.work += (last + last) * mu;
        }
        self.redundant_rows(0, feastol, opttol, inf);
        0
    }
}

/// One term of a row evaluated at `38e49a`'s point: adds `a * v`, or returns the saturated sum (+-1e100) when
/// `v` is infinite (|v| >= 1e30) and `a` is nonzero.
fn point_term(v: f64, a: f64, sum: &mut f64) -> Option<f64> {
    if v < 1e30 {
        if v <= -1e30 {
            if 0.0 < a {
                return Some(-1e100);
            }
            if a < 0.0 {
                return Some(1e100);
            }
        }
    } else {
        if 0.0 < a {
            return Some(1e100);
        }
        if a < 0.0 {
            return Some(-1e100);
        }
    }
    *sum += a * v;
    None
}

impl State {
    /// `50495e`: with bounds tightened by one propagation pass over the rows at this level (`50108e`, kept
    /// aside; the column bounds do not change), drop empty rows and `<` rows that cannot be violated, and
    /// detect rows that cannot be satisfied. Returns 0 or 3.
    pub fn implied_redundant_rows(&mut self, feastol: f64, inf: f64, prebound: i32) -> i32 {
        let rows = self.rows_list();
        let (lo, hi) = match self.propagate_bounds(&rows, feastol, inf, prebound) {
            Ok(b) => b,
            Err(rc) => return rc,
        };
        // SolutionTarget (env+0x2b9c) == 2 would copy lo/hi into the bounds here; the parameter stops at 1.
        let mut removed = 0;
        for &rr in &rows {
            let r = rr as usize;
            let c = self.row_cnt[r];
            if c == 0 {
                let rhs = self.rhs[r];
                if rhs < -feastol || (feastol < rhs && self.sense[r] == b'=') {
                    self.bad_row = rr;
                    return INFEASIBLE;
                }
                self.row_cnt[r] = -1;
                removed += 1;
                continue;
            }
            if c < 0 {
                continue;
            }
            let (maxact, nmax, minact, nmin) = self.row_range(r, &lo, &hi, inf);
            let b = self.row_beg[r] as usize;
            let len = self.row_len[r] as usize;
            self.work += len as f64 * 5.0;
            let mut d = maxact - minact;
            if d <= 1.0 {
                d = 1.0;
            }
            let d = if d <= 10000.0 { d } else { 10000.0 };
            if self.sense[r] == b'=' {
                if (nmin == 0 && feastol * d < minact) || (nmax == 0 && maxact < -(d * feastol)) {
                    self.bad_row = rr;
                    return INFEASIBLE;
                }
            } else if nmax != 0 || -(feastol * d) <= maxact {
                if nmin == 0 && d * feastol < minact {
                    self.bad_row = rr;
                    return INFEASIBLE;
                }
            } else {
                for k in b..b + len {
                    let j = self.row_ind[k];
                    if j >= 0 && self.col_cnt[j as usize] >= 0 {
                        self.tracker.mark_col(j);
                        self.col_cnt[j as usize] -= 1;
                        self.row_ind[k] = -1;
                    }
                }
                self.row_cnt[r] = -2;
                self.work += len as f64 * 4.0;
                removed += 1;
            }
        }
        self.rows_removed += removed;
        self.work += rows.len() as f64 * 5.0;
        0
    }

    /// Max and min activity minus rhs of row `r` under bounds `lo`/`hi`, plain sums without rounding guard:
    /// (finite max part, infinite max count, finite min part, infinite min count). As inlined in `50495e`/`50108e`.
    fn row_range(&self, r: usize, lo: &[f64], hi: &[f64], inf: f64) -> (f64, i32, f64, i32) {
        let (mut maxact, mut minact) = (-self.rhs[r], -self.rhs[r]);
        let (mut nmax, mut nmin) = (0, 0);
        let b = self.row_beg[r] as usize;
        for k in b..b + self.row_len[r] as usize {
            let j = self.row_ind[k];
            if j < 0 || self.col_cnt[j as usize] < 0 {
                continue;
            }
            let (j, a) = (j as usize, self.row_val[k]);
            if a <= 0.0 {
                if -inf < lo[j] {
                    maxact += lo[j] * a;
                } else {
                    nmax += 1;
                }
                if inf <= hi[j] {
                    nmin += 1;
                } else {
                    minact += hi[j] * a;
                }
            } else {
                if hi[j] < inf {
                    maxact += hi[j] * a;
                } else {
                    nmax += 1;
                }
                if -inf < lo[j] {
                    minact += a * lo[j];
                } else {
                    nmin += 1;
                }
            }
        }
        (maxact, nmax, minact, nmin)
    }

    /// `50108e`: copies of the column bounds (state+0x398 / +0x3a0) tightened by one pass over `rows`: each row's
    /// min activity (and, for `=` rows, max activity) bounds every column in it. Changes under 100 feastol and
    /// bounds beyond 1e8 are ignored; crossing bounds give 3 with `bad_col`. Without PREBOUND only the copy.
    fn propagate_bounds(&mut self, rows: &[i32], feastol: f64, inf: f64, prebound: i32) -> Result<(Vec<f64>, Vec<f64>), i32> {
        let mut lo = self.lb.clone();
        let mut hi = self.ub.clone();
        // The original copies all bounds when (m+1)*0.3 < nnz*|rows| (QC aside), else only those of the rows'
        // columns, which are the only ones read; only the second way counts work.
        if !(((self.m + 1) as f64) * 0.3 < (self.nnz * rows.len() as i64) as f64) {
            for &r in rows {
                if 0 < self.row_cnt[r as usize] {
                    self.work += self.row_len[r as usize] as f64 * 6.0;
                }
            }
        }
        if prebound == 0 {
            return Ok((lo, hi));
        }
        for &rr in rows {
            let r = rr as usize;
            if self.row_cnt[r] <= 0 {
                continue;
            }
            let (maxact, nmax, minact, nmin) = self.row_range(r, &lo, &hi, inf);
            let b = self.row_beg[r] as usize;
            let len = self.row_len[r] as usize;
            self.work += len as f64 * 5.0;
            let mut d = maxact - minact;
            if d <= 1.0 {
                d = 1.0;
            }
            if !(d <= 1e13) {
                continue;
            }
            let dcap = if d <= 1e6 { d } else { 1e6 };
            let mn = if 0.0 <= minact { 0.0 } else { minact };
            let mx = if 0.0 <= -maxact { 0.0 } else { -maxact };
            let eq = self.sense[r] == b'=';
            for k in b..b + len {
                let jj = self.row_ind[k];
                if jj < 0 || self.col_cnt[jj as usize] < 0 {
                    continue;
                }
                let j = jj as usize;
                let a = self.row_val[k];
                if !(dcap * 1e-10 <= a.abs()) {
                    continue;
                }
                let (u, mut l) = (hi[j], lo[j]);
                let (mut nl, mut nu) = (l, u);
                if a <= 1e-8 {
                    if a < -1e-8 {
                        if nmin == 0 {
                            nl = u - mn / a;
                        } else if nmin == 1 && inf <= u {
                            nl = -minact / a;
                        }
                    }
                } else if nmin == 0 {
                    nu = l - mn / a;
                } else if nmin == 1 && l <= -inf {
                    nu = -minact / a;
                }
                let (mut f8, mut f0) = (nl, nu);
                if eq {
                    f8 = self.lb[j];
                    f0 = self.ub[j];
                    let a2 = -a;
                    if a2 <= 1e-8 {
                        if a2 < -1e-8 {
                            if nmax == 0 {
                                f8 = u - mx / a2;
                            } else if nmax == 1 && inf <= u {
                                f8 = maxact / a2;
                            }
                        }
                    } else if nmax == 0 {
                        f0 = l - mx / a2;
                    } else if nmax == 1 && l <= -inf {
                        f0 = maxact / a2;
                    }
                    if f8 <= nl {
                        f8 = nl;
                    }
                    if nu <= f0 {
                        f0 = nu;
                    }
                }
                if -1e8 < f8 && feastol * 100.0 + l < f8 {
                    if u + feastol < f8 {
                        self.bad_col = jj;
                        return Err(INFEASIBLE);
                    }
                    lo[j] = f8;
                    l = f8;
                }
                if f0 < 1e8 && f0 < hi[j] - feastol * 100.0 {
                    if f0 < l - feastol {
                        self.bad_col = jj;
                        return Err(INFEASIBLE);
                    }
                    hi[j] = f0;
                }
            }
            self.work += len as f64 * 12.0;
        }
        Ok((lo, hi))
    }
}
