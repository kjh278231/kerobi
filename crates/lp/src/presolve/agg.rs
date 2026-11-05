//! Aggregation (`519a52` around `37f484`, docs/m1_presolve_agg.md). The CSR state becomes the linked-list matrix
//! the MIP presolve uses, up to five aggregation rounds run on it, and it goes back to gap-free CSR in list order.
//! Later reductions and the presolved model read that order, so nodes, the free/pending lists and the +0xd8 count
//! are simulated as the original keeps them (node addresses are not visible; list order and counts are).
//! Deterministic work (+0x458) is added term by term in the original order (docs/m1_presolve_agg.md §10); only
//! `519a52`'s own terms skip the +0x448 multiplier.
use super::reduce::ColTols;
use super::sort::sort_by_key;
use super::state::{guarded_add, PostRec, State};

const NIL: u32 = u32::MAX;
/// GURO_PAR_ZERO (env+0x2498)
const ZERO: f64 = 1e-13;

/// Aggregation parameters beyond `ColTols`.
pub struct AggPar {
    /// AggFill after `390989` resolved -1 (10, or 20 when model+0xf0 is set)
    pub fill: i32,
    /// GURO_PAR_PREPAIRAGG (env+0x2594)
    pub pair_agg: i32,
    /// GURO_PAR_PREBOUND (env+0x2544)
    pub prebound: i32,
}

#[derive(Clone, Copy, Debug)]
struct Node {
    val: f64,
    row: i32,
    /// -1 = dead
    col: i32,
    /// next in the row (+0x10) / in the column, free or pending list (+0x18)
    nr: u32,
    nc: u32,
}

/// state+0x160..+0x1a0: node arena, row and column heads, free list (+0x1a0, count +0x198) and pending list
/// (+0x190, count +0x188: nodes out of their column that may still sit in a row list).
pub(super) struct Mat {
    nd: Vec<Node>,
    rh: Vec<u32>,
    ch: Vec<u32>,
    free: u32,
    nfree: i64,
    pend: u32,
    npend: i64,
    /// +0x1b0 scatter (row- or column-indexed), cleared after each use
    scat: Vec<u32>,
}

/// Per-row caches of `37f484` (`local_1e0`, `local_1d8`, `local_1e8`, `local_180`, `local_178`); hi == 0 means stale.
struct RowCache {
    hi: Vec<f64>,
    lo: Vec<f64>,
    amax: Vec<f64>,
    amin: Vec<f64>,
    rowmax: Vec<f64>,
}

impl Mat {
    /// `3b85ea`: grab `k` more nodes for the free list.
    fn grow(&mut self, k: usize) {
        for _ in 0..k {
            let p = self.nd.len() as u32;
            self.nd.push(Node { val: 0.0, row: 0, col: -1, nr: NIL, nc: self.free });
            self.free = p;
        }
        self.nfree += k as i64;
    }
}

impl State {
    /// `519a52`. Returns a status (3 infeasible) or 0.
    pub fn aggregate(&mut self, t: &ColTols, ap: &AggPar) -> i32 {
        let mut mat = self.agg_lists();
        self.agg_bits(0x20, true); // 53c507
        // state+8 (problem type) is 0 for a general LP; type 4 would use 0.1 with Presolve < 2
        let ratio = 0.01;
        let mut cls = vec![0u8; self.n];
        let mut base = self.rows_removed;
        let mut it = 0;
        let mut first = true;
        loop {
            let rc = self.agg_round(&mut mat, &mut cls, first, t, ap);
            if rc != 0 {
                return rc;
            }
            first = false;
            // rows flagged 0x20, then the rows of flagged columns, get their activity recomputed
            let lv = self.tracker.level;
            let mut list = Vec::new();
            let mut flagged = 0;
            for idx in 0..self.tracker.rows.b[lv] {
                let i = self.tracker.rows.list[lv][idx];
                if self.tracker.rows.flag[i as usize] & 0x20 != 0 {
                    flagged += 1;
                    list.push(i);
                    self.mark[i as usize] = true;
                }
            }
            // 519a52's own terms carry no multiplier
            self.work += self.tracker.rows.b[lv] as f64 * 4.0;
            for idx in 0..self.tracker.cols.b[lv] {
                let j = self.tracker.cols.list[lv][idx] as usize;
                if self.tracker.cols.flag[j] & 0x20 == 0 {
                    continue;
                }
                flagged += 1;
                let mut p = mat.ch[j];
                let mut live = 0;
                while p != NIL {
                    let q = mat.nd[p as usize];
                    if q.col >= 0 {
                        if !self.mark[q.row as usize] {
                            self.mark[q.row as usize] = true;
                            list.push(q.row);
                        }
                        live += 1;
                    }
                    p = q.nc;
                }
                self.work += live as f64 * 4.0;
            }
            let bc = self.tracker.cols.b[lv] as f64;
            self.work += bc + bc;
            self.agg_activity(&mat, &list, t.infbound);
            for &i in &list {
                self.mark[i as usize] = false;
            }
            if (self.m as f64) * ratio > (self.rows_removed - base) as f64 {
                break;
            }
            it += 1;
            if flagged <= 0 {
                break;
            }
            base = self.rows_removed;
            if it >= 5 {
                break;
            }
        }
        self.node_free = mat.nfree;
        self.node_extra |= mat.nd.len() as i64 > self.node_cap;
        self.agg_csr(&mat);
        0
    }

    /// CSR -> lists (`519a52` L79-296): rows ascending by column, columns in CSR order, live entries only.
    fn agg_lists(&mut self) -> Mat {
        let (n, m) = (self.n, self.m);
        let mut start = vec![0usize; m + 1];
        for i in 0..m {
            start[i + 1] = start[i] + self.row_cnt[i].max(0) as usize;
        }
        let used = start[m] as i64;
        self.work += m as f64 * 3.0;
        if self.node_extra || self.node_cap < used + self.node_free {
            self.node_cap = used + self.node_free;
            self.node_extra = false;
        }
        let cap = self.node_cap as usize;
        let mut nd = vec![Node { val: 0.0, row: 0, col: -1, nr: NIL, nc: NIL }; cap];
        let mut fill = start.clone();
        let mut ch = vec![NIL; n];
        for j in 0..n {
            if self.col_cnt[j] < 0 {
                continue;
            }
            let mut tail = NIL;
            let b = self.col_beg[j] as usize;
            for k in b..b + self.col_len[j].max(0) as usize {
                let i = self.col_ind[k];
                if i < 0 || self.row_cnt[i as usize] < 0 {
                    continue;
                }
                let p = fill[i as usize];
                fill[i as usize] += 1;
                nd[p] = Node { val: self.col_val[k], row: i, col: j as i32, nr: NIL, nc: NIL };
                if tail == NIL {
                    ch[j] = p as u32;
                } else {
                    nd[tail as usize].nc = p as u32;
                }
                tail = p as u32;
            }
            self.work += self.col_len[j] as f64 * 7.0;
        }
        self.work += n as f64 * 4.0;
        let mut rh = vec![NIL; m];
        for i in 0..m {
            debug_assert_eq!(fill[i], start[i + 1], "row {i}: live count differs from the column CSR");
            if start[i] < start[i + 1] {
                rh[i] = start[i] as u32;
                for p in start[i]..start[i + 1] - 1 {
                    nd[p].nr = (p + 1) as u32;
                }
                self.work += (start[i + 1] - 1 - start[i]) as f64 * 4.0;
            }
        }
        self.work += m as f64 * 6.0;
        for p in used as usize..cap {
            nd[p].nc = if p + 1 < cap { (p + 1) as u32 } else { NIL };
        }
        let free = if (used as usize) < cap { used as u32 } else { NIL };
        Mat { nd, rh, ch, free, nfree: cap as i64 - used, pend: NIL, npend: 0, scat: vec![NIL; n.max(m)] }
    }

    /// Lists -> CSR (`519a52` L405-694): no gaps, list order; live counts stay as aggregation kept them.
    fn agg_csr(&mut self, mat: &Mat) {
        self.work += self.n as f64 + self.n as f64;
        self.row_ind.clear();
        self.row_val.clear();
        for i in 0..self.m {
            self.row_beg[i] = self.row_ind.len() as i64;
            let mut c = 0;
            let mut p = mat.rh[i];
            while p != NIL {
                let q = mat.nd[p as usize];
                if q.col >= 0 {
                    self.row_ind.push(q.col);
                    self.row_val.push(q.val);
                    c += 1;
                }
                p = q.nr;
            }
            self.row_len[i] = c;
            self.work += c as f64 * 3.0;
            debug_assert!(self.row_cnt[i] < 0 || self.row_cnt[i] == c, "row {i}");
        }
        self.work += self.m as f64 * 4.0;
        self.col_ind.clear();
        self.col_val.clear();
        for j in 0..self.n {
            self.col_beg[j] = self.col_ind.len() as i64;
            let mut c = 0;
            let mut p = mat.ch[j];
            while p != NIL {
                let q = mat.nd[p as usize];
                if q.col >= 0 {
                    self.col_ind.push(q.row);
                    self.col_val.push(q.val);
                    c += 1;
                }
                p = q.nc;
            }
            self.col_len[j] = c;
            self.work += c as f64 * 3.0;
            debug_assert!(self.col_cnt[j] < 0 || self.col_cnt[j] == c, "col {j}");
        }
        self.work += self.n as f64 * 4.0;
    }

    /// `53c507` (set) / `53c72d` (clear): bit on every row and column of the current level's a-list.
    fn agg_bits(&mut self, bit: u8, set: bool) {
        let lv = self.tracker.level;
        for q in [&mut self.tracker.rows, &mut self.tracker.cols] {
            self.work += (q.a[lv] as f64 + q.a[lv] as f64) * self.work_mult;
            for idx in 0..q.a[lv] {
                let f = &mut q.flag[q.list[lv][idx] as usize];
                if set {
                    *f |= bit;
                } else {
                    *f &= !bit;
                }
            }
        }
    }

    /// `370047` / `36fcb5`: activities of the listed rows from the row lists (summed from 0, then -rhs added).
    fn agg_activity(&mut self, mat: &Mat, rows: &[i32], inf: f64) {
        for &i in rows {
            let i = i as usize;
            if self.row_cnt[i] < 0 {
                continue;
            }
            let (mut mx, mut mn, mut imx, mut imn) = (0.0, 0.0, 0, 0);
            let mut p = mat.rh[i];
            let mut visited = 0;
            while p != NIL {
                let q = mat.nd[p as usize];
                p = q.nr;
                visited += 1;
                if q.col < 0 {
                    continue;
                }
                let (k, a) = (q.col as usize, q.val);
                let (l, u) = (self.lb[k], self.ub[k]);
                if a <= 0.0 {
                    if -inf < l { mx = guarded_add(mx, l * a, true) } else { imx += 1 }
                    if u < inf { mn = guarded_add(mn, a * u, false) } else { imn += 1 }
                } else {
                    if u < inf { mx = guarded_add(mx, u * a, true) } else { imx += 1 }
                    if -inf < l { mn = guarded_add(mn, a * l, false) } else { imn += 1 }
                }
            }
            self.work += self.work_mult * 7.0 * visited as f64;
            self.inf_max[i] = imx;
            self.inf_min[i] = imn;
            self.act_max[i] = guarded_add(mx, -self.rhs[i], true);
            self.act_min[i] = guarded_add(mn, -self.rhs[i], false);
            let tiny = self.rhs[i].abs() * 1e-13;
            if 0.0 < self.act_min[i] && self.act_min[i] < tiny {
                self.act_min[i] = 0.0;
            }
            if self.act_max[i] < 0.0 && -self.act_max[i] < tiny {
                self.act_max[i] = 0.0;
            }
        }
        self.work += self.work_mult * 8.0 * rows.len() as f64;
    }

    /// Add (`add`) or remove the contribution `a * bound` of a column with bounds [l, u] to row i's activities
    /// (max side first), as the inline updates of `37f484`/`3791fd` do.
    fn agg_contrib(&mut self, i: usize, a: f64, l: f64, u: f64, add: bool, inf: f64) {
        let s = if add { 1 } else { -1 };
        let term = |x: f64| if add { x * a } else { -(x * a) };
        let (xmax, fmax, xmin, fmin) = if a <= 0.0 { (l, -inf < l, u, u < inf) } else { (u, u < inf, l, -inf < l) };
        if fmax {
            self.act_max[i] = guarded_add(self.act_max[i], term(xmax), true);
        } else {
            self.inf_max[i] += s;
        }
        if fmin {
            self.act_min[i] = guarded_add(self.act_min[i], term(xmin), false);
        } else {
            self.inf_min[i] += s;
        }
    }

    /// `53d18c`: mark (tracker + 0x30) every live column of row r.
    fn agg_mark_row(&mut self, mat: &Mat, r: usize) {
        let mut p = mat.rh[r];
        let mut k = 0;
        while p != NIL {
            let q = mat.nd[p as usize];
            if q.col >= 0 {
                self.tracker.mark_col(q.col);
                self.tracker.cols.flag[q.col as usize] |= 0x30;
            }
            p = q.nr;
            k += 1;
        }
        self.work += self.work_mult * 4.0 * k as f64;
    }

    /// `53d2b0`: mark (tracker + 0x30) the row of every live node of column j.
    fn agg_mark_col(&mut self, mat: &Mat, j: usize) {
        let mut p = mat.ch[j];
        let mut k = 0;
        while p != NIL {
            let q = mat.nd[p as usize];
            if q.col >= 0 {
                self.tracker.mark_row(q.row);
                self.tracker.rows.flag[q.row as usize] |= 0x30;
            }
            p = q.nc;
            k += 1;
        }
        self.work += self.work_mult * 5.0 * k as f64;
    }

    /// `36f5c4` (also inlined in `3791fd`): dead nodes leave column j for the pending list.
    fn agg_compact_col(&mut self, mat: &mut Mat, j: usize) {
        let mut p = mat.ch[j];
        let mut prev = NIL;
        let mut k = 0.0;
        while p != NIL {
            k += 1.0;
            let next = mat.nd[p as usize].nc;
            if mat.nd[p as usize].col < 0 {
                mat.nd[p as usize].nc = mat.pend;
                mat.pend = p;
                mat.npend += 1;
                self.nnz -= 1;
            } else {
                if prev == NIL { mat.ch[j] = p } else { mat.nd[prev as usize].nc = p }
                prev = p;
            }
            p = next;
        }
        if prev == NIL { mat.ch[j] = NIL } else { mat.nd[prev as usize].nc = NIL }
        self.work += (k + k) * self.work_mult;
    }

    /// Inline row compaction: dead nodes are only unlinked from row r.
    fn agg_compact_row(&mut self, mat: &mut Mat, r: usize) {
        let mut p = mat.rh[r];
        let mut prev = NIL;
        let mut k = 0.0;
        while p != NIL {
            k += 1.0;
            let next = mat.nd[p as usize].nr;
            if mat.nd[p as usize].col >= 0 {
                if prev == NIL { mat.rh[r] = p } else { mat.nd[prev as usize].nr = p }
                prev = p;
            }
            p = next;
        }
        if prev == NIL { mat.rh[r] = NIL } else { mat.nd[prev as usize].nr = NIL }
        self.work += (k + k) * self.work_mult;
    }

    /// `36f8e0` + head reset: every node of column e goes to pending; live ones leave their row's count.
    fn agg_release_col(&mut self, mat: &mut Mat, e: usize) {
        let mut p = mat.ch[e];
        let mut k = 0;
        while p != NIL {
            k += 1;
            let q = mat.nd[p as usize];
            if q.col >= 0 {
                self.row_cnt[q.row as usize] -= 1;
            }
            mat.npend += 1;
            self.nnz -= 1;
            mat.nd[p as usize].nc = mat.pend;
            mat.nd[p as usize].col = -1;
            mat.pend = p;
            p = q.nc;
        }
        mat.ch[e] = NIL;
        self.work += k as f64 * 3.0 * self.work_mult;
    }

    /// Kill every node of row r and drop the row's list. Returns the nodes visited (work).
    fn agg_kill_row(mat: &mut Mat, r: usize) -> f64 {
        let mut p = mat.rh[r];
        let mut k = 0.0;
        while p != NIL {
            mat.nd[p as usize].col = -1;
            p = mat.nd[p as usize].nr;
            k += 1.0;
        }
        mat.rh[r] = NIL;
        k
    }

    /// `53d3d9`: pending nodes join the free list; columns and rows flagged 0x10 in the current level's b-lists
    /// are compacted (dead column nodes go straight to free) and lose the flag.
    fn agg_gc(&mut self, mat: &mut Mat) {
        let mu = self.work_mult;
        if mat.pend != NIL {
            let mut tail = mat.pend;
            let mut k = 0;
            while mat.nd[tail as usize].nc != NIL {
                tail = mat.nd[tail as usize].nc;
                k += 1;
            }
            self.work += k as f64 * mu;
            mat.nd[tail as usize].nc = mat.free;
            mat.free = mat.pend;
            mat.nfree += mat.npend;
            mat.pend = NIL;
            mat.npend = 0;
        }
        let lv = self.tracker.level;
        for idx in 0..self.tracker.cols.b[lv] {
            let j = self.tracker.cols.list[lv][idx] as usize;
            if self.tracker.cols.flag[j] & 0x10 == 0 {
                continue;
            }
            self.tracker.cols.flag[j] &= !0x10;
            let mut p = mat.ch[j];
            let mut prev = NIL;
            let mut k = 0.0;
            while p != NIL {
                k += 1.0;
                let next = mat.nd[p as usize].nc;
                if mat.nd[p as usize].col < 0 {
                    mat.nd[p as usize].nc = mat.free;
                    mat.free = p;
                    mat.nfree += 1;
                    self.nnz -= 1;
                } else {
                    if prev == NIL { mat.ch[j] = p } else { mat.nd[prev as usize].nc = p }
                    prev = p;
                }
                p = next;
            }
            if prev == NIL { mat.ch[j] = NIL } else { mat.nd[prev as usize].nc = NIL }
            self.work += (k + k) * mu;
        }
        self.work += self.tracker.cols.b[lv] as f64 * 3.0 * mu;
        for idx in 0..self.tracker.rows.b[lv] {
            let i = self.tracker.rows.list[lv][idx] as usize;
            if self.tracker.rows.flag[i] & 0x10 == 0 {
                continue;
            }
            self.tracker.rows.flag[i] &= !0x10;
            // keeps the first row_cnt live nodes and cuts the list there
            let mut left = self.row_cnt[i];
            let mut p = mat.rh[i];
            let mut prev = NIL;
            let mut k = 0.0;
            while p != NIL {
                let next = mat.nd[p as usize].nr;
                if mat.nd[p as usize].col >= 0 {
                    if prev == NIL { mat.rh[i] = p } else { mat.nd[prev as usize].nr = p }
                    prev = p;
                    left -= 1;
                    if left == 0 {
                        break;
                    }
                }
                p = next;
                k += 1.0;
            }
            if prev == NIL { mat.rh[i] = NIL } else { mat.nd[prev as usize].nr = NIL }
            self.work += (k + k) * mu;
        }
        self.work += self.tracker.rows.b[lv] as f64 * 3.0 * mu;
    }

    /// A node for (i, k) with value v at the front of column k: the reused node of the eliminated column (it keeps
    /// its row slot) or a free one put at the front of row i.
    fn agg_new_node(&mut self, mat: &mut Mat, i: usize, k: usize, v: f64, reuse: &mut u32) {
        let p = if *reuse != NIL {
            std::mem::replace(reuse, NIL)
        } else {
            if mat.nfree == 0 {
                mat.grow(self.m.max(3000)); // 3b85ea
            }
            let p = mat.free;
            mat.free = mat.nd[p as usize].nc;
            mat.nfree -= 1;
            self.nnz += 1;
            mat.nd[p as usize].row = i as i32;
            mat.nd[p as usize].nr = mat.rh[i];
            mat.rh[i] = p;
            self.row_cnt[i] += 1;
            p
        };
        let q = &mut mat.nd[p as usize];
        q.col = k as i32;
        q.val = v;
        q.nc = mat.ch[k];
        mat.ch[k] = p;
        self.col_cnt[k] += 1;
    }

    /// One fill update: add v to entry (i, k) (`e`, NIL if none). A new entry needs |v| >= ZERO; an entry whose
    /// new value is within max(1, 10|v|) * ZERO dies. With `act`, row i's activities follow (k's bounds).
    #[allow(clippy::too_many_arguments)]
    fn agg_upd(&mut self, mat: &mut Mat, e: u32, i: usize, k: usize, v: f64, act: bool, reuse: &mut u32, inf: f64) {
        let (l, u) = (self.lb[k], self.ub[k]);
        if e == NIL {
            if ZERO <= v.abs() {
                self.agg_new_node(mat, i, k, v, reuse);
                if act {
                    self.agg_contrib(i, v, l, u, true, inf);
                }
            }
            return;
        }
        let old = mat.nd[e as usize].val;
        if act {
            self.agg_contrib(i, old, l, u, false, inf);
        }
        let new = old + v;
        mat.nd[e as usize].val = new;
        let tol = if 1.0 <= v.abs() * 10.0 { v.abs() * 10.0 } else { 1.0 };
        if new.abs() <= tol * ZERO {
            mat.nd[e as usize].col = -1;
            self.row_cnt[i] -= 1;
            self.col_cnt[k] -= 1;
        } else if act {
            self.agg_contrib(i, new, l, u, true, inf);
        }
    }

    /// `377cb6` with `3773aa`: may the pivot node p's column be substituted out of its row without the
    /// multiplier growth bounds (state+0x80/+0x88) passing AGGMULTLIMIT?
    fn agg_mult_ok(&mut self, mat: &Mat, rc: &mut RowCache, p: u32, lim: f64) -> bool {
        let q = mat.nd[p as usize];
        let r = q.row as usize;
        if rc.hi[r] == 0.0 {
            rc.amax[r] = 0.0;
            rc.amin[r] = 1e101;
            let (mut lo, mut hi) = (1.0f64, 1.0f64);
            let mut bad = false;
            let mut s = mat.rh[r];
            let mut visited = 0;
            while s != NIL {
                let x = mat.nd[s as usize];
                s = x.nr;
                visited += 1;
                if x.col < 0 {
                    continue;
                }
                let a = x.val.abs();
                if rc.amin[r] >= a {
                    rc.amin[r] = a;
                }
                if rc.amax[r] <= a {
                    rc.amax[r] = a;
                }
                let k = x.col as usize;
                let ml = self.mult_lo[k];
                if (ml < lo && {
                    lo = ml;
                    ml < 1.0 / lim
                }) || {
                    let mh = self.mult_hi[k];
                    hi < mh && {
                        hi = mh;
                        lim < mh
                    }
                } {
                    bad = true;
                    visited -= 1;
                    break;
                }
            }
            self.work += self.work_mult * 4.0 * visited as f64;
            if bad {
                rc.hi[r] = lim + lim;
                rc.lo[r] = 1.0 / (lim + lim);
            } else {
                rc.hi[r] = hi;
                rc.lo[r] = lo;
            }
        }
        let a = q.val.abs();
        let j = q.col as usize;
        let inv = 1.0 / lim;
        !(lim < rc.hi[r] || rc.lo[r] < inv || lim < (rc.amax[r] * self.mult_hi[j]) / a || (rc.amin[r] * self.mult_lo[j]) / a < inv)
    }

    /// `378c30__0` (`upper`) / `__1`: is column j's upper (lower) bound implied by one of its rows?
    fn agg_implied(&mut self, mat: &Mat, j: usize, upper: bool, tol_i: f64, t: &ColTols) -> bool {
        let mut visited = 0;
        let found = self.agg_implied_walk(mat, j, upper, tol_i, t, &mut visited);
        self.work += self.work_mult * 8.0 * visited as f64;
        found
    }

    /// `agg_implied` without the work: `visited` gets the nodes passed before the one that implies the bound.
    fn agg_implied_walk(&self, mat: &Mat, j: usize, upper: bool, tol_i: f64, t: &ColTols, visited: &mut i32) -> bool {
        let inf = t.infbound;
        let mut p = mat.ch[j];
        let (base, f) = if upper {
            let l = self.lb[j];
            (if -l < inf { -l } else { 0.0 }, inf <= -l)
        } else {
            let u = self.ub[j];
            (if u < inf { u } else { 0.0 }, inf <= u)
        };
        while p != NIL {
            let q = mat.nd[p as usize];
            p = q.nc;
            *visited += 1;
            if q.col < 0 || q.val.abs() < tol_i {
                continue;
            }
            let (i, a) = (q.row as usize, q.val);
            let eq = self.sense[i] == b'=';
            if upper {
                let v = if -a >= 0.0 {
                    if !(eq && (f as i32) == self.inf_max[i]) {
                        continue;
                    }
                    self.act_max[i] / -a
                } else {
                    if (f as i32) != self.inf_min[i] {
                        continue;
                    }
                    self.act_min[i] / -a
                };
                if -self.ub[j] - t.feastol <= base - v {
                    *visited -= 1;
                    return true;
                }
            } else {
                let v = if a >= 0.0 {
                    if !(eq && (f as i32) == self.inf_max[i]) {
                        continue;
                    }
                    self.act_max[i]
                } else {
                    if (f as i32) != self.inf_min[i] {
                        continue;
                    }
                    self.act_min[i]
                };
                if self.lb[j] - t.feastol <= base - v / a {
                    *visited -= 1;
                    return true;
                }
            }
        }
        false
    }

    /// `3707ac`: one propagation pass over the listed rows into the bound copies (lc, uc). Returns the number of
    /// tightenings, or Err(3) with bad_row/bad_col set.
    fn agg_propagate(&mut self, mat: &Mat, rows: &[i32], lc: &mut [f64], uc: &mut [f64], t: &ColTols, prebound: i32) -> Result<i32, i32> {
        if prebound == 0 {
            return Ok(0);
        }
        let (inf, ft) = (t.infbound, t.feastol);
        let mut cnt = 0;
        for &r in rows {
            let r = r as usize;
            if self.row_cnt[r] <= 0 {
                continue;
            }
            let (mut mx, mut mn) = (-self.rhs[r], -self.rhs[r]);
            let (mut nmx, mut nmn) = (0, 0);
            let mut p = mat.rh[r];
            let mut visited = 0;
            while p != NIL {
                let q = mat.nd[p as usize];
                p = q.nr;
                visited += 1;
                if q.col < 0 {
                    continue;
                }
                let (k, a) = (q.col as usize, q.val);
                if a <= 0.0 {
                    if -inf < lc[k] { mx = guarded_add(mx, lc[k] * a, true) } else { nmx += 1 }
                    if uc[k] < inf { mn = guarded_add(mn, a * uc[k], false) } else { nmn += 1 }
                } else {
                    if uc[k] < inf { mx = guarded_add(mx, uc[k] * a, true) } else { nmx += 1 }
                    if -inf < lc[k] { mn = guarded_add(mn, a * lc[k], false) } else { nmn += 1 }
                }
            }
            self.work += visited as f64 * 5.0 * self.work_mult;
            let d = (mx - mn).max(1.0);
            if 1e13 < d {
                continue;
            }
            let eq = self.sense[r] == b'=';
            if (nmn == 0 && ft < mn) || (eq && nmx == 0 && mx < -ft) {
                self.bad_row = r as i32;
                return Err(3);
            }
            let rr = d.min(1e6);
            let mc = if 0.0 <= -mx { 0.0 } else { -mx };
            let nc = if 0.0 <= mn { 0.0 } else { mn };
            let mut p = mat.rh[r];
            while p != NIL {
                let q = mat.nd[p as usize];
                p = q.nr;
                if q.col < 0 || q.val.abs() < rr * 1e-10 {
                    continue;
                }
                let (k, a) = (q.col as usize, q.val);
                let (lk, uk) = (lc[k], uc[k]);
                let (mut nu, mut nl) = (uk, lk);
                if a > 1e-8 {
                    if nmn == 0 {
                        nu = lk - nc / a;
                    } else if nmn == 1 && lk <= -inf {
                        nu = -mn / a;
                    }
                } else if a < -1e-8 {
                    if nmn == 0 {
                        nl = uk - nc / a;
                    } else if nmn == 1 && inf <= uk {
                        nl = -mn / a;
                    }
                }
                let (mut fl, mut fu) = (nl, nu);
                if eq {
                    let na = -a;
                    let (mut l2, mut u2) = (self.lb[k], self.ub[k]);
                    if na > 1e-8 {
                        if nmx == 0 {
                            u2 = lk - mc / na;
                        } else if nmx == 1 && lk <= -inf {
                            u2 = mx / na;
                        }
                    } else if na < -1e-8 {
                        if nmx == 0 {
                            l2 = uk - mc / na;
                        } else if nmx == 1 && inf <= uk {
                            l2 = mx / na;
                        }
                    }
                    fl = if l2 <= nl { nl } else { l2 };
                    fu = if nu <= u2 { nu } else { u2 };
                }
                if -1e8 < fl && lk + ft * 1000.0 < fl {
                    if uk + ft < fl {
                        self.bad_col = k as i32;
                        return Err(3);
                    }
                    cnt += 1;
                    lc[k] = fl;
                }
                if fu < 1e8 && fu < uc[k] - ft * 1000.0 {
                    if fu < lc[k] - ft {
                        self.bad_col = k as i32;
                        return Err(3);
                    }
                    cnt += 1;
                    uc[k] = fu;
                }
            }
            self.work += self.row_cnt[r] as f64 * 6.0 * self.work_mult;
        }
        self.work += rows.len() as f64 * 4.0 * self.work_mult;
        Ok(cnt)
    }

    /// `37f484` prescan (first call only): an `=` row with a tiny coefficient that the special two-term
    /// aggregation of stage D targets. Follows the original's walk for its work terms.
    fn agg_prescan(&mut self, mat: &Mat, rows: &[i32]) -> bool {
        let inf = 1e30;
        let mu = self.work_mult;
        // first live node of row r whose column is not t (the row is known to have one), and the nodes skipped
        let first_other = |r: usize, t: usize| {
            let mut p = mat.rh[r];
            let mut k = 0;
            while mat.nd[p as usize].col < 0 || mat.nd[p as usize].col as usize == t {
                p = mat.nd[p as usize].nr;
                k += 1;
            }
            (mat.nd[p as usize], k)
        };
        for (idx, &r) in rows.iter().enumerate() {
            let r = r as usize;
            if self.row_cnt[r] != 2 || self.sense[r] != b'=' {
                continue;
            }
            // first two live nodes; k = position of the second
            let mut v = Vec::new();
            let mut p = mat.rh[r];
            let mut k = 0;
            while p != NIL {
                let q = mat.nd[p as usize];
                if q.col >= 0 {
                    v.push((q.col as usize, q.val));
                    if v.len() == 2 {
                        break;
                    }
                }
                p = q.nr;
                k += 1;
            }
            self.work += mu * 3.0 * k as f64;
            let ((c1, a1), (c2, a2)) = (v[0], v[1]);
            if 1e-6 <= a1.abs() && 1e-6 <= a2.abs() {
                continue;
            }
            let t = if a2.abs() < 1e-6 {
                if 1e-10 < (a1.abs() - 1.0).abs() {
                    continue;
                }
                c2
            } else {
                if 1e-10 < (a2.abs() - 1.0).abs() {
                    continue;
                }
                c1
            };
            if self.col_cnt[t] != 3 {
                continue;
            }
            // first and last other rows of t before one that is not a two-term inequality
            let (mut r1, mut r2) = (usize::MAX, usize::MAX);
            let mut k = 0;
            let mut p = mat.ch[t];
            while p != NIL {
                let q = mat.nd[p as usize];
                if q.col >= 0 && q.row as usize != r {
                    let i = q.row as usize;
                    if self.row_cnt[i] != 2 || self.sense[i] == b'=' {
                        break;
                    }
                    if r1 == usize::MAX { r1 = i } else { r2 = i }
                }
                p = q.nc;
                k += 1;
            }
            self.work += mu * 5.0 * k as f64;
            if r2 == usize::MAX {
                continue;
            }
            let (q, k) = first_other(r1, t);
            self.work += mu * 3.0 * k as f64;
            if 1e-10 <= (q.val.abs() - 1.0).abs() {
                continue;
            }
            let x = q.col as usize;
            let (q2, k) = first_other(r2, t);
            self.work += mu * 3.0 * k as f64;
            if 1e-10 < (q2.val.abs() - 1.0).abs() || q2.col as usize != x {
                continue;
            }
            if !(self.col_cnt[x] == 3 && self.lb[x] <= -inf && inf <= self.ub[x]) {
                continue;
            }
            let mut p = mat.ch[x];
            let mut k = 0;
            while p != NIL {
                let q = mat.nd[p as usize];
                // original tests col_cnt of the node's column (x, already 3): the first live node is r3
                if q.col >= 0 && 2 < self.col_cnt[q.col as usize] {
                    break;
                }
                p = q.nc;
                k += 1;
            }
            self.work += mu * 3.0 * k as f64;
            if p == NIL || 1e-10 < (mat.nd[p as usize].val.abs() - 1.0).abs() {
                continue;
            }
            let i3 = mat.nd[p as usize].row as usize;
            let (mut big, mut k) = (0, 0);
            let mut p = mat.rh[i3];
            while p != NIL {
                let q = mat.nd[p as usize];
                if q.col >= 0 && 10000.0 < q.val.abs() {
                    big += 1;
                }
                p = q.nr;
                k += 1;
            }
            self.work += mu * 3.0 * k as f64;
            if big == self.row_cnt[i3] - 1 {
                self.work += mu * 5.0 * idx as f64;
                return true;
            }
        }
        self.work += mu * 5.0 * rows.len() as f64;
        false
    }

    /// `37f484`: one aggregation round (stages A-D) on the lists. Returns a status or 0.
    fn agg_round(&mut self, mat: &mut Mat, cls: &mut [u8], first: bool, t: &ColTols, ap: &AggPar) -> i32 {
        let (n, m) = (self.n, self.m);
        let inf = t.infbound;
        let fill = ap.fill;
        let rho = if t.numeric_focus == 1 { 0.5 } else if t.numeric_focus < 2 { 0.01 } else { 0.9 };
        let rho2 = rho * 0.01;
        let pmin = 1e-6;
        let rows0 = self.rows_removed;
        let cols0 = self.cols_removed;
        let lv = self.tracker.level;
        let rlist: Vec<i32> = self.tracker.rows.list[lv][..self.tracker.rows.a[lv]].to_vec();
        let ccnt = self.tracker.cols.a[lv];
        if self.agg_scan == 0 {
            // shortcut: the tiny-coefficient pattern switches stage D to a special path (37f484 L2325-2470)
            // that is not ported; no Netlib model has it. Port it when a model trips this.
            let found = self.agg_prescan(mat, &rlist); // adds work: keep out of the assert
            assert!(!found, "37f484 prescan pattern found: special stage D path not ported");
            self.agg_scan = -1;
        }
        let (tol_a, tol_i) = if self.agg_scan > 0 { (1e-8, 1e-8) } else { (pmin, t.feastol) };
        let mut rc = RowCache { hi: vec![0.0; m], lo: vec![0.0; m], amax: vec![0.0; m], amin: vec![0.0; m], rowmax: vec![0.0; m] };

        // candidate columns and their class (0 both bounds needed, 1 upper only, 2 lower only, 3 implied free)
        let mu = self.work_mult;
        let cand: Vec<i32> = if ccnt == n && first {
            let mut lc = self.lb.clone();
            let mut uc = self.ub.clone();
            let all: Vec<i32> = (0..m as i32).collect();
            self.work += m as f64 * mu;
            match self.agg_propagate(mat, &all, &mut lc, &mut uc, t, ap.prebound) {
                Err(s) => return s,
                Ok(c) => {
                    if 0 < c && t.numeric_focus == 0 {
                        if let Err(s) = self.agg_propagate(mat, &all, &mut lc, &mut uc, t, ap.prebound) {
                            return s;
                        }
                    }
                }
            }
            self.work += n as f64 * mu;
            for j in 0..n {
                if self.col_cnt[j] > 0 && self.col_flag[j] == 0 {
                    let (l, u) = (self.lb[j], self.ub[j]);
                    let need = (-inf < l && lc[j] <= l + 0.01) || (u < inf && u - 0.01 <= uc[j]);
                    cls[j] = if need {
                        if -inf < l { if u < inf { 0 } else { 2 } } else { 1 }
                    } else {
                        3
                    };
                }
            }
            self.work += n as f64 * 7.0 * mu;
            self.agg_bits(0x20, false); // 53c72d
            (0..n as i32).collect()
        } else {
            // 53c9ee: columns flagged 0x20, then the columns of rows flagged 0x20
            assert!(!first, "37f484 entered with a partial column list (53af0f path) is not ported");
            let mu3 = mu * 3.0;
            self.work += ccnt as f64 * mu3;
            let mut l = Vec::new();
            for idx in 0..ccnt {
                let j = self.tracker.cols.list[lv][idx] as usize;
                if self.tracker.cols.flag[j] & 0x20 != 0 {
                    self.tracker.cols.flag[j] &= !0x20;
                    self.mark[j] = true;
                    l.push(j as i32);
                }
            }
            for idx in 0..self.tracker.rows.a[lv] {
                let i = self.tracker.rows.list[lv][idx] as usize;
                if self.tracker.rows.flag[i] & 0x20 == 0 {
                    continue;
                }
                let mut p = mat.rh[i];
                let mut k = 0;
                while p != NIL {
                    let q = mat.nd[p as usize];
                    if q.col >= 0 && !self.mark[q.col as usize] {
                        self.mark[q.col as usize] = true;
                        l.push(q.col);
                    }
                    p = q.nr;
                    k += 1;
                }
                self.work += mu3 * k as f64;
                self.tracker.rows.flag[i] &= !0x20;
            }
            self.work += self.tracker.rows.a[lv] as f64 * mu3;
            self.work += l.len() as f64 * mu3;
            for &j in &l {
                self.mark[j as usize] = false;
            }
            l.retain(|&j| self.col_cnt[j as usize] >= 0);
            for &j in &l {
                let j = j as usize;
                if self.col_cnt[j] > 0 && self.col_flag[j] == 0 {
                    let (l, u) = (self.lb[j], self.ub[j]);
                    cls[j] = if -inf < l { if u < inf { 0 } else { 2 } } else if u < inf { 1 } else { 3 };
                }
            }
            self.work += l.len() as f64 * 6.0 * mu;
            l
        };

        if let Err(s) = self.agg_stage_a(mat, &mut rc, cls, &cand, tol_a, tol_i, rho, rho2, t, fill) {
            return s;
        }
        if t.dual_reductions && t.precrush < 2 {
            self.agg_stage_b(mat, &mut rc, cls, &cand, tol_a, tol_i, rho, t, fill);
        }
        // stage C needs DualReductions and PREPAIRAGG 1 or 3 (2 is for MIP)
        if t.dual_reductions && (ap.pair_agg == 1 || ap.pair_agg == 3) {
            self.agg_stage_c(mat, &rlist, pmin, t);
        }
        self.agg_stage_d(mat, &rlist, pmin, t);
        if rows0 < self.rows_removed || cols0 < self.cols_removed {
            self.agg_gc(mat);
        }
        0
    }

    /// Stage A (`37f484` L757-1287): an `=` row substitutes out a column whose bounds its rows imply.
    #[allow(clippy::too_many_arguments)]
    fn agg_stage_a(&mut self, mat: &mut Mat, rc: &mut RowCache, cls: &[u8], cand: &[i32], tol_a: f64, tol_i: f64, rho: f64,
                   rho2: f64, t: &ColTols, fill: i32) -> Result<(), i32> {
        let (n, m) = (self.n, self.m);
        let lim_l = t.agg_mult_limit;
        let ff = fill as f64;
        let mut est = vec![-1i32; n];
        let mut bucket = vec![0i32; m + 2];
        for &j in cand {
            let j = j as usize;
            est[j] = -1;
            let c = self.col_cnt[j];
            if !(0 < c && self.col_flag[j] == 0 && 1.0 / lim_l <= self.mult_lo[j] && self.mult_hi[j] <= lim_l) {
                continue;
            }
            let lim = if 2 < c {
                let x = (ff + 2.0) / ((c - 2) as f64) + 3.0001;
                (if x <= (n + 1) as f64 { x } else { (n + 1) as f64 }) as i32
            } else {
                n as i32 + 1
            };
            let mut best = lim;
            let mut p = mat.ch[j];
            while p != NIL {
                let cur = p;
                let q = mat.nd[cur as usize];
                p = q.nc;
                if q.col < 0 || q.val.abs() < tol_a {
                    continue;
                }
                let r = q.row as usize;
                if self.sense[r] != b'=' || best <= self.row_cnt[r] {
                    continue;
                }
                if self.row_cnt[r] == 1 {
                    best = lim;
                    break;
                }
                if self.agg_mult_ok(mat, rc, cur, lim_l) {
                    best = self.row_cnt[r];
                }
            }
            self.work += c as f64 * 7.0 * self.work_mult;
            if best != lim {
                let c2 = c - 2;
                if (c2 as f64) * ((best - 2) as f64) <= (fill + 2) as f64 {
                    let e = if c < 2 { 0.0 } else { (c2 * (best - 1) + 1) as f64 };
                    if e <= m as f64 {
                        est[j] = e as i32;
                        bucket[e as usize + 1] += 1;
                    }
                }
            }
        }
        let mu = self.work_mult;
        self.work += cand.len() as f64 * 6.0 * mu;
        for i in 1..m + 2 {
            bucket[i] += bucket[i - 1];
        }
        self.work += (m + 1) as f64 * mu;
        let mut order = vec![0i32; bucket[m + 1] as usize];
        let mut saved = vec![0i32; n];
        for &j in cand {
            let j = j as usize;
            if est[j] >= 0 {
                let b = &mut bucket[est[j] as usize];
                order[*b as usize] = j as i32;
                *b += 1;
                saved[j] = self.col_cnt[j];
            }
        }
        self.work += cand.len() as f64 * 5.0 * mu;
        let total = bucket[m] as usize;
        let mut keys: Vec<f64> = Vec::new();
        let mut nodes: Vec<i32> = Vec::new();
        for &j in &order[..total] {
            let j = j as usize;
            let c = self.col_cnt[j];
            if !(0 < c && self.col_flag[j] == 0 && c - saved[j] < saved[j] * 2 + 10000) {
                continue;
            }
            let ok = match cls[j] {
                0 => self.agg_implied(mat, j, true, tol_i, t) && self.agg_implied(mat, j, false, tol_i, t),
                1 => self.agg_implied(mat, j, true, tol_i, t),
                2 => self.agg_implied(mat, j, false, tol_i, t),
                _ => true,
            };
            if !ok {
                continue;
            }
            let lim = if 2 < c { ((ff + 2.0) / ((c - 2) as f64) + 3.0001) as i32 } else { n as i32 + 1 };
            let mut colmax = 0.0f64;
            keys.clear();
            nodes.clear();
            let mut give_up = false;
            let mut p = mat.ch[j];
            let mut visited = 0.0;
            while p != NIL {
                let q = mat.nd[p as usize];
                if q.col >= 0 {
                    let a = q.val.abs();
                    if colmax <= a {
                        colmax = a;
                    }
                    let r = q.row as usize;
                    if tol_a <= a && self.sense[r] == b'=' && self.row_cnt[r] <= lim {
                        if self.row_cnt[r] == 1 {
                            give_up = true;
                            break;
                        }
                        keys.push(self.row_cnt[r] as f64);
                        nodes.push(p as i32);
                    }
                }
                p = q.nc;
                visited += 1.0;
            }
            if give_up {
                keys.clear();
            }
            let kk = keys.len() as f64;
            self.work += (visited * 6.0 + kk + kk) * mu;
            if keys.is_empty() {
                continue;
            }
            if keys.len() >= 2 {
                sort_by_key(&mut keys, &mut nodes);
            }
            let thr = colmax * rho;
            let (mut ratio_best, mut cnt_best, mut piv) = (0.0f64, lim, NIL);
            let mut seen = 0;
            for (&key, &p) in keys.iter().zip(nodes.iter()) {
                let cnt = key as i32;
                if cnt_best < cnt {
                    break;
                }
                seen += 1;
                let q = mat.nd[p as usize];
                let r = q.row as usize;
                if rc.rowmax[r] == 0.0 {
                    let mut mx = 0.0f64;
                    let mut s = mat.rh[r];
                    let mut k = 0;
                    while s != NIL {
                        let x = mat.nd[s as usize];
                        if x.col >= 0 && mx <= x.val.abs() {
                            mx = x.val.abs();
                        }
                        s = x.nr;
                        k += 1;
                    }
                    self.work += k as f64 * 3.0 * mu;
                    rc.rowmax[r] = mx;
                }
                let rmax = rc.rowmax[r];
                let a = q.val.abs();
                if (cnt != cnt_best || rmax / a < ratio_best)
                    && (thr <= a || rho * rmax <= a || (rmax * rho2 <= a && cnt < 3))
                    && self.agg_mult_ok(mat, rc, p as u32, lim_l)
                {
                    ratio_best = rmax / a;
                    piv = p as u32;
                    cnt_best = cnt;
                }
            }
            self.work += seen as f64 * 4.0 * mu;
            if piv == NIL {
                continue;
            }
            let need = (cnt_best + fill) as i64;
            if mat.nfree <= self.col_cnt[j] as i64 + need {
                self.agg_gc(mat);
            }
            if !(need + (self.col_cnt[j] as i64) < mat.nfree) {
                continue;
            }
            let r = mat.nd[piv as usize].row as usize;
            let a = mat.nd[piv as usize].val;
            mat.nd[piv as usize].col = -1;
            self.agg_compact_row(mat, r);
            self.agg_compact_col(mat, j);
            self.agg_subst(mat, rc, a, r, j, t.infbound);
            let k = Self::agg_kill_row(mat, r);
            self.work += (k + k) * mu;
        }
        self.work += total as f64 * 10.0 * mu;
        Ok(())
    }

    /// Type-2 record from the lists (`39b9e0`, LP branch): column j = c0 + sum(-a_rk/a x_k) over row r's live
    /// nodes (`cnt` of them), with the multiplier growth bounds of those columns raised; `col` holds j's other
    /// entries. Counts a removed row and column.
    fn agg_push_subst(&mut self, mat: &Mat, j: usize, r: usize, c0: f64, a: f64, cnt: i32) {
        let lc = self.col_cnt[j] - 1;
        let mut row = Vec::with_capacity(cnt.max(0) as usize);
        let mut p = mat.rh[r];
        let mut k = 0;
        while p != NIL {
            let q = mat.nd[p as usize];
            p = q.nr;
            k += 1;
            if q.col < 0 {
                continue;
            }
            row.push((q.col, -q.val / a));
            if q.col as usize != j {
                let k = q.col as usize;
                let s = (q.val / a).abs();
                let lo = self.mult_lo[j] * s;
                if lo <= self.mult_lo[k] {
                    self.mult_lo[k] = lo;
                }
                let hi = self.mult_hi[j] * s;
                if self.mult_hi[k] <= hi {
                    self.mult_hi[k] = hi;
                }
            }
        }
        self.work += k as f64 * 5.0 * self.work_mult;
        let mut col = Vec::new();
        if 0 < lc {
            let mut p = mat.ch[j];
            let mut k = 0;
            while p != NIL {
                let q = mat.nd[p as usize];
                p = q.nc;
                k += 1;
                if q.col >= 0 && q.row as usize != r {
                    col.push((q.row, q.val));
                }
            }
            self.work += k as f64 * 6.0 * self.work_mult;
        }
        self.cols_removed += 1;
        self.rows_removed += 1;
        self.post.push(PostRec::Subst { j: j as i32, r: r as i32, c0, a, obj: self.obj[j], mode: lc, row, col });
    }

    /// `3791fd`: substitute column j out through `=` row r (pivot value a, already taken out of both lists).
    fn agg_subst(&mut self, mat: &mut Mat, rc: &mut RowCache, a: f64, r: usize, j: usize, inf: f64) {
        let rhs_r = self.rhs[r];
        self.agg_push_subst(mat, j, r, rhs_r / a, a, self.row_cnt[r] - 1);
        rc.hi[r] = 0.0;
        rc.rowmax[r] = 0.0;
        self.objcon += self.obj[j] * (rhs_r / a);
        let mu = self.work_mult;
        let mut s_row = 0.0f64;
        let mut len_r = 0;
        let mut p = mat.rh[r];
        while p != NIL {
            let q = mat.nd[p as usize];
            p = q.nr;
            let k = q.col as usize;
            self.col_cnt[k] -= 1;
            self.obj[k] -= (self.obj[j] * q.val) / a;
            s_row += self.col_cnt[k] as f64;
            len_r += 1;
        }
        self.work += len_r as f64 * 4.0 * mu;
        self.agg_mark_row(mat, r);
        self.agg_mark_col(mat, j);
        let (lj, uj) = (self.lb[j], self.ub[j]);
        let mut s_col = 0.0f64;
        let (mut rows, mut vals, mut slot) = (Vec::new(), Vec::new(), Vec::new());
        let mut p = mat.ch[j];
        while p != NIL {
            let q = mat.nd[p as usize];
            let i = q.row as usize;
            let d = (rhs_r * q.val) / a;
            s_col += self.row_cnt[i] as f64;
            self.rhs[i] -= d;
            rc.hi[i] = 0.0;
            rc.rowmax[i] = 0.0;
            self.agg_contrib(i, q.val, lj, uj, false, inf);
            self.act_min[i] = guarded_add(self.act_min[i], d, false);
            self.act_max[i] = guarded_add(self.act_max[i], d, true);
            rows.push(i);
            vals.push(q.val);
            slot.push(p);
            p = q.nc;
        }
        let u = rows.len() as f64;
        self.work += u * 5.0 * mu;
        self.work += u * 6.0 * mu;
        self.row_cnt[r] = -3;
        self.obj[j] = 0.0;
        self.col_cnt[j] = -3 - r as i32;
        mat.ch[j] = NIL;
        if s_row <= s_col {
            // B1: row r's columns outside, j's entries inside
            let mut p = mat.rh[r];
            while p != NIL {
                let qk = mat.nd[p as usize];
                p = qk.nr;
                let k = qk.col as usize;
                if self.tracker.cols.flag[k] & 0x10 != 0 {
                    self.agg_compact_col(mat, k);
                }
                let mut s = mat.ch[k];
                let mut ns = 0.0;
                while s != NIL {
                    mat.scat[mat.nd[s as usize].row as usize] = s;
                    s = mat.nd[s as usize].nc;
                    ns += 1.0;
                }
                self.work += (ns + ns) * mu;
                for tt in 0..rows.len() {
                    let i = rows[tt];
                    let v = (-vals[tt] * qk.val) / a;
                    let e = mat.scat[i];
                    let mut re = slot[tt];
                    self.agg_upd(mat, e, i, k, v, true, &mut re, inf);
                    slot[tt] = re;
                    if e != NIL {
                        mat.scat[i] = NIL;
                    }
                }
                self.work += ns * 16.0 * mu;
                let mut s = mat.ch[k];
                let mut nc = 0.0;
                while s != NIL {
                    let x = mat.nd[s as usize];
                    if x.col >= 0 {
                        mat.scat[x.row as usize] = NIL;
                    }
                    s = x.nc;
                    nc += 1.0;
                }
                self.work += (nc + nc) * mu;
            }
            self.work += len_r as f64 * 5.0 * mu;
        } else {
            // B2: j's entries outside, row r's columns inside (scatter by column)
            for tt in 0..rows.len() {
                let i = rows[tt];
                if self.tracker.rows.flag[i] & 0x10 != 0 {
                    self.agg_compact_row(mat, i);
                }
                let mut s = mat.rh[i];
                let mut ns = 0.0;
                while s != NIL {
                    let x = mat.nd[s as usize];
                    if x.col >= 0 {
                        mat.scat[x.col as usize] = s;
                    }
                    s = x.nr;
                    ns += 1.0;
                }
                self.work += (ns + ns) * mu;
                let mut p = mat.rh[r];
                let mut ni = 0;
                while p != NIL {
                    let qk = mat.nd[p as usize];
                    p = qk.nr;
                    ni += 1;
                    let k = qk.col as usize;
                    let v = (-qk.val * vals[tt]) / a;
                    let e = mat.scat[k];
                    let mut re = slot[tt];
                    self.agg_upd(mat, e, i, k, v, true, &mut re, inf);
                    if re != slot[tt] {
                        mat.scat[j] = NIL;
                    }
                    slot[tt] = re;
                    if e != NIL {
                        mat.scat[k] = NIL;
                    }
                }
                self.work += ni as f64 * 16.0 * mu;
                let mut s = mat.rh[i];
                let mut nc = 0.0;
                while s != NIL {
                    let x = mat.nd[s as usize];
                    if x.col >= 0 {
                        mat.scat[x.col as usize] = NIL;
                    }
                    s = x.nr;
                    nc += 1.0;
                }
                self.work += (nc + nc) * mu;
            }
            self.work += u * 5.0 * mu;
        }
        // nodes of j not reused for fill leave their rows
        for &s in &slot {
            if s != NIL {
                let i = mat.nd[s as usize].row as usize;
                self.row_cnt[i] -= 1;
                mat.nd[s as usize].col = -1;
                mat.nd[s as usize].nc = mat.pend;
                mat.pend = s;
                mat.npend += 1;
                self.nnz -= 1;
            }
        }
        self.work += u * 3.0 * mu;
    }

    /// Stage B (`37f484` L1288-1430, L2555-3890): a `<` row turns a free-ish column into its slack.
    #[allow(clippy::too_many_arguments)]
    fn agg_stage_b(&mut self, mat: &mut Mat, rc: &mut RowCache, cls: &[u8], cand: &[i32], tol_a: f64, tol_i: f64, rho: f64,
                   t: &ColTols, fill: i32) {
        let n = self.n;
        let mu = self.work_mult;
        let lim_l = t.agg_mult_limit;
        let ff = fill as f64;
        let (mut keys, mut nodes) = (Vec::new(), Vec::new());
        for &j in cand {
            let j = j as usize;
            let c = self.col_cnt[j];
            if !(0 < c && self.col_flag[j] == 0 && 1.0 / lim_l <= self.mult_lo[j] && self.mult_hi[j] <= lim_l) {
                continue;
            }
            let lim = if c < 3 { n as i32 } else { ((ff + 1.0) / (c as f64 - 2.0) + 1.0001) as u32 as i32 };
            if lim < 2 {
                continue;
            }
            let ok = match cls[j] {
                0 => false, // LP skips columns that need both bounds
                1 => self.agg_implied(mat, j, true, tol_i, t),
                2 => self.agg_implied(mat, j, false, tol_i, t),
                _ => true,
            };
            if !ok {
                continue;
            }
            let mut colmax = 0.0f64;
            keys.clear();
            nodes.clear();
            let mut give_up = false;
            let mut p = mat.ch[j];
            let mut visited = 0.0;
            while p != NIL {
                let q = mat.nd[p as usize];
                if q.col >= 0 {
                    let a = q.val.abs();
                    if colmax <= a {
                        colmax = a;
                    }
                    let i = q.row as usize;
                    if self.row_cnt[i] == 1 {
                        give_up = true;
                        break;
                    }
                    if self.sense[i] == b'<' && tol_a <= a && self.row_cnt[i] <= lim {
                        keys.push(self.row_cnt[i] as f64);
                        nodes.push(p as i32);
                    }
                }
                p = q.nc;
                visited += 1.0;
            }
            if give_up {
                keys.clear();
            }
            self.work += (visited * 3.0 + keys.len() as f64 * 4.0) * mu;
            if keys.is_empty() {
                continue;
            }
            if keys.len() >= 2 {
                sort_by_key(&mut keys, &mut nodes);
            }
            let thr = colmax * rho;
            let (mut best, mut cnt_best, mut piv) = (0.0f64, n as i32 + 1, NIL);
            let mut seen = 0;
            for (&key, &p) in keys.iter().zip(nodes.iter()) {
                let cnt = key as i32;
                if cnt_best < cnt {
                    break;
                }
                seen += 1;
                let x = if cnt < cnt_best { thr } else { best };
                let a = mat.nd[p as usize].val.abs();
                if x < a && self.agg_mult_ok(mat, rc, p as u32, lim_l) {
                    best = a;
                    piv = p as u32;
                    cnt_best = cnt;
                }
            }
            self.work += seen as f64 * 3.0 * mu;
            if piv == NIL {
                continue;
            }
            let need = (cnt_best + fill) as i64;
            if mat.nfree <= self.col_cnt[j] as i64 + need + 1 {
                self.agg_gc(mat);
            }
            if mat.nfree <= self.col_cnt[j] as i64 + need + 1 {
                continue;
            }
            self.agg_slack(mat, rc, piv, j, t.infbound);
        }
        self.work += cand.len() as f64 * 9.0 * mu;
    }

    /// Stage B substitution: with `<` row r (pivot a on column j), x_j = rhs/a - sum(a_rk/a x_k) - s x'_j where
    /// the new x'_j >= 0 takes j's place (s = sign a). Row r goes away; j's entries become -s a_ij.
    fn agg_slack(&mut self, mat: &mut Mat, rc: &mut RowCache, piv: u32, j: usize, inf: f64) {
        let a = mat.nd[piv as usize].val;
        let r = mat.nd[piv as usize].row as usize;
        let s = if a < 0.0 { -1.0 } else { 1.0 };
        let aa = a.abs();
        let (lj, uj) = (self.lb[j], self.ub[j]);
        let mu = self.work_mult;
        let mut s_col = 0.0f64;
        let mut p = mat.ch[j];
        let mut k = 0;
        while p != NIL {
            let q = mat.nd[p as usize];
            p = q.nc;
            k += 1;
            if q.col < 0 {
                continue;
            }
            let i = q.row as usize;
            s_col += self.row_cnt[i] as f64;
            rc.hi[i] = 0.0;
            self.agg_contrib(i, q.val, lj, uj, false, inf);
        }
        self.work += k as f64 * 3.0 * mu;
        let mut s_row = 0.0f64;
        let mut p = mat.rh[r];
        let mut k = 0;
        while p != NIL {
            let q = mat.nd[p as usize];
            p = q.nr;
            k += 1;
            if q.col >= 0 {
                s_row += self.col_cnt[q.col as usize] as f64;
            }
        }
        self.work += k as f64 * 3.0 * mu;
        mat.nd[piv as usize].val = aa;
        let rhs_r = self.rhs[r];
        self.agg_push_subst(mat, j, r, rhs_r / a, a, self.row_cnt[r]);
        self.cols_removed -= 1;
        self.post.push(PostRec::SlackCol { r: r as i32, j: j as i32, a: aa });
        self.mult_lo[j] = 1.0;
        self.mult_hi[j] = 1.0;
        self.lb[j] = 0.0;
        self.ub[j] = inf;
        rc.hi[r] = 0.0;
        mat.nd[piv as usize].col = -1;
        self.col_cnt[j] -= 1;
        self.objcon += (rhs_r / a) * self.obj[j];
        let mut p = mat.rh[r];
        let mut k = 0;
        while p != NIL {
            let q = mat.nd[p as usize];
            p = q.nr;
            k += 1;
            if q.col >= 0 {
                let k = q.col as usize;
                self.col_cnt[k] -= 1;
                self.obj[k] -= (self.obj[j] * q.val) / a;
            }
        }
        self.work += k as f64 * 5.0 * mu;
        self.obj[j] = -self.obj[j] * s;
        self.agg_compact_row(mat, r);
        self.agg_compact_col(mat, j);
        self.agg_mark_row(mat, r);
        self.agg_mark_col(mat, j);
        self.row_cnt[r] = -3;
        let mut none = NIL;
        let mut outer = 0;
        if s_row <= s_col {
            let mut p = mat.rh[r];
            while p != NIL {
                let qk = mat.nd[p as usize];
                p = qk.nr;
                outer += 1;
                let k = qk.col as usize;
                if self.tracker.cols.flag[k] & 0x10 != 0 {
                    self.agg_compact_col(mat, k);
                }
                let mut x = mat.ch[k];
                let mut ns = 0;
                while x != NIL {
                    mat.scat[mat.nd[x as usize].row as usize] = x;
                    x = mat.nd[x as usize].nc;
                    ns += 1;
                }
                self.work += ns as f64 * 3.0 * mu;
                let mut y = mat.ch[j];
                let mut ni = 0;
                while y != NIL {
                    let qi = mat.nd[y as usize];
                    y = qi.nc;
                    ni += 1;
                    let i = qi.row as usize;
                    let v = (-qi.val * qk.val) / a;
                    let e = mat.scat[i];
                    self.agg_upd(mat, e, i, k, v, true, &mut none, inf);
                    if e != NIL {
                        mat.scat[i] = NIL;
                    }
                }
                self.work += ni as f64 * 10.0 * mu;
                let mut x = mat.ch[k];
                let mut nc = 0;
                while x != NIL {
                    let q = mat.nd[x as usize];
                    if q.col >= 0 {
                        mat.scat[q.row as usize] = NIL;
                    }
                    x = q.nc;
                    nc += 1;
                }
                self.work += nc as f64 * 3.0 * mu;
            }
        } else {
            let mut y = mat.ch[j];
            while y != NIL {
                let qi = mat.nd[y as usize];
                y = qi.nc;
                outer += 1;
                let i = qi.row as usize;
                if self.tracker.rows.flag[i] & 0x10 != 0 {
                    self.agg_compact_row(mat, i);
                }
                let mut x = mat.rh[i];
                let mut ns = 0;
                while x != NIL {
                    let q = mat.nd[x as usize];
                    if q.col >= 0 {
                        mat.scat[q.col as usize] = x;
                    }
                    x = q.nr;
                    ns += 1;
                }
                self.work += ns as f64 * 3.0 * mu;
                let mut p = mat.rh[r];
                let mut ni = 0;
                while p != NIL {
                    let qk = mat.nd[p as usize];
                    p = qk.nr;
                    ni += 1;
                    let k = qk.col as usize;
                    let v = (-qi.val * qk.val) / a;
                    let e = mat.scat[k];
                    self.agg_upd(mat, e, i, k, v, true, &mut none, inf);
                    if e != NIL {
                        mat.scat[k] = NIL;
                    }
                }
                self.work += ni as f64 * 10.0 * mu;
                let mut x = mat.rh[i];
                let mut nc = 0;
                while x != NIL {
                    let q = mat.nd[x as usize];
                    if q.col >= 0 {
                        mat.scat[q.col as usize] = NIL;
                    }
                    x = q.nr;
                    nc += 1;
                }
                self.work += nc as f64 * 3.0 * mu;
            }
        }
        self.work += outer as f64 * 6.0 * mu;
        self.tracker.mark_col(j as i32); // 53d139
        self.tracker.cols.flag[j] |= 0x30;
        let mut y = mat.ch[j];
        let mut k = 0;
        while y != NIL {
            let cur = y as usize;
            let q = mat.nd[cur];
            y = q.nc;
            k += 1;
            let i = q.row as usize;
            let d = (rhs_r * q.val) / a;
            self.rhs[i] -= d;
            let na = -q.val * s;
            mat.nd[cur].val = na;
            self.agg_contrib(i, na, 0.0, inf, true, inf);
            self.act_min[i] = guarded_add(self.act_min[i], d, false);
            self.act_max[i] = guarded_add(self.act_max[i], d, true);
        }
        self.work += k as f64 * 5.0 * mu;
        let k = Self::agg_kill_row(mat, r);
        self.work += k * mu;
    }

    /// Stage C (`37f484` L1431-1925): a 4-term `=` row with zero rhs, P1(x_p1 - x_n1) + P2(x_p2 - x_n2) = 0, whose
    /// columns can be shifted so that min(x_p, x_n) = 0: two columns are merged into the other two.
    fn agg_stage_c(&mut self, mat: &mut Mat, rlist: &[i32], pmin: f64, t: &ColTols) {
        let mut failed = Vec::new();
        for &r in rlist {
            let r = r as usize;
            if !(self.row_cnt[r] == 4 && self.sense[r] == b'=' && self.rhs[r].abs() <= 1e-10) {
                continue;
            }
            let mut pos: Vec<(usize, f64)> = Vec::new();
            let mut neg: Vec<(usize, f64)> = Vec::new();
            let mut fail = false;
            let mut p = mat.rh[r];
            let mut visited = 0;
            while p != NIL {
                let q = mat.nd[p as usize];
                p = q.nr;
                if q.col >= 0 {
                    let k = q.col as usize;
                    if self.col_flag[k] != 0 || 1e-10 < self.lb[k].abs() || self.ub[k] - self.lb[k] < 1e-10 {
                        fail = true;
                        break;
                    }
                    let side = if q.val <= 0.0 { &mut neg } else { &mut pos };
                    if side.len() == 2 {
                        fail = true;
                        break;
                    }
                    side.push((k, q.val));
                }
                visited += 1;
            }
            self.work += visited as f64 * 4.0 * self.work_mult;
            let mut committed = false;
            if !fail && pos.len() == 2 && neg.len() == 2 {
                let ((p1, pv1), (p2, pv2)) = (pos[0], pos[1]);
                let ((mut n1, mut nv1), (mut n2, mut nv2)) = (neg[0], neg[1]);
                let close = |x: f64| x.abs() <= 1e-10;
                if (close(nv1 + pv1) || close(pv1 + nv2)) && (close(nv1 + pv1) || close(nv1 + pv2)) {
                    let mut paired = close(nv1 + pv1) && self.agg_pairable(mat, p1, n1);
                    if !paired && close(pv1 + nv2) && self.agg_pairable(mat, p1, n2) {
                        std::mem::swap(&mut n1, &mut n2);
                        std::mem::swap(&mut nv1, &mut nv2);
                        paired = true;
                    }
                    if paired && close(pv2 + nv2) && self.agg_pairable(mat, p2, n2) {
                        committed = true;
                        self.agg_pair(mat, r, [p1, p2, n1, n2], [pv1, pv2, nv1, nv2], pmin, t);
                    }
                }
            }
            if !committed {
                let mut p = mat.rh[r];
                let mut k = 0.0;
                while p != NIL {
                    let q = mat.nd[p as usize];
                    p = q.nr;
                    if q.col >= 0 && self.col_flag[q.col as usize] == 0 {
                        failed.push(q.col as usize);
                        self.col_flag[q.col as usize] = 0x1000;
                    }
                    k += 1.0;
                }
                self.work += (k + k) * self.work_mult;
            }
        }
        let nf = failed.len() as f64;
        self.work += (nf + nf) * self.work_mult;
        for j in failed {
            self.col_flag[j] = 0;
        }
    }

    /// `395c97`: can columns p and n be shifted together (rows only allow it if p + n stays put on `=` rows and does
    /// not go down on `<` rows, and rows with only one of them do not mind it going up)?
    fn agg_pairable(&mut self, mat: &mut Mat, pc: usize, nc: usize) -> bool {
        if self.obj[pc] + self.obj[nc] < -1e-10 {
            return false;
        }
        let mu = self.work_mult;
        let mut x = mat.ch[nc];
        let mut k = 0;
        while x != NIL {
            let q = mat.nd[x as usize];
            if q.col >= 0 {
                mat.scat[q.row as usize] = x;
            }
            x = q.nc;
            k += 1;
        }
        self.work += mu * 3.0 * k as f64;
        let mut ok = true;
        let mut x = mat.ch[pc];
        let mut k = 0;
        while x != NIL {
            let q = mat.nd[x as usize];
            x = q.nc;
            k += 1;
            if q.col < 0 {
                continue;
            }
            let i = q.row as usize;
            let e = mat.scat[i];
            let eq = self.sense[i] == b'=';
            if e == NIL {
                if eq || q.val < 0.0 {
                    ok = false;
                    k -= 1;
                    break;
                }
            } else {
                let s = q.val + mat.nd[e as usize].val;
                if (eq && 1e-10 < s.abs()) || (!eq && s < -1e-10) {
                    ok = false;
                    k -= 1;
                    break;
                }
                mat.scat[i] = NIL;
            }
        }
        self.work += mu * 4.0 * k as f64;
        // rows of n left in the scatter (no p entry), and cleanup
        let mut x = mat.ch[nc];
        let mut k = 0;
        while x != NIL {
            let q = mat.nd[x as usize];
            x = q.nc;
            k += 1;
            if q.col < 0 {
                continue;
            }
            let i = q.row as usize;
            if mat.scat[i] != NIL {
                if ok && (self.sense[i] == b'=' || q.val < 0.0) {
                    ok = false;
                }
                mat.scat[i] = NIL;
            }
        }
        self.work += mu * 3.0 * k as f64;
        ok
    }

    /// Stage C after pairing (`37f484` from 0x3891f4): ratio and growth checks, then merge.
    fn agg_pair(&mut self, mat: &mut Mat, r: usize, c: [usize; 4], v: [f64; 4], pmin: f64, t: &ColTols) {
        let [p1, p2, n1, n2] = c;
        let [pv1, pv2, nv1, nv2] = v;
        let inf = t.infbound;
        let lim_l = t.agg_mult_limit;
        let rho = pv2 / pv1;
        if 1e6 < pv1 || pv1 < pmin || 1e6 < pv2 || pv2 < pmin || 1e6 < rho || rho < pmin {
            return;
        }
        let (e1, k1, f1) = if self.ub[n2] * rho < self.ub[p1] { (n2, p1, 1.0 / rho) } else { (p1, n2, rho) };
        let (e2, k2, f2) = if self.ub[p2] * rho <= self.ub[n1] { (p2, n1, 1.0 / rho) } else { (n1, p2, rho) };
        let inv = 1.0 / lim_l;
        let grow_ok = |k: usize, e: usize, f: f64, st: &State| {
            inv <= st.mult_lo[k] && st.mult_hi[k] <= lim_l && inv <= f * st.mult_lo[e] && f * st.mult_hi[e] <= lim_l
        };
        if !(grow_ok(k1, e1, f1, self) && grow_ok(k2, e2, f2, self)) {
            return;
        }
        // state+0xfc, +0x278, +0x100 are 0 for LP: nudge the costs so the merged columns stay bounded
        let s = ((self.obj[k1] + self.obj[e1]) + self.obj[e2]) + self.obj[k2];
        if s < t.opttol + t.opttol {
            let d = t.opttol - s * 0.5;
            self.obj[k2] += d;
            self.obj[k1] += d;
        }
        let need = (self.col_cnt[e1] + self.col_cnt[e2]) as i64;
        if mat.nfree < need {
            self.agg_gc(mat);
        }
        if mat.nfree < need {
            return;
        }
        let a1 = if e1 == p1 { pv1 } else { nv2 };
        let a2 = if e2 == p2 { pv2 } else { nv1 };
        // 39d14c: type-21 record
        let (c1, c2) = (self.col_cnt[e1] - 1, self.col_cnt[e2] - 1);
        let lo = self.mult_lo[e1] * f1;
        if lo <= self.mult_lo[k1] {
            self.mult_lo[k1] = lo;
        }
        let hi = f1 * self.mult_hi[e1];
        if self.mult_hi[k1] <= hi {
            self.mult_hi[k1] = hi;
        }
        let lo = self.mult_lo[e2] * f2;
        if lo <= self.mult_lo[k2] {
            self.mult_lo[k2] = lo;
        }
        let hi = f2 * self.mult_hi[e2];
        if self.mult_hi[k2] <= hi {
            self.mult_hi[k2] = hi;
        }
        let mut entries = Vec::new();
        for (e, cnt) in [(e1, c1), (e2, c2)] {
            if 0 < cnt {
                let mut x = mat.ch[e];
                let mut k = 0;
                while x != NIL {
                    let q = mat.nd[x as usize];
                    if q.col >= 0 && q.row as usize != r {
                        entries.push((q.row, q.val));
                    }
                    x = q.nc;
                    k += 1;
                }
                self.work += self.work_mult * 6.0 * k as f64;
            }
        }
        self.cols_removed += 2;
        self.rows_removed += 1;
        self.post.push(PostRec::PairAgg {
            r: r as i32, e1: e1 as i32, e2: e2 as i32, k1: k1 as i32, k2: k2 as i32, n1: c1, n2: c2, f1, f2, a1, a2,
            obj1: self.obj[e1], obj2: self.obj[e2], ub1: self.ub[e1], ub2: self.ub[e2], entries,
        });
        self.agg_mark_row(mat, r);
        self.agg_mark_col(mat, e1);
        self.agg_mark_col(mat, e2);
        self.ub[k1] = if inf <= self.ub[e1] { inf } else { self.ub[e1] / f1 };
        self.ub[k2] = if inf <= self.ub[e2] { inf } else { self.ub[e2] / f2 };
        let k = Self::agg_kill_row(mat, r);
        self.work += k * self.work_mult;
        self.row_cnt[r] = -3;
        self.obj[k1] += f1 * self.obj[e1];
        self.obj[k2] += f2 * self.obj[e2];
        self.obj[e1] = 0.0;
        self.obj[e2] = 0.0;
        self.col_cnt[k1] -= 1;
        self.col_cnt[k2] -= 1;
        self.col_cnt[e1] = -3 - r as i32;
        self.col_cnt[e2] = -3 - r as i32;
        for (e, k, f) in [(e1, k1, f1), (e2, k2, f2)] {
            self.agg_fold(mat, e, k, f, None);
        }
    }

    /// Fold column e into column k: entries f * a_ie are added (no activity update), with `rhs` = (b, a_e) also
    /// moving b a_ie / a_e out of each row's rhs (stage D). Then e's nodes go to pending and k is compacted.
    fn agg_fold(&mut self, mat: &mut Mat, e: usize, k: usize, f: f64, rhs: Option<(f64, f64)>) {
        let mu = self.work_mult;
        let mut x = mat.ch[k];
        let mut c = 0;
        while x != NIL {
            let q = mat.nd[x as usize];
            if q.col >= 0 {
                mat.scat[q.row as usize] = x;
            }
            x = q.nc;
            c += 1;
        }
        self.work += c as f64 * 3.0 * mu;
        let mut none = NIL;
        let mut y = mat.ch[e];
        let mut c = 0;
        while y != NIL {
            let q = mat.nd[y as usize];
            y = q.nc;
            c += 1;
            if q.col < 0 {
                continue;
            }
            let i = q.row as usize;
            let v = match rhs {
                None => f * q.val,
                Some((b, ae)) => {
                    self.rhs[i] -= (b * q.val) / ae;
                    -(f * q.val) / ae
                }
            };
            let s = mat.scat[i];
            self.agg_upd(mat, s, i, k, v, false, &mut none, 0.0);
            if s != NIL {
                mat.scat[i] = NIL;
            }
        }
        self.work += c as f64 * 10.0 * mu;
        let mut x = mat.ch[k];
        let mut c = 0;
        while x != NIL {
            let q = mat.nd[x as usize];
            if q.col >= 0 {
                mat.scat[q.row as usize] = NIL;
            }
            x = q.nc;
            c += 1;
        }
        self.work += c as f64 * 3.0 * mu;
        self.agg_release_col(mat, e);
        if rhs.is_none() {
            self.agg_compact_col(mat, k);
        }
    }

    /// Stage D (`37f484` L1937-2490): a two-term `=` row a_e x_e + a_k x_k = b eliminates x_e (the larger |a|);
    /// e's bounds move onto k.
    fn agg_stage_d(&mut self, mat: &mut Mat, rlist: &[i32], pmin: f64, t: &ColTols) {
        let inf = t.infbound;
        let lim_l = t.agg_mult_limit;
        let inv = 1.0 / lim_l;
        let mu = self.work_mult;
        for &r in rlist {
            let r = r as usize;
            if self.row_cnt[r] != 2 || self.sense[r] != b'=' {
                continue;
            }
            // first two live nodes; pos = position of the second
            let mut two = Vec::with_capacity(2);
            let mut p = mat.rh[r];
            let mut pos = 0;
            while p != NIL {
                let q = mat.nd[p as usize];
                if q.col >= 0 {
                    two.push((q.col as usize, q.val));
                    if two.len() == 2 {
                        break;
                    }
                }
                p = q.nr;
                pos += 1;
            }
            self.work += pos as f64 * 3.0 * mu;
            let ((c1, a1), (c2, a2)) = (two[0], two[1]);
            if !(pmin <= a1.abs() && pmin <= a2.abs()) {
                continue; // the tiny-coefficient path needs the prescan pattern (asserted absent)
            }
            let ((e, ae), (k, ak)) = if a1.abs() < a2.abs() { ((c2, a2), (c1, a1)) } else { ((c1, a1), (c2, a2)) };
            let ratio = (ak / ae).abs();
            if !(inv <= self.mult_lo[k] && self.mult_hi[k] <= lim_l && inv <= self.mult_lo[e] * ratio && ratio * self.mult_hi[e] <= lim_l) {
                continue;
            }
            let (le, ue) = (self.lb[e], self.ub[e]);
            if !((le <= -inf || -1e8 <= le) && (inf <= ue || ue <= 1e8) && (self.col_cnt[e] as i64) <= mat.nfree) {
                continue;
            }
            let b = self.rhs[r];
            let mut code = 0;
            if ak / ae <= 0.0 {
                if -inf < le {
                    let x = (b - le * ae) / ak;
                    if self.lb[k] + 1e-10 < x {
                        code = 1;
                        self.lb[k] = x;
                    }
                }
                if ue < inf {
                    let x = (b - ue * ae) / ak;
                    if x < self.ub[k] - 1e-10 {
                        code += 2;
                        self.ub[k] = x;
                    }
                }
            } else {
                if -inf < le {
                    let x = (b - le * ae) / ak;
                    if x < self.ub[k] - 1e-10 {
                        code = 2;
                        self.ub[k] = x;
                    }
                }
                if ue < inf {
                    let x = (b - ue * ae) / ak;
                    if self.lb[k] + 1e-10 < x {
                        code += 1;
                        self.lb[k] = x;
                    }
                }
            }
            if code != 0 {
                self.post.push(PostRec::AggBound { code, e: e as i32, k: k as i32, r: r as i32, b, ae, ak });
            }
            let mut p = mat.rh[r];
            let mut ke = 0.0;
            while mat.nd[p as usize].col != e as i32 {
                p = mat.nd[p as usize].nr;
                ke += 1.0;
            }
            self.work += (ke + ke) * mu;
            mat.nd[p as usize].col = -1;
            let c0 = b / ae;
            self.agg_push_subst(mat, e, r, c0, ae, 1);
            self.objcon += self.obj[e] * c0;
            let mut p = mat.rh[r];
            let mut c = 0;
            while p != NIL {
                let q = mat.nd[p as usize];
                p = q.nr;
                c += 1;
                if q.col >= 0 {
                    let kk = q.col as usize;
                    self.col_cnt[kk] -= 1;
                    self.obj[kk] -= (self.obj[e] * q.val) / ae;
                }
            }
            self.work += c as f64 * 5.0 * mu;
            self.agg_mark_row(mat, r);
            self.agg_mark_col(mat, e);
            self.obj[e] = 0.0;
            self.row_cnt[r] = -3;
            self.col_cnt[e] = -3 - r as i32;
            self.agg_fold(mat, e, k, ak, Some((b, ae)));
            let c = Self::agg_kill_row(mat, r);
            self.work += (c + c) * mu;
            self.agg_compact_col(mat, k);
        }
        self.work += rlist.len() as f64 * 12.0 * mu;
    }
}

