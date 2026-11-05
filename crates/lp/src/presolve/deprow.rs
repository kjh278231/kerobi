//! `47c445` (PreDepRow): linearly dependent rows, as presolve calls it (mode 1, then mode 2), on the CSR state
//! (state+0xe0 = 1). Mode 1 eliminates the `=` rows (`515b74` builds the matrix), mode 2 the free columns
//! (`516cb4`) with a Markowitz LU of its own; an `=` row left empty with |rhs'| <= 10 feastol is dropped, a free
//! column left empty with |c'| <= 10 feastol is fixed at 0. Notes: docs/m1_presolve_deprow.md.
use super::reduce::ColTols;
use super::state::State;

const NIL: u32 = u32::MAX;
/// GURO_PAR_ZERO (env+0x2498): fill-in and cancellation threshold. shortcut: not in Params, default assumed;
/// add it before running with GURO_PAR_ZERO set.
const ZERO: f64 = 1e-13;
/// pivots and lines below this are "tiny" (local_1c0)
const TINY: f64 = 1e-6;
/// max(MarkowitzTol, 0.01). shortcut: MarkowitzTol (env+0x2478, default 0.0078125) is not in Params; add it
/// before running with MarkowitzTol > 0.01.
const MARKOWITZ: f64 = 0.01;

/// A matrix entry (32-byte node of the original): `row`/`col` are sub-matrix indices, `col` -1 = dead.
#[derive(Clone, Copy)]
struct Node {
    val: f64,
    row: i32,
    col: i32,
    next_r: u32,
    next_c: u32,
}

/// Count buckets (bucket k holds the lines with k+1 entries): circular doubly linked lists whose head's `prev`
/// is -2 - tail and tail's `next` -2 - head. Lines join at the tail.
struct Buckets {
    head: Vec<i32>,
    prev: Vec<i32>,
    next: Vec<i32>,
}

impl Buckets {
    /// Lines with count >= 1 in descending index order per bucket (pushed at the front); returns the empty ones.
    fn new(cnt: &[i32], nb: usize) -> (Buckets, i32) {
        let mut b = Buckets { head: vec![-1; nb], prev: vec![0; cnt.len()], next: vec![0; cnt.len()] };
        let mut tail = vec![-1i32; nb];
        let mut empty = 0;
        for (i, &c) in cnt.iter().enumerate() {
            if c < 1 {
                empty += 1;
                continue;
            }
            let k = c as usize - 1;
            b.next[i] = b.head[k];
            if b.head[k] < 0 {
                tail[k] = i as i32;
            } else {
                b.prev[b.head[k] as usize] = i as i32;
            }
            b.head[k] = i as i32;
        }
        for (k, &t) in tail.iter().enumerate() {
            if b.head[k] >= 0 {
                b.prev[b.head[k] as usize] = -2 - t;
                b.next[t as usize] = -2 - b.head[k];
            }
        }
        (b, empty)
    }

    fn remove(&mut self, i: i32, cnt: i32) {
        let k = cnt as usize - 1;
        let iu = i as usize;
        let (p, nx) = (self.prev[iu], self.next[iu]);
        if p == -2 - i {
            self.head[k] = -1;
        } else if p < 0 {
            self.head[k] = nx;
            self.prev[nx as usize] = p;
            self.next[(-2 - p) as usize] = -2 - nx;
        } else if nx < 0 {
            self.next[p as usize] = nx;
            self.prev[(-2 - nx) as usize] = -2 - p;
        } else {
            self.prev[nx as usize] = p;
            self.next[p as usize] = nx;
        }
    }

    fn append(&mut self, i: i32, cnt: i32) {
        let k = cnt as usize - 1;
        let iu = i as usize;
        let h = self.head[k];
        if h < 0 {
            self.prev[iu] = -2 - i;
            self.next[iu] = -2 - i;
            self.head[k] = i;
        } else {
            self.next[iu] = -2 - h;
            let t = -2 - self.prev[h as usize];
            self.prev[iu] = t;
            self.next[t as usize] = i;
            self.prev[h as usize] = -2 - i;
        }
    }

    /// Restart bucket `k` at line `at` (or the one after it when `at` is not the pivot's line), so the next
    /// search goes on where this one stopped (`47c445` LAB_0052315f / LAB_0052283a).
    fn rotate(&mut self, k: usize, at: i32, pivot_line: i32) {
        let h = self.head[k] as usize;
        self.prev[h] = -2 - self.prev[h];
        self.next[self.prev[h] as usize] = h as i32;
        let c = if at != pivot_line { self.next[at as usize] } else { at };
        self.head[k] = c;
        let t = self.prev[c as usize];
        self.next[t as usize] = -2 - c;
        self.prev[c as usize] = -2 - t;
    }
}

/// How the elimination ended: the finish step runs, or memory limits stopped it (nothing removed).
enum End {
    Finish,
    Abort,
}

/// The elimination of `47c445` on its sub-matrix. Mode 1: rows = `=` rows, columns = all columns, `b` = rhs per
/// row. Mode 2: rows = all rows, columns = free columns, `b` = cost per column.
struct Elim {
    mode: i32,
    nr: usize,
    nc: usize,
    nodes: Vec<Node>,
    rhead: Vec<u32>,
    chead: Vec<u32>,
    rcnt: Vec<i32>,
    ccnt: Vec<i32>,
    /// local_190 bits 1 (row list holds dead nodes) and 2 (column list)
    rflag: Vec<bool>,
    cflag: Vec<bool>,
    /// row buckets (by row count, heads over nc) and column buckets (heads over nr)
    rb: Buckets,
    cb: Buckets,
    b: Vec<f64>,
    empty_r: i32,
    empty_c: i32,
    pivots: i32,
    /// local_290: live entries in the active part
    nnz: i64,
    /// node accounting: free nodes in the current chunk list (local_2c8), freed nodes not yet merged
    /// (local_140), nodes in allocated chunks (local_228), chunk size (local_230)
    free_main: i64,
    free2: i64,
    alloc: i64,
    chunk: i64,
    /// state+0x1b0 scatter (node per row or column)
    scat: Vec<u32>,
    work: f64,
    mu: f64,
}

impl Elim {
    /// Nodes in the original's traversal order of (row, col, value); each list keeps insertion order.
    #[allow(clippy::too_many_arguments)]
    fn new(mode: i32, nr: usize, nc: usize, entries: &[(i32, i32, f64)], b: Vec<f64>, nnz: i64, mu: f64, work: f64) -> Elim {
        let mut nodes = Vec::with_capacity(entries.len());
        let (mut rhead, mut chead) = (vec![NIL; nr], vec![NIL; nc]);
        let (mut rtail, mut ctail) = (vec![NIL; nr], vec![NIL; nc]);
        let (mut rcnt, mut ccnt) = (vec![0i32; nr], vec![0i32; nc]);
        for &(r, c, v) in entries {
            let p = nodes.len() as u32;
            nodes.push(Node { val: v, row: r, col: c, next_r: NIL, next_c: NIL });
            let (ru, cu) = (r as usize, c as usize);
            if rtail[ru] == NIL { rhead[ru] = p } else { nodes[rtail[ru] as usize].next_r = p }
            rtail[ru] = p;
            if ctail[cu] == NIL { chead[cu] = p } else { nodes[ctail[cu] as usize].next_c = p }
            ctail[cu] = p;
            rcnt[ru] += 1;
            ccnt[cu] += 1;
        }
        let (cb, empty_c) = Buckets::new(&ccnt, nr);
        let (rb, empty_r) = Buckets::new(&rcnt, nc);
        let chunk = (nnz / 5).max(nr.max(nc) as i64 * 5);
        Elim {
            mode, nr, nc, nodes, rhead, chead, rcnt, ccnt, rflag: vec![false; nr], cflag: vec![false; nc], rb, cb, b,
            empty_r, empty_c, pivots: 0, nnz, free_main: chunk, free2: 0, alloc: chunk, chunk,
            scat: vec![NIL; nr.max(nc)], work, mu,
        }
    }

    fn row_nodes(&self, r: usize) -> Vec<u32> {
        let mut v = Vec::new();
        let mut p = self.rhead[r];
        while p != NIL {
            v.push(p);
            p = self.nodes[p as usize].next_r;
        }
        v
    }

    fn col_nodes(&self, c: usize) -> Vec<u32> {
        let mut v = Vec::new();
        let mut p = self.chead[c];
        while p != NIL {
            v.push(p);
            p = self.nodes[p as usize].next_c;
        }
        v
    }

    fn alive(&self, p: u32) -> bool {
        self.nodes[p as usize].col >= 0
    }

    /// Drop dead nodes from row `r`'s list; returns (entries visited, max |a| of the live ones).
    fn compact_row(&mut self, r: usize) -> (usize, f64) {
        let all = self.row_nodes(r);
        let live: Vec<u32> = all.iter().copied().filter(|&p| self.alive(p)).collect();
        let mut mx = 0.0f64;
        for &p in &live {
            let a = self.nodes[p as usize].val.abs();
            if mx <= a {
                mx = a;
            }
        }
        self.rhead[r] = live.first().copied().unwrap_or(NIL);
        for w in live.windows(2) {
            self.nodes[w[0] as usize].next_r = w[1];
        }
        if let Some(&l) = live.last() {
            self.nodes[l as usize].next_r = NIL;
        }
        self.rflag[r] = false;
        (all.len(), mx)
    }

    /// Drop dead nodes from column `c`'s list (they become free: `free2`, or `free_main` when `main`).
    fn compact_col(&mut self, c: usize, main: bool) -> (usize, f64) {
        let all = self.col_nodes(c);
        let live: Vec<u32> = all.iter().copied().filter(|&p| self.alive(p)).collect();
        let dead = (all.len() - live.len()) as i64;
        if main { self.free_main += dead } else { self.free2 += dead }
        let mut mx = 0.0f64;
        for &p in &live {
            let a = self.nodes[p as usize].val.abs();
            if mx <= a {
                mx = a;
            }
        }
        self.chead[c] = live.first().copied().unwrap_or(NIL);
        for w in live.windows(2) {
            self.nodes[w[0] as usize].next_c = w[1];
        }
        if let Some(&l) = live.last() {
            self.nodes[l as usize].next_c = NIL;
        }
        self.cflag[c] = false;
        (all.len(), mx)
    }

    /// One entry fewer in row `i` (taken out of its bucket by the caller or here); re-bucket or count it empty.
    fn row_lost(&mut self, i: i32) {
        let iu = i as usize;
        self.rcnt[iu] -= 1;
        if self.rcnt[iu] == 0 {
            self.empty_r += 1;
        } else {
            self.rb.append(i, self.rcnt[iu]);
        }
        self.rflag[iu] = true;
    }

    fn col_lost(&mut self, k: i32) {
        let ku = k as usize;
        self.ccnt[ku] -= 1;
        if self.ccnt[ku] == 0 {
            self.empty_c += 1;
        } else {
            self.cb.append(k, self.ccnt[ku]);
        }
        self.cflag[ku] = true;
    }

    /// `47bbb6`: take column `c` out of its rows; all its nodes become free.
    fn remove_col(&mut self, c: usize) {
        for p in self.col_nodes(c) {
            if self.alive(p) {
                let i = self.nodes[p as usize].row;
                self.rb.remove(i, self.rcnt[i as usize]);
                self.row_lost(i);
            }
            self.free2 += 1;
            self.nodes[p as usize].col = -1;
        }
    }

    /// `47bec3` (and its inline copies): take row `r` out of its columns.
    fn kill_row(&mut self, r: usize) {
        for p in self.row_nodes(r) {
            if self.alive(p) {
                let k = self.nodes[p as usize].col;
                self.cb.remove(k, self.ccnt[k as usize]);
                self.col_lost(k);
                self.nodes[p as usize].col = -1;
            }
        }
    }

    fn row_singleton(&mut self, r: usize, keep: f64) {
        self.rb.remove(r as i32, self.rcnt[r]);
        if self.rflag[r] {
            self.compact_row(r);
        }
        let p = self.rhead[r] as usize;
        let (c, a) = (self.nodes[p].col as usize, self.nodes[p].val);
        let cc = self.ccnt[c];
        self.work += cc as f64 * (self.mu * 10.0);
        if TINY <= a.abs() || self.mode != 1 {
            self.nnz -= cc as i64;
            self.cb.remove(c as i32, cc);
            self.nodes[p].col = -1;
            if self.mode == 1 {
                for q in self.col_nodes(c) {
                    let q = self.nodes[q as usize];
                    if q.col >= 0 {
                        self.b[q.row as usize] -= (self.b[r] * q.val) / a;
                    }
                }
            }
            self.remove_col(c);
            self.pivots += 1;
            self.chead[c] = NIL;
            self.rcnt[r] = -1;
            self.ccnt[c] = -1;
        } else {
            self.empty_r += 1;
            self.rcnt[r] = 0;
            self.nnz -= 1;
            self.kill_row(r);
            if 1e-10 < a.abs() {
                self.b[r] = keep;
            }
        }
        self.rhead[r] = NIL;
    }

    fn col_singleton(&mut self, c: usize, keep: f64) {
        self.cb.remove(c as i32, self.ccnt[c]);
        if self.cflag[c] {
            self.compact_col(c, false);
        }
        let p = self.chead[c] as usize;
        let (r, a) = (self.nodes[p].row as usize, self.nodes[p].val);
        let rc = self.rcnt[r];
        self.work += rc as f64 * (self.mu * 10.0);
        if TINY <= a.abs() || self.mode != 2 {
            self.nnz -= rc as i64;
            self.rb.remove(r as i32, rc);
            self.nodes[p].col = -1;
            if self.mode == 2 {
                for q in self.row_nodes(r) {
                    let q = self.nodes[q as usize];
                    if q.col >= 0 {
                        self.b[q.col as usize] -= (self.b[c] * q.val) / a;
                    }
                }
            }
            for q in self.row_nodes(r) {
                if self.alive(q) {
                    let k = self.nodes[q as usize].col;
                    self.cb.remove(k, self.ccnt[k as usize]);
                    self.col_lost(k);
                    self.nodes[q as usize].col = -1;
                }
            }
            self.rhead[r] = NIL;
            self.pivots += 1;
            self.free2 += 1;
            self.rcnt[r] = -1;
            self.ccnt[c] = -1;
        } else {
            self.empty_c += 1;
            self.ccnt[c] = 0;
            self.nnz -= 1;
            self.remove_col(c);
            if 1e-10 < a.abs() {
                self.b[c] = keep;
            }
        }
        self.chead[c] = NIL;
    }

    /// Column singletons first, then one row singleton at a time, each followed by all column singletons.
    fn singletons(&mut self, keep: f64) {
        loop {
            while self.cb.head[0] >= 0 {
                self.col_singleton(self.cb.head[0] as usize, keep);
            }
            if self.rb.head[0] < 0 {
                break;
            }
            self.row_singleton(self.rb.head[0] as usize, keep);
        }
    }

    fn remaining(&self) -> (i32, i32) {
        (self.nr as i32 - self.empty_r - self.pivots, self.nc as i32 - self.empty_c - self.pivots)
    }

    /// Mode 1 with >= 3001 rows and columns left (first time only): rows meeting a column of more than 3000
    /// entries leave the elimination and are kept. Returns false when there is no such column.
    fn dense_cols(&mut self, keep: f64) -> bool {
        let Some(first) = (0..self.nc).find(|&c| self.ccnt[c] >= 3001) else {
            return false;
        };
        let mut hits = vec![0i32; self.nr];
        for c in first..self.nc {
            if 3000 < self.ccnt[c] {
                for p in self.col_nodes(c) {
                    if self.alive(p) {
                        hits[self.nodes[p as usize].row as usize] += 1;
                    }
                }
            }
        }
        for (r, &h) in hits.iter().enumerate() {
            if h != 0 && 0 < self.rcnt[r] {
                self.rb.remove(r as i32, self.rcnt[r]);
                self.nnz -= self.rcnt[r] as i64;
                self.kill_row(r);
                self.rcnt[r] = 0;
                self.empty_r += 1;
                self.rhead[r] = NIL;
                self.b[r] = keep;
            }
        }
        true
    }

    /// Stop when the active part is too dense for this sparse elimination (LAB_00520b12).
    fn too_dense(&self, auto: bool) -> bool {
        let (rows, cols) = self.remaining();
        let nz = self.nnz as f64;
        if !auto {
            return (rows as f64) * 0.3 * (cols as f64) < nz;
        }
        if 100.max(self.empty_r) < rows {
            let d = rows as f64;
            return d * 8.0 * d < nz || d * 0.3 * (cols as f64) < nz;
        }
        false
    }

    /// Lowest bucket index >= 1 holding a line, or `n` when none (the scan of `47c445` before the search).
    fn first_bucket(head: &[i32], n: usize) -> usize {
        if n <= 1 {
            return 1;
        }
        (1..n).find(|&k| head[k] >= 0).unwrap_or(n)
    }

    /// Markowitz search over buckets of growing count, then the pivot or the tiny line found on the way.
    fn step(&mut self, u: i32, keep: f64, lim_fill: i64, lim_alloc: i64) -> Option<End> {
        let kc = Self::first_bucket(&self.cb.head, self.nr);
        let kr = Self::first_bucket(&self.rb.head, self.nc);
        let mut k = kc.min(kr);
        let lb = kc as f64 * kr as f64;
        let limit = (k as i32 / 2).max(8).min(u);
        let mut best = self.nr as f64 * self.nc as f64;
        let mut piv: Option<u32> = None;
        let mut ratio = 0i32;
        let mut bad = -1i32;
        let mut cnt = 0i32;
        // k < u always holds here (every live line sits in a bucket below u)
        'k: loop {
            let pending = bad;
            // columns of count k+1
            let mut c = self.cb.head[k];
            let mut early = c < 0; // LAB_0052306c: leave the column part, stop if the limit is reached
            if c >= 0 {
                'cols: loop {
                    let mx = loop {
                        let (_, mx) = self.compact_col(c as usize, false);
                        if mx < TINY && self.mode == 2 {
                            bad = c;
                            if 1e-10 < mx {
                                self.b[c as usize] = keep;
                            }
                            early = true;
                            break 'cols;
                        }
                        if ZERO <= mx {
                            break mx;
                        }
                        c = self.cb.next[c as usize];
                        if c < 0 {
                            early = true;
                            break 'cols;
                        }
                    };
                    for p in self.col_nodes(c as usize) {
                        let nd = self.nodes[p as usize];
                        let mk = (self.rcnt[nd.row as usize] - 1) as f64 * k as f64;
                        let a = nd.val.abs();
                        if (mk < best && MARKOWITZ * mx < a) || (mk == best && ratio as f64 * mx < a) {
                            ratio = (a / mx) as i32;
                            best = mk;
                            piv = Some(p);
                        }
                    }
                    cnt += 1;
                    if pending >= 0 {
                        early = true;
                        break 'cols;
                    }
                    if limit <= cnt {
                        if let Some(p) = piv {
                            if self.cb.next[c as usize] >= 0 {
                                self.cb.rotate(k, c, self.nodes[p as usize].col);
                            }
                        }
                        break 'k;
                    }
                    if best <= lb {
                        if let Some(p) = piv {
                            if self.cb.next[c as usize] >= 0 {
                                self.cb.rotate(k, c, self.nodes[p as usize].col);
                            }
                        }
                        break 'k;
                    }
                    c = self.cb.next[c as usize];
                    if c < 0 {
                        break 'cols;
                    }
                }
            }
            if early && limit <= cnt {
                break 'k;
            }
            let pending = bad;
            if best <= lb {
                break 'k;
            }
            // rows of count k+1
            let mut r = self.rb.head[k];
            while r >= 0 {
                let (_, mx) = self.compact_row(r as usize);
                if mx < TINY && self.mode == 1 {
                    bad = r + self.nc as i32;
                    if 1e-10 < mx {
                        self.b[r as usize] = keep;
                    }
                    break;
                }
                for p in self.row_nodes(r as usize) {
                    let nd = self.nodes[p as usize];
                    let mk = (self.ccnt[nd.col as usize] - 1) as f64 * k as f64;
                    let a = nd.val.abs();
                    if (mk < best && MARKOWITZ * mx < a) || (mk == best && ratio as f64 * mx < a) {
                        ratio = (a / mx) as i32;
                        best = mk;
                        piv = Some(p);
                    }
                }
                cnt += 1;
                if pending >= 0 {
                    if limit <= cnt {
                        break 'k;
                    }
                    break;
                }
                if limit <= cnt {
                    if let Some(p) = piv {
                        if self.rb.next[r as usize] >= 0 {
                            self.rb.rotate(k, r, self.nodes[p as usize].row);
                        }
                    }
                    break 'k;
                }
                if best <= lb {
                    if let Some(p) = piv {
                        if self.rb.next[r as usize] >= 0 {
                            self.rb.rotate(k, r, self.nodes[p as usize].row);
                        }
                    }
                    break;
                }
                r = self.rb.next[r as usize];
            }
            // LAB_00521154
            if best <= lb {
                break;
            }
            k += 1;
            if u <= k as i32 {
                break;
            }
        }
        let mut w = cnt as f64 * (self.mu * 10.0) * k as f64 + self.work;
        if bad >= 0 {
            self.work = w;
            if bad < self.nc as i32 {
                let c = bad as usize;
                self.cb.remove(bad, self.ccnt[c]);
                self.nnz -= self.ccnt[c] as i64;
                self.remove_col(c);
                self.empty_c += 1;
                self.ccnt[c] = 0;
                self.chead[c] = NIL;
            } else {
                let r = (bad - self.nc as i32) as usize;
                self.rb.remove(r as i32, self.rcnt[r]);
                self.nnz -= self.rcnt[r] as i64;
                self.kill_row(r);
                self.empty_r += 1;
                self.rcnt[r] = 0;
                self.rhead[r] = NIL;
            }
            return None;
        }
        let p = piv.expect("47c445: the search always finds a pivot") as usize;
        let (r, c) = (self.nodes[p].row as usize, self.nodes[p].col as usize);
        self.rb.remove(r as i32, self.rcnt[r]);
        self.cb.remove(c as i32, self.ccnt[c]);
        let fill = (self.rcnt[r] - 1).wrapping_mul(self.ccnt[c] - 1) as i64;
        if lim_fill < fill {
            self.work = w;
            return Some(End::Abort);
        }
        if self.free_main <= fill {
            self.free_main += self.free2;
            self.free2 = 0;
            for j in 0..self.nc {
                if self.cflag[j] {
                    w += self.compact_col(j, true).0 as f64 * self.mu;
                }
            }
            for i in 0..self.nr {
                if self.rflag[i] {
                    w += self.compact_row(i).0 as f64 * self.mu;
                }
            }
            if !(fill < self.free_main) {
                let sz = (fill * 2).max(self.chunk);
                self.work = w;
                self.alloc += sz;
                if lim_alloc < self.alloc {
                    return Some(End::Abort);
                }
                self.free_main += sz;
            }
        }
        let (rc, cc) = (self.rcnt[r], self.ccnt[c]);
        self.work = w + self.mu * 20.0 * rc as f64 * cc as f64;
        self.nnz -= (rc + cc) as i64 - 1;
        self.pivots += 1;
        let pv = self.nodes[p].val;
        self.nodes[p].col = -1;
        self.compact_row(r);
        self.compact_col(c, false);
        let (mut sr, mut sc) = (0.0f64, 0.0f64);
        let colc = self.col_nodes(c);
        let rowr = self.row_nodes(r);
        for &q in &colc {
            let nd = self.nodes[q as usize];
            if self.mode == 1 {
                self.b[nd.row as usize] -= (self.b[r] * nd.val) / pv;
            }
            sr += self.rcnt[nd.row as usize] as f64;
            self.rb.remove(nd.row, self.rcnt[nd.row as usize]);
        }
        for &q in &rowr {
            let nd = self.nodes[q as usize];
            if self.mode == 2 {
                self.b[nd.col as usize] -= (self.b[c] * nd.val) / pv;
            }
            sc += self.ccnt[nd.col as usize] as f64;
            self.cb.remove(nd.col, self.ccnt[nd.col as usize]);
        }
        self.rcnt[r] = -1;
        self.ccnt[c] = -1;
        if sc <= sr {
            // column by column: scatter column k, run down the pivot column
            for &q in &rowr {
                let (kk, ark) = (self.nodes[q as usize].col as usize, self.nodes[q as usize].val);
                if self.cflag[kk] {
                    self.compact_col(kk, false);
                }
                for e in self.col_nodes(kk) {
                    self.scat[self.nodes[e as usize].row as usize] = e;
                }
                for &q2 in &colc {
                    let (i, aic) = (self.nodes[q2 as usize].row as usize, self.nodes[q2 as usize].val);
                    let d = (-aic * ark) / pv;
                    self.update(i, kk, d, i);
                }
                for e in self.col_nodes(kk) {
                    if self.alive(e) {
                        self.scat[self.nodes[e as usize].row as usize] = NIL;
                    }
                }
            }
        } else {
            // row by row: scatter row i, run along the pivot row
            for &q in &colc {
                let (i, aic) = (self.nodes[q as usize].row as usize, self.nodes[q as usize].val);
                if self.rflag[i] {
                    self.compact_row(i);
                }
                for e in self.row_nodes(i) {
                    self.scat[self.nodes[e as usize].col as usize] = e;
                }
                for &q2 in &rowr {
                    let (kk, ark) = (self.nodes[q2 as usize].col as usize, self.nodes[q2 as usize].val);
                    let d = (-ark * aic) / pv;
                    self.update(i, kk, d, kk);
                }
                for e in self.row_nodes(i) {
                    if self.alive(e) {
                        self.scat[self.nodes[e as usize].col as usize] = NIL;
                    }
                }
            }
        }
        for q in self.col_nodes(c) {
            if self.alive(q) {
                self.row_lost(self.nodes[q as usize].row);
            }
            self.free2 += 1;
            self.nodes[q as usize].col = -1;
        }
        for q in self.row_nodes(r) {
            if self.alive(q) {
                self.col_lost(self.nodes[q as usize].col);
                self.nodes[q as usize].col = -1;
            }
        }
        self.chead[c] = NIL;
        self.rhead[r] = NIL;
        None
    }

    /// Add `d` to entry (i, k) found through the scatter slot `s`, or create it at the front of both lists.
    fn update(&mut self, i: usize, k: usize, d: f64, s: usize) {
        let e = self.scat[s];
        if e == NIL {
            if ZERO <= d.abs() {
                self.free_main -= 1;
                self.nnz += 1;
                let p = self.nodes.len() as u32;
                self.nodes.push(Node { val: d, row: i as i32, col: k as i32, next_r: self.rhead[i], next_c: self.chead[k] });
                self.rhead[i] = p;
                self.chead[k] = p;
                self.rcnt[i] += 1;
                self.ccnt[k] += 1;
            }
            return;
        }
        let nd = &mut self.nodes[e as usize];
        nd.val += d;
        if nd.val.abs() < ZERO {
            nd.col = -1;
            self.nnz -= 1;
            self.rcnt[i] -= 1;
            self.ccnt[k] -= 1;
        }
        self.scat[s] = NIL;
    }

    /// The elimination loop (LAB_0051e164); ends when nothing is left, the rest is too dense, or a memory limit.
    fn run(&mut self, auto: bool, keep: f64, lim_fill: i64, lim_alloc: i64) -> End {
        let mut first = true;
        loop {
            // shortcut: every 100 rounds the original calls 3ba5ac (log) and 3ba9de, whose work-budget stop
            // (state+0x1dc < 0) ends the elimination; the budget is not modeled (see mod.rs). Port with P2.
            self.singletons(keep);
            let (rows, cols) = self.remaining();
            let mut u = rows.min(cols);
            if u == 0 {
                return End::Finish;
            }
            if first {
                first = false;
                if auto && self.mode == 1 && 3001 <= u && self.dense_cols(keep) {
                    let (rows, cols) = self.remaining();
                    u = rows.min(cols);
                    if u == 0 {
                        return End::Finish;
                    }
                    if self.cb.head[0] >= 0 || self.rb.head[0] >= 0 {
                        continue;
                    }
                }
            }
            if 50 < u && self.too_dense(auto) {
                return End::Finish;
            }
            if let Some(e) = self.step(u, keep, lim_fill, lim_alloc) {
                return e;
            }
        }
    }
}

impl State {
    /// `47c445(work_mult, state, 0, env, 0, mode, work)`: mode 1 drops dependent `=` rows, mode 2 fixes
    /// dependent free columns at 0. `auto` = PreDepRow left at -1 (picks the density stop). Returns 3 when a
    /// dependent row has |rhs'| >= 1e4 feastol, 5 when a dependent free column has |c'| >= 1e4 feastol, else 0.
    pub fn dependent_rows(&mut self, mode: i32, t: &ColTols, auto: bool) -> i32 {
        let mu = self.work_mult;
        let ft10 = t.feastol * 10.0;
        let big = (self.n as i64 * 2).max(self.nnz);
        let (lim_fill, lim_alloc) = (big * 2, if 100_000_000 < self.nnz { big * 5 } else { big * 10 });
        self.work += mu * 3.0 * self.nnz as f64;
        let (mut e, idx) = if mode == 1 { self.eq_matrix(mu) } else { self.free_matrix(mu) };
        if idx.is_empty() {
            return 0;
        }
        let end = e.run(auto, ft10 * 100.0, lim_fill, lim_alloc);
        self.work = e.work;
        if let End::Abort = end {
            return 0;
        }
        if mode == 1 {
            let mut removed = 0;
            for (s, &i) in idx.iter().enumerate() {
                if e.rcnt[s] != 0 {
                    continue;
                }
                let b = e.b[s].abs();
                if ft10 < b {
                    if b < ft10 * 1000.0 {
                        continue;
                    }
                    return 3;
                }
                removed += 1;
                let i = i as usize;
                self.mark_cols_of_row(i);
                let rb = self.row_beg[i] as usize;
                for k in rb..rb + self.row_len[i] as usize {
                    let j = self.row_ind[k];
                    if j >= 0 && self.col_cnt[j as usize] >= 0 {
                        self.col_cnt[j as usize] -= 1;
                        self.row_ind[k] = -1;
                    }
                }
                self.row_cnt[i] = -2;
            }
            self.rows_removed += removed;
        } else {
            for (s, &j) in idx.iter().enumerate() {
                if e.ccnt[s] != 0 {
                    continue;
                }
                let o = e.b[s].abs();
                if ft10 < o {
                    if ft10 * 1000.0 <= o {
                        return 5;
                    }
                } else {
                    self.fix_at(j as usize, 0.0, t.infbound, true);
                }
            }
        }
        0
    }

    /// `515b74`: the `=` rows with live entries over all columns, column by column (rows keep column order).
    /// Returns the matrix and the sub-row -> row map.
    fn eq_matrix(&mut self, mu: f64) -> (Elim, Vec<i32>) {
        let mut map = vec![-1i32; self.m];
        let mut idx = Vec::new();
        let mut nnz = 0i64;
        for (i, slot) in map.iter_mut().enumerate() {
            if self.sense[i] == b'=' && 0 < self.row_cnt[i] {
                *slot = idx.len() as i32;
                idx.push(i as i32);
                nnz += self.row_cnt[i] as i64;
            }
        }
        self.work += self.m as f64 * 4.0;
        let neq = idx.len();
        let mut ent = Vec::new();
        if neq != 0 {
            self.work += self.m as f64 * 5.0;
            for j in 0..self.n {
                if self.col_cnt[j] < 0 {
                    continue;
                }
                let b = self.col_beg[j] as usize;
                for k in b..b + self.col_len[j].max(0) as usize {
                    let i = self.col_ind[k];
                    if i >= 0 && self.row_cnt[i as usize] >= 0 && map[i as usize] >= 0 {
                        ent.push((map[i as usize], j as i32, self.col_val[k]));
                    }
                }
                self.work += self.col_len[j] as f64 * 9.0;
            }
            self.work += self.n as f64 * 3.0 + neq as f64;
            let mut per = vec![0i32; neq];
            for e in &ent {
                per[e.0 as usize] += 1;
            }
            for &c in &per {
                self.work += c as f64;
            }
            self.work += neq as f64 * 8.0;
            self.work += neq as f64 * 3.0;
        }
        let b: Vec<f64> = idx.iter().map(|&i| self.rhs[i as usize]).collect();
        // column-major traversal: column lists in column-CSR order, row lists in column order
        let e = Elim::new(1, neq, self.n, &ent, b, nnz, mu, self.work);
        (e, idx)
    }

    /// `516cb4`: the free columns (lb <= -1e30, ub >= 1e30, live, not special) over all rows, row by row.
    fn free_matrix(&mut self, mu: f64) -> (Elim, Vec<i32>) {
        let mut map = vec![-1i32; self.n];
        let mut idx = Vec::new();
        let mut nnz = 0i64;
        for (j, slot) in map.iter_mut().enumerate() {
            if 0 < self.col_cnt[j] && self.col_flag[j] == 0 && self.lb[j] <= -1e30 && 1e30 <= self.ub[j] {
                *slot = idx.len() as i32;
                idx.push(j as i32);
                nnz += self.col_cnt[j] as i64;
            }
        }
        self.work += self.n as f64 * 6.0;
        let nf = idx.len();
        let mut ent = Vec::new();
        if nf != 0 {
            self.work += (nf + 2 + self.m) as i32 as f64;
            // gathering the free columns' costs and counts
            self.work += self.n as f64 * 6.0;
            for i in 0..self.m {
                if self.row_cnt[i] < 0 {
                    continue;
                }
                let b = self.row_beg[i] as usize;
                for k in b..b + self.row_len[i].max(0) as usize {
                    let j = self.row_ind[k];
                    if j >= 0 && self.col_cnt[j as usize] >= 0 && map[j as usize] >= 0 {
                        ent.push((i as i32, map[j as usize], self.row_val[k]));
                    }
                }
                self.work += self.row_len[i] as f64 * 9.0;
            }
            self.work += self.m as f64 * 3.0 + nf as f64;
            let mut per = vec![0i32; nf];
            for e in &ent {
                per[e.1 as usize] += 1;
            }
            for &c in &per {
                self.work += c as f64;
            }
            self.work += nf as f64 * 4.0;
            self.work += nf as f64 * 3.0;
        }
        let b: Vec<f64> = idx.iter().map(|&j| self.obj[j as usize]).collect();
        // row-major traversal: row lists in row-CSR order, column lists in row order
        let e = Elim::new(2, self.m, nf, &ent, b, nnz, mu, self.work);
        (e, idx)
    }
}

#[cfg(test)]
mod tests {
    use super::Buckets;

    fn order(b: &Buckets, k: usize) -> Vec<i32> {
        let mut v = Vec::new();
        let mut i = b.head[k];
        while i >= 0 {
            v.push(i);
            i = b.next[i as usize];
        }
        v
    }

    #[test]
    fn buckets_keep_the_original_order() {
        // lines 0..5 with counts 2,1,2,0,2: bucket 1 starts in descending index order, line 3 is empty
        let (mut b, empty) = Buckets::new(&[2, 1, 2, 0, 2], 3);
        assert_eq!((empty, order(&b, 0), order(&b, 1)), (1, vec![1], vec![4, 2, 0]));
        b.remove(2, 2);
        b.append(2, 2);
        assert_eq!(order(&b, 1), vec![4, 0, 2]);
        // restart at the line after 4 (4 is not the pivot's line), the tail wraps around
        b.rotate(1, 4, 2);
        assert_eq!(order(&b, 1), vec![0, 2, 4]);
        b.remove(4, 2);
        b.remove(0, 2);
        b.remove(2, 2);
        assert_eq!(b.head[1], -1);
    }
}
