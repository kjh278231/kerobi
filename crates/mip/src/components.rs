//! `52cba4` connected components and `521497` biconnected blocks (docs/m2_red_final.md §3). For linear models both
//! searches add no work; they act only through a sub-solve (`5307f4` / `522aa2` → `98b671`), which is not ported.
use crate::state::{State, NIL};
use crate::Params;

impl State {
    /// `52cba4(st, env, T21, &ncomp)`: `52da19` splits the live row/column graph into components; with two or more,
    /// `5307f4` solves every one but the largest as its own MIP and `ncomp` gets the columns solved that way.
    pub fn components(&mut self, _p: &Params, ncomp: &mut i32) -> i32 {
        *ncomp = 0;
        let (nc, _) = self.component_search();
        // shortcut: `5307f4` (a sub-MIP solve of each smaller component) is not ported; it runs whenever nc > 1, which
        // none of the 38 models reach. Port it with the first model that has two or more components here.
        assert!(nc <= 1, "52cba4: {nc} components; the sub-solve 5307f4 is not ported");
        0
    }

    /// `52da19(st, env, &nc, …, size, W)` for linear models: rows in index order seed a breadth-first search (row →
    /// its live columns → their rows) over rows with live entries. Returns the component count and each component's
    /// size Σ row_cnt (capped at 2e9). No work. The pass for columns met by no row (st+0x250) is for quadratic
    /// objectives only.
    pub(crate) fn component_search(&self) -> (usize, Vec<i64>) {
        let (n, m) = (self.n, self.m);
        let (mut row_seen, mut col_seen) = (vec![false; m], vec![false; n]);
        let (mut sizes, mut rows, mut cols) = (Vec::new(), Vec::new(), Vec::new());
        for i in 0..m {
            if self.row_cnt[i] <= 0 || row_seen[i] {
                continue;
            }
            row_seen[i] = true;
            let mut size = self.row_cnt[i] as i64;
            let mut head = rows.len();
            rows.push(i);
            while head < rows.len() {
                let r = rows[head];
                head += 1;
                let first = cols.len();
                let mut e = self.row_head[r];
                while e != NIL {
                    let y = self.nd[e as usize];
                    if y.col >= 0 && !col_seen[y.col as usize] {
                        col_seen[y.col as usize] = true;
                        cols.push(y.col as usize);
                    }
                    e = y.nr;
                }
                for t in first..cols.len() {
                    let mut e = self.col_head[cols[t]];
                    while e != NIL {
                        let y = self.nd[e as usize];
                        if y.col >= 0 && !row_seen[y.row as usize] {
                            row_seen[y.row as usize] = true;
                            rows.push(y.row as usize);
                            size += self.row_cnt[y.row as usize] as i64;
                        }
                        e = y.nc;
                    }
                }
            }
            sizes.push(size.min(2_000_000_000));
        }
        (sizes.len(), sizes)
    }

    /// `521497(st, env, T22)` (PREBICONNECTED): linear models only (no SOS, PWL, QC, general constraints, NL). The
    /// column graph joins neighbours along each row of two or more entries (and its last to its first); a depth-first
    /// search finds blocks cut off by a non-root articulation column that is integer with ub = lb + 1, and each such
    /// block goes to `522aa2`. No work and no state change of its own.
    pub fn biconnected(&mut self, _p: &Params) -> i32 {
        let blocks = self.binary_cut_blocks();
        // shortcut: `522aa2` (→ `98b671`, a solve of the block with the cut column fixed) is not ported; none of the
        // 38 models has such a block. Port it with the first model that does.
        assert!(blocks.is_empty(), "521497: {} blocks; the block solve 522aa2 is not ported", blocks.len());
        0
    }

    /// The blocks `521497` hands to `522aa2`: (cut column, its child's subtree in DFS order).
    pub(crate) fn binary_cut_blocks(&self) -> Vec<(usize, Vec<usize>)> {
        let (n, m) = (self.n, self.m);
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        for i in 0..m {
            if self.row_cnt[i] <= 1 {
                continue;
            }
            let mut cs = Vec::new();
            let mut e = self.row_head[i];
            while e != NIL {
                let y = self.nd[e as usize];
                if y.col >= 0 {
                    cs.push(y.col as usize);
                }
                e = y.nr;
            }
            for t in 0..cs.len() {
                let (a, b) = (cs[t], cs[(t + 1) % cs.len()]);
                adj[a].push(b);
                adj[b].push(a);
            }
        }
        for a in adj.iter_mut() {
            a.sort_unstable();
            a.dedup();
        }
        cut_blocks(&adj, |v| self.col_cnt[v] >= 0, |p| self.vtype[p] != b'C' && self.ub[p] == self.lb[p] + 1.0)
    }
}

/// `521497`'s DFS over a sorted, deduplicated adjacency: (non-root articulation p with `cut_ok(p)`, the subtree of the
/// child it cuts off, in DFS order); roots are taken in index order among vertices with edges and `root_ok`.
fn cut_blocks(adj: &[Vec<usize>], root_ok: impl Fn(usize) -> bool, cut_ok: impl Fn(usize) -> bool) -> Vec<(usize, Vec<usize>)> {
    let n = adj.len();
    // iterative Hopcroft-Tarjan in the original's edge order; parent n marks a root
    let (mut parent, mut disc, mut low) = (vec![-1i64; n], vec![-1i64; n], vec![-1i64; n]);
    let mut blocks = Vec::new();
    for v in 0..n {
        if adj[v].is_empty() || !root_ok(v) || parent[v] >= 0 {
            continue;
        }
        parent[v] = n as i64;
        disc[v] = 0;
        low[v] = 0;
        let (mut order, mut stack, mut cnt) = (vec![v], vec![(v, 0usize)], 1i64);
        while let Some(&(u, it)) = stack.last() {
            let next = adj[u][it..].iter().position(|&w| parent[w] < 0).map(|k| it + k);
            for &x in &adj[u][it..next.unwrap_or(adj[u].len())] {
                if parent[u] != x as i64 && disc[x] < disc[u] {
                    low[u] = low[u].min(disc[x]);
                }
            }
            if let Some(k) = next {
                let w = adj[u][k];
                parent[w] = u as i64;
                order.push(w);
                disc[w] = cnt;
                low[w] = cnt;
                cnt += 1;
                stack.last_mut().unwrap().1 = k;
                stack.push((w, 0));
                continue;
            }
            stack.pop();
            let Some(top) = stack.last_mut() else { break };
            let (p, c) = (top.0, adj[top.0][top.1]);
            top.1 += 1;
            if disc[p] <= low[c] && parent[p] < n as i64 && cut_ok(p) {
                blocks.push((p, order[disc[c] as usize..cnt as usize].to_vec()));
            }
            low[p] = low[p].min(low[c]);
        }
        for &x in &order {
            disc[x] = -1;
            low[x] = -1;
        }
    }
    blocks
}

#[cfg(test)]
mod tests {
    use super::cut_blocks;

    #[test]
    fn blocks_of_path_and_triangle() {
        let path = vec![vec![1], vec![0, 2], vec![1]];
        assert_eq!(cut_blocks(&path, |_| true, |_| true), vec![(1, vec![2])]);
        let tri = vec![vec![1, 2], vec![0, 2], vec![0, 1]];
        assert!(cut_blocks(&tri, |_| true, |_| true).is_empty());
    }
}
