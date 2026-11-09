//! `3e40d1` inequalities to equalities by dual arguments (docs/m2/m2_red_outer_b.md §2).
use crate::state::{State, INF, NIL};
use crate::Params;

/// `x − floor(x + 1e-10) <= 1e-10` as the original tests it (false on NaN).
fn integral(x: f64) -> bool {
    1e-10 >= x - (1e-10 + x).floor()
}

impl State {
    /// Row `r` divided by `a` has an integral rhs, only integer columns and integral coefficients (an empty row
    /// with a fractional rhs passes, as in the original).
    fn int_row(&self, r: usize, a: f64) -> bool {
        if !integral(self.rhs[r] / a) {
            return self.row_head[r] == NIL;
        }
        let mut q = self.row_head[r];
        while q != NIL {
            let nd = self.nd[q as usize];
            q = nd.nr;
            if nd.col >= 0 && (self.vtype[nd.col as usize] == b'C' || !integral(nd.val / a)) {
                return false;
            }
        }
        true
    }

    /// `3e40d1(st, env)`: a column (`5399fc` list, live, no flags, not fixed, in no `=` row) whose cost does not
    /// resist moving it down (up) and which sits in exactly one `<` row with a <= 0 (a > 0) blocks that move only
    /// by that row; if the row alone already implies the bound on that side (and for integer columns the row is
    /// integral after dividing by a), the row is tight at some optimum and becomes `=`. With both sides possible
    /// the shorter row wins (the up row on ties). Counter +0x400, row marked. Returns nothing (3cd406 ignores it).
    pub fn dual_ineq_to_eq(&mut self, _p: &Params) -> i32 {
        let cols = self.cols_list();
        self.work += ((self.nnz as f64 * 5.0 * cols.len() as f64) / (self.n as i32 + 1) as f64) * self.work_mult;
        for &jj in &cols {
            let j = jj as usize;
            if !(0 < self.col_cnt[j]) || self.col_flag[j] != 0 {
                continue;
            }
            let (l, u) = (self.lb[j], self.ub[j]);
            if l > u - 1e-10 {
                continue;
            }
            let c = self.obj[j];
            // -1: no row yet, -2: more than one row (or the cost resists the move), else the row
            let mut up: i64 = if 1e-10 < c { -2 } else { -1 };
            let mut dn: i64 = if c < -1e-10 { -2 } else { -1 };
            let (mut a_up, mut a_dn) = (0.0, 0.0);
            let mut q = self.col_head[j];
            let mut eq = false;
            while q != NIL {
                let nd = self.nd[q as usize];
                q = nd.nc;
                if nd.col < 0 {
                    continue;
                }
                let r = nd.row as usize;
                if self.sense[r] == b'=' {
                    eq = true;
                    break;
                }
                let le = self.sense[r] == b'<';
                if nd.val > 0.0 {
                    if le && up == -1 {
                        up = r as i64;
                        a_up = nd.val;
                    } else {
                        up = -2;
                    }
                } else if le && dn == -1 {
                    dn = r as i64;
                    a_dn = nd.val;
                } else {
                    dn = -2;
                }
            }
            if eq || (dn < 0 && up < 0) {
                continue;
            }
            if dn >= 0 {
                let r = dn as usize;
                let mut ok = true;
                if -INF < l {
                    let (lo, _) = self.implied_col_bounds(j, 1, Some((r, a_dn)));
                    ok = !(self.lb[j] - 1e-9 > lo);
                }
                if ok && self.vtype[j] != b'C' {
                    ok = self.int_row(r, a_dn);
                }
                if !ok {
                    dn = -2;
                }
            }
            let mut pick = dn;
            if up >= 0 {
                let r = up as usize;
                let mut ok = true;
                if INF > self.ub[j] {
                    let (_, hi) = self.implied_col_bounds(j, 1, Some((r, a_up)));
                    ok = !(hi > self.ub[j] + 1e-9);
                }
                if ok && self.vtype[j] != b'C' {
                    ok = self.int_row(r, a_up);
                }
                if ok && !(dn >= 0 && self.row_cnt[r] > self.row_cnt[dn as usize]) {
                    pick = up;
                }
            }
            if pick < 0 {
                continue;
            }
            // the LP-state block (+0xfc == 0 and +0x100 set) that frees the other bound is not taken in MIP
            self.sense[pick as usize] = b'=';
            self.ineq_to_eq += 1;
            self.tracker.mark_row(pick as i32);
        }
        0
    }
}
