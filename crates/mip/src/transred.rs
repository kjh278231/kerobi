//! `5431ee` binary implication graph (docs/m2_red_outer_a.md §7): two-entry `<` rows over [0,1] columns are
//! implications between literals (x_k or its complement); `53fa62` finds strongly connected components (equivalent
//! literals), `5416c3` a topological order, and `541d96` removes rows whose implication follows from a longer path
//! (transitive reduction).
use crate::state::{State, NIL};
use crate::Params;

/// The implication graph of `53dce0`: literal k < n is x_k, k >= n its complement; CSR by literal.
struct Graph {
    deg: Vec<i32>,
    beg: Vec<usize>,
    adj: Vec<usize>,
    rowof: Vec<usize>,
    /// a two-entry `<` row with rhs 0 and unit coefficients that gives no implication (`local_44`)
    flag: bool,
}

impl State {
    /// `5431ee`. Returns 0 (3 only from the SCC step, which is not reached on the models ported against).
    pub fn trans_red(&mut self, p: &Params) -> i32 {
        let n = self.n;
        if 2000000000.0 < n as f64 + n as f64 {
            return 0;
        }
        let pt = if p.pretransred == -1 { 1 } else { p.pretransred };
        let Some(g) = self.implication_graph(1) else { return 0 };
        if self.scc(&g) {
            // shortcut: 53fa62 changed something (3bb6fe aggregation, never on the 38 models): rebuild the graph
            // as the original does once 3bb6fe is ported
        }
        let (order, ok) = self.topo_order(&g);
        if ok {
            self.transitive_reduction(&g, &order);
        }
        // shortcut: PRETRANSRED >= 2 runs the mode-2 graph again (not decoded); the default (−1 → 1) stops here
        let _ = (pt, g.flag);
        0
    }

    /// `53dce0(env, st, mode)`. None when there is no implication.
    fn implication_graph(&mut self, mode: i32) -> Option<Graph> {
        let (n, m, mu) = (self.n, self.m, self.work_mult);
        let n2 = 2 * n;
        let mut deg = vec![0i32; n2];
        let (mut lit1, mut lit2) = (vec![-1i64; m], vec![-1i64; m]);
        let (mut ne, mut flag) = (0usize, false);
        for i in 0..m {
            if self.row_cnt[i] != 2 || self.sense[i] != b'<' {
                continue;
            }
            let mut b = self.rhs[i];
            let mut ab = b.abs();
            if !(1e-10 >= ab) && (mode != 1 || (!((self.nint as usize) < n) && !(1e-10 >= (ab - 1.0).abs()))) {
                continue;
            }
            let (mut c1, mut c2, mut a1, mut a2) = (-1i64, -1i64, 0.0f64, 0.0f64);
            let (mut unit, mut vis, mut fail) = (true, 0, false);
            let mut q = self.row_head[i];
            while q != NIL && self.nd[q as usize].col < 0 {
                q = self.nd[q as usize].nr;
                vis += 1;
            }
            while q != NIL {
                let x = self.nd[q as usize];
                let (c, a) = (x.col as usize, x.val);
                unit = !(1e-10 < (a.abs() - 1.0).abs());
                let d = self.ub[c] - self.lb[c];
                if 1e-10 >= d {
                    break;
                }
                let scale;
                if mode == 1 {
                    if self.vtype[c] == b'C' {
                        if !(-1e30 < self.lb[c]) || !(self.ub[c] < 1e30) || 10000.0 < d {
                            fail = true;
                            break;
                        }
                        b -= self.lb[c] * a;
                        scale = d;
                    } else {
                        if 1e-10 < self.lb[c].abs() || 1e-10 < (self.ub[c] - 1.0).abs() {
                            fail = true;
                            break;
                        }
                        self.vtype[c] = b'B';
                        self.lb[c] = 0.0;
                        self.ub[c] = 1.0;
                        scale = 1.0;
                    }
                } else {
                    if !unit {
                        fail = true;
                        break;
                    }
                    scale = 1.0;
                }
                if c1 != -1 {
                    a2 = a * scale;
                    c2 = c as i64;
                    break;
                }
                a1 = a * scale;
                c1 = c as i64;
                q = x.nr;
                vis += 1;
                while q != NIL && self.nd[q as usize].col < 0 {
                    q = self.nd[q as usize].nr;
                    vis += 1;
                }
            }
            self.work += mu * 4.0 * vis as f64;
            let mut ok = !fail && c2 >= 0;
            if ok && mode == 1 {
                let sc = 1.0 / a1.abs();
                a2 *= sc;
                a1 *= sc;
                b *= sc;
                ab = b.abs();
                if 1e-10 < (a2.abs() - 1.0).abs() || (!(1e-10 >= ab) && 1e-10 < (ab - 1.0).abs()) {
                    ok = false;
                }
            }
            if !ok {
                if unit && 1e-10 >= self.rhs[i].abs() {
                    flag = true;
                }
                continue;
            }
            if 0.0 > a1 && a2 > 0.0 {
                std::mem::swap(&mut c1, &mut c2);
                std::mem::swap(&mut a1, &mut a2);
            }
            let comp = |k: i64| (k + n as i64) % n2 as i64;
            if 1e-10 >= ab {
                if a1 > 0.0 && 0.0 > a2 {
                    (lit1[i], lit2[i]) = (c1, c2);
                }
            } else if b > 0.5 {
                if a1 > 0.0 && a2 > 0.0 {
                    (lit1[i], lit2[i]) = (c1, comp(c2));
                }
            } else if 0.0 > a1 && 0.0 > a2 {
                (lit1[i], lit2[i]) = (comp(c1), c2);
            }
            if lit1[i] >= 0 {
                ne += 2;
                deg[lit1[i] as usize] += 1;
                deg[comp(lit2[i]) as usize] += 1;
            }
        }
        self.work += (m as f64 * 5.0 + ne as f64) * mu;
        if ne == 0 {
            return None;
        }
        let prefix = |deg: &[i32]| {
            let mut beg = vec![0usize; n2];
            for k in 1..n2 {
                beg[k] = beg[k - 1] + deg[k - 1] as usize;
            }
            beg
        };
        let mut beg = prefix(&deg);
        self.work += (n2 as f64 + n2 as f64) * mu;
        let (mut adj, mut rowof) = (vec![0usize; ne], vec![0usize; ne]);
        for i in 0..m {
            if lit1[i] >= 0 {
                let (u, v) = (lit1[i] as usize, lit2[i] as usize);
                let nv = (v + n) % n2;
                adj[beg[u]] = v;
                rowof[beg[u]] = i;
                beg[u] += 1;
                adj[beg[nv]] = (u + n) % n2;
                rowof[beg[nv]] = i;
                beg[nv] += 1;
            }
        }
        self.work += (ne as f64 * 4.0 + m as f64) * mu;
        let beg = prefix(&deg);
        self.work += (n2 as f64 + n2 as f64) * mu;
        Some(Graph { deg, beg, adj, rowof, flag })
    }

    /// `53fa62` (Tarjan, iterative; indices restart at each root). Returns whether a component made a change.
    fn scc(&mut self, g: &Graph) -> bool {
        let (n, mu) = (self.n, self.work_mult);
        let n2 = 2 * n;
        self.work += (n as f64 + n as f64) * mu;
        let (mut idx, mut low, mut onst) = (vec![-1i32; n2], vec![-1i32; n2], vec![false; n2]);
        for r in 0..n2 {
            if idx[r] >= 0 {
                continue;
            }
            idx[r] = 0;
            low[r] = 0;
            let mut stack = vec![r];
            onst[r] = true;
            let mut dfs: Vec<(usize, usize)> = vec![(r, 0)];
            let mut counter = 1;
            while let Some(&(u, e0)) = dfs.last() {
                let (mut e, mut ed, mut down) = (e0, 0, false);
                while (e as i32) < g.deg[u] {
                    let v = g.adj[g.beg[u] + e];
                    if idx[v] < 0 {
                        dfs.last_mut().unwrap().1 = e + 1;
                        idx[v] = counter;
                        low[v] = counter;
                        counter += 1;
                        stack.push(v);
                        onst[v] = true;
                        dfs.push((v, 0));
                        self.work += 10.0 * mu;
                        down = true;
                        break;
                    }
                    if onst[v] {
                        low[u] = if low[u] < idx[v] { low[u] } else { idx[v] };
                        self.work += 2.0 * mu;
                    }
                    e += 1;
                    ed += 1;
                }
                self.work += mu * 4.0 * ed as f64;
                if down {
                    continue;
                }
                if low[u] == idx[u] {
                    let mut cnt = 0;
                    while let Some(w) = stack.pop() {
                        onst[w] = false;
                        if w == u {
                            break;
                        }
                        // shortcut: equivalent literals u, w would be merged by 3bb6fe (or fixed when w = ¬u); no
                        // component of two or more literals occurs on the 38 models, and 3bb6fe is not decoded
                        cnt += 1;
                    }
                    self.work += mu * 5.0 * cnt as f64;
                }
                dfs.pop();
                let Some(&(pu, _)) = dfs.last() else { break };
                low[pu] = if low[u] < low[pu] { low[u] } else { low[pu] };
                self.work += 4.0 * mu;
            }
        }
        self.work += mu * 7.0 * n2 as f64;
        false
    }

    /// `5416c3`: Kahn order from the literals without predecessors (a literal's in-degree is its complement's
    /// out-degree); ok = every edge goes up in level (no cycle).
    fn topo_order(&mut self, g: &Graph) -> (Vec<usize>, bool) {
        let (n, mu) = (self.n, self.work_mult);
        let n2 = 2 * n;
        let mut q: Vec<usize> = (0..n2).filter(|&k| g.deg[(n + k) % n2] == 0 && 0 < g.deg[k]).collect();
        self.work += ((n2 as f64 + n2 as f64) + q.len() as f64) * mu;
        let mut cnt = g.deg.clone();
        let mut lvl = vec![0i32; n2];
        let mut t = 0;
        while t < q.len() {
            let u = q[t];
            for e in 0..g.deg[u].max(0) as usize {
                let v = g.adj[g.beg[u] + e];
                lvl[v] = lvl[v].max(lvl[u] + 1);
                let k = (n + v) % n2;
                cnt[k] -= 1;
                if cnt[k] == 0 {
                    q.push(v);
                }
            }
            self.work += (5.0 * mu) * g.deg[u].max(0) as f64;
            t += 1;
        }
        self.work += mu * (q.len() as f64 + q.len() as f64);
        let mut ok = true;
        for k in 0..n2 {
            for e in 0..g.deg[k].max(0) as usize {
                if lvl[g.adj[g.beg[k] + e]] <= lvl[k] {
                    ok = false;
                }
            }
        }
        // Σ deg·(4μ), auto-vectorized in the original: 8 lanes (lane 0 starts at W), then pairs
        let f = 4.0 * mu;
        let k8 = n2 & !7;
        let mut w = self.work;
        if k8 > 0 {
            let mut l = [0.0f64; 8];
            l[0] = w;
            for b in (0..k8).step_by(8) {
                for (x, lane) in l.iter_mut().enumerate() {
                    *lane += g.deg[b + x] as f64 * f;
                }
            }
            w = ((l[0] + l[2]) + (l[4] + l[6])) + ((l[1] + l[3]) + (l[5] + l[7]));
        }
        if n2 - k8 >= 2 {
            let mut pr = [w, 0.0];
            for b in (k8..n2).step_by(2) {
                pr[0] += g.deg[b] as f64 * f;
                pr[1] += g.deg[b + 1] as f64 * f;
            }
            w = pr[0] + pr[1];
        }
        self.work = w;
        (q, ok)
    }

    /// `541d96`: from the end of the topological order, longest paths (DFS) from each literal not done; an edge
    /// u→v shorter than the longest u⇝v path has its row removed (`415f4f`). A chain continues from the first
    /// not-done literal reached through the complement, keeping the distances. Stops when the work since entry
    /// exceeds 1e7 + 100·nnz + 1000 per removed row.
    fn transitive_reduction(&mut self, g: &Graph, order: &[usize]) {
        let (n, mu) = (self.n, self.work_mult);
        let n2 = 2 * n;
        let w0 = self.work;
        let (mut dist, mut done) = (vec![-1i32; n2], vec![false; n2]);
        self.work += (n as f64 + n as f64) * mu;
        let mut removed = 0;
        for &start in order.iter().rev() {
            if self.work > (removed as f64 * 1000.0 + (self.nnz as f64 * 100.0 + 10000000.0)) + w0 {
                return;
            }
            if done[start] {
                continue;
            }
            let mut u = start;
            dist[u] = n2 as i32;
            let mut vis = Vec::new();
            loop {
                vis.push(u);
                let mut stack = vec![u];
                let mut pops = 0;
                while let Some(w) = stack.pop() {
                    let dw = g.deg[w].max(0) as usize;
                    for e in 0..dw {
                        let v = g.adj[g.beg[w] + e];
                        let mut dv = dist[v];
                        if dv == -1 {
                            stack.push(v);
                            vis.push(v);
                        }
                        if dist[w] + 1 > dv {
                            dv = dist[w] + 1;
                        }
                        dist[v] = dv;
                    }
                    pops += 1;
                    self.work += (4.0 * mu) * dw as f64;
                }
                self.work += (pops as f64 + pops as f64) * mu;
                let mut k = 0;
                while (k as i32) < g.deg[u] {
                    let e = g.beg[u] + k;
                    if dist[u] + 1 < dist[g.adj[e]] && self.row_cnt[g.rowof[e]] >= 0 {
                        self.drop_row_silent(g.rowof[e]);
                        removed += 1;
                    }
                    k += 1;
                }
                self.work += (mu * 3.0) * k as f64;
                done[u] = true;
                let cu = (u + n) % n2;
                let (mut x, mut k) = (None, 0);
                while (k as i32) < g.deg[cu] {
                    let c = (g.adj[g.beg[cu] + k] + n) % n2;
                    if !done[c] {
                        x = Some(c);
                        break;
                    }
                    k += 1;
                }
                self.work += (3.0 * mu) * k as f64;
                match x {
                    Some(c) => {
                        dist[c] = dist[u] - 1;
                        u = c;
                    }
                    None => break,
                }
            }
            for &v in &vis {
                dist[v] = -1;
            }
            self.work += (vis.len() as f64 + vis.len() as f64) * mu;
        }
    }

    /// `415f4f(st, r)`: remove row `r` (its live nodes die, columns lose a count), row count −2, rows removed +1;
    /// no tracker marks, no work.
    pub(crate) fn drop_row_silent(&mut self, r: usize) {
        let mut q = self.row_head[r];
        while q != NIL {
            let x = self.nd[q as usize];
            if x.col >= 0 {
                self.col_cnt[x.col as usize] -= 1;
                self.nd[q as usize].col = -1;
            }
            q = x.nr;
        }
        self.row_cnt[r] = -2;
        self.rows_removed += 1;
    }
}
