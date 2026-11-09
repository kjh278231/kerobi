//! `36ee92` node arena compression (docs/m2/m2_red_outer_b.md §4): `510953` copies the live matrix to CSR, `514031`
//! lays the linked lists out again from it.
use crate::state::{Node, State, NIL};
use crate::Params;

impl State {
    /// `36ee92(env, st)`: when the free list has grown past the live nodes (nnz < free, free > 1e5 and twice the
    /// last chunk size) or the arena is mostly unused, rebuild it: row lists come out in column order, column lists
    /// keep their order, only nodes of live rows and columns stay, the rest of the arena becomes the free list
    /// (in position order) and the pending list is emptied. The chunk size (+0x1a8) becomes the old free count.
    /// Work without the multiplier. nnz (+0xd8) is left as it is. Returns 0.
    pub fn compress(&mut self, _p: &Params) -> i32 {
        let (nnz, nfree) = (self.nnz, self.nfree);
        if nnz == 0 || !(self.node_cap * 2 < nnz + nfree || (nnz < nfree && 100000 < nfree && self.chunk * 2 < nfree)) {
            return 0;
        }
        self.chunk = nfree;
        let (n, m) = (self.n, self.m);
        let live = |s: &State, mut q: u32, row: bool| {
            let mut v = Vec::new();
            while q != NIL {
                let nd = s.nd[q as usize];
                if nd.col >= 0 {
                    v.push((nd.row, nd.col, nd.val));
                }
                q = if row { nd.nr } else { nd.nc };
            }
            v
        };
        // 510953: row copies (only their sizes reach the state, through the work), then column copies
        self.work = (n as f64 + n as f64) + self.work;
        for i in 0..m {
            let k = live(self, self.row_head[i], true).len();
            self.work = k as f64 * 3.0 + self.work;
        }
        self.work = m as f64 * 4.0 + self.work;
        let cols: Vec<Vec<(i32, i32, f64)>> = (0..n).map(|j| live(self, self.col_head[j], false)).collect();
        for c in &cols {
            self.work = c.len() as f64 * 3.0 + self.work;
        }
        self.work = n as f64 * 4.0 + self.work;
        // 514031: row slots from the live counts
        let mut start = vec![0usize; m + 1];
        for i in 0..m {
            start[i + 1] = start[i] + self.row_cnt[i].max(0) as usize;
        }
        let total = start[m];
        self.work = m as f64 * 3.0 + self.work;
        // shortcut: the chunk chain +0x1b8 (nodes `3b85ea`/`3bea24` added past the constructor's arena) is taken
        // as "the arena grew"; then the arena is reallocated to live + free nodes (dumped states never grew).
        let cap = if self.nd.len() > self.node_cap as usize || (self.node_cap as usize) < total + nfree as usize {
            self.node_cap = (total + nfree as usize) as i64;
            total + nfree as usize
        } else {
            self.node_cap as usize
        };
        let mut nd = vec![Node { val: 0.0, row: 0, col: -1, nr: NIL, nc: NIL }; cap];
        let mut next = start.clone();
        for (j, c) in cols.iter().enumerate() {
            let mut tail = NIL;
            if self.col_cnt[j] >= 0 {
                for &(r, col, v) in c {
                    if r < 0 || self.row_cnt[r as usize] < 0 {
                        continue;
                    }
                    let at = next[r as usize];
                    next[r as usize] += 1;
                    nd[at] = Node { val: v, row: r, col, nr: NIL, nc: NIL };
                    if tail == NIL { self.col_head[j] = at as u32 } else { nd[tail as usize].nc = at as u32 }
                    tail = at as u32;
                }
                self.work = c.len() as f64 * 7.0 + self.work;
            }
            if tail == NIL {
                self.col_head[j] = NIL;
            }
        }
        self.work = n as f64 * 4.0 + self.work;
        for i in 0..m {
            let (b, e) = (start[i], start[i + 1]);
            if b == e {
                self.row_head[i] = NIL;
                continue;
            }
            self.row_head[i] = b as u32;
            for at in b..e - 1 {
                nd[at].nr = (at + 1) as u32;
            }
            self.work = (e - 1 - b) as f64 * 4.0 + self.work;
        }
        self.work = m as f64 * 6.0 + self.work;
        self.nfree = (cap - total) as i64;
        self.free = if total < cap { total as u32 } else { NIL };
        for at in total..cap {
            nd[at].nc = if at + 1 < cap { (at + 1) as u32 } else { NIL };
        }
        self.nd = nd;
        self.pend = NIL;
        self.npend = 0;
        0
    }
}
