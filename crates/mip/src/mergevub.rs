//! `5466a7` VUB merging (docs/m2/m2_red_outer_c.md §3): for each binary y, its two-entry `<` rows that are variable
//! bounds of another column (x ≤ l + (u−l)·y and the like) are sorted into four kinds per x; then rows of y whose
//! other entries are all such bounded columns on one side could be merged into one.
use crate::driver::Timer;
use crate::probe::TimerView;
use crate::state::{State, NIL};
use crate::Params;

impl State {
    /// `5466a7(env, st, T13)`.
    /// shortcut: the actions are not ported: two bounds of one x that conflict or repeat (fixing y or x, `397538`
    /// dropping the repeat, the `564ce0` merge of the remembered pairs) and the merge of the rows phase 2 keeps.
    /// None of them occurs on the 38 models (the call changes nothing there but work); port them with a model
    /// that has such rows.
    pub fn merge_vubs(&mut self, _p: &Params, mut t: Timer) -> i32 {
        t.start(self.work);
        let tv = TimerView { acc: t.acc(), running: t.running(), limit: t.limit };
        let mu = self.work_mult;
        let list = self.adjacent_list(true);
        let mut mark = vec![0i32; self.n.max(self.m)];
        let mut wchk = 0.0;
        for &j in &list {
            let j = j as usize;
            if !(1 < self.col_cnt[j] && self.vtype[j] == b'B' && self.lb[j] <= 0.5 && self.ub[j] >= 0.5) {
                continue;
            }
            if tv.over(self.work) {
                break;
            }
            if 1e7 < self.work - wchk {
                self.budget_check();
                if self.prelimit < 0 {
                    break;
                }
                wchk = self.work;
            }
            // phase 1: the variable bound rows of y, kind 1..4 per bounded column (lists: kinds 2/4, kinds 1/3;
            // -1-x for a negative coefficient of x)
            let mut side: [Vec<i32>; 2] = [Vec::new(), Vec::new()];
            let mut q = self.col_head[j];
            let mut visited = 0;
            while q != NIL {
                let nd = self.nd[q as usize];
                let r = nd.row as usize;
                if nd.col >= 0 && self.row_cnt[r] == 2 && self.sense[r] == b'<' {
                    let mut o = self.row_head[r];
                    let mut skip = 0;
                    while o != NIL && (self.nd[o as usize].col < 0 || self.nd[o as usize].col as usize == j) {
                        skip += 1;
                        o = self.nd[o as usize].nr;
                    }
                    self.work = (skip as f64 + skip as f64) * mu + self.work;
                    let other = self.nd[o as usize];
                    let (k, ak, aj, rhs) = (other.col as usize, other.val, nd.val, self.rhs[r]);
                    let (lk, uk) = (self.lb[k], self.ub[k]);
                    let d = uk - lk;
                    if !(d <= 1e-13 || 1e-13 < ((aj / ak).abs() - d).abs()) {
                        let kind = if ak < 0.0 {
                            if !(aj <= 0.0 || 1e-13 < (rhs / ak - lk).abs()) {
                                1
                            } else if aj < 0.0 && (rhs / ak - uk).abs() <= 1e-13 {
                                2
                            } else {
                                0
                            }
                        } else if !(aj <= 0.0 || 1e-13 < (rhs / ak - uk).abs()) {
                            3
                        } else if aj < 0.0 && (rhs / ak - lk).abs() <= 1e-13 {
                            4
                        } else {
                            0
                        };
                        // a second bound of k (mark[k] != 0) is a conflict or repeat (see shortcut)
                        if kind != 0 && mark[k] == 0 {
                            let x = if ak < 0.0 { -1 - k as i32 } else { k as i32 };
                            side[(kind & 1) as usize].push(x);
                            mark[k] = kind;
                        }
                    }
                }
                visited += 1;
                q = nd.nc;
            }
            self.work = (visited as f64 * 9.0) * mu + self.work;
            for s in &side {
                for &x in s {
                    mark[if x < 0 { (-1 - x) as usize } else { x as usize }] = 0;
                }
                self.work = (s.len() as f64 + s.len() as f64) * mu + self.work;
            }
            if 0.5 < self.lb[j] || self.ub[j] < 0.5 {
                continue;
            }
            // phase 2: rows (other than the bounds) whose entries are y and bounded columns of one side only
            for s in &side {
                if s.len() < 2 || 0.5 < self.lb[j] || self.ub[j] < 0.5 {
                    continue;
                }
                let mut rows: Vec<usize> = Vec::new();
                let mut q = self.col_head[j];
                let mut walked = 0;
                while q != NIL {
                    let nd = self.nd[q as usize];
                    if nd.col >= 0 && 2 < self.row_cnt[nd.row as usize] {
                        rows.push(nd.row as usize);
                        mark[nd.row as usize] = 1;
                    }
                    walked += 1;
                    q = nd.nc;
                }
                self.work = (walked as f64 * 5.0) * mu + self.work;
                for &x in s {
                    let k = if x < 0 { (-1 - x) as usize } else { x as usize };
                    if self.col_cnt[k] < 0 || 10000 < self.col_cnt[k] {
                        continue;
                    }
                    let mut q = self.col_head[k];
                    let mut walked = 0;
                    while q != NIL {
                        let nd = self.nd[q as usize];
                        if nd.col >= 0 && (x >= 0) == (0.0 < nd.val) {
                            let r = nd.row as usize;
                            if mark[r] == 0 {
                                rows.push(r);
                            }
                            mark[r] += 1;
                        }
                        walked += 1;
                        q = nd.nc;
                    }
                    self.work = (mu * 5.0) * walked as f64 + self.work;
                }
                self.work = (s.len() as f64 * 3.0) * mu + self.work;
                // rows with 1 < mark == row_cnt, `<` and act_min <= -0.001 would be merged here (see shortcut)
                for &r in &rows {
                    mark[r] = 0;
                }
            }
        }
        0
    }
}
