//! `374521` dual bounds (docs/m2_red_outer_a.md §6): the MIP form of LP `505077` + `4f84ef` in one function.
//! Continuous columns whose bounds are implied by their rows treat those bounds as free; each free bound gives the
//! column's reduced cost a sign; row duals start at y <= 0 (`<`) or free (`=`) and are tightened by those signs
//! through a circular queue. Columns whose reduced cost then has a strict sign go to that bound; `<` rows whose dual
//! is bounded away from 0 become `=`.
use crate::state::{guarded_add, State, INF, NIL};
use crate::Params;

impl State {
    /// `374521`. Returns 0 or 3 (a column's dual sign cannot hold: `bad_row` = the column; crossing row duals:
    /// `bad_col` = the row, as the original stores them).
    pub fn dual_bounds(&mut self, p: &Params) -> i32 {
        let (n, m, mu) = (self.n, self.m, self.work_mult);
        let (ft, ot) = (p.feastol, p.opttol);
        // shortcut: state+0x100 is taken as (NumericFocus > 0), as in the LP port; Method, Crossover and
        // SolutionTarget are not in Params.
        let implied_free = p.numeric_focus <= 0;
        let special = |s: &State, j: usize| s.vtype[j] != b'C' || s.col_flag[j] != 0;
        let mut j0 = 0;
        while j0 < n && special(self, j0) {
            j0 += 1;
        }
        self.work += (j0 as f64 + j0 as f64) * mu;
        if j0 == n {
            return 0;
        }
        // the original shifts special columns' counts below zero for the status steps
        let cnt: Vec<i32> = (0..n).map(|j| if special(self, j) { self.col_cnt[j] - (m as i32 + 1) } else { self.col_cnt[j] }).collect();
        self.work += mu * 3.0 * n as f64;
        let mut imax = self.inf_max.clone();
        let mut imin = self.inf_min.clone();
        // bit 0: lower bound free, bit 1: upper bound free (scratch +0x168, untouched for other columns)
        let mut st = vec![0u8; n];
        for j in 0..n {
            if 0 < cnt[j] {
                st[j] = if -INF < self.lb[j] { if self.ub[j] < INF { 0 } else { 2 } } else if self.ub[j] < INF { 1 } else { 3 };
            }
        }
        self.work += n as f64 * 4.0 * mu;
        if implied_free {
            self.work += (n as f64 + n as f64) * mu;
            self.work += m as f64 * mu;
            let mut order: Vec<usize> = (0..n).filter(|&j| 0 < cnt[j]).collect();
            order.sort_by_key(|&j| cnt[j]); // stable counting sort
            self.work += mu * 3.0 * n as f64;
            for upper in [true, false] {
                for &j in &order {
                    let sj = st[j];
                    if (upper && sj > 1) || (!upper && sj & 0xfd == 1) {
                        continue;
                    }
                    let (l, u) = (self.lb[j], self.ub[j]);
                    let far = if upper { l <= -INF } else { !(u < INF) };
                    let k = far as i32;
                    let (mut v, mut found) = (0, false);
                    let mut q = self.col_head[j];
                    while q != NIL {
                        let x = self.nd[q as usize];
                        if x.col >= 0 {
                            let (a, i) = (x.val, x.row as usize);
                            let eq = self.sense[i] == b'=';
                            found = if upper {
                                if a <= 0.0 {
                                    eq && imax[i] == k && (if far { -self.act_max[i] / a } else { l - self.act_max[i] / a }) < u + 1e-9
                                } else {
                                    imin[i] == k && (if far { -self.act_min[i] / a } else { l - self.act_min[i] / a }) < u + 1e-9
                                }
                            } else if 0.0 <= a {
                                eq && imax[i] == k && l - 1e-9 < if far { -self.act_max[i] / a } else { u - self.act_max[i] / a }
                            } else {
                                imin[i] == k && l - 1e-9 < if far { -self.act_min[i] / a } else { u - self.act_min[i] / a }
                            };
                            if found {
                                break;
                            }
                        }
                        v += 1;
                        q = x.nc;
                    }
                    self.work += mu * 8.0 * v as f64;
                    if !found {
                        continue;
                    }
                    st[j] = if sj == 0 { if upper { 2 } else { 1 } } else { 3 };
                    let mut v = 0;
                    let mut q = self.col_head[j];
                    while q != NIL {
                        let x = self.nd[q as usize];
                        if x.col >= 0 {
                            // the freed bound now counts as infinite in the activity it feeds
                            if (x.val <= 0.0) == upper { imin[x.row as usize] += 1 } else { imax[x.row as usize] += 1 }
                        }
                        v += 1;
                        q = x.nc;
                    }
                    self.work += mu * 5.0 * v as f64;
                }
                self.work += mu * 4.0 * order.len() as f64;
            }
        }
        let sense: Vec<u8> =
            (0..n).map(|j| if cnt[j] < 1 { b'F' } else { [b'F', b'>', b'<', b'='][st[j] as usize] }).collect();
        self.work += (n as f64 + n as f64) * mu;
        let mut ylo = vec![0.0f64; m];
        let mut yhi = vec![0.0f64; m];
        for i in 0..m {
            if self.row_cnt[i] >= 1 {
                ylo[i] = -INF;
                yhi[i] = if self.sense[i] == b'=' { INF } else { 0.0 };
            }
        }
        self.work += m as f64 * 4.0 * mu;
        self.work += mu * 3.0 * n as f64;
        // per column: range of −c + Σ a·y (finite part, infinite count), summed from 0 with −c added last
        let (mut dmax, mut dmin, mut cmax, mut cmin) = (vec![0.0f64; n], vec![0.0f64; n], vec![0i32; n], vec![0i32; n]);
        for j in 0..n {
            let mut v = 0;
            let mut q = self.col_head[j];
            while q != NIL {
                let x = self.nd[q as usize];
                if x.col >= 0 {
                    let (a, i) = (x.val, x.row as usize);
                    let (to_max, to_min) = if a <= 0.0 { (ylo[i], yhi[i]) } else { (yhi[i], ylo[i]) };
                    let max_fin = if a <= 0.0 { -INF < to_max } else { to_max < INF };
                    if max_fin { dmax[j] = guarded_add(dmax[j], to_max * a, true) } else { cmax[j] += 1 }
                    let min_fin = if a <= 0.0 { to_min < INF } else { -INF < to_min };
                    if min_fin { dmin[j] = guarded_add(dmin[j], a * to_min, false) } else { cmin[j] += 1 }
                }
                v += 1;
                q = x.nc;
            }
            dmax[j] = guarded_add(dmax[j], -self.obj[j], true);
            dmin[j] = guarded_add(dmin[j], -self.obj[j], false);
            self.work += mu * 8.0 * v as f64;
        }
        self.work += n as f64 * 6.0 * mu;
        let mut queue: Vec<usize> = (0..n).filter(|&j| self.col_cnt[j] >= 0).collect();
        let cntq = queue.len();
        queue.resize(n, 0);
        let mut inq = vec![true; n];
        self.work += n as f64 * 3.0 * mu;
        let mut pops = 0;
        if cntq >= 2 {
            let (mut head, mut tail, mut rounds) = (0usize, cntq - 1, 0);
            let push = |k: usize, inq: &mut [bool], queue: &mut [usize], tail: &mut usize| {
                if !inq[k] {
                    *tail += 1;
                    if *tail == n {
                        *tail = 0;
                    }
                    inq[k] = true;
                    queue[*tail] = k;
                }
            };
            loop {
                let j = queue[head];
                let sj = sense[j];
                if sj & 0xfe == b'<' && cmin[j] == 0 && ft < dmin[j] {
                    self.bad_row = j as i32;
                    return 3;
                }
                if (sj == b'=' || sj == b'>') && cmax[j] == 0 && dmax[j] < -ft {
                    self.bad_row = j as i32;
                    return 3;
                }
                let mut v = 0;
                let mut q = self.col_head[j];
                while q != NIL {
                    let x = self.nd[q as usize];
                    q = x.nc;
                    v += 1;
                    if x.col < 0 {
                        continue;
                    }
                    let (i, a) = (x.row as usize, x.val);
                    let (yh, yl) = (yhi[i], ylo[i]);
                    let (mut nl, mut nh) = (yl, yh);
                    if sj & 0xfe == b'<' {
                        let (dm, k) = (dmin[j], cmin[j]);
                        let d0 = if 0.0 <= dm { 0.0 } else { dm };
                        if a <= 1e-8 {
                            if a < -1e-8 {
                                if k == 0 {
                                    nl = yh - d0 / a;
                                } else if k == 1 && INF <= yh {
                                    nl = -dm / a;
                                }
                            }
                        } else if k == 0 {
                            nh = yl - d0 / a;
                        } else if k == 1 && yl <= -INF {
                            nh = -dm / a;
                        }
                    }
                    let mut th = nh;
                    if sj == b'=' || sj == b'>' {
                        let (a2, dx, k) = (-a, dmax[j], cmax[j]);
                        let d0 = if 0.0 <= -dx { 0.0 } else { -dx };
                        let mut tl = yl;
                        th = yh;
                        if a2 <= 1e-8 {
                            if a2 < -1e-8 {
                                if k == 0 {
                                    tl = yh - d0 / a2;
                                } else if k == 1 && INF <= yh {
                                    tl = dx / a2;
                                }
                            }
                        } else if k == 0 {
                            th = yl - d0 / a2;
                        } else if k == 1 && yl <= -INF {
                            th = dx / a2;
                        }
                        if tl <= nl {
                            tl = nl;
                        }
                        nl = tl;
                        if nh <= th {
                            th = nh;
                        }
                    }
                    let mut yl_now = yl;
                    if -1e8 < nl && ft * 1000.0 + yl < nl {
                        if !(nl <= yh + ft) {
                            self.bad_col = i as i32;
                            return 3;
                        }
                        let finite = -INF < yl;
                        let w = self.shift_dual(i, nl, yl, finite, false, &mut dmax, &mut dmin, &mut cmax, &mut cmin,
                                                &mut |k| push(k, &mut inq, &mut queue, &mut tail));
                        self.work += w * if finite { 5.0 } else { 6.0 } * mu;
                        ylo[i] = nl;
                        yl_now = nl;
                    }
                    let yh2 = yhi[i];
                    if th < 1e8 && th < yh2 - ft * 1000.0 {
                        if th < yl_now - ft {
                            self.bad_col = i as i32;
                            return 3;
                        }
                        let finite = yh2 < INF;
                        let w = self.shift_dual(i, th, yh2, finite, true, &mut dmax, &mut dmin, &mut cmax, &mut cmin,
                                                &mut |k| push(k, &mut inq, &mut queue, &mut tail));
                        self.work += w * if finite { 5.0 } else { 6.0 } * mu;
                        yhi[i] = th;
                    }
                }
                self.work += v as f64 * 10.0 * mu;
                head += 1;
                inq[j] = false;
                if head == n {
                    rounds += 1;
                    head = 0;
                    if 10 < rounds {
                        break;
                    }
                }
                pops += 1;
                if head == tail {
                    break;
                }
            }
        }
        if cntq > 0 {
            self.work += pops as f64 * 7.0 * mu;
        }
        // shortcut: the Presolve=2 branch ('F' columns to a bound with reduced cost about 0) is not ported
        for j in 0..n {
            if !(0 < self.col_cnt[j]) {
                continue;
            }
            let (u, l) = (self.ub[j], self.lb[j]);
            if !(1e-10 < u - l && self.col_flag[j] == 0) {
                continue;
            }
            let rc = if cmax[j] != 0 || -ot <= dmax[j] || l <= -INF {
                if cmin[j] == 0 && ot < dmin[j] && u < INF { self.fix_col(j, u, true) } else { 0 }
            } else {
                self.fix_col(j, l, true)
            };
            if rc != 0 {
                return rc;
            }
        }
        self.work += n as f64 * 7.0 * mu;
        for i in 0..m {
            if 0 < self.row_cnt[i] && (ft < ylo[i] || yhi[i] < -ft) && self.sense[i] != b'=' {
                self.ineq_to_eq += 1;
                self.sense[i] = b'=';
                self.tracker.mark_row(i as i32);
            }
        }
        self.work += m as f64 * 3.0 * mu;
        0
    }

    /// Row `i`'s lower (`upper` false) or upper dual bound moves from `old` to `new`; the live columns of the row get
    /// their dual ranges moved and are queued. A finite old bound moves by the difference (guarded), an infinite one
    /// replaces an infinite count by `b·new` (plain addition). Returns the nodes walked.
    #[allow(clippy::too_many_arguments)]
    fn shift_dual(&mut self, i: usize, new: f64, old: f64, finite: bool, upper: bool, dmax: &mut [f64], dmin: &mut [f64],
                  cmax: &mut [i32], cmin: &mut [i32], push: &mut dyn FnMut(usize)) -> f64 {
        let mut walked = 0;
        let mut q = self.row_head[i];
        while q != NIL {
            let x = self.nd[q as usize];
            q = x.nr;
            walked += 1;
            if x.col < 0 {
                continue;
            }
            let (k, b) = (x.col as usize, x.val);
            // lower dual feeds the max for b <= 0, the min for b > 0; the upper dual the other way
            let on_max = (b <= 0.0) != upper;
            if finite {
                let t = if upper { -((old - new) * b) } else { (new - old) * b };
                if on_max { dmax[k] = guarded_add(dmax[k], t, true) } else { dmin[k] = guarded_add(dmin[k], t, false) }
            } else if on_max {
                cmax[k] -= 1;
                dmax[k] = b * new + dmax[k];
            } else {
                cmin[k] -= 1;
                dmin[k] = b * new + dmin[k];
            }
            push(k);
        }
        walked as f64
    }
}
