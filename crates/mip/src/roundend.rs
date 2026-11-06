//! Small round-end reductions of `3cd406` (docs/m2_red_roundend.md §1–3): `58d4ce` (Q diagonal; nothing for linear
//! models), `5a9282` dominated singleton columns of a row, `5ab429` cost singletons in a row with a coefficient gcd.
use crate::state::{State, INF, NIL};
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

/// `a7c550(a, b)`: gcd of two doubles by Euclid with a 1e-10 tolerance; 0 when either is below 1e-10 (or the result
/// below 1e-9); a value >= 1e100 counts as "none" (the other is returned).
pub(crate) fn fgcd(a: f64, b: f64) -> f64 {
    let (a, b) = (a.abs(), b.abs());
    if 1e-10 > a || 1e-10 > b {
        return 0.0;
    }
    if a >= 1e100 {
        return b;
    }
    if b >= 1e100 {
        return a;
    }
    let mut hi = if b > a { b } else { a };
    let mut lo = if a < b { a } else { b };
    while 1e-10 < lo {
        if lo > hi {
            return 0.0;
        }
        let t = hi - ((1e-10 + hi) / lo).floor() * lo;
        hi = lo;
        lo = if t > 0.0 { t } else { 0.0 };
    }
    if 1e-9 > hi { 0.0 } else { hi }
}

/// `3c5996(a, m)`: inverse of `a` modulo `m` by the extended Euclid algorithm (0 when gcd is 0).
pub(crate) fn mod_inverse(a: i32, m: i32) -> i32 {
    let (mut u, mut v, mut s0, mut s1, mut g) = (a, m, 1i32, 0i32, a);
    let mut x = 1;
    if m != 0 {
        loop {
            x = s1;
            g = v;
            let (q, r) = (u / v, u % v);
            let t = s0.wrapping_sub(q.wrapping_mul(x));
            s0 = x;
            s1 = t;
            u = v;
            v = r;
            if v == 0 {
                break;
            }
        }
    }
    if g == 0 { 0 } else { ((x as i64 + m as i64) as i32 as i64 % m as i64) as i32 }
}

impl State {
    /// `5ab429(st, env)`: a row whose columns are integer except for at most one cost singleton (|a| = 1) per cost
    /// direction, with an integral coefficient gcd g in [2, 1e6] (the residual rhs at the singletons' preferred bounds
    /// included): each integer singleton becomes x = r + g·y (`574764`). No work of its own.
    pub fn r5ab429(&mut self, p: &Params) -> i32 {
        let mut rc = 0;
        for r in 0..self.m {
            let mut slot = [-1i32; 2];
            let mut res = self.rhs[r];
            let mut g = 1e100;
            let mut q = self.row_head[r];
            let mut skip = false;
            while q != NIL {
                let nd = self.nd[q as usize];
                q = nd.nr;
                if nd.col < 0 {
                    continue;
                }
                let j = nd.col as usize;
                let (c, a) = (self.obj[j], nd.val);
                let side = (0.0 < c * a) as usize;
                if c.abs() > 1e-10 && self.col_cnt[j] == 1 && (a.abs() - 1.0).abs() < 1e-10 && self.col_flag[j] == 0
                    && slot[side] == -1
                {
                    res -= if c > 0.0 { self.lb[j] } else { self.ub[j] } * a;
                    slot[side] = nd.col;
                } else if self.vtype[j] == b'C' {
                    skip = true;
                    break;
                } else {
                    g = fgcd(g, a.abs());
                }
            }
            if skip {
                continue;
            }
            if res.abs() > 1e-10 {
                g = fgcd(g, res.abs());
            }
            if !(1.5 > g) && !(g > 1e6) && g - (g + 1e-10).floor() <= 1e-10 {
                let gi = (g + 0.5).floor() as i32;
                for &j in &slot {
                    if j < 0 || self.vtype[j as usize] == b'C' {
                        continue;
                    }
                    let ju = j as usize;
                    let v = if self.obj[ju] > 0.0 { self.lb[ju] } else { self.ub[ju] };
                    let rem = ((v as i64) % gi as i64) as i32;
                    let mut fixed = 0;
                    rc = self.mod_subst(p, ju, 1, gi, rem, &mut fixed);
                    if rc != 0 {
                        return rc;
                    }
                }
            }
        }
        rc
    }

    /// `574764(st, env, j, a, g, rem, &done)`: integer column j with a·x ≡ rem (mod g) becomes x = r + g·y (type 9
    /// record {j, r, g}): bounds, rows (rhs, activities), objective and objective constant move to y; `done` = 1
    /// unless infeasible (3, bad_col); a fixed y is removed with `3de18e`. r is shifted toward lb when that stays
    /// below a NumericFocus-dependent size.
    pub fn mod_subst(&mut self, p: &Params, j: usize, a: i32, g: i32, rem: i32, done: &mut i32) -> i32 {
        let rem = rem % g;
        *done = 0;
        let mut r: i64 = 0;
        if rem != 0 {
            let inv = mod_inverse(a.abs(), g) as i64;
            let rr = if rem < 0 { rem as i64 + g as i64 } else { rem as i64 };
            r = (inv * rr) % g as i64;
            if a < 0 {
                r = -r;
            }
        }
        let gf = g as f64;
        let mut rf = r as f64;
        if self.lb[j] > -INF && INF > self.ub[j] {
            let lim = match p.numeric_focus {
                1 => 1e4,
                2 => 100.0,
                x if x > 2 => 0.0,
                _ => 1e6,
            };
            let t = ((self.lb[j] - rf) / gf - 1e-10).ceil() * gf;
            if lim > t {
                r = (rf + t) as i64;
                rf = r as f64;
            }
        }
        self.push_t9(j as i32, rf, gf);
        self.mark_rows_of_col(j);
        self.tracker.mark_col(j as i32);
        let (l, u) = (self.lb[j], self.ub[j]);
        if l > -INF {
            let nl = ((l - rf) / gf - 1e-10).ceil();
            self.bound_change(j, l, u, gf * nl, u);
            self.lb[j] = nl;
        }
        let (l, u) = (self.lb[j], self.ub[j]);
        if INF > u {
            let nu = ((u - rf) / gf + 1e-10).floor();
            self.bound_change(j, l, u, l, gf * nu);
            self.ub[j] = nu;
        }
        let mut q = self.col_head[j];
        if r != 0 {
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 {
                    let t = rf * nd.val;
                    let i = nd.row as usize;
                    self.nd[q as usize].val = nd.val * gf;
                    self.act_max[i] += t;
                    self.act_min[i] += t;
                    self.rhs[i] -= t;
                }
                q = nd.nc;
            }
            self.objcon = rf * self.obj[j] + self.objcon;
            self.work = (self.col_cnt[j] as f64 * 6.0) * self.work_mult + self.work;
        } else {
            while q != NIL {
                self.nd[q as usize].val = gf * self.nd[q as usize].val;
                q = self.nd[q as usize].nc;
            }
            self.work = (self.col_cnt[j] as f64 * 2.0) * self.work_mult + self.work;
        }
        self.obj[j] = gf * self.obj[j];
        let (l, u) = (self.lb[j], self.ub[j]);
        if l > p.feastol + u {
            self.bad_col = j as i32;
            return 3;
        }
        *done = 1;
        if l == u {
            return self.fix_col(j, l, true);
        }
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gcd_and_inverse() {
        assert_eq!(fgcd(1e100, 4.0), 4.0);
        assert_eq!(fgcd(4.0, 6.0), 2.0);
        assert_eq!(fgcd(2.0, 7.0), 1.0);
        assert_eq!(fgcd(0.0, 3.0), 0.0);
        assert_eq!(fgcd(1.5, 3.0), 1.5);
        // 3·5 = 15 ≡ 1 (mod 7)
        assert_eq!(mod_inverse(3, 7), 5);
        assert_eq!(mod_inverse(1, 2), 1);
    }
}
