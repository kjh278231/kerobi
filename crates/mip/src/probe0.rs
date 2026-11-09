//! `423e35(st, env, T10, probe mode)` (PREPROBE0): one-level probing of the [0, 1] integer columns on two bound
//! copies (docs/m2_red_roundend_b.md §2). x = 0 and x = 1 are propagated through the column's rows only (`426093`,
//! `427846`); an infeasible side fixes x, bounds implied by both sides are tightened, a `<` row redundant on one
//! side gets its x coefficient strengthened (an `=` row fixes x), and columns fixed on both sides are fixed.
use crate::driver::Timer;
use crate::probe::{sort_desc, TimerView};
use crate::probe_engine::Rng;
use crate::state::{State, NIL};
use crate::subst::sort_asc;
use crate::Params;

/// A bound copy (`424c23`, 0x1e0 bytes): bounds [lb, ub], per side the changed columns and their positions.
struct Ctx {
    bnd: [Vec<f64>; 2],
    list: [Vec<usize>; 2],
    pos: [Vec<i32>; 2],
}

impl Ctx {
    fn set(&mut self, k: usize, j: usize, v: f64) {
        if self.pos[k][j] == -1 {
            self.pos[k][j] = self.list[k].len() as i32;
            self.list[k].push(j);
        }
        self.bnd[k][j] = v;
    }
}

impl State {
    /// `427846(dir, st, ctx, node)`: the node's row (dir 1: a·x ≤ rhs, dir −1: the `=` row's other side) under the
    /// copy's bounds tightens the bounds of its other columns; true if infeasible.
    fn row_prop(&mut self, c: &mut Ctx, q: u32, dir: f64) -> bool {
        let mu = self.work_mult;
        let r = self.nd[q as usize].row as usize;
        let (mut hi, mut lo, mut cnt) = (0.0f64, 0.0f64, 0);
        let mut x = self.row_head[r];
        while x != NIL {
            let nd = self.nd[x as usize];
            if nd.col >= 0 {
                let (a, j) = (nd.val, nd.col as usize);
                let (l, u) = (c.bnd[0][j], c.bnd[1][j]);
                if a <= 0.0 {
                    hi = if l <= -1e30 { 1e100 } else { hi + l * a };
                    lo = if 1e30 <= u { -1e100 } else { lo + u * a };
                } else {
                    lo = if l <= -1e30 { -1e100 } else { lo + l * a };
                    hi = if 1e30 <= u { 1e100 } else { hi + u * a };
                }
            }
            cnt += 1;
            x = nd.nr;
        }
        self.work += mu * 5.0 * cnt as f64;
        let rhs = self.rhs[r];
        let (lr, hr) = (lo - rhs, hi - rhs);
        let slack = if dir != 1.0 { hr } else { -lr };
        if slack < -1e-4 {
            return true;
        }
        let slack = if slack > 0.0 { slack } else { 0.0 };
        let rng = (hr - lr).max(100.0) * 1e-6;
        let tol = 1e-10 * rng / 1e-6;
        let (mut cnt, mut inf) = (0, false);
        let mut x = self.row_head[r];
        while x != NIL {
            let nd = self.nd[x as usize];
            if nd.col >= 0 && x != q {
                let j = nd.col as usize;
                let (l, u) = (c.bnd[0][j], c.bnd[1][j]);
                let a = nd.val * dir;
                if slack < ((u - rng) - l) * a.abs() {
                    let int = self.vtype[j] != b'C';
                    if tol < a {
                        let mut v = slack / a + l;
                        if int {
                            v = (v + 1e-5).floor();
                        }
                        if v < l - 1e-6 {
                            inf = true;
                            break;
                        }
                        if v <= u - 1e-6 && v.abs() <= 1e10 {
                            c.set(1, j, v);
                        }
                    } else if a < -tol {
                        let mut v = slack / a + u;
                        if int {
                            v = (v - 1e-5).ceil();
                        }
                        if u + 1e-6 < v {
                            inf = true;
                            break;
                        }
                        if l + 1e-6 <= v && v.abs() <= 1e10 {
                            c.set(0, j, v);
                        }
                    }
                }
            }
            cnt += 1;
            x = nd.nr;
        }
        self.work += mu * 4.0 * cnt as f64;
        inf
    }

    /// `426093(obj, st, ctx, j, skip, v, flag)`: x_j = v in the copy, then two passes of `427846` over x_j's rows
    /// (rows marked in `skip` left out); true if infeasible.
    /// shortcut: the objective test (flag from `4f2c32`) compares with the cutoff, 1e100 without a MIP context, and
    /// can never fire; port it with a MIP context.
    fn col_prop(&mut self, c: &mut Ctx, j: usize, skip: &[i32], v: f64) -> bool {
        let mu = self.work_mult;
        let mut inf = false;
        if v == 0.0 {
            if v < c.bnd[0][j] - 1e-6 {
                inf = true;
            } else if !(c.bnd[1][j] - 1e-6 < v) {
                c.set(1, j, v);
            }
        } else if c.bnd[1][j] + 1e-6 < v {
            inf = true;
        } else if !(v < c.bnd[0][j] + 1e-6) {
            c.set(0, j, v);
        }
        for _ in 0..2 {
            let mut cnt = 0;
            let mut q = self.col_head[j];
            while q != NIL && !inf {
                let nd = self.nd[q as usize];
                if nd.col >= 0 && skip[nd.row as usize] == 0 {
                    let r = nd.row as usize;
                    if self.inf_min[r] == 0 {
                        inf = self.row_prop(c, q, 1.0);
                    }
                    if self.sense[r] == b'=' && self.inf_max[r] == 0 && self.row_prop(c, q, -1.0) {
                        inf = true;
                        cnt += 1;
                        break;
                    }
                }
                cnt += 1;
                q = nd.nc;
            }
            self.work += mu * 5.0 * cnt as f64;
        }
        inf
    }

    /// `423e35(st, env, T10, probe)`; `t` is T10 before its start.
    pub fn probe0(&mut self, _p: &Params, probe: i32, t: Timer) -> i32 {
        let mut t = t;
        t.start(self.work);
        let tv = TimerView { acc: t.acc(), running: t.running(), limit: t.limit };
        let (n, m, mu) = (self.n, self.m, self.work_mult);
        let mut w70 = self.work;
        let mut cx: [Ctx; 2] = std::array::from_fn(|_| {
            self.work += n as f64 * 3.0 * mu;
            Ctx { bnd: [self.lb.clone(), self.ub.clone()], list: [vec![], vec![]], pos: [vec![-1; n], vec![-1; n]] }
        });
        // rows by (binaries × length), ascending, up to 3·nnz + 1000 (30·nnz + 10000 for probe mode ≥ 2); the rest
        // are skipped
        let mut skip = vec![0i32; m];
        let budget = if probe < 2 { (self.nnz * 3) as f64 + 1000.0 } else { (self.nnz * 30) as f64 + 10000.0 };
        let (mut rk, mut ri) = (vec![], vec![]);
        for i in 0..m {
            let rc = self.row_cnt[i];
            if rc <= 0 {
                continue;
            }
            let mut s = 0.0;
            let mut q = self.row_head[i];
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col != -1 && self.vtype[nd.col as usize] == b'B' {
                    s += rc as f64;
                }
                q = nd.nr;
            }
            self.work += mu * 3.0 * rc as f64;
            if 0.0 < s {
                ri.push(i);
                rk.push(s);
            }
        }
        self.work += m as f64 * 4.0 * mu;
        sort_asc(&mut rk, &mut ri);
        let mut acc = 0.0;
        let cut = rk.iter().position(|&k| {
            acc += k;
            budget < acc
        });
        for &i in &ri[cut.unwrap_or(ri.len())..] {
            skip[i] = 1;
        }
        self.work += (ri.len() as f64 + ri.len() as f64) * mu;
        // candidates: [0, 1] integer columns, most row entries first (times 1 + 1e-4·random, seed env+0x2cd0 = 0)
        let mut rng = Rng(0);
        let (mut ck, mut cj) = (vec![], vec![]);
        for j in 0..n {
            if self.col_cnt[j] <= 0 || self.vtype[j] == b'C' || self.lb[j] != 0.0 || self.ub[j] != 1.0 {
                continue;
            }
            let mut s = 0i32;
            let mut q = self.col_head[j];
            while q != NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 && skip[nd.row as usize] == 0 {
                    s += self.row_cnt[nd.row as usize];
                }
                q = nd.nc;
            }
            self.work += mu * 4.0 * self.col_cnt[j] as f64;
            cj.push(j);
            ck.push((rng.f64() * 1e-4 + 1.0) * (-s) as f64);
        }
        self.work += n as f64 * 5.0 * mu;
        sort_desc(&mut ck, &mut cj);
        let mut pos = vec![-1i32; n];
        let mut at = 0;
        while at < cj.len() {
            let j = cj[at];
            if self.col_cnt[j] <= 0 {
                at += 1;
                continue;
            }
            let l = self.lb[j];
            if 0.5 < l || self.ub[j] < 0.5 {
                let rc = self.fix_col(j, if 0.5 < l { 1.0 } else { 0.0 }, true);
                if rc != 0 {
                    return rc;
                }
            } else {
                let i0 = self.col_prop(&mut cx[0], j, &skip, 0.0);
                let i1 = self.col_prop(&mut cx[1], j, &skip, 1.0);
                let rc = match (i0, i1) {
                    (false, false) => self.probe0_both(&mut cx, j, &skip, &mut pos),
                    (false, true) => self.fix_col(j, 0.0, true),
                    _ => self.fix_col(j, 1.0, true),
                };
                if rc != 0 {
                    return rc;
                }
                for c in cx.iter_mut() {
                    for k in 0..2 {
                        let src = if k == 0 { &self.lb } else { &self.ub };
                        for &x in &c.list[k] {
                            c.bnd[k][x] = src[x];
                            c.pos[k][x] = -1;
                        }
                        self.work += mu * 4.0 * c.list[k].len() as f64;
                        c.list[k].clear();
                    }
                }
            }
            if tv.over(self.work) {
                break;
            }
            if 1e7 < self.work - w70 {
                // `3ba5ac` (log) and `3ba9de` (budget; its stop flag is never set from Model.presolve())
                self.budget_check();
                if self.prelimit < 0 {
                    return 0;
                }
                w70 = self.work;
            }
            at += 1;
        }
        self.work += (at as f64 + at as f64) * mu;
        0
    }

    /// Both sides feasible: tighten bounds implied by both, strengthen x_j's coefficients in rows redundant on one
    /// side (an `=` row fixes x_j instead and ends here), fix columns fixed on both sides.
    fn probe0_both(&mut self, cx: &mut [Ctx; 2], j: usize, skip: &[i32], pos: &mut [i32]) -> i32 {
        let (n, mu) = (self.n, self.work_mult);
        for k in 0..2 {
            for (t, &x) in cx[0].list[k].iter().enumerate() {
                pos[x] = t as i32;
            }
            let l0 = cx[0].list[k].len() as f64;
            self.work += (l0 + l0) * mu;
            for &x in &cx[1].list[k] {
                if pos[x] == -1 {
                    continue;
                }
                let (v0, v1) = (cx[0].bnd[k][x], cx[1].bnd[k][x]);
                let (l, u) = (self.lb[x], self.ub[x]);
                if k == 0 {
                    let v = if v1 <= v0 { v1 } else { v0 };
                    if l + 1e-6 < v {
                        self.bound_change(x, l, u, v, u);
                        self.bounds_tightened += 1;
                        self.lb[x] = v;
                        self.tracker.mark_col(x as i32);
                    }
                } else {
                    let v = if v0 <= v1 { v1 } else { v0 };
                    if v < u - 1e-6 {
                        self.bound_change(x, l, u, l, v);
                        self.bounds_tightened += 1;
                        self.ub[x] = v;
                        self.tracker.mark_col(x as i32);
                    }
                }
            }
            self.work += cx[1].list[k].len() as f64 * 5.0 * mu;
            for &x in &cx[0].list[k] {
                pos[x] = -1;
            }
            self.work += (l0 + l0) * mu;
        }
        let mut q = self.col_head[j];
        while q != NIL {
            let cur = q as usize;
            let nd = self.nd[cur];
            q = nd.nc;
            if nd.col < 0 || skip[nd.row as usize] != 0 {
                continue;
            }
            let r = nd.row as usize;
            let mut hit = None;
            for (b, c) in cx.iter().enumerate() {
                let (mut lo, mut hi, mut cnt) = (0.0f64, 0.0f64, 0);
                let mut x = self.row_head[r];
                while x != NIL {
                    let e = self.nd[x as usize];
                    if e.col >= 0 {
                        let (a, k) = (e.val, e.col as usize);
                        let (l, u) = (c.bnd[0][k], c.bnd[1][k]);
                        if a <= 0.0 {
                            hi = if l <= -1e30 { 1e100 } else { hi + l * a };
                            lo = if 1e30 <= u { -1e100 } else { lo + u * a };
                        } else {
                            lo = if l <= -1e30 { -1e100 } else { lo + l * a };
                            hi = if 1e30 <= u { 1e100 } else { hi + u * a };
                        }
                    }
                    cnt += 1;
                    x = e.nr;
                }
                self.work += cnt as f64 * 5.0 * mu;
                let (hr, lr) = (hi - self.rhs[r], lo - self.rhs[r]);
                if hr < -1e-4 {
                    hit = Some((b, -hr));
                    break;
                }
                if self.sense[r] == b'=' && 1e-4 < lr {
                    hit = Some((b, lr));
                    break;
                }
            }
            let Some((b, d)) = hit else { continue };
            if self.sense[r] == b'=' {
                return self.fix_col(j, (1 - b) as f64, true);
            }
            if d.abs() > 1e-10 {
                self.tracker.mark_row(r as i32);
                self.tracker.mark_col(nd.col);
            }
            let (a, rh) = (nd.val, self.rhs[r]);
            let (na, nrh) = if b == 1 { (d + a, rh) } else { (a - d, rh - d) };
            if a <= 0.0 {
                self.act_min[r] -= a;
            } else {
                self.act_max[r] -= a;
            }
            if na <= 0.0 {
                self.act_min[r] = na + self.act_min[r];
            } else {
                self.act_max[r] = na + self.act_max[r];
            }
            self.act_min[r] -= nrh - self.rhs[r];
            self.act_max[r] -= nrh - self.rhs[r];
            self.nd[cur].val = na;
            self.rhs[r] = nrh;
            if na.abs() < 1e-9 {
                self.row_cnt[r] -= 1;
                self.col_cnt[nd.col as usize] -= 1;
                self.nd[cur].col = -1;
            }
            self.coefs_changed += 1;
        }
        self.work += self.col_cnt[j] as f64 * 4.0 * mu;
        for s in 0..2 {
            let k0 = 1 - s;
            for &x in &cx[0].list[k0] {
                pos[x] = -1;
            }
            let l0 = cx[0].list[k0].len() as f64;
            self.work += (l0 + l0) * mu;
            for (t, &x) in cx[1].list[s].iter().enumerate() {
                if (s == 0 && self.ub[x] - 1e-10 < cx[1].bnd[0][x]) || (s == 1 && cx[1].bnd[1][x] < self.lb[x] + 1e-10) {
                    pos[x] = t as i32;
                }
            }
            self.work += cx[1].list[s].len() as f64 * 4.0 * mu;
            let list = cx[0].list[k0].clone();
            for &x in &list {
                if !(x < n && 0 < self.col_cnt[x] && self.col_flag[x] == 0 && x != j && pos[x] != -1) {
                    continue;
                }
                let (l, u) = (self.lb[x], self.ub[x]);
                if (u - l).abs() < 1e-10 {
                    let v = if 0.0 < u && l < 0.0 { 0.0 } else { (l + u) * 0.5 };
                    let rc = self.fix_col(x, v, true);
                    if rc != 0 {
                        return rc;
                    }
                }
                // shortcut: else, within AGGMULTLIMIT, `3baaef` would tie x to x_j (x at a bound on both sides);
                // never called on the 38 models, not ported
            }
            self.work += list.len() as f64 * 7.0 * mu;
        }
        0
    }
}
