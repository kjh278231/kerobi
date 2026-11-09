//! `59c193` VUB lifting (docs/m2/m2_red_roundend.md §4): two-entry `<` rows x·a + y·b <= rhs with one binary y that are
//! tight variable upper bounds of x. Two VUBs of the same x through the same binary: one goes (same sign) or x's
//! bound moves (opposite signs). Then VUBs of x whose binaries share a clique-like row (every binary uses up the
//! whole slack) merge into one row over all those binaries.
use crate::probe::sort_desc;
use crate::state::{State, NIL};
use crate::Params;

impl State {
    /// `415f4f(st, i)`: row `i` goes (-2), its live nodes die (columns lose an entry). No work, no marks.
    fn drop_row(&mut self, i: usize) {
        let mut q = self.row_head[i];
        while q != NIL {
            let nd = self.nd[q as usize];
            if nd.col >= 0 {
                self.col_cnt[nd.col as usize] -= 1;
                self.nd[q as usize].col = -1;
            }
            q = nd.nr;
        }
        self.row_cnt[i] = -2;
        self.rows_removed += 1;
    }

    /// `59c193(st, env, W)`. Returns 0 (or `3bea24`'s status).
    pub fn vub_lift(&mut self, _p: &Params) -> i32 {
        let (n, m) = (self.n, self.m);
        if n as i32 == self.nbin || self.nbin == 0 {
            return 0;
        }
        let mu = self.work_mult;
        let w0 = self.work;
        let budget = self.nnz as f64 * 10000.0 + 1e6;
        // per row: its binary node and its other node, while the row is a candidate VUB
        let (mut bn, mut cn) = (vec![NIL; m], vec![NIL; m]);
        let mut vubs = 0;
        for i in 0..m {
            if !(self.sense[i] == b'<' && self.row_cnt[i] == 2 && self.inf_min[i] == 0) {
                continue;
            }
            let (mut nb, mut nc) = (0, 0);
            let mut q = self.row_head[i];
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 {
                    if self.vtype[nd.col as usize] == b'B' {
                        bn[i] = q;
                        nb += 1;
                    } else {
                        cn[i] = q;
                        nc += 1;
                    }
                }
                q = nd.nr;
            }
            self.work += mu * 4.0 * 2.0;
            let mut keep = false;
            if nb == 1 && nc == 1 {
                let a = self.nd[cn[i] as usize].val;
                let x = self.nd[cn[i] as usize].col as usize;
                keep = !(0.001 > a.abs())
                    && !((self.nd[bn[i] as usize].val / a).abs() > 1e8)
                    && (a.abs() * (self.ub[x] - self.lb[x]) + self.act_min[i]).abs() < 1e-10;
            }
            if keep {
                vubs += 1;
            } else {
                bn[i] = NIL;
                cn[i] = NIL;
            }
        }
        self.work += (m as f64 * 3.0) * mu;
        if vubs < 2 {
            return 0;
        }
        let gone = if n > 0 { self.vub_pairs(&mut bn, &mut cn) } else { 0 };
        if vubs - gone < 2 {
            return 0;
        }
        // rows where every binary (|a|) equals the slack at least activity: sign 1 (`<`), or -1 at most activity of
        // an `=` row
        let mut sign = vec![0i32; m];
        let mut cliques = 0;
        for i in 0..m {
            if self.row_cnt[i] < 2 {
                continue;
            }
            for s in [1, -1] {
                let slack = if s == 1 {
                    if self.inf_min[i] != 0 {
                        continue;
                    }
                    self.act_min[i]
                } else {
                    if self.sense[i] != b'=' || self.inf_max[i] != 0 {
                        continue;
                    }
                    -self.act_max[i]
                };
                let (mut nb, mut all) = (0, true);
                let mut q = self.row_head[i];
                while q != NIL {
                    let nd = self.nd[q as usize];
                    if nd.col >= 0 && self.vtype[nd.col as usize] == b'B' {
                        nb += 1;
                        if (nd.val.abs() + slack).abs() > 1e-10 {
                            all = false;
                            break;
                        }
                    }
                    q = nd.nr;
                }
                self.work += mu * 4.0 * self.row_cnt[i] as f64;
                if all && nb > 1 {
                    sign[i] = s;
                    cliques += 1;
                    break;
                }
            }
        }
        self.work += (cliques as f64 + m as f64 * 5.0) * mu;
        if cliques == 0 || n == 0 {
            return 0;
        }
        let limit = w0 + budget;
        let mut cnt = vec![0i32; n.max(m)];
        for pass in 0..2 {
            let mut j = 0;
            while j < n {
                if self.work > limit {
                    break;
                }
                if j % 1000 == 0 {
                    // `3ba9de`: limits; stop is never set from Model.presolve()
                    self.budget_check();
                }
                if self.col_cnt[j] < 0 || self.vtype[j] == b'B' {
                    j += 1;
                    continue;
                }
                let want = |v: f64| if pass == 0 { v < 0.0 } else { v > 0.0 };
                // VUB rows of x_j on this pass's side
                let mut rows = Vec::new();
                let mut q = self.col_head[j];
                while q != NIL {
                    let nd = self.nd[q as usize];
                    if nd.col >= 0 && cn[nd.row as usize] != NIL && want(self.nd[cn[nd.row as usize] as usize].val) {
                        rows.push(nd.row as usize);
                    }
                    q = nd.nc;
                }
                self.work += mu * 5.0 * self.col_cnt[j] as f64;
                if rows.len() < 2 {
                    j += 1;
                    continue;
                }
                let rc = self.vub_merge(j, &rows, &sign, &mut cnt, &mut bn, &mut cn, &want);
                if rc != 0 {
                    return rc;
                }
                j += 1;
            }
        }
        0
    }

    /// `59c193` part 2: two VUBs of x through the same binary. Same sign: the weaker row goes. Opposite signs: x's
    /// bound moves by the smaller |b/a| (rounded for integers) and all of x's VUBs stop being candidates. Returns
    /// the number of rows that stopped being candidates.
    fn vub_pairs(&mut self, bn: &mut [u32], cn: &mut [u32]) -> i32 {
        let n = self.n;
        let mu = self.work_mult;
        // per binary and side of a: the last column seen (`P`) and its row (`Q`)
        let (mut p_neg, mut p_pos) = (vec![-1i32; n], vec![-1i32; n]);
        let (mut q_neg, mut q_pos) = (vec![0usize; n], vec![0usize; n]);
        let mut gone = 0;
        for j in 0..n {
            if self.col_cnt[j] < 0 || self.vtype[j] == b'B' || 1e-13 >= self.ub[j] - self.lb[j] {
                continue;
            }
            let mut q = self.col_head[j];
            while q != NIL {
                let nd = self.nd[q as usize];
                q = nd.nc;
                if nd.col < 0 {
                    continue;
                }
                let i = nd.row as usize;
                if cn[i] == NIL {
                    continue;
                }
                let a = self.nd[cn[i] as usize].val;
                let k = self.nd[bn[i] as usize].col as usize;
                let neg = 0.0 > a;
                let (pp, qq) = if neg { (&mut p_neg, &mut q_neg) } else { (&mut p_pos, &mut q_pos) };
                if !(j as i32 <= pp[k]) {
                    self.work += mu * 3.0;
                    qq[k] = i;
                    pp[k] = j as i32;
                    continue;
                }
                let i2 = qq[k];
                let b = self.nd[bn[i] as usize].val;
                let b2 = self.nd[bn[i2] as usize].val;
                if !(0.0 > b * b2) {
                    // same sign: drop the row with the larger |b/a|
                    let r2 = (b2 / self.nd[cn[i2] as usize].val).abs();
                    self.work += mu * 20.0;
                    let drop = if (b / a).abs() > r2 {
                        qq[k] = i;
                        i2
                    } else {
                        i
                    };
                    self.drop_row(drop);
                    cn[drop] = NIL;
                    gone += 1;
                    bn[drop] = NIL;
                    continue;
                }
                let r1 = (b / a).abs();
                let r2 = (b2 / self.nd[cn[i2] as usize].val).abs();
                let d = if r1 < r2 { r1 } else { r2 };
                let (l, u) = (self.lb[j], self.ub[j]);
                if neg {
                    let mut v = d + l;
                    self.work += mu * 20.0;
                    if self.vtype[j] != b'C' {
                        v = (v - 1e-5).ceil();
                    }
                    v = if v < u { v } else { u };
                    if !(v > l) {
                        continue;
                    }
                    self.bound_change(j, l, u, v, u);
                    self.tracker.mark_col(j as i32);
                    self.lb[j] = v;
                } else {
                    let mut v = u - d;
                    self.work += mu * 20.0;
                    if self.vtype[j] != b'C' {
                        v = (v + 1e-5).floor();
                    }
                    v = if v > l { v } else { l };
                    if !(u > v) {
                        continue;
                    }
                    self.bound_change(j, l, u, l, v);
                    self.tracker.mark_col(j as i32);
                    self.ub[j] = v;
                }
                let mut q = self.col_head[j];
                while q != NIL {
                    let nd = self.nd[q as usize];
                    if nd.col >= 0 && bn[nd.row as usize] != NIL {
                        gone += 1;
                        cn[nd.row as usize] = NIL;
                        bn[nd.row as usize] = NIL;
                    }
                    q = nd.nc;
                }
                self.bounds_tightened += 1;
                break;
            }
        }
        gone
    }

    /// `59c193` part 4 for column j and its VUB rows `rows` on one side: the clique-like rows met by those rows'
    /// binaries (with the sign that makes them a cover) at least twice, most met first; for each, the VUB rows whose
    /// binaries it covers merge into the first (scaled to |a| = 1, the others' binaries added, the others dropped).
    #[allow(clippy::too_many_arguments)]
    fn vub_merge(&mut self, j: usize, rows: &[usize], sign: &[i32], cnt: &mut [i32], bn: &mut [u32], cn: &mut [u32],
                 want: &dyn Fn(f64) -> bool) -> i32 {
        let mu = self.work_mult;
        let mut met: Vec<i32> = Vec::new();
        for &i in rows {
            let b = self.nd[bn[i] as usize];
            let k = b.col as usize;
            let mut q = self.col_head[k];
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 {
                    let r = nd.row as usize;
                    if 0.0 < nd.val * b.val * sign[r] as f64 {
                        if cnt[r] == 0 {
                            met.push(r as i32);
                        }
                        cnt[r] += 1;
                    }
                }
                q = nd.nc;
            }
            self.work += mu * 7.0 * self.col_cnt[k] as f64;
        }
        let all = met.len();
        let mut kept = Vec::new();
        for &r in &met {
            if cnt[r as usize] < 2 {
                cnt[r as usize] = 0;
            } else {
                kept.push(r);
            }
        }
        self.work = self.work + mu * (all as f64 + all as f64);
        // `ab72b6`: by count, descending (keys are small integers, exact as f64)
        let mut key: Vec<f64> = kept.iter().map(|&r| cnt[r as usize] as f64).collect();
        sort_desc(&mut key, &mut kept);
        for &r in &kept {
            cnt[r as usize] = 0;
        }
        self.work += (kept.len() as f64 + kept.len() as f64) * mu;
        for &rq in &kept {
            let rq = rq as usize;
            let mut q = self.row_head[rq];
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 {
                    cnt[nd.col as usize] = if sign[rq] as f64 * nd.val > 0.0 { 1 } else { -1 };
                }
                q = nd.nr;
            }
            self.work = mu * 4.0 * self.row_cnt[rq] as f64 + self.work;
            let mut grp = Vec::new();
            let mut q = self.col_head[j];
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 {
                    let i = nd.row as usize;
                    if cn[i] != NIL {
                        let b = self.nd[bn[i] as usize];
                        if cnt[b.col as usize] as f64 * b.val > 0.0 && want(self.nd[cn[i] as usize].val) {
                            grp.push(i);
                        }
                    }
                }
                q = nd.nc;
            }
            self.work = self.work + mu * 6.0 * self.col_cnt[j] as f64;
            let mut q = self.row_head[rq];
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 {
                    cnt[nd.col as usize] = 0;
                }
                q = nd.nr;
            }
            if grp.len() < 2 {
                continue;
            }
            let i0 = grp[0];
            let s = 1.0 / self.nd[cn[i0] as usize].val.abs();
            self.nd[bn[i0] as usize].val *= s;
            self.nd[cn[i0] as usize].val *= s;
            self.rhs[i0] *= s;
            for &it in &grp[1..] {
                let b = self.nd[bn[it] as usize];
                let f = b.val / self.nd[cn[it] as usize].val.abs();
                self.new_node(f, i0, b.col as usize);
                if 0.0 > f {
                    self.rhs[i0] += f;
                }
                self.drop_row(it);
                cn[it] = NIL;
                bn[it] = NIL;
            }
            self.activity(Some(&[i0 as i32]));
            self.tracker.mark_row(i0 as i32);
            self.mark_cols_of_row(i0);
            cn[i0] = NIL;
            bn[i0] = NIL;
            self.work += (grp.len() as f64 * 15.0 + 10.0) * mu;
        }
        0
    }
}
