//! `418542` dual aggregation (docs/m2_red_outer_b.md §3).
use crate::helpers::minsd;
use crate::state::{State, NIL};
use crate::Params;

impl State {
    /// `418542(st, env)`: count each column's `<` rows by coefficient sign (columns in an `=` row are out); then
    /// in every `<` row with finite activities, find the first binary that alone can make the row slack, and look
    /// for another column of that row that only this row keeps from moving where its cost wants: such a column
    /// would be tied to the binary (`3bb6fe`). One more row pass when fewer than (n − removed)/50 succeeded.
    /// shortcut: the aggregation `3bb6fe` (and the activity update of its rows) is not ported; no candidate ever
    /// reaches it on the 38 models (coverage: 0 calls), so a candidate counts as not aggregated. Port it with
    /// `5431ee`'s SCC branch, which calls it too.
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
        loop {
            let succ = 0;
            for i in 0..m {
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
                            bin = Some(k);
                            break;
                        }
                    }
                    before += 1;
                    q = nd.nr;
                }
                self.work += before as f64 * (4.0 * mu);
                let Some(k) = bin else { continue };
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
                        // 3bb6fe(±(u−l), l or u, env, st, c, k, .., pass == 0, &ok) would tie x_c to x_k here
                        continue;
                    }
                }
                self.work += (4.0 * mu) * walked as f64;
            }
            self.work += (5.0 * mu) * m as f64;
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
