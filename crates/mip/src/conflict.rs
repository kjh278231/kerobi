//! Conflict graph `*(state+0x3f0)` (0x48 bytes, made by `5b84b1`): literal pairs that cannot both be true, kept as a
//! literal CSR plus a list of pending pairs (docs/m2_red_probe.md §3.5). Literal encoding: `j` = x_j, `-1-j` = ¬x_j;
//! CSR slot `lit(e) = 2·var(e) + (e < 0)`. Probing fills it (`5b88ce`) and merges it (`5b9aad`); `4eba40` follows it.

/// `var(e)`
pub fn var(e: i32) -> usize {
    (if e < 0 { -1 - e } else { e }) as usize
}

/// CSR slot of literal `e`
pub fn lit(e: i32) -> usize {
    2 * var(e) + (e < 0) as usize
}

#[derive(Clone, Debug, Default)]
pub struct ConflictGraph {
    /// +0x00 variables the CSR covers (0 = no CSR), +0x04 state+0x408 when it was last cleaned
    pub n0: usize,
    pub rc0: i32,
    /// +0x08 literal starts (2·n0+1, empty = NULL) and +0x10 neighbours
    pub ptr: Vec<i64>,
    pub adj: Vec<i32>,
    /// +0x28 / +0x30 pending pairs (pa[t] <= pb[t]); +0x18 is their count
    pub pa: Vec<i32>,
    pub pb: Vec<i32>,
    /// +0x40 per-column "drop this column's edges at the next merge" (`5b91ad`), +0x38 its length; empty = NULL
    pub flag: Vec<u8>,
}

impl ConflictGraph {
    /// Edges in the CSR (`ptr[2·n0]`, 0 without a CSR).
    pub fn edges(&self) -> i64 {
        if self.ptr.is_empty() { 0 } else { self.ptr[2 * self.n0] }
    }

    /// `5b88ce(env, st, G, a, b, nocheck, &full, W)`: queue pair (a, b) unless the graph is at `limit` (`5b86ce`;
    /// skipped with `nocheck`). Returns full.
    pub fn push(&mut self, a: i32, b: i32, nocheck: bool, limit: i64, w: &mut f64, mu: f64) -> bool {
        if !nocheck && limit < self.edges() + self.pa.len() as i64 {
            return true;
        }
        self.pa.push(a.min(b));
        self.pb.push(a.max(b));
        *w += mu * 2.0;
        false
    }

    /// `5b9aad(env, st, G, 0, out, W)`: drop pending pairs of dead or flagged columns, merge the rest into the CSR
    /// (new neighbours after the old ones), then clean the lists of the touched variable range: dead columns go,
    /// duplicates in lists of length > 1 go (`out` counts 2·merged − duplicates). `n` = state+0xc, `rc` = +0x408.
    /// shortcut: only p4 == 0 (probing's calls); another caller passing p4 != 0 needs the `rc > rc0` restart.
    pub fn merge(&mut self, n: usize, rc: i32, col_cnt: &[i32], w: &mut f64, mu: f64) -> i64 {
        let mut out = 0i64;
        let (mut minv, mut maxv) = (n as i64, -1i64);
        let mut kept = 0usize;
        let mut deg: Vec<i32> = Vec::new();
        let flagged = |f: &[u8], v: usize| !f.is_empty() && f[v] != 0;
        if !self.pa.is_empty() {
            deg = vec![0; 2 * n];
            let np = self.pa.len();
            for t in 0..np {
                let (a, b) = (self.pa[t], self.pb[t]);
                let (va, vb) = (var(a), var(b));
                let keep = col_cnt[va] >= 0 && !flagged(&self.flag, va) && col_cnt[vb] >= 0 && !flagged(&self.flag, vb);
                if keep {
                    self.pa[kept] = a;
                    self.pb[kept] = b;
                    kept += 1;
                    minv = minv.min(va as i64).min(vb as i64);
                    maxv = maxv.max(va as i64).max(vb as i64);
                    deg[lit(a)] += 1;
                    deg[lit(b)] += 1;
                }
            }
            *w += (6.0 * mu) * np as f64;
            self.pa.truncate(kept);
            self.pb.truncate(kept);
        }
        let has_flag = !self.flag.is_empty();
        if self.n0 == 0 {
            if kept == 0 {
                return self.finish(out);
            }
            if has_flag {
                self.flag.resize(n, 0);
            }
            self.ptr = vec![0; 2 * n + 1];
        } else {
            if has_flag || !(n == self.n0 && self.rc0 == rc && kept == 0) {
                if has_flag {
                    self.flag.resize(n, 0);
                }
            } else {
                return self.finish(out);
            }
            if self.n0 < n {
                let e = self.ptr[2 * self.n0];
                self.ptr.resize(2 * n + 1, e);
                *w += (2.0 * mu) * (n - self.n0) as f64;
            }
        }
        self.n0 = n;
        if kept > 0 {
            out = 2 * kept as i64;
            let add = 2 * kept as i64;
            let mut e0 = self.ptr[2 * n];
            let mut f0 = e0 + add;
            self.adj.resize(f0 as usize, 0);
            let maxu = maxv as usize;
            if maxu + 1 < n {
                let len = e0 - self.ptr[2 * maxu + 2];
                if len > 0 {
                    f0 -= len;
                    e0 -= len;
                    self.adj.copy_within(e0 as usize..(e0 + len) as usize, f0 as usize);
                }
                for v in maxu + 1..n {
                    self.ptr[2 * v + 1] += add;
                    self.ptr[2 * v + 2] += add;
                }
                *w += (2.0 * mu) * (n - maxu - 1) as f64;
            }
            let mut v = maxv;
            while v >= minv {
                let u = v as usize;
                for l in [2 * u + 1, 2 * u] {
                    f0 -= deg[l] as i64;
                    let len = e0 - self.ptr[l];
                    self.ptr[l + 1] = f0;
                    if len > 0 {
                        f0 -= len;
                        e0 -= len;
                        self.adj.copy_within(e0 as usize..(e0 + len) as usize, f0 as usize);
                    }
                }
                v -= 1;
            }
            *w += ((n as i64 - minv) as f64 * 2.0 + (maxv - v) as f64 * 4.0) * mu;
            for t in 0..kept {
                let (a, b) = (self.pa[t], self.pb[t]);
                for (x, y) in [(a, b), (b, a)] {
                    let c = &mut self.ptr[lit(x) + 1];
                    self.adj[*c as usize] = y;
                    *c += 1;
                }
            }
            *w += (4.0 * mu) * kept as f64;
            self.pa.clear();
            self.pb.clear();
        }
        if !has_flag && maxv < minv {
            return self.finish(out);
        }
        let s0 = if has_flag { 0 } else { minv as usize };
        let mut mark = if minv <= maxv { vec![0u8; n] } else { Vec::new() };
        let mut wr = self.ptr[2 * s0];
        for v in s0..n {
            if col_cnt[v] < 0 || flagged(&self.flag, v) {
                self.ptr[2 * v] = wr;
                self.ptr[2 * v + 1] = wr;
                continue;
            }
            for s in 0..2 {
                let l = 2 * v + s;
                let (b, e) = (self.ptr[l], self.ptr[l + 1]);
                self.ptr[l] = wr;
                let dd = minv <= v as i64 && v as i64 <= maxv && e - b > 1;
                for q in b..e {
                    let x = self.adj[q as usize];
                    let u = var(x);
                    if col_cnt[u] < 0 || flagged(&self.flag, u) {
                        continue;
                    }
                    if dd {
                        let bit = (x < 0) as u8 + 1;
                        if mark[u] & bit != 0 {
                            out -= 1;
                            continue;
                        }
                        mark[u] |= bit;
                    }
                    self.adj[wr as usize] = x;
                    wr += 1;
                }
                *w += (mu * 2.0) * (e - b) as f64;
                if dd {
                    for q in self.ptr[l]..wr {
                        mark[var(self.adj[q as usize])] = 0;
                    }
                    *w += (mu * 2.0) * (wr - self.ptr[l]) as f64;
                }
            }
        }
        *w += (mu * 4.0) * (n - s0) as f64;
        self.ptr[2 * n] = wr;
        self.adj.truncate(wr as usize);
        if wr == 0 {
            self.ptr.clear();
            self.adj.clear();
        }
        if wr == 0 || s0 == 0 {
            self.rc0 = rc;
        }
        self.finish(out)
    }

    /// `5b9aad`'s common exit: no CSR means n0 = 0; the flag array is always dropped.
    fn finish(&mut self, out: i64) -> i64 {
        if self.ptr.is_empty() {
            self.n0 = 0;
        }
        self.flag.clear();
        out
    }
}

/// `5b86ce`'s limit on edges + pending pairs: nnz + 2·(live columns + live rows + SOS + general constraints + QC
/// + st+0x2f8), times 100 plus 1e6 (500 plus 1e7 with PRECLIQUE or Presolve >= 2), cut to an integer.
///
/// shortcut: SOS, general constraints, QC and st+0x2f8 counts are 0 (linear models).
pub fn limit(nnz: i64, live_cols: i32, live_rows: i32, preclique: i32, presolve: i32) -> i64 {
    let l = nnz + 2 * (live_cols as i64 + live_rows as i64);
    let lim = if preclique < 2 && presolve < 2 { l as f64 * 100.0 + 1e6 } else { l as f64 * 500.0 + 1e7 };
    lim as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_dedups_and_keeps_order() {
        // 3 variables; pairs (x0, ¬x1) twice and (x1, x2): x0's list gets ¬x1 once
        let mut g = ConflictGraph::default();
        let mut w = 0.0;
        for (a, b) in [(0, -2), (-2, 0), (1, 2)] {
            assert!(!g.push(a, b, false, 1 << 40, &mut w, 1.0));
        }
        let out = g.merge(3, 0, &[1, 1, 1], &mut w, 1.0);
        assert_eq!(out, 2 * 3 - 2);
        // lists by slot: 0 (x0), 1 (¬x0), 2 (x1), 3 (¬x1), 4 (x2), 5 (¬x2)
        assert_eq!(g.ptr, vec![0, 1, 1, 2, 3, 4, 4]);
        assert_eq!(g.adj, vec![-2, 2, 0, 1]);
        assert!(g.pa.is_empty());
    }
}
