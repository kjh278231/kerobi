//! `418542` dual aggregation (docs/m2/m2_red_outer_b.md §3).
use crate::helpers::minsd;
use crate::state::{State, NIL};
use crate::Params;

impl State {
    /// `3bb6fe`: test, then substitute x_c = b + x*x_k. The first pass only tests growth.
    /// Dead nodes stay in both lists; the caller recomputes the marked rows before reading activities.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn dual_agg_subst(&mut self, p: &Params, c: usize, k: usize, x: f64, b: f64,
                               buf: &mut [u32], mark: &mut [bool], test: bool) -> bool {
        let ax = x.abs();
        if self.mult_lo[k] * ax < 1.0 / p.agg_mult_limit || p.agg_mult_limit < self.mult_hi[k] * ax {
            return false;
        }
        if test {
            return true;
        }
        let mu = self.work_mult;
        let mut q = self.col_head[c];
        let mut walked = 0;
        while q != NIL {
            let nd = self.nd[q as usize];
            if nd.col >= 0 { buf[nd.row as usize] = NIL; }
            walked += 1;
            q = nd.nc;
        }
        self.work = (walked as f64 + walked as f64) * mu + self.work;
        let mut q = self.col_head[k];
        let mut walked = 0;
        while q != NIL {
            let nd = self.nd[q as usize];
            if nd.col >= 0 { buf[nd.row as usize] = q; }
            walked += 1;
            q = nd.nc;
        }
        self.work = (walked as f64 + walked as f64) * mu + self.work;
        let mut q = self.col_head[c];
        let (mut walked, mut added) = (0, 0);
        while q != NIL {
            let nd = self.nd[q as usize];
            if nd.col >= 0 && buf[nd.row as usize] == NIL {
                buf[nd.row as usize] = self.new_node(0.0, nd.row as usize, k);
                added += 1;
            }
            walked += 1;
            q = nd.nc;
        }
        self.work = (added as f64 * 6.0 + walked as f64 + walked as f64) * mu + self.work;
        let mut q = self.col_head[c];
        let mut walked = 0;
        while q != NIL {
            let nd = self.nd[q as usize];
            if nd.col >= 0 {
                let r = nd.row as usize;
                let dest = buf[r] as usize;
                self.nd[dest].val = nd.val * x + self.nd[dest].val;
                self.rhs[r] -= nd.val * b;
                if self.nd[dest].val.abs() < 1e-10 {
                    self.row_cnt[r] -= 1;
                    self.col_cnt[k] -= 1;
                    self.nd[dest].col = -1;
                }
                self.row_cnt[r] -= 1;
                self.col_cnt[c] -= 1;
                self.nd[q as usize].col = -1;
                mark[r] = true;
                self.tracker.mark_row(r as i32);
            }
            walked += 1;
            q = nd.nc;
        }
        self.work = walked as f64 * 6.0 * mu + self.work;
        self.obj[k] = self.obj[c] * x + self.obj[k];
        self.objcon = self.obj[c] * b + self.objcon;
        self.obj[c] = 0.0;
        self.col_cnt[c] = -3;
        self.cols_removed += 1;
        self.push_agg(c, k, x, b);
        self.tracker.mark_col(k as i32);
        true
    }

    /// `418542(st, env)`: count each column's `<` rows by coefficient sign (columns in an `=` row are out); then
    /// in every `<` row with finite activities, find the first binary that alone can make the row slack, and look
    /// for another column of that row that only this row keeps from moving where its cost wants: such a column
    /// would be tied to the binary (`3bb6fe`). One more row pass when fewer than (n − removed)/50 succeeded.
    pub fn dual_agg(&mut self, p: &Params) -> i32 {
        let ft = minsd(p.feastol, 1e-6);
        let (n, m, mu) = (self.n, self.m, self.work_mult);
        let mut neg = vec![0i32; n];
        let mut pos = vec![0i32; n];
        for j in 0..n {
            let mut walked = 0;
            let mut q = self.col_head[j];
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 {
                    let k = nd.col as usize;
                    if self.sense[nd.row as usize] == b'=' {
                        pos[k] = 2_000_000_000;
                        neg[k] = 2_000_000_000;
                        break;
                    }
                    if nd.val > 0.0 { pos[k] += 1 } else { neg[k] += 1 }
                }
                walked += 1;
                q = nd.nc;
            }
            self.work += walked as f64 * (3.0 * mu);
        }
        self.work += mu * n as f64;
        let mut pass = 0;
        let mut mark = vec![false; m];
        let mut buf = vec![NIL; m];
        loop {
            let mut succ = 0;
            for i in 0..m {
                if mark[i] {
                    self.activity(Some(&[i as i32]));
                    mark[i] = false;
                }
                if self.sense[i] == b'=' || self.row_cnt[i] <= 0 || self.inf_min[i] != 0 || self.inf_max[i] != 0 {
                    continue;
                }
                let mut before = 0;
                let mut bin = None;
                let mut q = self.row_head[i];
                while q != NIL {
                    let nd = self.nd[q as usize];
                    if nd.col >= 0 {
                        let k = nd.col as usize;
                        let aa = nd.val.abs();
                        if self.vtype[k] != b'C'
                            && self.lb[k] == 0.0
                            && self.ub[k] == 1.0
                            && self.act_min[i] + aa >= -1e-10
                            && ft >= self.act_max[i] - aa
                        {
                            bin = Some((k, nd.val));
                            break;
                        }
                    }
                    before += 1;
                    q = nd.nr;
                }
                self.work += before as f64 * (4.0 * mu);
                let Some((k, ak)) = bin else { continue };
                let mut walked = 0;
                let mut q = self.row_head[i];
                while q != NIL {
                    let nd = self.nd[q as usize];
                    q = nd.nr;
                    walked += 1;
                    if nd.col < 0 || nd.col as usize == k {
                        continue;
                    }
                    let c = nd.col as usize;
                    let d = self.ub[c] - self.lb[c];
                    let a = nd.val;
                    if !(d > 1e-6) || !(a.abs() * d > 1e-6) || self.col_flag[c] != 0 {
                        continue;
                    }
                    let up = a > 1e-4 && 1e-10 >= self.obj[c] && pos[c] == 1;
                    let dn = -1e-4 > a && self.obj[c] >= -1e-10 && neg[c] == 1;
                    if up || dn {
                        let sign = if up { 1.0 } else { -1.0 };
                        let (x, b) = if 0.0 > sign * ak { (d, self.lb[c]) } else { (-d, self.ub[c]) };
                        if self.dual_agg_subst(p, c, k, x, b, &mut buf, &mut mark, pass == 0) {
                            succ += 1;
                            if pass > 0 {
                                pos[k] = 2_000_000_000;
                                neg[k] = 2_000_000_000;
                            }
                        }
                    }
                }
                self.work += (4.0 * mu) * walked as f64;
            }
            self.work += (5.0 * mu) * m as f64;
            if pass == 1 && succ > 0 {
                for i in 0..m {
                    if mark[i] {
                        self.activity(Some(&[i as i32]));
                        mark[i] = false;
                    }
                }
                self.work += m as f64 * mu;
            }
            if succ < (self.n as i32 - self.cols_removed) / 50 {
                break;
            }
            pass += 1;
            if pass >= 2 {
                break;
            }
        }
        0
    }
}
