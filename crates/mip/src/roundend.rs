//! Small round-end reductions of `3cd406` (docs/m2_red_roundend.md §1–3): `58d4ce` (Q diagonal; nothing for linear
//! models), `5a9282` dominated singleton columns of a row, `5ab429` cost singletons in a row with a coefficient gcd.
use crate::state::{State, NIL};
use crate::subst::sort_asc;
use crate::Params;

impl State {
    /// `58d4ce(env, st, W)`: columns with a zero Q diagonal go to `3c2f36`. Without a Q objective (+0x250 NULL) it
    /// returns 0 at once, without work (docs/m2_red_roundend.md §1).
    /// shortcut: the Q branch is not ported (linear models only); port it with Q objectives.
    pub fn q_diag(&mut self, _p: &Params) -> i32 {
        0
    }

    /// `5a9282(st, env)`: in a row (each side of an `=` row), when the singleton columns could more than cover what
    /// the other columns leave of the rhs, the singletons with the best cost ratio −c/a go to their bound of least
    /// activity (`3de18e`, type 1 records) while the rest can still cover it. Returns 0 or `3de18e`'s value.
    pub fn r5a9282(&mut self, p: &Params) -> i32 {
        // guard: state+0x100 == 0 and +0x330 (PWL objectives) == 0 hold for linear models
        if p.dual_reductions == 0 || self.precrush == 2 {
            return 0;
        }
        let rows = self.rows_list();
        let mu = self.work_mult;
        for &ri in &rows {
            let r = ri as usize;
            if self.row_cnt[r] < 0 {
                continue;
            }
            let mut s = -1.0;
            while s <= 1.0 {
                if self.sense[r] == b'=' || s == 1.0 {
                    let rc = self.side(r, s);
                    if rc != 0 {
                        return rc;
                    }
                }
                s += 2.0;
            }
        }
        self.work += mu * 5.0 * rows.len() as f64;
        0
    }

    /// One side `s·row ≤ s·rhs` of `5a9282`.
    fn side(&mut self, r: usize, s: f64) -> i32 {
        let mu = self.work_mult;
        let single = |st: &State, j: usize, a: f64| {
            st.col_cnt[j] == 1 && st.col_flag[j] == 0 && (st.vtype[j] == b'C' || (1.0 - a.abs()).abs() < 1e-10)
        };
        // rest: rhs less the others' least activity; top: the singletons' largest activity
        let (mut rest, mut top) = (s * self.rhs[r], 0.0);
        let (mut nsing, mut walked) = (0, 0);
        // all others integer with integral coefficients / no integer singleton / no continuous singleton
        let (mut int_rest, mut no_int, mut no_cont) = (true, true, true);
        let mut q = self.row_head[r];
        while q != NIL {
            let nd = self.nd[q as usize];
            q = nd.nr;
            walked += 1;
            if nd.col < 0 {
                continue;
            }
            let j = nd.col as usize;
            let a = s * nd.val;
            if single(self, j, a) {
                top += if a < 0.0 { self.lb[j] } else { self.ub[j] } * a;
                nsing += 1;
                if self.vtype[j] == b'C' {
                    no_cont = false;
                } else {
                    no_int = false;
                }
            } else {
                rest -= if a <= 0.0 { self.ub[j] } else { self.lb[j] } * a;
                if self.vtype[j] == b'C' || 1e-10 < a - (a + 1e-10).floor() {
                    int_rest = false;
                }
            }
        }
        self.work += mu * 5.0 * walked as f64;
        let b = self.rhs[r];
        let ok = 1 < nsing
            && nsing != self.row_cnt[r]
            && rest < 1e10
            && top < 1e10
            && rest + 1e-6 <= top
            && (no_int || (int_rest && b - (b + 1e-10).floor() <= 1e-10 && no_cont));
        if !ok {
            return 0;
        }
        let (mut cols, mut key, mut coef) = (Vec::new(), Vec::new(), Vec::new());
        let (mut q, mut walked) = (self.row_head[r], 0);
        while q != NIL {
            let nd = self.nd[q as usize];
            q = nd.nr;
            walked += 1;
            if nd.col >= 0 && single(self, nd.col as usize, s * nd.val) {
                let a = s * nd.val;
                cols.push(nd.col);
                coef.push(a);
                key.push(-self.obj[nd.col as usize] / a);
            }
        }
        self.work += mu * 10.0 * walked as f64;
        // the coefficient goes along with its column (Gurobi keeps it in a per-column scratch array)
        let mut ix: Vec<usize> = (0..cols.len()).collect();
        sort_asc(&mut key, &mut ix);
        let mut l = 0;
        while l < ix.len() {
            let j = cols[ix[l]] as usize;
            let d = ((self.ub[j] - self.lb[j]) * coef[ix[l]]).abs();
            if 1e10 < d {
                break;
            }
            top -= d;
            if top < rest - 1e-10 {
                break;
            }
            l += 1;
        }
        self.work = mu * 4.0 * l as f64 + self.work;
        for &k in &ix[..l] {
            let j = cols[k] as usize;
            let v = if coef[k] <= 0.0 { self.ub[j] } else { self.lb[j] };
            let rc = self.fix_col(j, v, true);
            if rc != 0 {
                return rc;
            }
        }
        self.work = self.work + mu * 3.0 * l as f64;
        0
    }
}
