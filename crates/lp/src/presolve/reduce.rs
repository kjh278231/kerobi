//! LP presolve reductions. Each function names its original; bodies follow the original's loop order, tests and
//! floating-point operation order, since the presolved model must match byte for byte.
use super::sort::sort_by_key;
use super::state::State;

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
            self.fix_col(ju, v, infbound);
        }
    }
}
