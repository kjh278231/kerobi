//! 05 §2.3 Ordering and symbolic analysis (`300f28`, `309dd0`, `30b0f2`, `30d189`, `3047bb`, `2e4f14`),
//! docs/m3/m3_ordering.md. AAᵀ pattern (`303e2a`) → stage labels with one bisection of every component > 600 rows
//! (`309dd0` mode 3) → approximate minimum degree (`3047bb`, Seed = 0) → column structures, supernodes, relaxed
//! amalgamation, postorder and block split (`2e4f14`). Matches Gurobi's permutation, supernodes and Factor NZ/Ops
//! on all 64 bench LPs.
//! shortcut: the ND trials of `300f28` (mode 1, seeds Seed+11+k, run only when ordering work/ops < 0.1, which on the
//! bench LPs happens only for instances without bisection where they equal mode 3), Seed ≠ 0 and dense columns
//! (`2fd14c`, m ≥ 1000) are not ported; port them when an instance needs them.
use rsolver_model::Model;

/// Result of the structure analysis (field doc = Gurobi location).
#[derive(Debug, Clone, PartialEq)]
pub struct Symbolic {
    /// B+0x100: nonzeros of AAᵀ without the diagonal, both triangles. The log prints `AA' NZ` = aat_nnz / 2.
    pub aat_nnz: i64,
    /// F+0x30 after `2e4f14`: final position of each row (the row permutation applied to A).
    pub perm: Vec<i32>,
    /// F+0x38: `Factor NZ`, entries of the supernodal L including the diagonal and amalgamation padding.
    pub nnz_l: i64,
    /// F+0x60: `Factor Ops`, Σ over columns of (column length)².
    pub ops: f64,
    /// F+0x68: the part of `ops` in supernodes narrower than 5 columns (`2e4f14` L2058-2084; feeds the work estimate).
    pub ops_small: f64,
    /// F+0x48: Σ over columns of A of (column length)², the cost of forming A·D·Aᵀ.
    pub ops_aat: f64,
    /// F+0x70: supernode widths by new column: width at the first column, −k at the k-th column after it.
    pub snode: Vec<i32>,
    /// F+0x78: the same after cutting supernodes into blocks of ≤ 192 columns (240 for tall, 24 for cheap factors).
    pub blocks: Vec<i32>,
    /// F+0xa8: length of each column of L, diagonal included.
    pub col_len: Vec<i32>,
    /// F+0xc8: per supernode, its rows ascending (its own columns first).
    pub row_idx: Vec<i32>,
    /// F+0xb0 as offsets: start of each column's rows in `row_idx` (supernode start + offset in the supernode).
    pub col_ptr: Vec<usize>,
}

/// Structure analysis of A·D·Aᵀ for the barrier (rows of `m` are the unknowns).
pub fn analyze(m: &Model) -> Symbolic {
    let nr = m.num_constrs();
    let (cp, ri) = aat(m);
    // the bisection draws from the model's RNG (model+0xb0): 0 for a model as read, set by presolve otherwise
    let pos = amd(nr, &cp, &ri, nd_labels(nr, &cp, &ri, &mut { m.rng }));
    let st = col_structs(nr, &cp, &ri, &pos);
    let (sns, rows) = supernodes(nr, &st);
    let mut newpos = vec![0usize; nr];
    for (k, &c) in sns.iter().flatten().enumerate() {
        newpos[c] = k;
    }
    let (mut nz, mut ops, mut ops_small, mut snode, mut col_len, mut row_idx, mut col_ptr) =
        (0.0f64, 0.0f64, 0.0f64, Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for (cols, &r) in sns.iter().zip(&rows) {
        let w = cols.len();
        let start = row_idx.len();
        let mut ix: Vec<i32> = cols.iter().map(|&c| newpos[c] as i32).collect();
        let last = *cols.iter().max().unwrap();
        ix.extend(st[last][1..].iter().map(|&i| newpos[i] as i32));
        ix.sort_unstable(); // distinct values: any sort gives the same order
        row_idx.extend(ix);
        for i in 0..w {
            snode.push(if i == 0 { w as i32 } else { -(i as i32) });
            col_len.push((r - i) as i32);
            col_ptr.push(start + i);
        }
        // 2e4f14 L2068-2087: trapezoid count and Σ k² over the supernode, both summed in double
        let (rf, wf) = (r as f64, w as f64);
        nz += (rf - (wf - 1.0) * 0.5) * wf;
        let (a, b) = (r as f64, (r - w) as f64);
        let o = a * (a + 1.0) * (2.0 * a + 1.0) / 6.0 - b * (b + 1.0) * (2.0 * b + 1.0) / 6.0;
        ops += o;
        if w < 5 {
            ops_small += o;
        }
    }
    let ops_aat: f64 = m.col_len.iter().map(|&l| (l as f64) * (l as f64)).sum();
    let blocks = split_blocks(&snode, &col_len, ops < 2.0 * ops_aat);
    let perm = pos.iter().map(|&p| newpos[p] as i32).collect();
    Symbolic { aat_nnz: cp[nr] as i64, perm, nnz_l: nz as i64, ops, ops_small, ops_aat, snode, blocks, col_len, row_idx, col_ptr }
}

/// `303e2a`: pattern of A·Aᵀ without the diagonal, both triangles, rows ascending in each column.
fn aat(m: &Model) -> (Vec<usize>, Vec<usize>) {
    let nr = m.num_constrs();
    let mut by_row: Vec<Vec<usize>> = vec![Vec::new(); nr];
    for j in 0..m.num_vars() {
        let b = m.col_beg[j] as usize;
        for &i in &m.col_ind[b..b + m.col_len[j] as usize] {
            by_row[i as usize].push(j);
        }
    }
    let (mut cp, mut ri, mut mark) = (vec![0usize], Vec::new(), vec![usize::MAX; nr]);
    for i in 0..nr {
        let s = ri.len();
        mark[i] = i;
        for &j in &by_row[i] {
            let b = m.col_beg[j] as usize;
            for &k in &m.col_ind[b..b + m.col_len[j] as usize] {
                if mark[k as usize] != i {
                    mark[k as usize] = i;
                    ri.push(k as usize);
                }
            }
        }
        ri[s..].sort_unstable();
        cp.push(ri.len());
    }
    (cp, ri)
}

/// `309dd0`/`3047bb` dense-node threshold: max(100, min((int)(50·nnz/n), 10·√n)).
fn dense_thresh(n: usize, nnz: usize) -> usize {
    let mut d = ((nnz as f64 * 50.0 / n as f64) as i32) as f64;
    let s = (n as f64).sqrt() * 10.0;
    if s <= d {
        d = s;
    }
    (d as i32).max(100) as usize
}

/// `309dd0` mode 3: rows of degree > threshold go to stage 2 (label −2); every remaining connected component larger
/// than max(600, n/100) is bisected once (`30b0f2`, RNG state `seed` = model+0xb0) and its separator joins stage 2.
fn nd_labels(n: usize, cp: &[usize], ri: &[usize], seed: &mut u64) -> Vec<i32> {
    let t = dense_thresh(n, cp[n]);
    let mut lab: Vec<i32> = (0..n).map(|i| if cp[i + 1] - cp[i] > t { -2 } else { -1 }).collect();
    let mut comp: Vec<usize> = lab.iter().map(|&l| if l == -1 { usize::MAX } else { usize::MAX - 1 }).collect();
    let mut nc = 0;
    for s in 0..n {
        if comp[s] != usize::MAX {
            continue;
        }
        comp[s] = nc;
        let mut stack = vec![s];
        while let Some(v) = stack.pop() {
            for &u in &ri[cp[v]..cp[v + 1]] {
                if comp[u] == usize::MAX {
                    comp[u] = nc;
                    stack.push(u);
                }
            }
        }
        nc += 1;
    }
    let mut size = vec![0usize; nc];
    for &c in comp.iter().filter(|&&c| c < nc) {
        size[c] += 1;
    }
    let leaf = (n / 100).max(600);
    let mut loc = vec![usize::MAX; n];
    for c in (0..nc).filter(|&c| size[c] > leaf) {
        let nodes: Vec<usize> = (0..n).filter(|&i| comp[i] == c).collect();
        for (k, &i) in nodes.iter().enumerate() {
            loc[i] = k;
        }
        let (mut xa, mut ad) = (vec![0usize], Vec::new());
        for &i in &nodes {
            ad.extend(ri[cp[i]..cp[i + 1]].iter().filter(|&&j| comp[j] == c).map(|&j| loc[j]));
            xa.push(ad.len());
        }
        let part = bisect(&xa, &ad, -1, seed);
        for (k, &i) in nodes.iter().enumerate() {
            if part[k] == 2 {
                lab[i] = -2;
            }
        }
    }
    lab
}

/// `a7b4d6`: 64-bit LCG step with a tempered 31-bit output.
fn rand31(seed: &mut u64) -> i32 {
    *seed = seed.wrapping_mul(0x5851f42d4c957f2d).wrapping_add(0x14057b7ef767814f);
    let mut u = (*seed >> 32) as u32;
    u ^= u >> 11;
    u ^= (u & 0x13a58ad) << 7;
    u ^= (u & 0x1df8c) << 15;
    ((u ^ (u >> 18)) & 0x7fffffff) as i32
}

/// `a786fc`…`a7936f`: binary max-heap on int keys with a node locator; equal keys never move.
struct Heap {
    node: Vec<usize>,
    key: Vec<i32>,
    loc: Vec<i64>,
}

impl Heap {
    fn new(n: usize) -> Self {
        Heap { node: Vec::new(), key: Vec::new(), loc: vec![-1; n] }
    }
    fn reset(&mut self) {
        for &v in &self.node {
            self.loc[v] = -1;
        }
        self.node.clear();
        self.key.clear();
    }
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
    /// Sift after the key at `i` changed: down at the root or when not above the parent, else up.
    fn fix(&mut self, i: usize) {
        if i > 0 && self.key[(i - 1) / 2] < self.key[i] {
            self.up(i);
        } else {
            self.down(i);
        }
    }
    fn insert(&mut self, v: usize, k: i32) {
        self.node.push(v);
        self.key.push(k);
        self.loc[v] = self.node.len() as i64 - 1;
        self.up(self.node.len() - 1);
    }
    fn peek(&self) -> Option<usize> {
        self.node.first().copied()
    }
    fn contains(&self, v: usize) -> bool {
        self.loc[v] >= 0
    }
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
    fn update(&mut self, v: usize, k: i32) {
        let i = self.loc[v] as usize;
        self.key[i] = k;
        self.fix(i);
    }
}

/// `30d189` separator cost: |S| / (|P0|·|P1|), or 100·total when a side is empty.
fn sep_cost(p: [i32; 3]) -> f64 {
    if p[0] == 0 || p[1] == 0 {
        (p[0] + p[1] + p[2]) as f64 * 100.0
    } else {
        p[2] as f64 / (p[0] as f64 * p[1] as f64)
    }
}

/// `30d189`: two-sided FM refinement of a vertex separator (part 2) with balance factor `ratio`; up to 10 passes,
/// each rolled back to its best cut. Queue k holds separator nodes for a move to side k, key vwgt − edeg[1−k].
fn fm_refine(ratio: f64, seed: &mut u64, xa: &[usize], ad: &[usize], vw: &[i32], wh: &mut [usize]) {
    let n = vw.len();
    let mut pw = [0i32; 3];
    let mut edeg = [vec![0i32; n], vec![0i32; n]];
    let mut nsep = 0;
    for v in 0..n {
        pw[wh[v]] += vw[v];
        if wh[v] == 2 {
            nsep += 1;
            for &u in &ad[xa[v]..xa[v + 1]] {
                if wh[u] != 2 {
                    edeg[wh[u]][v] += vw[u];
                }
            }
        }
    }
    let limit = (5 * nsep).min(500) as i64;
    let mut q = [Heap::new(n), Heap::new(n)];
    let mut moved = vec![0i64; n];
    for _pass in 0..10 {
        q[0].reset();
        q[1].reset();
        let mut perm: Vec<usize> = (0..n).filter(|&v| wh[v] == 2).collect();
        for k in (0..perm.len()).rev() {
            let r = rand31(seed) as usize % (k + 1);
            perm.swap(r, k);
        }
        for &v in &perm {
            q[0].insert(v, vw[v] - edeg[1][v]);
            q[1].insert(v, vw[v] - edeg[0][v]);
        }
        moved.fill(-1);
        let mut mincut = sep_cost(pw);
        let mut mincutorder: i64 = -1;
        let (mut swaps, mut hist, mut mlist) = (Vec::new(), Vec::new(), Vec::new());
        while swaps.len() < n {
            if limit < swaps.len() as i64 - mincutorder {
                break;
            }
            let t = [q[0].peek(), q[1].peek()];
            let to = match t {
                [None, _] => 1,
                [Some(t0), Some(t1)] if pw[1] as f64 <= ratio * pw[0] as f64 => {
                    if (pw[1] as f64 * ratio) < pw[0] as f64 {
                        1
                    } else {
                        let (w0, e1) = (vw[t0], edeg[1][t0]);
                        let (a, b) = (pw[0] + w0, pw[1] - e1);
                        let c0 = if a == 0 || b == 0 {
                            (pw[0] + pw[1] + pw[2]) as f64 * 100.0
                        } else {
                            (pw[2] - w0 + e1) as f64 / (a as f64 * b as f64)
                        };
                        let (e0, w1) = (edeg[0][t1], vw[t1]);
                        let (a, b) = (pw[0] - e0, pw[1] + w1);
                        let c1 = if a == 0 || b == 0 {
                            (pw[0] + pw[1] + pw[2]) as f64 * 100.0
                        } else {
                            (pw[2] - w1 + e0) as f64 / (b as f64 * a as f64)
                        };
                        usize::from(c1 <= c0)
                    }
                }
                _ => 0,
            };
            let Some(v) = t[to] else { break };
            let other = 1 - to;
            q[to].delete(v);
            q[other].delete(v);
            wh[v] = to;
            moved[v] = swaps.len() as i64;
            swaps.push(v);
            hist.push(mlist.len());
            pw[2] -= vw[v];
            pw[to] += vw[v];
            for &u in &ad[xa[v]..xa[v + 1]] {
                if wh[u] == 2 {
                    edeg[to][u] += vw[v];
                    // asm 0x3281eb tests contains(q[other], v), not u: v was just deleted, so this never fires
                    if q[other].contains(v) {
                        q[other].update(u, vw[u] - edeg[to][u]);
                    }
                } else if wh[u] == other {
                    wh[u] = 2;
                    pw[other] -= vw[u];
                    mlist.push(u);
                    edeg[0][u] = 0;
                    edeg[1][u] = 0;
                    pw[2] += vw[u];
                    for &x in &ad[xa[u]..xa[u + 1]] {
                        if wh[x] == 2 {
                            edeg[other][x] -= vw[u];
                            if q[to].contains(x) {
                                q[to].update(x, vw[x] - edeg[other][x]);
                            }
                        } else {
                            edeg[wh[x]][u] += vw[x];
                        }
                    }
                    if moved[u] == -1 {
                        q[to].insert(u, vw[u] - edeg[other][u]);
                        moved[u] = -2;
                    }
                }
            }
            let cost = sep_cost(pw);
            if cost < mincut {
                mincutorder = swaps.len() as i64 - 1;
                mincut = cost;
            }
        }
        hist.push(mlist.len());
        for i in ((mincutorder + 1) as usize..swaps.len()).rev() {
            let v = swaps[i];
            let to = wh[v];
            let other = 1 - to;
            wh[v] = 2;
            pw[to] -= vw[v];
            edeg[0][v] = 0;
            edeg[1][v] = 0;
            pw[2] += vw[v];
            for &u in &ad[xa[v]..xa[v + 1]] {
                if wh[u] == 2 {
                    edeg[to][u] -= vw[v];
                } else {
                    edeg[wh[u]][v] += vw[u];
                }
            }
            for &u in &mlist[hist[i]..hist[i + 1]] {
                pw[2] -= vw[u];
                pw[other] += vw[u];
                wh[u] = other;
                for &x in &ad[xa[u]..xa[u + 1]] {
                    if wh[x] == 2 {
                        edeg[other][x] += vw[u];
                    }
                }
            }
        }
        if mincutorder == -1 {
            break;
        }
    }
}

/// `30b0f2`: multilevel vertex bisection; returns 0/1 per side and 2 for the separator. Coarsening (≤ 19 levels,
/// while > 99 nodes and shrinking) visits nodes by degree with equal-degree runs shuffled and matches each to its
/// lowest-degree unmatched neighbour (first unmatched when `round` = 3). The coarsest graph starts all-separator.
fn bisect(xa: &[usize], ad: &[usize], round: i32, seed: &mut u64) -> Vec<usize> {
    let mut levels = vec![(xa.to_vec(), ad.to_vec(), vec![1i32; xa.len() - 1])];
    let mut maps: Vec<(Vec<usize>, Vec<usize>)> = Vec::new();
    loop {
        let (xl, al, vl) = levels.last().unwrap();
        let nl = xl.len() - 1;
        let deg: Vec<usize> = (0..nl).map(|i| xl[i + 1] - xl[i]).collect();
        let mut order: Vec<usize> = (0..nl).collect();
        order.sort_by_key(|&i| deg[i]); // stable, like the counting sort
        let mut i = 0;
        while i < nl {
            let mut j = i + 1;
            while j < nl && deg[order[j]] == deg[order[i]] {
                j += 1;
            }
            for k in (i + 1..j).rev() {
                let r = rand31(seed) as usize % (k - i + 1);
                order.swap(k, i + r);
            }
            i = j;
        }
        let mut mat = vec![usize::MAX; nl];
        let (mut cptr, mut clist) = (Vec::new(), Vec::new());
        for &v in &order {
            if mat[v] != usize::MAX {
                continue;
            }
            let (mut best, mut bd) = (usize::MAX, nl + 1);
            for &u in &al[xl[v]..xl[v + 1]] {
                if mat[u] == usize::MAX && (round == 3 || deg[u] < bd) {
                    (best, bd) = (u, deg[u]);
                    if round == 3 {
                        break;
                    }
                }
            }
            mat[v] = cptr.len();
            cptr.push(clist.len());
            clist.push(v);
            if best != usize::MAX {
                mat[best] = cptr.len() - 1;
                clist.push(best);
            }
        }
        let nc = cptr.len();
        cptr.push(clist.len());
        let mut mark = vec![usize::MAX; nc];
        let (mut cx, mut ca, mut cw) = (vec![0usize], Vec::new(), Vec::with_capacity(nc));
        for c in 0..nc {
            mark[c] = c;
            let mut w = 0;
            for &f in &clist[cptr[c]..cptr[c + 1]] {
                w += vl[f];
                for &x in &al[xl[f]..xl[f + 1]] {
                    if mark[mat[x]] != c {
                        mark[mat[x]] = c;
                        ca.push(mat[x]);
                    }
                }
            }
            cw.push(w);
            cx.push(ca.len());
        }
        levels.push((cx, ca, cw));
        maps.push((cptr, clist));
        if !(nc > 99 && nc != nl && levels.len() - 1 < 19) {
            break;
        }
    }
    let mut lev = levels.len() - 1;
    let mut wh = vec![2usize; levels[lev].2.len()];
    let ratio = rand31(seed) as f64 * 4.656612873077393e-10 * 5.0 + 5.0; // a7b54f
    loop {
        let (xl, al, vl) = &levels[lev];
        fm_refine(ratio, seed, xl, al, vl, &mut wh);
        if lev == 0 {
            return wh;
        }
        let (cptr, clist) = &maps[lev - 1];
        let mut fine = vec![0usize; levels[lev - 1].2.len()];
        for c in 0..vl.len() {
            for &f in &clist[cptr[c]..cptr[c + 1]] {
                fine[f] = wh[c];
            }
        }
        wh = fine;
        lev -= 1;
    }
}

/// `3047bb` with Seed = 0: quotient-graph approximate minimum degree in stages. `label[i]` = −stage on entry;
/// returns the elimination position of each node. Lists hold elements first (`elen`), then variables.
fn amd(n: usize, cp: &[usize], ri: &[usize], mut label: Vec<i32>) -> Vec<usize> {
    let ni = n as i32;
    let mut adj: Vec<Vec<usize>> = (0..n).map(|i| ri[cp[i]..cp[i + 1]].to_vec()).collect();
    let mut ln: Vec<i32> = adj.iter().map(|a| a.len() as i32).collect();
    let mut elen = vec![0i32; n];
    let mut deg = ln.clone(); // degree before the dense rows are cut (sic)
    let mut nv = vec![1i32; n];
    let mut parent: Vec<usize> = (0..n).collect();
    let mut mark = vec![usize::MAX; n];
    let mut hash = vec![0.0f64; n];
    let mut w = vec![0i32; n];
    let maxstage = label.iter().map(|&l| -l).max().unwrap_or(0).max(0);
    let t = dense_thresh(n, cp[n]) as i32;
    let mut ndense = 0;
    for i in 0..n {
        if ln[i] > t {
            ndense += 1;
            label[i] = ni - label[i];
            ln[i] = -1;
        }
    }
    if ndense > 0 {
        for i in 0..n {
            let keep = ln[i].max(0) as usize;
            let a: Vec<usize> = adj[i][..keep].iter().copied().filter(|&j| label[j] < 0).collect();
            ln[i] = a.len() as i32;
            adj[i] = a;
        }
    }
    let mut pos = 0i32;
    for i in 0..n {
        if label[i] == -1 && ln[i] == 0 {
            label[i] = pos;
            pos += 1;
            ln[i] = -1;
        }
    }
    let (mut head, mut next, mut prev) = (vec![-1i64; n], vec![-1i64; n], vec![-1i64; n]);
    while pos < ni {
        let mut stage = maxstage;
        for i in 0..n {
            if nv[i] > 0 {
                if label[i] < 0 && -label[i] < stage {
                    stage = -label[i];
                } else if label[i] > ni && label[i] - ni < stage {
                    stage = label[i] - ni;
                }
            }
        }
        head.fill(-1);
        let mut mindeg = n;
        macro_rules! insert {
            ($i:expr) => {{
                let i = $i;
                let d = deg[i] as usize;
                let h = head[d];
                if h != -1 {
                    prev[h as usize] = i as i64;
                }
                next[i] = h;
                prev[i] = -1;
                head[d] = i as i64;
                mindeg = mindeg.min(d);
            }};
        }
        macro_rules! remove {
            ($i:expr) => {{
                let i = $i;
                let (a, b) = (prev[i], next[i]);
                if a == -1 {
                    head[deg[i] as usize] = b;
                } else {
                    next[a as usize] = b;
                }
                if b != -1 {
                    prev[b as usize] = a;
                }
            }};
        }
        for i in (0..n).rev() {
            if label[i] == -stage && nv[i] > 0 {
                insert!(i);
            }
        }
        while mindeg < n {
            let Some(d) = (mindeg..n).find(|&d| head[d] != -1) else { break };
            let me = head[d] as usize;
            mindeg = d;
            remove!(me);
            label[me] = pos;
            mark[me] = me;
            // Lme: variables of the absorbed elements, then the pivot's own variables
            let (mut lme, mut degme) = (Vec::new(), 0i32);
            let ems: Vec<usize> = adj[me][..elen[me] as usize].to_vec();
            for e in ems {
                for &i in &adj[e][..ln[e].max(0) as usize] {
                    if mark[i] != me && nv[i] > 0 && label[i] < 0 {
                        lme.push(i);
                        mark[i] = me;
                        degme += nv[i];
                    }
                }
                elen[e] = -1;
                ln[e] = -1;
            }
            for &i in &adj[me][elen[me] as usize..ln[me] as usize] {
                if mark[i] != me && nv[i] > 0 && label[i] < 0 {
                    lme.push(i);
                    mark[i] = me;
                    degme += nv[i];
                }
            }
            // |Le \ Lme| for every element next to Lme; w = 0 absorbs the element
            for &i in &lme {
                for &e in &adj[i][..elen[i] as usize] {
                    w[e] = nv[e];
                }
            }
            for &i in &lme {
                for &e in &adj[i][..elen[i] as usize] {
                    w[e] -= nv[i];
                    if w[e] == 0 {
                        ln[e] = 0;
                    }
                }
            }
            for &i in &lme {
                if label[i] == -stage {
                    remove!(i);
                }
            }
            // prune lists, approximate degree, hash; very dense variables wait for the end of the stage
            for &i in &lme {
                let mut h = (label[i] * ni) as f64;
                let mut dsum = 0i32;
                let mut keep = Vec::with_capacity(ln[i].max(0) as usize + 1);
                for &e in &adj[i][..elen[i] as usize] {
                    if ln[e] > 0 {
                        dsum += w[e];
                        h += e as f64;
                        keep.push(e);
                    }
                }
                keep.push(me);
                let ne = keep.len() as i32;
                for &j in &adj[i][elen[i] as usize..ln[i] as usize] {
                    if mark[j] != me && nv[j] > 0 {
                        dsum += nv[j];
                        keep.push(j);
                        h += j as f64;
                    }
                }
                elen[i] = ne;
                ln[i] = keep.len() as i32;
                adj[i] = keep;
                deg[i] = (degme + deg[i] - nv[i]).min(dsum);
                hash[i] = h;
                if 5 * ni - 5 * pos < dsum + nv[i] {
                    label[i] = ni - label[i];
                    ln[i] = -1;
                }
            }
            // mass elimination: variables left with only the new element join the pivot
            let mut l2 = Vec::with_capacity(lme.len());
            for &i in &lme {
                if label[i] == -stage && ln[i] == 1 {
                    nv[me] += nv[i];
                    degme -= nv[i];
                    nv[i] = 0;
                    parent[i] = me;
                    ln[i] = -1;
                } else if label[i] < ni {
                    l2.push(i);
                }
            }
            pos += nv[me];
            nv[me] = degme;
            for &i in &l2 {
                deg[i] = (degme - nv[i] + deg[i]).min(ni - 1);
            }
            // supervariables: equal-hash runs of the hash-sorted list with identical lists
            let mut key: Vec<f64> = l2.iter().map(|&i| hash[i]).collect();
            shell_sort(&mut l2, &mut key);
            let mut lm: Vec<i64> = l2.iter().map(|&i| i as i64).collect();
            let mut k = 0;
            while k < lm.len() {
                let mut e2 = k + 1;
                while e2 < lm.len() && key[e2] == key[k] {
                    e2 += 1;
                }
                for a in k..e2 {
                    for b in a + 1..e2 {
                        if lm[a] < 0 || lm[b] < 0 {
                            continue;
                        }
                        let (i, j) = (lm[a] as usize, lm[b] as usize);
                        if label[i] == label[j]
                            && ln[i] == ln[j]
                            && elen[i] == elen[j]
                            && adj[i][..ln[i] as usize] == adj[j][..ln[j] as usize]
                        {
                            nv[i] += nv[j];
                            deg[i] -= nv[j];
                            nv[j] = 0;
                            parent[j] = i;
                            ln[j] = -1;
                            lm[b] = -1;
                        }
                    }
                }
                k = e2;
            }
            let le: Vec<usize> = lm.iter().filter(|&&x| x >= 0).map(|&x| x as usize).collect();
            elen[me] = 0;
            ln[me] = le.len() as i32;
            for &i in &le {
                if label[i] == -stage {
                    insert!(i);
                }
            }
            adj[me] = le;
        }
        for i in 0..n {
            if label[i] == ni + stage {
                label[i] = pos;
                ln[i] = -1;
                pos += nv[i];
                nv[i] = 0;
            }
        }
    }
    // absorbed variables take the positions before their principal, in index order
    for i in 0..n {
        if label[i] < 0 {
            let mut r = parent[i];
            while parent[r] != r {
                r = parent[r];
            }
            label[i] = label[r];
            label[r] += 1;
        }
    }
    label.iter().map(|&p| p as usize).collect()
}

/// `ab9789` for n < 100: shell sort of `a` by `key` (gaps n/2, then gap/2.2 with 2 → 1); a key moves only past
/// strictly greater ones (keys are hashes, never NaN).
/// shortcut: `ab9789` runs a median-of-3 quicksort first when n ≥ 100 (never on the bench LPs); shell sort is used
/// for every n, so ties may come out in another order there. Port `ab9789_quick` when an Lme reaches 100.
fn shell_sort(a: &mut [usize], key: &mut [f64]) {
    let n = a.len();
    let mut gap = n / 2;
    while gap > 0 {
        loop {
            for i in gap..n {
                let (v, x) = (key[i], a[i]);
                let mut j = i;
                while j >= gap && key[j - gap] > v {
                    key[j] = key[j - gap];
                    a[j] = a[j - gap];
                    j -= gap;
                }
                key[j] = v;
                a[j] = x;
            }
            if gap == 2 && n > 1 {
                gap = 1;
                continue;
            }
            break;
        }
        gap = (gap as f64 / 2.2) as usize;
    }
}

/// Row structure of each column of L (diagonal first, ascending) for elimination positions `pos`.
fn col_structs(n: usize, cp: &[usize], ri: &[usize], pos: &[usize]) -> Vec<Vec<usize>> {
    let mut perm = vec![0usize; n];
    for (i, &p) in pos.iter().enumerate() {
        perm[p] = i;
    }
    let mut kids: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut st: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut mark = vec![usize::MAX; n];
    for k in 0..n {
        let mut s = vec![k];
        mark[k] = k;
        for &j0 in &ri[cp[perm[k]]..cp[perm[k] + 1]] {
            let j = pos[j0];
            if j > k && mark[j] != k {
                mark[j] = k;
                s.push(j);
            }
        }
        for &c in &kids[k] {
            for &j in &st[c][1..] {
                if mark[j] != k {
                    mark[j] = k;
                    s.push(j);
                }
            }
        }
        s.sort_unstable();
        if s.len() > 1 {
            kids[s[1]].push(k);
        }
        st[k] = s;
    }
    st
}

/// `2e4f14` L700-1360: maximal supernodes, relaxed amalgamation (child into parent while the extra Σk² stays
/// below 3b²+1000, b = rows below the child; 0.1b² from 192 columns), merge of sibling single columns with equal
/// structure (> 24 rows, up to 24 columns), postorder. Returns supernode columns in final order and row counts.
fn supernodes(n: usize, st: &[Vec<usize>]) -> (Vec<Vec<usize>>, Vec<usize>) {
    let ln: Vec<usize> = st.iter().map(|s| s.len()).collect();
    let (mut width, mut first, mut sn_of) = (Vec::new(), Vec::new(), vec![0usize; n]);
    let mut c = 0;
    while c < n {
        let mut w = 1;
        while w < n - c && ln[c + w] + w == ln[c] && st[c][w] == c + w {
            w += 1;
        }
        sn_of[c..c + w].fill(width.len());
        first.push(c);
        width.push(w);
        c += w;
    }
    let ns = width.len();
    let nil = usize::MAX;
    let mut ncol: Vec<i64> = width.iter().map(|&w| w as i64).collect();
    let mut nrow: Vec<i64> = first.iter().map(|&f| ln[f] as i64).collect();
    let (mut par, mut child, mut sib) = (vec![ns; ns], vec![nil; ns + 1], vec![nil; ns]);
    for s in 0..ns {
        if ncol[s] != nrow[s] {
            par[s] = sn_of[st[first[s]][ncol[s] as usize]];
        }
        sib[s] = child[par[s]];
        child[par[s]] = s;
    }
    let (mut head, mut next) = (vec![nil; ns], vec![nil; n]);
    for col in 0..n {
        next[col] = head[sn_of[col]];
        head[sn_of[col]] = col;
    }
    let sq = |k: i64| k as f64 * (k + 1) as f64 * (2 * k + 1) as f64 / 6.0;
    for s in 0..ns {
        let p = par[s];
        if p >= ns {
            continue;
        }
        let (cs, rs, rp) = (ncol[s], nrow[s], nrow[p]);
        let b = rs - cs;
        let lim = if cs < 192 { b as f64 * 3.0 } else { b as f64 * 0.1 };
        if rs > 1 && (sq(rp + cs) - sq(rp)) - (sq(rs) - sq(b)) < lim * b as f64 + 1000.0 {
            let mut col = head[s];
            while col != nil {
                let t = next[col];
                next[col] = head[p];
                head[p] = col;
                col = t;
            }
            if child[p] == s {
                child[p] = sib[s];
            } else {
                let mut q = child[p];
                while sib[q] != s {
                    q = sib[q];
                }
                sib[q] = sib[s];
            }
            let mut ch = child[s];
            while ch != nil {
                let t = sib[ch];
                par[ch] = p;
                sib[ch] = child[p];
                child[p] = ch;
                ch = t;
            }
            nrow[p] += cs;
            ncol[p] += cs;
            ncol[s] = -1;
        }
    }
    for p in 0..ns {
        if ncol[p] <= 0 {
            continue;
        }
        let mut c = child[p];
        while c != nil {
            if ncol[c] == 1 && nrow[c] > 24 && sib[c] != nil {
                let c0 = head[c];
                let mut prev = c;
                loop {
                    if ncol[c] > 23 {
                        break;
                    }
                    let s2 = sib[prev];
                    let k2 = head[s2];
                    if ncol[s2] == 1 && ln[k2] == ln[c0] && st[k2][1..] == st[c0][1..] {
                        nrow[c] += 1;
                        ncol[c] += 1;
                        next[k2] = head[c];
                        head[c] = k2;
                        let mut ch = child[s2];
                        while ch != nil {
                            let t = sib[ch];
                            par[ch] = c;
                            sib[ch] = child[c];
                            child[c] = ch;
                            ch = t;
                        }
                        ncol[s2] = -1;
                        sib[prev] = sib[s2];
                    } else {
                        prev = s2;
                    }
                    if sib[prev] == nil {
                        break;
                    }
                }
            }
            c = sib[c];
        }
    }
    // reverse of a stack preorder = postorder (children before parents)
    let (mut stack, mut visit) = (Vec::new(), Vec::new());
    let mut r = child[ns];
    while r != nil {
        stack.push(r);
        r = sib[r];
    }
    while let Some(x) = stack.pop() {
        visit.push(x);
        let mut c = child[x];
        while c != nil {
            stack.push(c);
            c = sib[c];
        }
    }
    let mut out = Vec::with_capacity(visit.len());
    let mut rows = Vec::with_capacity(visit.len());
    for &s in visit.iter().rev() {
        let mut cols = Vec::new();
        let mut col = head[s];
        while col != nil {
            cols.push(col);
            col = next[col];
        }
        rows.push(cols.len() + ln[*cols.iter().max().unwrap()] - 1);
        out.push(cols);
    }
    (out, rows)
}

/// `2e4f14` L2152-2350 with one ordering thread: cut each supernode into blocks of at most `lim` columns
/// (24 when `small`), capped at 192 (240 when the block's first column has ≥ 1920 rows).
fn split_blocks(snode: &[i32], col_len: &[i32], small: bool) -> Vec<i32> {
    let lim = if small { 24 } else { 2_000_000_000 };
    let mut out = Vec::with_capacity(snode.len());
    let mut j = 0;
    while j < snode.len() {
        let end = j + snode[j] as usize;
        while j < end {
            let b = lim.min(end - j).min(if col_len[j] < 1920 { 192 } else { 240 });
            out.push(b as i32);
            out.extend((1..b as i32).map(|i| -i));
            j += b;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn netlib(name: &str) -> Model {
        let p = format!("{}/../../../re/bench/data/netlib/{name}.mps.gz", env!("CARGO_MANIFEST_DIR"));
        rsolver_model::mps::read_file(&p).unwrap()
    }

    #[test]
    fn shell_sort_keeps_equal_keys_in_place_for_gap_one() {
        let mut a = vec![0, 1, 2];
        let mut k = vec![1.0, 0.0, 1.0];
        shell_sort(&mut a, &mut k);
        assert_eq!(a, vec![1, 0, 2]);
    }

    // expected values: Gurobi 13.0.3 oracle (tools/proto/bar_order_oracle.py, bench/oracle/bar_order)
    #[test]
    fn afiro_matches_gurobi() {
        let s = analyze(&netlib("afiro"));
        assert_eq!(s.aat_nnz, 126);
        assert_eq!(
            s.perm,
            vec![17, 11, 13, 16, 18, 0, 2, 3, 4, 5, 7, 10, 15, 8, 26, 25, 24, 23, 22, 21, 19, 9, 20, 14, 6, 12, 1]
        );
        assert_eq!((s.nnz_l, s.ops, s.ops_aat), (218, 2146.0, 245.0));
        assert_eq!(&s.blocks[..8], &[7, -1, -2, -3, -4, -5, -6, 12]);
        assert_eq!(&s.row_idx[..11], &[0, 1, 2, 3, 4, 5, 6, 18, 19, 25, 26]);
        assert_eq!(s.col_ptr[7], 11);
    }

    #[test]
    fn factor_counts_match_gurobi() {
        // (name, AA' NZ x2, Factor NZ, Factor Ops, Σ i·perm[i])
        for (nm, aat, nz, ops, ck) in [
            ("adlittle", 656, 692, 10160.0, 46349),
            ("kb2", 804, 562, 8522.0, 18626),
            ("israel", 22106, 12564, 1187368.0, 1230623),
            ("scsd6", 3904, 2838, 61350.0, 1025846),
            ("scagr25", 3844, 6286, 89960.0, 17333440),
            ("seba", 102800, 61106, 12947118.0, 30551947),
        ] {
            let s = analyze(&netlib(nm));
            let got: i64 = s.perm.iter().enumerate().map(|(i, &p)| i as i64 * p as i64).sum();
            assert_eq!((s.aat_nnz, s.nnz_l, s.ops, got), (aat, nz, ops, ck), "{nm}");
        }
    }

    /// All bench LPs against bench/oracle/bar_order/expected.tsv (column 2 marks the 11 that bisect).
    #[test]
    fn bench_lps_match_gurobi() {
        let tsv = format!("{}/../../bench/oracle/bar_order/expected.tsv", env!("CARGO_MANIFEST_DIR"));
        let ck = |v: &[i32]| v.iter().enumerate().map(|(i, &x)| (i as i64 + 1) * x as i64).sum::<i64>();
        let (mut ok, mut nbis) = (0, 0);
        for line in std::fs::read_to_string(tsv).unwrap().lines().filter(|l| !l.starts_with('#')) {
            let f: Vec<&str> = line.split('\t').collect();
            let v: Vec<i64> = f[1..].iter().map(|x| x.parse().unwrap()).collect();
            let s = analyze(&netlib(f[0]));
            let got = [s.aat_nnz, s.nnz_l, s.ops as i64, ck(&s.perm), ck(&s.blocks), ck(&s.col_len), ck(&s.row_idx)];
            assert_eq!(&got[..], &v[1..], "{}", f[0]);
            nbis += v[0];
            ok += 1;
        }
        assert_eq!((ok, nbis), (64, 11));
    }

    #[test]
    fn wide_supernodes_split_at_192() {
        let s = analyze(&netlib("fit1p"));
        let w: Vec<i32> = s.blocks.iter().copied().filter(|&x| x > 0).collect();
        assert_eq!(w, vec![192, 192, 192, 51]);
    }
}
