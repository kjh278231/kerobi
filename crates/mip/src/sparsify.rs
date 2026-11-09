//! `470936` sparsify (docs/m2/m2_red_final.md §2): add a multiple of an `=` row to rows that share most of its support
//! when that removes at least half the row's nonzeros; inequality rows of 10+ entries get a slack column first
//! (removed again when untouched). MIP states keep no record (`39dded` is LP only).
use crate::driver::Timer;
use crate::probe::TimerView;
use crate::state::{State, NIL};
use crate::Params;

/// A node of the trial copy (`470936`'s own pool).
#[derive(Clone, Copy)]
struct TNode {
    val: f64,
    row: i32,
    col: i32,
    nr: u32,
    nc: u32,
}

/// Candidate multiplier of row r for the pivot row (pass 1): `cnt[r]` entries agree on `mult[r]`; ratios below `tol`
/// (main pass only) reset the row.
fn tally(cnt: &mut [i32], mult: &mut [f64], r: usize, ratio: f64, tol: Option<f64>) {
    let ar = ratio.abs();
    if cnt[r] == 0 {
        mult[r] = if 10000.0 <= ar || tol.is_some_and(|t| ar <= t) { 0.0 } else { ratio };
        cnt[r] = 1;
    } else if tol.is_none_or(|t| t < ar) {
        if (ratio - mult[r]).abs() < 1e-10 {
            cnt[r] += 1;
        } else if ar < mult[r].abs() {
            mult[r] = ratio;
            cnt[r] = 1;
        }
    } else {
        mult[r] = 0.0;
        cnt[r] = 0;
    }
}

impl State {
    /// `470936(st, env, T20, 0, 0)` (no MIP context). Returns 0 (or a callee status).
    pub fn sparsify(&mut self, p: &Params, t: Timer) -> i32 {
        let mut t = t;
        t.start(self.work);
        let tv = TimerView { acc: t.acc(), running: t.running(), limit: t.limit };
        let (n, m, mu) = (self.n, self.m, self.work_mult);
        let w0 = self.work;
        if p.presparsify < 0 {
            if !(9 < m && 9 < n) {
                return 0;
            }
            let mut hist = vec![0i32; m + 1];
            let (mut ndist, mut nb, mut nalive) = (0, 0, 0);
            for j in 0..n {
                let c = self.col_cnt[j];
                if 0 < c {
                    nalive += 1;
                    ndist += (hist[c as usize] == 0) as i32;
                    hist[c as usize] += 1;
                    nb += (self.vtype[j] == b'B') as i32;
                }
            }
            // the 4th and 5th arguments (MIP context +0x620 → +0x7fc / +0x808) are 0 here: all live columns binary
            if ndist < 10 || nb < nalive {
                return 0;
            }
            let (mut nrows, mut nnz) = (0i32, 0i64);
            for i in 0..m {
                if 0 < self.row_cnt[i] {
                    nrows += 1;
                    nnz += self.row_cnt[i] as i64;
                }
            }
            let (fr, fz) = (nrows as f64, nnz as f64);
            if fz < fr * 40.0 || fz < nalive as f64 * 40.0 || nalive as f64 * 10.0 < fr || 1000000.0 < fr
                || 2000000000.0 < fz * 1.5
            {
                return 0;
            }
            match self.sparsify_trial(&tv, nnz as i32) {
                Some(removed) if !(removed < fz * 0.5 || removed < fz * 0.8) => {}
                _ => return 0,
            }
        }
        if tv.over(self.work) {
            return 0;
        }
        self.budget_check();
        if self.prelimit < 0 {
            return 0;
        }
        // st+0xe0 (CSR state) is 0 for MIP; without slacks only when all columns were binary and nnz >= 1e7
        let slacks = 1 <= p.presparsify || self.nbin < n as i32 || self.nnz < 10000000;
        let mut k = 0;
        if slacks {
            for i in 0..m {
                if 9 < self.row_cnt[i] && self.sense[i] != b'=' {
                    k += 1;
                    self.inf_max[i] += 1;
                }
            }
        }
        let n0 = self.add_cols(k);
        let mut js = n0;
        if slacks {
            for i in 0..m {
                if 9 < self.row_cnt[i] && self.sense[i] != b'=' {
                    self.add_node(js, i, 1.0);
                    js += 1;
                    self.sense[i] = b'=';
                }
            }
        }
        let mut cnt = vec![0i32; m];
        let mut mult = vec![0.0f64; m];
        let mut pos = vec![NIL; self.n];
        let mut wchk = w0;
        for i in 0..m {
            if !(9 < self.row_cnt[i] && self.sense[i] == b'=') {
                continue;
            }
            if tv.over(self.work) {
                break;
            }
            if 1e7 < self.work - wchk {
                self.budget_check();
                if self.prelimit < 0 {
                    return 0; // the slacks stay
                }
                wchk = self.work;
                if self.budget / 5.0 < wchk - w0 {
                    break;
                }
            }
            let h = self.row_head[i];
            let mut minabs = 1e100;
            let mut q = h;
            while q != NIL {
                let x = self.nd[q as usize];
                if x.col >= 0 && x.val.abs() <= minabs {
                    minabs = x.val.abs();
                }
                q = x.nr;
            }
            if h != NIL && minabs < 1e-13 {
                continue;
            }
            let tol = (1e-10 / minabs).max(1e-6);
            // pass 1: multipliers
            let mut q = h;
            while q != NIL {
                let a = self.nd[q as usize];
                if a.col >= 0 {
                    let mut c = self.col_head[a.col as usize];
                    while c != NIL {
                        let b = self.nd[c as usize];
                        if b.col >= 0 {
                            tally(&mut cnt, &mut mult, b.row as usize, -b.val / a.val, Some(tol));
                        }
                        c = b.nc;
                    }
                    self.work = (self.col_cnt[a.col as usize] as f64 * 10.0) * mu + self.work;
                }
                q = a.nr;
            }
            // pass 2: combine
            let mut q = h;
            while q != NIL {
                let a = self.nd[q as usize];
                if a.col >= 0 {
                    let mut c = self.col_head[a.col as usize];
                    while c != NIL {
                        if self.nd[c as usize].col >= 0 {
                            let r = self.nd[c as usize].row as usize;
                            let li = self.row_cnt[i];
                            if i != r && cnt[r] * -2 + li <= -(li / 2).max(10) {
                                self.work = ((li + self.row_cnt[r]) as f64 * 5.0) * mu + self.work;
                                self.sparsify_add_row(i, r, mult[r], &mut pos);
                            }
                            cnt[self.nd[c as usize].row as usize] = 0;
                        }
                        c = self.nd[c as usize].nc;
                    }
                }
                q = self.nd[q as usize].nr;
            }
        }
        // untouched slacks go again (the node joins the pending list)
        for j in n0..js {
            if self.col_cnt[j] != 1 {
                continue;
            }
            let mut q = self.col_head[j];
            while q != NIL && self.nd[q as usize].col < 0 {
                q = self.nd[q as usize].nc;
            }
            let x = self.nd[q as usize];
            let r = x.row as usize;
            if x.val == 1.0 && self.sense[r] == b'=' {
                self.row_cnt[r] -= 1;
                self.col_cnt[j] = -1;
                self.sense[r] = b'<';
                self.col_head[j] = NIL;
                self.npend += 1;
                self.inf_max[r] -= 1;
                self.cols_removed += 1;
                let nd = &mut self.nd[q as usize];
                nd.col = -1;
                nd.nc = self.pend;
                self.pend = q;
                self.nnz -= 1;
            }
        }
        0
    }

    /// Row r += f·row i: matched entries in place (dropped below 1e-10), fill-in nodes from the free list at the
    /// list heads, rhs, then r's activity (`370047(st, env, 1, &r)`).
    fn sparsify_add_row(&mut self, i: usize, r: usize, f: f64, pos: &mut [u32]) {
        let walk = |s: &State, h: u32| {
            let mut v = Vec::new();
            let mut q = h;
            while q != NIL {
                v.push(q);
                q = s.nd[q as usize].nr;
            }
            v
        };
        let (ri, rr) = (walk(self, self.row_head[i]), walk(self, self.row_head[r]));
        for &q in &rr {
            if self.nd[q as usize].col >= 0 {
                pos[self.nd[q as usize].col as usize] = NIL;
            }
        }
        for &q in &ri {
            if self.nd[q as usize].col >= 0 {
                pos[self.nd[q as usize].col as usize] = q;
            }
        }
        for &q in &rr {
            let x = self.nd[q as usize];
            if x.col < 0 || pos[x.col as usize] == NIL {
                continue;
            }
            let y = pos[x.col as usize];
            pos[x.col as usize] = NIL;
            let v = self.nd[y as usize].val * f + x.val;
            self.nd[q as usize].val = v;
            if v.abs() < 1e-10 {
                self.row_cnt[x.row as usize] -= 1;
                self.col_cnt[x.col as usize] -= 1;
                self.nd[q as usize].col = -1;
            }
        }
        for &q in &ri {
            let y = self.nd[q as usize];
            if y.col < 0 || pos[y.col as usize] == NIL {
                continue;
            }
            self.add_node(y.col as usize, r, y.val * f);
            pos[y.col as usize] = NIL;
        }
        self.rhs[r] = f * self.rhs[i] + self.rhs[r];
        self.activity(Some(&[r as i32]));
    }

    /// The auto-mode trial (PreSparsify < 0): the same pass on a copy of the live matrix (rows in column order, all
    /// rows of 10+ entries, inequality rows needing one more match), pool 1.3·nnz. Returns the nonzeros removed, or
    /// None when the pool overflows or the budget ends presolve. Only the work reaches the state.
    fn sparsify_trial(&mut self, tv: &TimerView, nnz: i32) -> Option<f64> {
        let (n, m, mu) = (self.n, self.m, self.work_mult);
        let cap = (nnz as f64 * 1.3) as i32;
        let mut len: Vec<i32> = (0..m).map(|i| self.row_cnt[i].max(0)).collect();
        let mut next = vec![0usize; m + 1];
        for i in 0..m {
            next[i + 1] = next[i] + len[i] as usize;
        }
        let start = next.clone();
        let mut pool = vec![TNode { val: 0.0, row: 0, col: -1, nr: NIL, nc: NIL }; start[m]];
        let mut chead = vec![NIL; n];
        for j in 0..n {
            let mut tail = NIL;
            let mut q = self.col_head[j];
            while q != NIL {
                let x = self.nd[q as usize];
                q = x.nc;
                if x.col < 0 {
                    continue;
                }
                let at = next[x.row as usize];
                next[x.row as usize] += 1;
                pool[at] = TNode { val: x.val, row: x.row, col: j as i32, nr: NIL, nc: NIL };
                if tail == NIL { chead[j] = at as u32 } else { pool[tail as usize].nc = at as u32 }
                tail = at as u32;
            }
        }
        let mut rhead = vec![NIL; m];
        for i in 0..m {
            if start[i] < start[i + 1] {
                rhead[i] = start[i] as u32;
                for at in start[i]..start[i + 1] - 1 {
                    pool[at].nr = at as u32 + 1;
                }
            }
        }
        let mut cnt = vec![0i32; m];
        let mut mult = vec![0.0f64; m];
        let mut pos = vec![NIL; n];
        let mut used = nnz;
        let mut wchk = self.work;
        for i in 0..m {
            if !(9 < len[i]) {
                continue;
            }
            if tv.over(self.work) {
                break;
            }
            if 1e7 < self.work - wchk {
                self.budget_check();
                if self.prelimit < 0 {
                    return None;
                }
                wchk = self.work;
            }
            let h = rhead[i];
            let mut q = h;
            while q != NIL {
                let a = pool[q as usize];
                if a.col >= 0 {
                    let mut c = chead[a.col as usize];
                    while c != NIL {
                        let b = pool[c as usize];
                        if b.col >= 0 {
                            tally(&mut cnt, &mut mult, b.row as usize, -b.val / a.val, None);
                        }
                        c = b.nc;
                    }
                    self.work = mu * 10.0 * self.col_cnt[a.col as usize] as f64 + self.work;
                }
                q = a.nr;
            }
            let mut q = h;
            while q != NIL {
                let a = pool[q as usize];
                if a.col >= 0 {
                    let mut c = chead[a.col as usize];
                    while c != NIL {
                        if pool[c as usize].col >= 0 {
                            let r = pool[c as usize].row as usize;
                            let li = len[i];
                            let t = li - 2 * cnt[r] + (self.sense[i] != b'=') as i32;
                            if i != r && t <= -(li / 2).max(10) {
                                self.work = mu * 5.0 * (li + len[r]) as f64 + self.work;
                                let f = mult[r];
                                let mut x = rhead[r];
                                while x != NIL {
                                    if pool[x as usize].col >= 0 {
                                        pos[pool[x as usize].col as usize] = NIL;
                                    }
                                    x = pool[x as usize].nr;
                                }
                                let mut y = h;
                                while y != NIL {
                                    if pool[y as usize].col >= 0 {
                                        pos[pool[y as usize].col as usize] = y;
                                    }
                                    y = pool[y as usize].nr;
                                }
                                let mut x = rhead[r];
                                while x != NIL {
                                    let xc = pool[x as usize].col;
                                    if xc >= 0 && pos[xc as usize] != NIL {
                                        let v = pool[pos[xc as usize] as usize].val * f + pool[x as usize].val;
                                        pos[xc as usize] = NIL;
                                        pool[x as usize].val = v;
                                        if v.abs() <= 1e-10 {
                                            len[pool[x as usize].row as usize] -= 1;
                                            pool[x as usize].col = -1;
                                        }
                                    }
                                    x = pool[x as usize].nr;
                                }
                                let mut y = h;
                                while y != NIL {
                                    let yn = pool[y as usize];
                                    if yn.col >= 0 && pos[yn.col as usize] != NIL {
                                        used += 1;
                                        if !(used < cap) {
                                            return None; // pool overflow (pos is cleared; nothing else to undo)
                                        }
                                        let at = pool.len() as u32;
                                        pool.push(TNode {
                                            val: yn.val * f,
                                            row: r as i32,
                                            col: yn.col,
                                            nr: rhead[r],
                                            nc: chead[yn.col as usize],
                                        });
                                        chead[yn.col as usize] = at;
                                        rhead[r] = at;
                                        len[r] += 1;
                                        pos[yn.col as usize] = NIL;
                                    }
                                    y = yn.nr;
                                }
                            }
                            cnt[pool[c as usize].row as usize] = 0;
                        }
                        c = pool[c as usize].nc;
                    }
                }
                q = pool[q as usize].nr;
            }
        }
        let after: i64 = len.iter().map(|&l| l.max(0) as i64).sum();
        Some((nnz as i64 - after) as i32 as f64)
    }
}
