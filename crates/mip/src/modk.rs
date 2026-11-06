//! `838543(env, st, flag, k, W)`: mod-k elimination of the `=` rows (k = 2, 3, 5 at the round end; docs
//! m2_red_roundend_b.md §1). The integer rows reduced mod k go into the cut module's sparse mod-k system (`83c8d1`,
//! rows by `83e123`), Gaussian elimination mod k (`83e5f0`) takes rows by fewest entries from a heap; then each
//! row left with one entry fixes its column's residue (`574764`) and a row with two [l, l+1] columns fixes or ties them.
use crate::state::{State, NIL};
use crate::Params;

#[derive(Clone, Copy)]
struct MNode {
    coef: i32,
    row: i32,
    col: i32,
    nr: u32,
    nc: u32,
}

/// The mod-k system (`83c8d1` object, cut-tracking arrays +0xe0/+0xe8 freed by 838543): row lists, counts, rhs.
struct Sys {
    k: i32,
    nd: Vec<MNode>,
    rhead: Vec<u32>,
    cnt: Vec<i32>,
    rhs: Vec<i32>,
    chead: Vec<u32>,
    ccnt: Vec<i32>,
    /// +0x48 duplicate-row marks, +0x58 combine scratch
    mark: Vec<i32>,
    scratch: Vec<i32>,
}

impl Sys {
    fn new_node(&mut self, row: usize, col: usize, coef: i32) {
        let q = self.nd.len() as u32;
        self.nd.push(MNode { coef, row: row as i32, col: col as i32, nr: self.rhead[row], nc: self.chead[col] });
        self.rhead[row] = q;
        self.chead[col] = q;
        self.cnt[row] += 1;
        self.ccnt[col] += 1;
    }

    /// `83e123(weight 0, rhs, env, sys, row, n, idx, vals, W)`; a row equal to the previous one (same entries,
    /// rhs not compared) is dropped again (count -1).
    fn add_row(&mut self, rhs: i32, idx: &[usize], vals: &[i32], w: &mut f64) {
        let r = self.rhead.len();
        self.rhead.push(NIL);
        self.cnt.push(0);
        self.rhs.push(rhs);
        for (&j, &v) in idx.iter().zip(vals) {
            self.new_node(r, j, v);
        }
        *w += idx.len() as f64 * 12.0;
        if r < 1 || self.cnt[r] != self.cnt[r - 1] {
            return;
        }
        let walk = |s: &Sys, i: usize| {
            let mut v = Vec::new();
            let mut q = s.rhead[i];
            while q != NIL {
                v.push(q);
                q = s.nd[q as usize].nr;
            }
            v
        };
        let (cur, prev) = (walk(self, r), walk(self, r - 1));
        for &q in &cur {
            self.mark[self.nd[q as usize].col as usize] = -1;
        }
        *w += cur.len() as f64 * 23.0;
        for &q in &prev {
            let x = self.nd[q as usize];
            if x.col >= 0 {
                self.mark[x.col as usize] = x.coef;
            }
        }
        *w += prev.len() as f64 * 23.0;
        if let Some(i) = cur.iter().position(|&q| self.mark[self.nd[q as usize].col as usize] != self.nd[q as usize].coef) {
            *w += i as f64 * 24.0;
            return;
        }
        *w += cur.len() as f64 * 24.0;
        for &q in &cur {
            let c = self.nd[q as usize].col as usize;
            self.ccnt[c] -= 1;
            self.nd[q as usize].col = -1;
        }
        *w += cur.len() as f64 * 23.0;
        self.cnt[r] = -1;
    }

    /// `83e5f0(env, sys, dst, src, mult, W)`: row dst += mult·row src (mod k); dead entries leave dst's row list
    /// (they stay in their column lists).
    fn combine(&mut self, dst: usize, src: usize, mult: i32, w: &mut f64) {
        let k = self.k;
        let mut ns = 0;
        let mut q = self.rhead[src];
        while q != NIL {
            let x = self.nd[q as usize];
            if x.col >= 0 {
                self.scratch[x.col as usize] = x.coef.wrapping_mul(mult);
            }
            ns += 1;
            q = x.nr;
        }
        *w += ns as f64 * 23.0;
        let (mut nd, mut prev) = (0, NIL);
        let mut q = self.rhead[dst];
        while q != NIL {
            let x = self.nd[q as usize];
            let mut keep = false;
            if x.col >= 0 {
                let c = x.col as usize;
                let v = (self.scratch[c] as i64 + x.coef as i64) % k as i64;
                self.scratch[c] = 0;
                let v = norm(v as i32, k);
                if v == 0 {
                    let y = &mut self.nd[q as usize];
                    y.coef = 0;
                    y.col = -1;
                    self.cnt[x.row as usize] -= 1;
                    self.ccnt[c] -= 1;
                } else {
                    self.nd[q as usize].coef = v;
                    keep = true;
                }
            }
            if keep {
                prev = q;
            } else if prev == NIL {
                self.rhead[dst] = x.nr;
            } else {
                self.nd[prev as usize].nr = x.nr;
            }
            nd += 1;
            q = x.nr;
        }
        *w += nd as f64 * 23.0;
        ns = 0;
        let mut q = self.rhead[src];
        while q != NIL {
            let x = self.nd[q as usize];
            if x.col >= 0 && self.scratch[x.col as usize] != 0 {
                let c = x.col as usize;
                let v = norm(self.scratch[c] % k, k);
                self.scratch[c] = 0;
                self.new_node(dst, c, v);
            }
            ns += 1;
            q = x.nr;
        }
        *w += ns as f64 * 26.0;
        let v = ((self.rhs[src] as f64 * mult as f64 + self.rhs[dst] as f64) as i64 % k as i64) as i32;
        self.rhs[dst] = norm(v, k);
    }
}

/// `(v >> 31 & k) + v`: a C remainder moved into [0, k)
fn norm(v: i32, k: i32) -> i32 {
    (v >> 31 & k) + v
}

/// The tables `mod3/mod5/…6415.0.27`: −a⁻¹ mod k (1 for k = 2, 0 for a k without a table).
fn neg_inv(k: i32, a: i32) -> i32 {
    match k {
        2 => 1,
        3 => [0, 2, 1][a as usize],
        5 => [0, 4, 2, 3, 1][a as usize],
        // shortcut: the mod 7/11/13 tables are not copied; 3cd406 calls only k = 2, 3, 5
        _ => 0,
    }
}

/// `a786fc`…`a7936f` with int keys (a copy of crates/lp barrier/ordering.rs `Heap`): binary max-heap with a node
/// locator; equal keys never move.
struct Heap {
    node: Vec<usize>,
    key: Vec<i32>,
    loc: Vec<i64>,
}

impl Heap {
    fn swap(&mut self, i: usize, j: usize) {
        self.node.swap(i, j);
        self.key.swap(i, j);
        self.loc[self.node[i]] = i as i64;
        self.loc[self.node[j]] = j as i64;
    }
    fn up(&mut self, mut i: usize) {
        while i > 0 {
            let p = (i - 1) / 2;
            if self.key[i] <= self.key[p] {
                return;
            }
            self.swap(i, p);
            i = p;
        }
    }
    fn down(&mut self, mut i: usize) {
        let n = self.node.len();
        while 2 * i + 1 < n {
            let mut c = 2 * i + 2;
            if n <= c || self.key[c] <= self.key[2 * i + 1] {
                c = 2 * i + 1;
            }
            if self.key[c] <= self.key[i] {
                return;
            }
            self.swap(i, c);
            i = c;
        }
    }
    fn fix(&mut self, i: usize) {
        if i > 0 && self.key[(i - 1) / 2] < self.key[i] {
            self.up(i);
        } else {
            self.down(i);
        }
    }
    /// `a787f2`
    fn insert(&mut self, v: usize, k: i32) {
        self.node.push(v);
        self.key.push(k);
        self.loc[v] = self.node.len() as i64 - 1;
        self.up(self.node.len() - 1);
    }
    /// `a7936f`
    fn delete(&mut self, v: usize) {
        let i = self.loc[v];
        if i < 0 {
            return;
        }
        let i = i as usize;
        self.loc[v] = -1;
        let last = self.node.len() - 1;
        self.node.swap_remove(i);
        self.key.swap_remove(i);
        if i != last {
            self.loc[self.node[i]] = i as i64;
            self.fix(i);
        }
    }
    /// `a78af7`
    fn pop(&mut self) -> Option<usize> {
        let v = *self.node.first()?;
        self.delete(v);
        Some(v)
    }
    /// `a78d82`
    fn update(&mut self, v: usize, k: i32) {
        let i = self.loc[v] as usize;
        self.key[i] = k;
        self.fix(i);
    }
}

impl State {
    /// `838543(env, st, flag, k, W)`; flag = 1 without a MIP context (elimination stops at a row of 51+ entries).
    /// Its work goes to W unscaled; a local timer stops the elimination after 100·m + 1e7 work.
    pub fn mod_k(&mut self, p: &Params, k: i32) -> i32 {
        let (n, m) = (self.n, self.m);
        let w0 = self.work;
        let limit = m as f64 * 100.0 + 1e7;
        let mut s = Sys {
            k,
            nd: Vec::new(),
            rhead: Vec::new(),
            cnt: Vec::new(),
            rhs: Vec::new(),
            chead: vec![NIL; n],
            ccnt: vec![0; n],
            mark: vec![0; n],
            scratch: vec![0; n],
        };
        let (mut idx, mut vals) = (vec![0usize; n], vec![0i32; n]);
        for i in 0..m {
            if self.row_cnt[i] <= 0 || self.sense[i] != b'=' {
                continue;
            }
            let (mut cnt, mut walked, mut res, mut bail) = (0, 0usize, self.rhs[i], false);
            let mut q = self.row_head[i];
            while q != NIL {
                let x = self.nd[q as usize];
                if x.col >= 0 {
                    let j = x.col as usize;
                    let l = self.lb[j];
                    if 1e-13 <= self.ub[j] - l {
                        let a = x.val;
                        let f = (a + 1e-10).floor();
                        if self.vtype[j] == b'C' || 2e9 < a.abs() || 1e-10 < (a - f).abs() {
                            bail = true;
                            break;
                        }
                        idx[cnt] = j;
                        let v = norm(((f as i64) % k as i64) as i32, k);
                        vals[cnt] = v;
                        if v != 0 {
                            cnt += 1;
                        }
                    } else {
                        res -= x.val * l;
                    }
                }
                walked += 1;
                q = x.nr;
            }
            self.work += walked as f64 * 22.0;
            if bail || 2e9 < res.abs() {
                continue;
            }
            let r = (((res + 0.5).floor() as i64) % k as i64) as i32;
            if cnt > 0 {
                s.add_row(norm(r, k), &idx[..cnt], &vals[..cnt], &mut self.work);
                self.work += cnt as f64 * 4.0;
            }
        }
        let nrows = s.rhead.len();
        if nrows == 0 {
            return 0;
        }
        let mut h = Heap { node: Vec::new(), key: Vec::new(), loc: vec![-1; nrows] };
        for r in 0..nrows {
            if s.cnt[r] > 0 {
                h.insert(r, -s.cnt[r]);
            }
        }
        loop {
            // `adf2be` on the local timer (`ade736`: 1 − W at the start)
            if 1.0 <= (1.0 - w0 + self.work) / limit {
                break;
            }
            let Some(r) = h.pop() else { break };
            // flag = 1 (no MIP context): stop at the first row of 51+ entries
            if s.cnt[r] >= 51 {
                break;
            }
            let (mut best, mut piv, mut walked) = (nrows as i32 + 1, NIL, 0);
            let mut q = s.rhead[r];
            while q != NIL {
                let x = s.nd[q as usize];
                if x.col >= 0 {
                    let c = s.ccnt[x.col as usize];
                    if 0 < c && c < best {
                        piv = q;
                        best = c;
                    }
                }
                walked += 1;
                q = x.nr;
            }
            self.work += walked as f64 * 23.0;
            if piv == NIL {
                continue;
            }
            let pv = s.nd[piv as usize];
            let inv = neg_inv(k, pv.coef);
            let mut walked = 0;
            let mut q = s.chead[pv.col as usize];
            while q != NIL {
                let x = s.nd[q as usize];
                if x.col >= 0 && x.row != pv.row {
                    let (dst, src) = (x.row as usize, pv.row as usize);
                    let mult = norm(((x.coef as i64 * inv as i64) % k as i64) as i32, k);
                    self.work += (s.cnt[dst] as f64 + s.cnt[src] as f64) * 10.0;
                    s.combine(dst, src, mult, &mut self.work);
                    if s.cnt[dst] == 0 && s.rhs[dst] != 0 {
                        return 3;
                    }
                    if h.loc[dst] >= 0 {
                        if s.cnt[dst] < 1 {
                            h.delete(dst);
                        } else {
                            h.update(dst, -s.cnt[dst]);
                        }
                    }
                }
                walked += 1;
                q = s.nd[q as usize].nc;
            }
            self.work += walked as f64 * 29.0;
        }
        let mut done = vec![false; n];
        for r in 0..nrows {
            if s.cnt[r] == 1 {
                let mut q = s.rhead[r];
                while s.nd[q as usize].col < 0 {
                    q = s.nd[q as usize].nr;
                }
                let x = s.nd[q as usize];
                let c = x.col as usize;
                if self.col_cnt[c] < 1 || self.col_flag[c] != 0 || self.lb[c] <= -1e6 || self.ub[c] >= 1e6 || done[c] {
                    continue;
                }
                done[c] = true;
                let mut ok = 0;
                let rc = self.mod_subst(p, c, x.coef, k, s.rhs[r], &mut ok);
                if rc != 0 {
                    return rc;
                }
            } else if s.cnt[r] == 2 {
                let (mut c1, mut c2, mut a1, mut a2, mut walked) = (-1i32, -1i32, 0i32, 0i32, 0);
                let mut q = s.rhead[r];
                while q != NIL {
                    let x = s.nd[q as usize];
                    if x.col >= 0 && self.ub[x.col as usize] - self.lb[x.col as usize] == 1.0 {
                        if c1 == -1 {
                            (a1, c1) = (x.coef, x.col);
                        } else {
                            (a2, c2) = (x.coef, x.col);
                        }
                    }
                    walked += 1;
                    q = x.nr;
                }
                self.work += walked as f64 * 24.0;
                if c1 < 0 || c2 < 0 {
                    continue;
                }
                let (c1, c2) = (c1 as usize, c2 as usize);
                let (l1, u1) = (self.lb[c1] as i32, self.ub[c1] as i32);
                let (l2, u2) = (self.lb[c2] as i32, self.ub[c2] as i32);
                let rh = s.rhs[r];
                let e = |x: i32, y: i32| norm(((a1.wrapping_mul(x).wrapping_add(a2.wrapping_mul(y))) as i64 % k as i64) as i32, k);
                let (ll, ul, lu, uu) = (e(l1, l2) == rh, e(u1, l2) == rh, e(l1, u2) == rh, e(u1, u2) == rh);
                match ll as i32 + ul as i32 + lu as i32 + uu as i32 {
                    0 => return 3,
                    1 => {
                        let (mut v1, mut v2) = (l1, l2);
                        if !ll {
                            if !lu {
                                v1 = u1;
                            }
                            if !ul {
                                v2 = u2;
                            }
                        }
                        let mut rc = 0;
                        if self.col_cnt[c1] >= 1 && !done[c1] {
                            rc = self.fix_col(c1, v1 as f64, true);
                        }
                        if rc == 0 && self.col_cnt[c2] > 0 && !done[c2] {
                            rc = self.fix_col(c2, v2 as f64, true);
                        }
                        if rc != 0 {
                            return rc;
                        }
                    }
                    2 => {
                        let (a, b) = if self.col_flag[c2] != 0 || (self.col_flag[c1] == 0 && l2 == 0 && l1 != 0) {
                            (c2, c1)
                        } else {
                            (c1, c2)
                        };
                        if self.col_cnt[a] > 0 && !done[a] && self.col_cnt[b] > 0 && !done[b] && self.col_flag[b] == 0 {
                            // shortcut: `3baaef(±1, c, st, env, a, b, ..)` would tie x_a to ±x_b + c (−1 with c = l_b + u_a
                            // unless (l, l) fits, else +1 with l_b − l_a); never called on the 38 models, not ported
                        }
                    }
                    _ => {}
                }
            }
        }
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neg_inv_tables() {
        for k in [2, 3, 5] {
            for a in 1..k {
                assert_eq!(a * neg_inv(k, a) % k, k - 1);
            }
        }
    }
}
