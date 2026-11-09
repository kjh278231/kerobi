//! `5a1293(st, env)` (PREDUALREDUNDANT, docs/m2/m2_red_roundend_b.md §4): a `<` row of a column with c·a > 0 that
//! is redundant when the column sits at its cheap bound, and dominated by the column's other rows of opposite sign
//! (`a7cec0` on the scaled sum), would be dropped (`39fb6d` record + `415f4f`).
use crate::knapsack::act_sum;
use crate::state::{State, NIL};
use crate::Params;

impl State {
    pub fn dual_redundant(&mut self, _p: &Params) -> i32 {
        let (n, mu) = (self.n, self.work_mult);
        if n == 0 {
            return 0;
        }
        let lim = self.work + self.nnz as f64 * 1000.0 + 1e6;
        let mut mark = vec![-1i32; n];
        for j in 0..n {
            let cc = self.col_cnt[j];
            if !(2..=1000).contains(&cc) {
                continue;
            }
            if lim < self.work {
                self.work += mu * 3.0 * j as f64;
                return 0;
            }
            if self.col_flag[j] & !0x40 != 0 {
                continue;
            }
            // pass 1: no `=` row, at most 1000 row entries in all
            let (mut cnt, mut tot, mut eq) = (0, 0i64, false);
            let mut q = self.col_head[j];
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 {
                    if self.sense[nd.row as usize] == b'=' {
                        eq = true;
                        break;
                    }
                    tot += self.row_cnt[nd.row as usize] as i64;
                }
                cnt += 1;
                q = nd.nc;
            }
            self.work += mu * 3.0 * cnt as f64;
            if eq || 1000 < tot {
                continue;
            }
            let c = self.obj[j];
            let mut ne = 0;
            let mut qe = self.col_head[j];
            while qe != NIL {
                let e = self.nd[qe as usize];
                qe = e.nc;
                ne += 1;
                let a = e.val;
                if e.col < 0 || a.abs() < 1e-4 || !((1e-6 < c && 0.0 < a) || (c < -1e-6 && a < 0.0)) {
                    continue;
                }
                let r = e.row as usize;
                let (l, u) = (self.lb[j], self.ub[j]);
                let mut ok = true;
                if a <= 0.0 {
                    if u < 1e30 {
                        let inf = self.inf_max[r];
                        ok = if inf == 0 {
                            (u - l) * a + self.act_max[r] < 1e-6
                        } else if inf == 1 && l <= -1e30 {
                            u * a + self.act_max[r] < 1e-6
                        } else {
                            false
                        };
                    }
                } else if -1e30 < l {
                    let inf = self.inf_max[r];
                    ok = if inf == 0 {
                        self.act_max[r] - (u - l) * a < 1e-6
                    } else if inf == 1 && 1e30 <= u {
                        a * l + self.act_max[r] < 1e-6
                    } else {
                        false
                    };
                }
                let mut nf = 0;
                let mut qf = self.col_head[j];
                while qf != NIL && ok {
                    let f = self.nd[qf as usize];
                    qf = f.nc;
                    if 0 <= f.col && f.val.abs() < 1e-4 {
                        ok = false;
                        break;
                    }
                    nf += 1;
                    if f.col < 0 {
                        continue;
                    }
                    let b = f.val;
                    if 0.0 <= a * b {
                        continue;
                    }
                    let inv = (1.0 / b).abs();
                    let rf = f.row as usize;
                    if self.vtype[j] != b'C' {
                        let x = self.rhs[rf] * inv;
                        let fr = x - x.floor();
                        let mut integral = 1e-10 >= fr || 0.9999999999 <= fr;
                        if integral {
                            let mut k = 0;
                            let mut x = self.row_head[rf];
                            while x != NIL {
                                let g = self.nd[x as usize];
                                if g.col >= 0 {
                                    let y = g.val * inv;
                                    let fr = y - y.floor();
                                    if self.vtype[g.col as usize] == b'C' || !(fr < 1e-10 || 0.9999999999 < fr) {
                                        integral = false;
                                        break;
                                    }
                                }
                                k += 1;
                                x = g.nr;
                            }
                            self.work += (k as f64 + k as f64) * mu;
                        }
                        if !integral {
                            ok = false;
                            break;
                        }
                    }
                    // row e scaled by 1/|a| plus row f scaled by 1/|b| must not exceed their rhs sum (`a7cec0`)
                    let (mut ind, mut val) = (vec![], vec![]);
                    let mut x = self.row_head[rf];
                    while x != NIL {
                        let g = self.nd[x as usize];
                        if g.col >= 0 {
                            mark[g.col as usize] = -1;
                        }
                        x = g.nr;
                    }
                    self.work += mu * 2.0 * self.row_cnt[rf] as f64;
                    let ia = (1.0 / a).abs();
                    let mut x = self.row_head[r];
                    while x != NIL {
                        let g = self.nd[x as usize];
                        if g.col >= 0 {
                            mark[g.col as usize] = ind.len() as i32;
                            ind.push(g.col);
                            val.push(g.val * ia);
                        }
                        x = g.nr;
                    }
                    let he = self.rhs[r];
                    self.work += mu * 6.0 * self.row_cnt[r] as f64;
                    let mut x = self.row_head[rf];
                    while x != NIL {
                        let g = self.nd[x as usize];
                        if g.col >= 0 {
                            let k = g.col as usize;
                            if mark[k] < 0 {
                                mark[k] = ind.len() as i32;
                                ind.push(g.col);
                                val.push(0.0);
                            }
                            val[mark[k] as usize] += g.val * inv;
                        }
                        x = g.nr;
                    }
                    let hf = self.rhs[rf];
                    self.work += mu * 6.0 * self.row_cnt[rf] as f64;
                    let mn = act_sum(&ind, &val, &self.lb, &self.ub, -1.0, &mut self.work);
                    ok = -mn <= inv * hf + he * ia + 1e-6;
                    self.work += (ind.len() as f64 + ind.len() as f64) * mu;
                }
                self.work += mu * 3.0 * nf as f64;
                if ok {
                    // shortcut: `39fb6d(env, st, j, row, a < 0)` (record) and `415f4f` (drop the row) are never
                    // reached on the 38 models; not ported
                }
            }
            self.work += mu * 8.0 * ne as f64;
        }
        self.work += mu * 3.0 * n as f64;
        0
    }
}
