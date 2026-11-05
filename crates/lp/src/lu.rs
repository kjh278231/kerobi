//! 04 §4 Basis factorization. Singleton phase of the original `1da0b2` (row singletons from a stack, slack rows
//! without structural entries, then column singletons in basis order) and the column-oriented dense solves of
//! `214344`/`2126cd`. On a triangular basis the bits depend only on the pivot order, so this reproduces the
//! original x_B exactly (60/60 bench LPs with a valid iteration-0 snapshot). shortcut: Markowitz, 2x2 blocks and dense dgetrf
//! (`036ade`) are not ported yet; `factor` reports an incomplete factorization when singletons do not finish.
use crate::spx::Spx;

/// Drop tolerance (GURO_PAR_ZERO, spx+0x3a8 / LU+0x10).
pub const ZERO: f64 = 1e-13;

pub struct Lu {
    /// Pivot steps in order: (row, basis position, diagonal).
    pub steps: Vec<(usize, usize, f64)>,
    /// Step at which each row was pivoted (-1 if not).
    pub prow: Vec<i64>,
    pub head: Vec<usize>,
    /// Step of each basis position.
    kpos: Vec<usize>,
    /// Lᵀ row of step k: (earlier step j, a/d_j), descending j (LU+0x70/0x78/0x80). U is diagonal after singletons.
    lrow: Vec<Vec<(usize, f64)>>,
    /// Sparse-solve multiplier LU+0x104: a sparse Lᵀ needs `mult·nnz ≤ m` (036ade L5709).
    sparse_mult: usize,
    /// Lᵀ DAG level of each step (LU+0x130) when BTRAN takes the level path (LU+0x140 ≥ 0, 036ade L5757).
    level: Option<Vec<usize>>,
}

impl Lu {
    pub fn complete(&self) -> bool {
        self.steps.len() == self.head.len()
    }

    pub fn factor(spx: &Spx, head: &[usize]) -> Lu {
        let (m, n) = (spx.m, spx.n);
        let mut rc = vec![0usize; m];
        for &j in head {
            if j < n {
                for (i, _) in spx.col(j) {
                    rc[i] += 1;
                }
            } else {
                rc[j - n] += 1 + m; // slack-basic row marker
            }
        }
        let mut rows: Vec<Vec<(usize, f64)>> = vec![Vec::new(); m];
        for (pos, &j) in head.iter().enumerate() {
            if j < n {
                for (i, a) in spx.col(j) {
                    if rc[i] <= m {
                        rows[i].push((pos, a));
                    }
                }
            }
        }
        let mut lu = Lu {
            steps: Vec::with_capacity(m),
            prow: vec![-1; m],
            head: head.to_vec(),
            kpos: Vec::new(),
            lrow: Vec::new(),
            sparse_mult: 2,
            level: None,
        };
        let mut ppos = vec![-1i64; m];
        let pivot = |lu: &mut Lu, ppos: &mut Vec<i64>, r: usize, pos: usize, d: f64| {
            let k = lu.steps.len() as i64;
            lu.prow[r] = k;
            ppos[pos] = k;
            lu.steps.push((r, pos, d));
        };
        // row singletons, LIFO stack seeded in ascending row order
        let mut stack: Vec<usize> = (0..m).filter(|&i| rc[i] == 1).collect();
        while let Some(r) = stack.pop() {
            if rc[r] == 0 {
                continue;
            }
            rc[r] = 0;
            let &(pos, d) = rows[r].iter().find(|(p, _)| ppos[*p] == -1).expect("row singleton has a free column");
            pivot(&mut lu, &mut ppos, r, pos, d);
            for (i, _) in spx.col(head[pos]) {
                if lu.prow[i] == -1 {
                    rc[i] -= 1;
                    if rc[i] == 1 {
                        stack.push(i);
                    }
                }
            }
        }
        // slacks of rows without structural entries
        for (pos, &j) in head.iter().enumerate() {
            if j >= n && rc[j - n] == m + 1 {
                rc[j - n] = 1;
                pivot(&mut lu, &mut ppos, j - n, pos, 1.0);
            }
        }
        // remaining column singletons and slacks, in basis order
        for (pos, &j) in head.iter().enumerate() {
            if ppos[pos] >= 0 {
                continue;
            }
            if j < n {
                if spx.len[j] == 1 {
                    let (i, a) = spx.col(j).next().unwrap();
                    if rc[i] <= m && lu.prow[i] < 0 {
                        pivot(&mut lu, &mut ppos, i, pos, a);
                    }
                }
            } else {
                let i = j - n;
                rc[i] -= m;
                pivot(&mut lu, &mut ppos, i, pos, 1.0);
            }
        }
        if lu.complete() {
            lu.transpose_views(spx);
        }
        lu
    }

    /// Row-wise Lᵀ, LU+0x104 and the Lᵀ levels (036ade L5700-5912). Singleton pivots leave no U off-diagonals:
    /// a row singleton's column has no entry in an earlier pivot row, and column singletons have one entry.
    fn transpose_views(&mut self, spx: &Spx) {
        let m = self.head.len();
        self.kpos = vec![0; m];
        self.lrow = vec![Vec::new(); m];
        for (k, &(_, pos, _)) in self.steps.iter().enumerate() {
            self.kpos[pos] = k;
        }
        for k in (0..m).rev() {
            let (_, pos, d) = self.steps[k];
            for (i, a) in spx.col(self.head[pos]) {
                let ki = self.prow[i];
                if ki > k as i64 {
                    self.lrow[ki as usize].push((k, a / d));
                }
                debug_assert!(ki >= k as i64, "U off-diagonal after singleton phase");
            }
        }
        let lnnz: usize = self.lrow.iter().map(Vec::len).sum();
        let t = lnnz as f64 / m as f64 + 1.0;
        let t = t * t * 5.0;
        self.sparse_mult = if lnnz <= m { 2 } else if m as f64 <= t { m } else { t as usize };
        if m >= 250 && self.sparse_mult < m {
            let mut lev = vec![0; m];
            for k in (0..m).rev() {
                for &(j, _) in &self.lrow[k] {
                    lev[j] = lev[j].max(lev[k] + 1);
                }
            }
            self.level = Some(lev);
        }
    }

    /// BTRAN `221173`: yᵀ = xᵀB⁻¹ for x given as (basis position, value) in input order. Returns (row, value) in the
    /// original's output order: sparse when it stays sparse, else all m rows in row order (output nnz = −1).
    /// Callers sum in this order, so the order is part of the result. shortcut: the Lᵀ linear-scan path (L956) and
    /// row etas are not ported; both need a Markowitz part or LU updates.
    pub fn btran(&self, x: &[(usize, f64)]) -> Vec<(usize, f64)> {
        assert!(self.complete(), "Markowitz phase not ported");
        let m = self.head.len();
        // Uᵀ is the diagonal: sparse (21a941, inputs in order) when (int)(LU+0x108 + LU+0x110)·nnz ≤ m, both 1.5 here
        let sparse_u = 3 * x.len() <= m;
        let mut xs: Vec<(usize, f64)> = Vec::with_capacity(x.len());
        let mut dense = vec![0.0; m]; // by step
        for &(pos, v) in x {
            let k = self.kpos[pos];
            let t = v / self.steps[k].2;
            if sparse_u {
                if ZERO < t.abs() {
                    xs.push((k, t));
                }
            } else if !(v.abs() <= ZERO || t.abs() <= ZERO) {
                dense[k] = t;
            }
        }
        if !sparse_u || self.sparse_mult * xs.len() > m {
            for &(k, t) in &xs {
                dense[k] = t;
            }
            for k in (0..m).rev() {
                // dense Lᵀ (L1576): values at or below ZERO are kept but not propagated
                let v = dense[k];
                if ZERO < v.abs() {
                    for &(j, l) in &self.lrow[k] {
                        dense[j] -= l * v;
                    }
                }
            }
            let mut y = vec![(0, 0.0); m];
            for (k, &(r, _, _)) in self.steps.iter().enumerate() {
                y[r] = (r, dense[k]);
            }
            return y;
        }
        let mut val: Vec<Option<f64>> = vec![None; m]; // touched steps
        for &(k, t) in &xs {
            val[k] = Some(t);
        }
        let mut out = Vec::new();
        let mut visit = |k: usize, val: &mut Vec<Option<f64>>, touched: &mut dyn FnMut(usize)| {
            let v = val[k].take().unwrap();
            if v.abs() >= ZERO {
                out.push((self.steps[k].0, v));
                for &(j, l) in &self.lrow[k] {
                    if val[j].is_none() {
                        val[j] = Some(0.0);
                        touched(j);
                    }
                    val[j] = Some(val[j].unwrap() - l * v);
                }
            }
        };
        match &self.level {
            None => {
                // heap path: descending step
                let mut heap: std::collections::BinaryHeap<usize> = xs.iter().map(|&(k, _)| k).collect();
                while let Some(k) = heap.pop() {
                    visit(k, &mut val, &mut |j| heap.push(j));
                }
            }
            Some(lev) => {
                // level path (L619): level buckets from the lowest, each in insertion order; fills go to higher levels
                let depth = lev.iter().copied().max().unwrap_or(0);
                let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); depth + 1];
                for &(k, _) in &xs {
                    buckets[lev[k]].push(k);
                }
                for l in 0..=depth {
                    let bucket = std::mem::take(&mut buckets[l]);
                    for k in bucket {
                        visit(k, &mut val, &mut |j| buckets[lev[j]].push(j));
                    }
                }
            }
        }
        out
    }

    /// Dense FTRAN: x (indexed by row) := B⁻¹x, returned by basis position.
    pub fn ftran_dense(&self, spx: &Spx, mut x: Vec<f64>) -> Vec<f64> {
        assert!(self.complete(), "Markowitz phase not ported");
        for (k, &(r, pos, d)) in self.steps.iter().enumerate() {
            if x[r].abs() > ZERO {
                let xr = x[r];
                for (i, a) in spx.col(self.head[pos]) {
                    if self.prow[i] > k as i64 {
                        x[i] -= (a / d) * xr;
                    }
                }
            }
        }
        for k in (0..self.steps.len()).rev() {
            let (r, pos, d) = self.steps[k];
            if x[r].abs() <= ZERO || (x[r] / d).abs() <= ZERO {
                x[r] = 0.0;
                continue;
            }
            x[r] /= d;
            let xr = x[r];
            for (i, a) in spx.col(self.head[pos]) {
                if self.prow[i] < k as i64 {
                    x[i] -= a * xr;
                }
            }
        }
        let mut xb = vec![0.0; self.head.len()];
        for &(r, pos, _) in &self.steps {
            xb[pos] = x[r];
        }
        xb
    }
}

/// Basic solution x_B = B⁻¹(b − N·x_N) with nonbasic values from the status array
/// (−1 lower, −2 upper, −4 fixed at lower, others 0); columns subtracted in order 0..n+m.
pub fn basic_solution(spx: &Spx, vstat: &[i32]) -> Vec<f64> {
    let head = basis_head(spx.m, vstat);
    let mut rhs = spx.b.clone();
    for (j, &s) in vstat.iter().enumerate() {
        let xj = match s {
            -2 => spx.u[j],
            -1 | -4 => spx.l[j],
            _ => 0.0,
        };
        if s < 0 && xj != 0.0 {
            for (i, a) in spx.col(j) {
                rhs[i] -= a * xj;
            }
        }
    }
    Lu::factor(spx, &head).ftran_dense(spx, rhs)
}

fn basis_head(m: usize, vstat: &[i32]) -> Vec<usize> {
    let mut head = vec![usize::MAX; m];
    for (j, &s) in vstat.iter().enumerate() {
        if s >= 0 {
            head[s as usize] = j;
        }
    }
    head
}

/// Iteration-0 reduced costs d (n+m) and exact DSE weights w (m, by basis position) for the basis in `vstat`.
/// d: basic 0, upper-bounded nonbasic (−2) negated, fixed structurals (−4) keep c_j (they are not in the row-wise
/// nonbasic copy); y = c_Bᵀ B⁻¹ from a dense BTRAN. w_p = ‖e_pᵀB⁻¹‖² summed in BTRAN output order (1bae9c EXACTNORM).
pub fn dual_start(spx: &Spx, vstat: &[i32]) -> (Vec<f64>, Vec<f64>) {
    let (m, n) = (spx.m, spx.n);
    let lu = Lu::factor(spx, &basis_head(m, vstat));
    let cb: Vec<(usize, f64)> = (0..m).map(|p| (p, spx.c[lu.head[p]])).collect();
    let mut y = vec![0.0; m];
    for (r, v) in lu.btran(&cb) {
        y[r] = v;
    }
    let d = (0..n + m)
        .map(|j| match vstat[j] {
            s if s >= 0 => 0.0,
            -4 if j < n => spx.c[j],
            s => {
                let t = spx.col(j).fold(spx.c[j], |t, (i, a)| t - a * y[i]);
                if s == -2 { -t } else { t }
            }
        })
        .collect();
    let w = (0..m).map(|p| sum_squares(&lu.btran(&[(p, 1.0)]).iter().map(|e| e.1).collect::<Vec<_>>())).collect();
    (d, w)
}

/// Σv² as 1bae9c sums it (L1971-2110): in sequence below 8 values, else 8 lanes over the 8-aligned head combined
/// ((0+2)+(4+6)) + ((1+3)+(5+7)) (asm 0xc3a26; the decompiled order is wrong), then the tail in sequence.
/// The vector buffer is 16-byte aligned, so no element is peeled first.
pub fn sum_squares(v: &[f64]) -> f64 {
    let (head, tail) = v.as_chunks::<8>();
    let mut a = [0.0; 8];
    for c in head {
        for q in 0..8 {
            a[q] += c[q] * c[q];
        }
    }
    let w = ((a[0] + a[2]) + (a[4] + a[6])) + ((a[1] + a[3]) + (a[5] + a[7])); // 0.0 when there is no full chunk
    tail.iter().fold(w, |w, t| w + t * t)
}

#[cfg(test)]
mod tests {
    use super::sum_squares;

    #[test]
    fn sum_squares_lane_order() {
        // 9 values: lanes take 0..8, the 9th is added last; (1e16 + 1) + ... shows the grouping
        let v = [1e8, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let lanes = ((1e16 + 1.0) + (1.0 + 1.0)) + ((1.0 + 1.0) + (1.0 + 1.0));
        assert_eq!(sum_squares(&v), lanes + 1.0);
        assert_eq!(sum_squares(&v[..7]), v[..7].iter().fold(0.0, |w, t| w + t * t));
    }
}
