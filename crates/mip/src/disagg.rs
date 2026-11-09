//! `5a04fc(st, env)` smart disaggregation (PRESMARTDISAGG >= 2, docs/m2/m2_red_roundend_b.md §6): `<` rows that force
//! one binary literal ("literal rows"), grouped by literal; a row whose big binary entries reach literals that
//! PRESMARTDISAGG or more literal rows share would be disaggregated through a new row and column (`3bde19`, `3bea24`).
use crate::state::{State, NIL};
use crate::Params;

impl State {
    pub fn smart_disagg(&mut self, p: &Params) -> i32 {
        let (n, m, mu) = (self.n, self.m, self.work_mult);
        let thr = p.presmartdisagg;
        let bin = |s: &State, j: usize| s.vtype[j] == b'B' && s.lb[j] == 0.0 && s.ub[j] == 1.0;
        let lit_of = |j: i32, a: f64| if a <= 0.0 { j + n as i32 } else { j };
        // 1. the literal of each `<` row whose min activity already meets the rhs up to one binary
        let mut lit = vec![-1i32; m];
        let mut cnt = 0;
        for i in 0..m {
            if self.row_cnt[i] <= 1 || self.sense[i] != b'<' || 0 < self.inf_min[i] || 0 < self.inf_max[i]
                || 1e-6 < self.act_max[i] + self.act_min[i]
            {
                continue;
            }
            let mut q = self.row_head[i];
            while q != NIL {
                let nd = self.nd[q as usize];
                q = nd.nr;
                if nd.col >= 0 && bin(self, nd.col as usize) && -1e-6 <= nd.val.abs() + self.act_min[i] {
                    lit[i] = if lit[i] == -1 { lit_of(nd.col, nd.val) } else { -2 };
                }
            }
            self.work += mu * 6.0 * self.row_cnt[i] as f64;
            if 0 <= lit[i] {
                cnt += 1;
            }
        }
        self.work += mu * 5.0 * m as f64;
        if cnt == 0 {
            return 0;
        }
        // 2. how many literal rows hold each other literal
        let mut uses = vec![0i32; 2 * n];
        self.work += (n as f64 + n as f64) * mu;
        for i in 0..m {
            if lit[i] < 0 {
                continue;
            }
            let mut q = self.row_head[i];
            while q != NIL {
                let nd = self.nd[q as usize];
                q = nd.nr;
                if nd.col >= 0 {
                    let l = if 0.0 < nd.val || nd.val == 0.0 { nd.col } else { nd.col + n as i32 };
                    if l != lit[i] {
                        uses[l as usize] += 1;
                    }
                }
            }
            self.work += mu * 5.0 * self.row_cnt[i] as f64;
        }
        // 3. keep literal rows with a free entry whose literal is shared often enough
        let mut kept = 0;
        for i in 0..m {
            if lit[i] < 0 {
                continue;
            }
            let mut found = false;
            let mut q = self.row_head[i];
            while q != NIL {
                let nd = self.nd[q as usize];
                q = nd.nr;
                if nd.col < 0 {
                    continue;
                }
                let j = nd.col as usize;
                if 1e-10 < self.ub[j] - self.lb[j]
                    && ((0.0 < nd.val && thr <= uses[j]) || (nd.val < 0.0 && thr <= uses[n + j]))
                {
                    found = true;
                    break;
                }
            }
            self.work += mu * 5.0 * self.row_cnt[i] as f64;
            if found {
                kept += 1;
            } else {
                lit[i] = -1;
            }
        }
        self.work += mu * 4.0 * m as f64;
        if kept == 0 {
            return 0;
        }
        // 4. literal rows by literal, ascending row order
        let mut start = vec![0usize; 2 * n + 1];
        for &l in &lit {
            if 0 <= l {
                start[l as usize + 1] += 1;
            }
        }
        for l in 0..2 * n {
            start[l + 1] += start[l];
        }
        let mut rows = vec![0usize; start[2 * n]];
        let mut at = start.clone();
        for (i, &l) in lit.iter().enumerate() {
            if 0 <= l {
                rows[at[l as usize]] = i;
                at[l as usize] += 1;
            }
        }
        self.work += (m as f64 * 5.0 + n as f64 * 6.0) * mu;
        // 5. rows with two or more big binary entries on a side
        let lim = self.work + self.nnz as f64 * 1000.0 + 1e6;
        let mut reach = vec![0i32; 2 * n];
        for i in 0..m {
            if lim < self.work {
                self.work += mu * 14.0 * i as f64;
                return 0;
            }
            if self.row_cnt[i] <= 1 || 0 < self.inf_min[i] + self.inf_max[i] || 0 <= lit[i] {
                continue;
            }
            let sides: &[f64] = if self.sense[i] == b'=' { &[-1.0, 1.0] } else { &[1.0] };
            for &s in sides {
                let d = if s < 0.0 { self.act_max[i] } else { -self.act_min[i] };
                let big = |st: &State, nd: &crate::state::Node| {
                    nd.col >= 0 && bin(st, nd.col as usize) && d * 0.51 + 1e-5 < (nd.val * s).abs()
                };
                let mut k = 0;
                let mut q = self.row_head[i];
                while q != NIL {
                    let nd = self.nd[q as usize];
                    k += big(self, &nd) as i32;
                    q = nd.nr;
                }
                self.work += mu * 6.0 * self.row_cnt[i] as f64;
                if k < 2 {
                    continue;
                }
                let (mut list, mut seen) = (vec![], 0usize);
                let mut q = self.row_head[i];
                while q != NIL {
                    let nd = self.nd[q as usize];
                    q = nd.nr;
                    if !big(self, &nd) {
                        continue;
                    }
                    let l = if nd.val * s <= 0.0 { nd.col as usize + n } else { nd.col as usize };
                    for &r in &rows[start[l]..start[l + 1]] {
                        seen += 1;
                        let mut x = self.row_head[r];
                        while x != NIL {
                            let e = self.nd[x as usize];
                            x = e.nr;
                            if e.col >= 0 && 1e-6 < self.ub[e.col as usize] - self.lb[e.col as usize] {
                                let l2 = lit_of(e.col, e.val) as usize;
                                if l2 != l {
                                    if reach[l2] == 0 {
                                        list.push(l2);
                                    }
                                    reach[l2] += 1;
                                }
                            }
                        }
                        self.work += mu * 5.0 * self.row_cnt[r] as f64;
                    }
                    self.work += mu * 3.0 * (start[l + 1] - start[l]) as f64;
                }
                for &l2 in &list {
                    // shortcut: reach[l2] >= PRESMARTDISAGG starts the disaggregation (`3bde19` new row, `3bea24`
                    // new column, node moves, `370047`); never on the 38 models, not ported
                    reach[l2] = 0;
                }
                self.work += list.len() as f64 * (mu * 3.0);
                self.work += mu * 3.0 * seen as f64;
            }
        }
        self.work += mu * 14.0 * m as f64;
        0
    }
}
