//! `52be10` with `5279b5`, `528f31` and `469c12` (docs/m2/m2_red_clique.md §5): the literal graph of ±1 binary set
//! packing rows. A two-literal row whose pair also sits in another row is removed; a triangle through such a pair
//! grows greedily into a clique that extends the seed row (`<`) or fixes the new literals at 0 (`=`).
use crate::state::{Node, State, NIL};
use crate::Params;

/// Literal graph (`5279b5`'s 12 slots): literal k < n is x_k, n + k its complement; edges point upward.
struct Graph {
    start: Vec<usize>,
    adj: Vec<i32>,
    /// r >= 0: an edge of two-literal row r; -1999999999 from a longer row; -1-r: seed used; -2e9: absorbed
    tag: Vec<i32>,
    mark: Vec<i64>,
    cnt: Vec<i32>,
    /// 0 untouched, -1 dominated (removed if `<`), c >= 1 seed row of clique c - 1
    rowflag: Vec<i64>,
    cstart: Vec<usize>,
    cbuf: Vec<i32>,
}

const LONG: i32 = -1_999_999_999;
const GONE: i32 = -2_000_000_000;

/// `a9dcac`: `aaedd9`'s sort (median-of-3 three-way quicksort to depth 50, Shell sort below 100) on i32 keys,
/// moving `pay` along; equal keys come out in the same order as Gurobi's.
fn sort_i32(keys: &mut [i32], pay: &mut [i32]) {
    quick(0, keys.len(), 0, keys, pay);
}

fn quick(mut lo: usize, mut hi: usize, mut depth: i32, a: &mut [i32], ix: &mut [i32]) {
    while depth < 50 && hi - lo >= 100 {
        let ln = hi - lo;
        let (d2, d3, d4) = (a[lo], a[hi - 1], a[ln / 2 + lo]);
        let mut piv = d2;
        if (d2 < d4 || d3 < d2) && (d4 < d2 || d2 < d3) {
            piv = d4;
            if (d4 < d2 || d3 < d4) && (d2 < d4 || d4 < d3) {
                piv = d3;
            }
        }
        let (mut i, mut j, mut k) = (lo, hi, lo);
        if !(hi <= lo || piv <= d2) {
            loop {
                i += 1;
                k = i;
                if hi <= i || !(a[i] < piv) {
                    break;
                }
            }
        }
        'outer: while i < j {
            let x = a[i];
            if piv <= x {
                if x == piv {
                    i += 1;
                    continue;
                }
                j -= 1;
                let mut y = a[j];
                while piv <= y {
                    if y == piv {
                        if i < j {
                            a[i] = y;
                            a[j] = x;
                            ix.swap(i, j);
                            i += 1;
                        }
                        continue 'outer;
                    }
                    if j <= i {
                        continue 'outer;
                    }
                    j -= 1;
                    y = a[j];
                }
                if i < j {
                    let (xi, xk, xj) = (ix[i], ix[k], ix[j]);
                    a[i] = a[k];
                    ix[i] = xk;
                    i += 1;
                    a[k] = a[j];
                    ix[k] = xj;
                    k += 1;
                    a[j] = x;
                    ix[j] = xi;
                }
            } else {
                a[i] = a[k];
                ix.swap(i, k);
                i += 1;
                a[k] = x;
                k += 1;
            }
        }
        depth += 1;
        if hi - j < k - lo {
            quick(j, hi, depth, a, ix);
            hi = k;
        } else {
            quick(lo, k, depth, a, ix);
            lo = j;
        }
    }
    let mut g = (hi - lo) / 2;
    while g > 0 {
        loop {
            for i in lo + g..hi {
                let (key, id) = (a[i], ix[i]);
                let mut p = i;
                while lo + g <= p && a[p - g] > key {
                    a[p] = a[p - g];
                    ix[p] = ix[p - g];
                    p -= g;
                }
                a[p] = key;
                ix[p] = id;
            }
            if g == 2 && lo + 1 < hi {
                g = 1;
                continue;
            }
            break;
        }
        g = (g as f64 / 2.2) as usize;
    }
}

impl State {
    /// One candidate row of `52be10`: (free binaries k, binaries fixed, nodes walked), the literals of the free ones
    /// pushed to `lits`. A non-binary, a coefficient not ±1, one not filling the row alone, or more than `kmax` free
    /// ones stop the walk (the row then fails `nfix + k == row_cnt`).
    fn pack_scan(&self, r: usize, d: f64, kmax: usize, lits: &mut Vec<i32>) -> (usize, i32, i32) {
        let (mut k, mut nfix, mut ent) = (0usize, 0, 0);
        let mut q = self.row_head[r];
        while q != NIL {
            let node = self.nd[q as usize];
            q = node.nr;
            ent += 1;
            if node.col < 0 {
                continue;
            }
            let j = node.col as usize;
            if self.vtype[j] != b'B' {
                break;
            }
            if self.lb[j] > 0.5 || self.ub[j] < 0.5 {
                nfix += 1;
                continue;
            }
            let a = node.val;
            if k >= kmax || (a.abs() - 1.0).abs() > 1e-10 || a.abs() + d < -1e-10 {
                break;
            }
            lits.push(if a <= 0.0 { (j + self.n) as i32 } else { j as i32 });
            k += 1;
        }
        (k, nfix, ent)
    }

    /// Row `r` qualifies for `52be10`'s scans: two or more entries, min activity − rhs below −ft, finite activities.
    fn pack_row(&self, r: usize, ft: f64) -> bool {
        2 <= self.row_cnt[r] && -ft > self.act_min[r] && self.inf_min[r] == 0 && self.inf_max[r] == 0
    }

    /// `469c12(coef, env, st, j, r)`: a new node from the free list at the head of column j's and row r's lists.
    /// shortcut: an empty free list (`3b85ea` growing the pool) is not ported; the spare nodes never run out here.
    pub(crate) fn add_node(&mut self, j: usize, r: usize, v: f64) {
        assert!(self.free != NIL, "52be10: node pool growth (3b85ea) not ported");
        let at = self.free;
        self.free = self.nd[at as usize].nc;
        self.nfree -= 1;
        self.nnz += 1;
        self.nd[at as usize] = Node { val: v, row: r as i32, col: j as i32, nr: self.row_head[r], nc: self.col_head[j] };
        self.col_head[j] = at;
        self.row_head[r] = at;
        self.col_cnt[j] += 1;
        self.row_cnt[r] += 1;
    }

    /// `52be10(st, env, timer)`. Returns 0 or a status (3 with `bad_col` when an `=` seed row cannot be met).
    /// shortcut: the reduction timer (`adf2be`) is not passed in; its budget is never reached on the 38 models.
    pub fn set_packing(&mut self, p: &Params) -> i32 {
        let (ft, mu, n) = (p.feastol, self.work_mult, self.n);
        let mut w0 = self.work;
        if 2e9 < 1.0 + n as f64 + n as f64 {
            return 0;
        }
        let list = self.rows_list();
        let limit = self.nnz * 2 + 10000;
        let mut hist = [0i64; 201];
        let mut lits = Vec::new();
        for &r in &list {
            let r = r as usize;
            if !self.pack_row(r, ft) {
                continue;
            }
            lits.clear();
            let (k, nfix, ent) = self.pack_scan(r, self.act_min[r], 200, &mut lits);
            self.work += (mu * 3.0) * ent as f64;
            if nfix + k as i32 == self.row_cnt[r] && k >= 2 {
                hist[k] += (k * (k - 1) / 2) as i64;
            }
        }
        self.work += list.len() as f64 * mu;
        if hist[2] < 10 {
            return 0;
        }
        let (mut maxk, mut tot) = (1usize, 0i64);
        loop {
            let t = tot + hist[maxk + 1];
            if limit <= t {
                break;
            }
            maxk += 1;
            tot = t;
            if maxk >= 200 {
                break;
            }
        }
        let (mut edges, mut tags): (Vec<(i32, i32)>, Vec<i32>) = (Vec::new(), Vec::new());
        for &r in &list {
            let r = r as usize;
            if !self.pack_row(r, ft) {
                continue;
            }
            lits.clear();
            let (k, nfix, ent) = self.pack_scan(r, self.act_min[r], maxk, &mut lits);
            self.work += (mu * 3.0) * ent as f64;
            if nfix + k as i32 == self.row_cnt[r] && k >= 2 {
                let tag = if k == 2 { r as i32 } else { LONG };
                for a in 0..k - 1 {
                    for b in a + 1..k {
                        edges.push((lits[a].min(lits[b]), lits[a].max(lits[b])));
                        tags.push(tag);
                    }
                }
            }
        }
        self.work += (tags.len() as f64 * 10.0 + list.len() as f64 * 5.0) * mu;
        if tags.len() <= 2 {
            return 0;
        }
        let mut g = self.pack_graph(&edges, &tags);
        if 2 < 2 * n {
            for k in 0..2 * n - 2 {
                if self.work - w0 > 1e7 {
                    self.budget_check(); // `3ba5ac`, `3ba9de`
                    if self.prelimit < 0 {
                        return 0;
                    }
                    w0 = self.work;
                }
                self.pack_triangles(&mut g, k);
            }
        }
        // dominated two-literal `<` rows
        let old = self.rows_removed;
        let mut ndel = 0;
        for &r in &list {
            let r = r as usize;
            if g.rowflag[r] != -1 || self.sense[r] != b'<' {
                continue;
            }
            ndel += 1;
            let mut c = 0;
            let mut q = self.row_head[r];
            while q != NIL {
                let at = q as usize;
                q = self.nd[at].nr;
                c += 1;
                let j = self.nd[at].col;
                if j >= 0 {
                    self.tracker.mark_col(j);
                    self.col_cnt[j as usize] -= 1;
                    self.nd[at].col = -1;
                }
            }
            self.work += (c as f64 + c as f64) * mu;
            self.row_cnt[r] = -2;
        }
        self.rows_removed = old + ndel;
        if (self.m as i32 - old) / 5 < ndel {
            self.gc(1);
        }
        // seed rows: the clique's literals after the first two (the row's own pair)
        for &r in &list {
            let r = r as usize;
            let id = g.rowflag[r];
            if id <= 0 {
                continue;
            }
            let (a, b) = (g.cstart[id as usize - 1], g.cstart[id as usize]);
            let (mut dpos, mut dneg, mut changed) = (0.0f64, 0.0f64, false);
            let mut pp = a + 2;
            while pp < b {
                let lit = g.cbuf[pp] as usize;
                pp += 1;
                if self.sense[r] == b'=' {
                    let (j, v) = if lit < n { (lit, 0.0) } else { (lit - n, 1.0) };
                    let bad = if lit < n { self.lb[j] > 0.5 } else { 0.5 > self.ub[j] };
                    if bad {
                        self.bad_col = j as i32;
                        return 3;
                    }
                    let skip = if lit < n { 0.5 > self.ub[j] } else { !(self.lb[j] <= 0.5) };
                    if !skip {
                        let rc = self.fix_col(j, v, true);
                        if rc != 0 {
                            return rc;
                        }
                    }
                    continue;
                }
                let (j, coef) = if lit < n {
                    (lit, 1.0)
                } else {
                    dneg -= 1.0;
                    dpos += 1.0;
                    (lit - n, -1.0)
                };
                if 0.5 > self.ub[j] {
                } else if self.lb[j] > 0.5 {
                    dneg -= coef;
                    dpos += coef;
                } else {
                    self.add_node(j, r, coef);
                    if coef > 0.0 {
                        dpos = 1.0 + dpos;
                    }
                    self.tracker.mark_col(j as i32);
                    changed = true;
                }
            }
            self.work += (pp.max(a + 2) - a - 2) as f64 * mu;
            if self.sense[r] == b'<' {
                self.act_max[r] = dpos + self.act_max[r];
                self.rhs[r] = dneg + self.rhs[r];
                if changed || dneg.abs() > 1e-10 {
                    self.tracker.mark_row(r as i32);
                }
            }
        }
        self.work += (list.len() as f64 + list.len() as f64) * mu;
        0
    }

    /// `5279b5`: upward adjacency per literal, sorted, duplicates merged; a two-literal row whose pair repeats is
    /// marked dominated (the surviving tag is the later one).
    fn pack_graph(&mut self, edges: &[(i32, i32)], tags: &[i32]) -> Graph {
        let (mu, n2, ne) = (self.work_mult, 2 * self.n, tags.len());
        let mut start = vec![0usize; n2 + 1];
        for e in edges {
            start[e.0 as usize + 2] += 1;
        }
        self.work += (ne as f64 + ne as f64) * mu;
        if n2 > 2 {
            for t in 3..=n2 {
                start[t] += start[t - 1];
            }
            self.work += mu * 2.0 * (n2 - 2) as f64;
        }
        let (mut adj, mut tag) = (vec![0i32; ne], vec![0i32; ne]);
        for (e, &(lo, hi)) in edges.iter().enumerate() {
            let p = start[lo as usize + 1];
            adj[p] = hi;
            tag[p] = tags[e];
            start[lo as usize + 1] = p + 1;
        }
        self.work += mu * 7.0 * ne as f64;
        for k in 0..n2 {
            let (a, b) = (start[k], start[k + 1]);
            sort_i32(&mut adj[a..b], &mut tag[a..b]);
        }
        self.work += (n2 as f64 + start[n2] as f64 * 2.0) * mu;
        let mut rowflag = vec![0i64; self.m];
        let mut w = 0;
        for k in 0..n2 {
            let (a, b) = (start[k], start[k + 1]);
            start[k] = w;
            for t in a..b {
                if t == a || adj[w - 1] < adj[t] {
                    adj[w] = adj[t];
                    tag[w] = tag[t];
                    w += 1;
                } else {
                    let pt = tag[w - 1];
                    if pt < 0 {
                        if tag[t] >= 0 {
                            rowflag[tag[t] as usize] = -1;
                        }
                    } else {
                        rowflag[pt as usize] = -1;
                        tag[w - 1] = tag[t];
                    }
                }
            }
        }
        self.work += mu * (start[n2] as f64 * 2.0 + n2 as f64);
        start[n2] = w;
        adj.truncate(w);
        tag.truncate(w);
        Graph { start, adj, tag, mark: vec![0; n2], cnt: vec![0; n2], rowflag, cstart: vec![0], cbuf: Vec::new() }
    }

    /// `528f31(μ, env, G, state+0x390, k)`: triangles through an unused two-literal edge (k, i), grown greedily.
    fn pack_triangles(&mut self, g: &mut Graph, k: usize) {
        let mu = self.work_mult;
        let d2 = mu * 2.0;
        let (s0, s1) = (g.start[k], g.start[k + 1]);
        for e in s0..s1 {
            g.mark[g.adj[e] as usize] = e as i64 + 1;
        }
        self.work += ((s1 - s0) as f64 * mu) * 2.0;
        for e in s0..s1 {
            if g.tag[e] < 0 {
                continue;
            }
            let (r, i) = (g.tag[e], g.adj[e] as usize);
            let (si, ei) = (g.start[i], g.start[i + 1]);
            let mut f = si;
            let mut p = -1i64;
            while f < ei {
                p = g.mark[g.adj[f] as usize] - 1;
                if p >= 0 && (g.tag[p as usize] >= 0 || g.tag[f] >= 0) {
                    break;
                }
                f += 1;
            }
            self.work += (f - si) as f64 * d2;
            if f == ei {
                continue;
            }
            let p = p as usize;
            let l = g.adj[f];
            g.rowflag[r as usize] = (g.cstart.len() - 1) as i64 + 1;
            g.tag[e] = -1 - r;
            if g.tag[p] >= 0 {
                g.rowflag[g.tag[p] as usize] = -1;
                g.tag[p] = GONE;
            }
            if g.tag[f] >= 0 {
                g.rowflag[g.tag[f] as usize] = -1;
                g.tag[f] = GONE;
            }
            let mut buf = vec![k as i32, i as i32, l];
            let mut cand = false;
            for f2 in f + 1..ei {
                let u = g.adj[f2] as usize;
                let p2 = g.mark[u] - 1;
                if p2 >= 0 {
                    g.mark[u] = -2;
                    g.cnt[u] = if g.tag[p2 as usize] < 0 { 0 } else { 1 };
                    cand = true;
                }
            }
            self.work += ((ei - f) as f64 * mu) * 3.0;
            if cand {
                loop {
                    let (size, v) = (buf.len() as i64, *buf.last().unwrap() as usize);
                    let (mut best, mut bc) = (-1i32, 0);
                    for gg in g.start[v]..g.start[v + 1] {
                        let u = g.adj[gg] as usize;
                        if g.mark[u] == 1 - size {
                            g.mark[u] = -size;
                            if g.tag[gg] >= 0 {
                                g.cnt[u] += 1;
                            }
                            if bc < g.cnt[u] {
                                best = u as i32;
                                bc = g.cnt[u];
                            }
                        }
                    }
                    self.work += (g.start[v + 1] - g.start[v]) as f64 * d2;
                    if best < 0 {
                        break;
                    }
                    buf.push(best);
                }
                let size = buf.len();
                if size > 3 {
                    for &q in &buf[3..] {
                        g.mark[q as usize] = GONE as i64;
                    }
                    self.work += (size - 3) as f64 * d2;
                    for &q in &buf {
                        let q = q as usize;
                        for gg in g.start[q]..g.start[q + 1] {
                            if g.mark[g.adj[gg] as usize] == GONE as i64 && g.tag[gg] >= 0 {
                                g.rowflag[g.tag[gg] as usize] = -1;
                                g.tag[gg] = GONE;
                            }
                        }
                        self.work += (g.start[q + 1] - g.start[q]) as f64 * d2;
                    }
                    self.work += (size as f64 + size as f64) * mu;
                }
            }
            g.cbuf.extend_from_slice(&buf);
            g.cstart.push(g.cbuf.len());
            if cand {
                for e2 in e + 1..s1 {
                    g.mark[g.adj[e2] as usize] = e2 as i64 + 1;
                }
                self.work += ((-1 - e as i64) + s1 as i64) as f64 * d2;
            }
        }
        self.work += mu * 4.0 * (s1 - s0) as f64;
        for e in s0..s1 {
            g.mark[g.adj[e] as usize] = 0;
        }
        self.work += mu * 2.0 * (s1 - s0) as f64;
    }
}
