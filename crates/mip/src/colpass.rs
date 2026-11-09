//! `3e39a6` column pass (docs/m2_red_inner.md §2): nearly fixed columns are fixed, empty columns go to their
//! cost-preferred bound, singleton columns are substituted out with their row (mode 0) or turn their `=` row into
//! an inequality (mode 1/2), and the rest get dual fixing (`3e2b99`).
use crate::helpers::{maxsd, minsd};
use crate::state::{State, INF, NIL};
use crate::Params;

/// Tolerances and limits of one `3e39a6` call.
struct Tols {
    ft: f64,
    ot: f64,
    agg: f64,
    psc: i32,
    /// `local_d0`: longest row a costed singleton column may slack out (mode 1/2)
    ld0: i32,
}

impl State {
    /// `3e39a6`. Returns 0, 3 (`bad_col`: lb > ub + ft), 5 (unbounded empty or dual-fixed column).
    pub fn col_pass(&mut self, p: &Params) -> i32 {
        let fixtol = match p.numeric_focus {
            1 => 1.0000000000000001e-11,
            f if f >= 2 => 1e-12,
            _ => 1e-10,
        };
        // MIP state: n+1 with PRESINGLECOL >= 4; else 10 when m > 10 and PRESINGLECOL is auto (no sub-MIP)
        let ld0 = if p.presinglecol >= 4 { self.n as i32 + 1 } else if self.m > 10 && p.presinglecol == -1 { 10 } else { 0 };
        let t = Tols { ft: p.feastol, ot: p.opttol, agg: p.agg_mult_limit, psc: p.presinglecol, ld0 };
        let list = self.cols_list();
        let mut rlist = Vec::new();
        for &jj in &list {
            let j = jj as usize;
            let cj = self.col_cnt[j];
            if cj < 0 {
                continue;
            }
            let (u, l) = (self.ub[j], self.lb[j]);
            if t.ft + u < l {
                self.bad_col = jj;
                return 3;
            }
            if u - l <= fixtol {
                let r = self.fix_near(j, t.ft);
                if r != 0 {
                    return r;
                }
                continue;
            }
            if p.dual_reductions == 0 {
                continue;
            }
            if self.col_flag[j] != 0 {
                let r = self.dual_fix(j, &t);
                if r != 0 {
                    return r;
                }
                continue;
            }
            if cj == 0 {
                let c = self.obj[j];
                let v = if c > t.ot {
                    if !(l > -INF) {
                        self.bad_col = jj;
                        return 5;
                    }
                    l
                } else if -t.ot > c {
                    if !(u < INF) {
                        self.bad_col = jj;
                        return 5;
                    }
                    u
                } else if l > -INF {
                    l
                } else if INF > u {
                    u
                } else {
                    0.0
                };
                self.push_fix(j, v);
                self.objcon = v * self.obj[j] + self.objcon;
                self.obj[j] = 0.0;
                self.col_cnt[j] = -1;
                continue;
            }
            let mut r11 = 0;
            if cj == 1 {
                match self.singleton_col(j, &t, &mut rlist) {
                    None => continue,
                    Some(x) => r11 = x,
                }
            }
            if self.col_cnt[j] + r11 > 1 {
                let r = self.dual_fix(j, &t);
                if r != 0 {
                    return r;
                }
            }
        }
        self.work += (6.0 * list.len() as f64) * self.work_mult;
        for &r in &rlist {
            self.row_int[r] = 0.0;
        }
        if !rlist.is_empty() {
            self.work += (rlist.len() as f64 * 3.0) * self.work_mult;
        }
        0
    }

    /// The singleton column branch: None when the column was substituted out or slacked (done), else Some(r11)
    /// (1 when its row is an inequality) for the dual-fixing test.
    fn singleton_col(&mut self, j: usize, t: &Tols, rlist: &mut Vec<usize>) -> Option<i32> {
        let vt = self.vtype[j];
        match t.psc {
            0 => return Some(0),
            1 if vt != b'C' => return Some(0),
            2 if vt == b'B' => return Some(0),
            _ => {}
        }
        let mut q = self.col_head[j];
        let mut dead = 0;
        while self.nd[q as usize].col < 0 {
            q = self.nd[q as usize].nc;
            dead += 1;
        }
        self.work += (dead as f64 + dead as f64) * self.work_mult;
        let node = q as usize;
        let (r, a) = (self.nd[node].row as usize, self.nd[node].val);
        let (c, u, l) = (self.obj[j], self.ub[j], self.lb[j]);
        if !(1e-6 <= a.abs() * (u - l)) && c.abs() * (u - l) > 1e-6 {
            return Some(0);
        }
        let eq = self.sense[r] == b'=';
        let r11 = if self.row_cnt[r] == 1 {
            if eq {
                return Some(0); // singleton `=` rows are 36c5cf's
            }
            1
        } else if eq {
            0
        } else {
            1
        };
        if r11 == 1 && ((c > 1e-9 && a > 0.0) || (c < -1e-9 && a < 0.0)) {
            return Some(1);
        }
        let (mut free, mut have, mut lo, mut hi) = (true, false, 0.0, 0.0);
        if l > -INF || INF > u {
            have = true;
            (lo, hi) = self.implied_col_bounds(j, 1, Some((r, a)));
            if (l > -INF && (l - 1e-9) > lo) || (INF > u && hi > 1e-9 + u) {
                free = false;
                let w = t.ft * 100.0;
                if lo > u + w || (l - w) > hi || !eq {
                    return Some(r11);
                }
            }
        }
        if vt != b'C' && !self.int_ok(j, r, a, rlist) {
            return Some(r11);
        }
        if free {
            // mode 0: x_j = (rhs − Σ a_k x_k)/a, the row goes
            if !self.growth_ok(j, r, a, t.agg) {
                return Some(r11);
            }
            self.nd[node].col = -1;
            let c0 = self.rhs[r] / a;
            self.push_subst_row(j, r, c0, a);
            self.objcon = c0 * c + self.objcon;
            let f = c / a;
            self.obj[j] = 0.0;
            self.row_cnt[r] = -3;
            self.col_cnt[j] = -3 - r as i32;
            let mut q = self.row_head[r];
            let mut walked = 0;
            while q != NIL {
                let at = q as usize;
                let nd = self.nd[at];
                q = nd.nr;
                walked += 1;
                if nd.col >= 0 {
                    let k = nd.col as usize;
                    self.tracker.mark_col(nd.col);
                    self.obj[k] -= nd.val * f;
                    self.col_cnt[k] -= 1;
                    self.nd[at].col = -1;
                }
            }
            self.work += (walked as f64 * 5.0) * self.work_mult;
            return None;
        }
        // mode 1/2: an `=` row whose one side does not imply x_j's bound: x_j becomes the row's slack
        let ac = c.abs();
        if ac >= 1e-10 && (1e-6 > a.abs() || ac > 1e6 || t.ld0 < self.row_cnt[r]) {
            return Some(r11);
        }
        let mode = if u >= INF {
            1
        } else if -INF >= l {
            2
        } else {
            if !have {
                (lo, hi) = self.implied_col_bounds(j, 1, Some((r, a)));
            }
            if 1e-9 + u > hi {
                1
            } else if lo > l - 1e-9 {
                2
            } else {
                return Some(r11);
            }
        };
        if !self.growth_ok(j, r, a, t.agg) {
            return Some(r11);
        }
        self.nd[node].col = -1;
        let c0 = self.rhs[r] / a;
        self.push_subst_row(j, r, c0, a);
        if ac >= 1e-10 {
            self.objcon = c0 * c + self.objcon;
            let f = c / a;
            self.obj[j] = 0.0;
            let mut q = self.row_head[r];
            let mut walked = 0;
            while q != NIL {
                let nd = self.nd[q as usize];
                q = nd.nr;
                walked += 1;
                if nd.col >= 0 {
                    self.obj[nd.col as usize] -= nd.val * f;
                }
            }
            self.work += (walked as f64 * 4.0) * self.work_mult;
        }
        self.rows_removed -= 1; // the row stays
        self.nd[node].col = j as i32;
        if (mode == 1 && 0.0 > a) || (mode == 2 && a > 0.0) {
            // the remaining bound's side becomes `<`
            let mut q = self.row_head[r];
            let mut walked = 0;
            while q != NIL {
                let at = q as usize;
                q = self.nd[at].nr;
                walked += 1;
                if self.nd[at].col >= 0 {
                    self.nd[at].val = -self.nd[at].val;
                }
            }
            self.work += (walked as f64 * 3.0) * self.work_mult;
            self.rhs[r] = -self.rhs[r];
            let (im, ix) = (self.inf_min[r], self.inf_max[r]);
            self.inf_max[r] = im;
            self.inf_min[r] = ix;
            let (am, ax) = (self.act_min[r], self.act_max[r]);
            self.act_max[r] = -am;
            self.act_min[r] = -ax;
        }
        self.tracker.mark_row(r as i32);
        self.mark_cols_of_row(r);
        self.sense[r] = b'<';
        let rc = self.fix_col(j, if mode == 1 { l } else { u }, false);
        self.col_cnt[j] = -3 - r as i32;
        if rc != 0 {
            return Some(rc); // shortcut: 3de18e returns 0 for linear models
        }
        None
    }

    /// Integer singleton column: does substituting it keep integrality (all of row `r`'s columns integer, the
    /// coefficients and rhs integral multiples of the row's smallest |a|)? Rows of 10 or more entries cache their
    /// answer in `row_int` for this call.
    fn int_ok(&mut self, j: usize, r: usize, a: f64, rlist: &mut Vec<usize>) -> bool {
        let _ = j;
        let frac = |q: f64| q - (1e-10 + q).floor();
        let mu = self.work_mult;
        if self.row_cnt[r] < 10 {
            if !(1e-10 >= frac(self.rhs[r] / a)) {
                return false;
            }
            let mut q = self.row_head[r];
            let mut cnt = 0;
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 && (self.vtype[nd.col as usize] == b'C' || 1e-10 < frac(nd.val / a)) {
                    self.work += (cnt as f64 * 4.0) * mu;
                    return false;
                }
                cnt += 1;
                q = nd.nr;
            }
            self.work += (cnt as f64 * 4.0) * mu;
            return true;
        }
        let g = self.row_int[r];
        if g != 0.0 {
            return !(a.abs() > g);
        }
        let mut dd = self.rhs[r].abs();
        if !(dd > 1e-10) {
            dd = 1e100;
        }
        let mut ok = true;
        let mut cnt = 0;
        let mut q = self.row_head[r];
        while q != NIL {
            let nd = self.nd[q as usize];
            if nd.col >= 0 {
                if self.vtype[nd.col as usize] == b'C' {
                    ok = false;
                    break;
                }
                dd = minsd(nd.val.abs(), dd);
            }
            cnt += 1;
            q = nd.nr;
        }
        self.work += (cnt as f64 * 4.0) * mu;
        if ok && 1e-10 >= frac(self.rhs[r] / dd) {
            let mut cnt = 0;
            let mut q = self.row_head[r];
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 && 1e-10 < frac(nd.val / dd) {
                    ok = false;
                    break;
                }
                cnt += 1;
                q = nd.nr;
            }
            self.work += (cnt as f64 * 4.0) * mu;
        } else {
            ok = false;
        }
        let g = if ok { dd + 1e-10 } else { 1e-10 };
        self.row_int[r] = g;
        rlist.push(r);
        !(a.abs() > g)
    }

    /// Multiplier growth check before substituting column `j` (coefficient `a`) out of row `r`: the other columns'
    /// accumulated multipliers stay within [1/AGGMULTLIMIT, AGGMULTLIMIT].
    fn growth_ok(&mut self, j: usize, r: usize, a: f64, agg: f64) -> bool {
        let (mut mn, mut mx, inv) = (1.0, 1.0, 1.0 / agg);
        let mut cnt = 0;
        let mut q = self.row_head[r];
        let mut ok = true;
        while q != NIL {
            let nd = self.nd[q as usize];
            let k = nd.col;
            if k >= 0 && k as usize != j {
                let k = k as usize;
                let t = (nd.val / a).abs();
                let v = minsd(self.mult_lo[k], self.mult_lo[j] * t);
                if mn > v {
                    mn = v;
                    if inv > v {
                        ok = false;
                        break;
                    }
                }
                let w = maxsd(self.mult_hi[k], self.mult_hi[j] * t);
                if w > mx {
                    mx = w;
                    if w > agg {
                        ok = false;
                        break;
                    }
                }
            }
            cnt += 1;
            q = nd.nr;
        }
        self.work += (cnt as f64 + cnt as f64) * self.work_mult;
        ok
    }

    /// `3e2b99(INF, OT, env, st, j)`: dual fixing. A column whose cost and every `<` row push it the same way goes
    /// to that (finite) bound with a record (`3de18e`); any `=` row stops it. Returns 0 or 5 (`bad_col`).
    /// shortcut: columns with flags (other than 0x40) and the infinite-bound side (`39e479`, removing the column and
    /// its rows) are not ported; neither occurs on the MIPLIB-38 defaults.
    fn dual_fix(&mut self, j: usize, t: &Tols) -> i32 {
        if self.col_flag[j] & !0x40 != 0 {
            return 0;
        }
        let (c, u, l) = (self.obj[j], self.ub[j], self.lb[j]);
        let mut down = false;
        if c >= -1e-10 {
            down = !(l > -INF) || l > -1e8;
        }
        let mut up = c <= 1e-10 && (u >= INF || u < 1e8);
        if !up && !down {
            return 0;
        }
        let mut q = self.col_head[j];
        let mut cnt = 0;
        while q != NIL {
            let nd = self.nd[q as usize];
            if nd.col >= 0 {
                if self.sense[nd.row as usize] == b'=' {
                    self.work += (cnt as f64 * 5.0) * self.work_mult;
                    return 0;
                }
                up &= nd.val <= 0.0;
                down &= 0.0 < nd.val;
            }
            cnt += 1;
            q = nd.nc;
        }
        self.work += (cnt as f64 * 5.0) * self.work_mult;
        if !up && !down {
            return 0;
        }
        if up {
            if u < INF {
                return self.fix_col(j, u, true);
            }
            if self.precrush != 0 {
                return 0;
            }
            if -t.ot > c {
                self.bad_col = j as i32;
                return 5;
            }
        } else {
            if l > -INF {
                return self.fix_col(j, l, true);
            }
            if self.precrush != 0 {
                return 0;
            }
            if c > t.ot {
                self.bad_col = j as i32;
                return 5;
            }
        }
        0
    }
}
