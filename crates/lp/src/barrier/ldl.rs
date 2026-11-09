//! 05 §2.6 Normal-equation factorization `M = L D Lᵀ` on a fixed (already permuted) pattern.
//! shortcut: up-looking scalar LDLᵀ (Davis' LDL) instead of Gurobi's supernodal kernel (`31c54b` → `2d8ee5`,
//! `32af6a`, MKL DGEMM/DTRSM), so results agree to rounding, not bit for bit; port the supernodal kernel when a
//! BarIterCount divergence is traced to factorization rounding (docs/m3/m3_barrier.md §4 step 4).

/// Symbolic structure plus numeric values of `L` (unit lower triangular, column-wise) and `D`.
#[derive(Clone, Debug)]
pub struct Ldl {
    pub n: usize,
    parent: Vec<usize>,
    lp: Vec<usize>,
    li: Vec<usize>,
    lx: Vec<f64>,
    pub d: Vec<f64>,
    /// F+0xe8: 1/d, or 0 for a dropped pivot; multipliers and solves multiply by it.
    e: Vec<f64>,
    /// Pivots dropped by the small-pivot rule in the last `numeric` call.
    pub fixed: usize,
}

const NONE: usize = usize::MAX;

impl Ldl {
    /// Elimination tree and column counts from the upper triangle of `M` given column-wise (`ap`, `ai`; rows ≤ column).
    pub fn symbolic(n: usize, ap: &[usize], ai: &[usize]) -> Ldl {
        let (mut parent, mut flag, mut cnt) = (vec![NONE; n], vec![NONE; n], vec![0usize; n]);
        for k in 0..n {
            flag[k] = k;
            for &r in &ai[ap[k]..ap[k + 1]] {
                let mut i = r;
                while i < k && flag[i] != k {
                    if parent[i] == NONE {
                        parent[i] = k;
                    }
                    cnt[i] += 1;
                    flag[i] = k;
                    i = parent[i];
                }
            }
        }
        let mut lp = vec![0; n + 1];
        for k in 0..n {
            lp[k + 1] = lp[k] + cnt[k];
        }
        let nz = lp[n];
        Ldl { n, parent, lp, li: vec![0; nz], lx: vec![0.0; nz], d: vec![0.0; n], e: vec![0.0; n], fixed: 0 }
    }

    /// Number of off-diagonal entries of `L` (Factor NZ counts these plus the diagonal).
    pub fn nnz(&self) -> usize {
        self.lp[self.n]
    }

    /// Numeric factorization of the upper triangle `(ap, ai, ax)`. LP pivot rule (`2d8ee5` L767–815, F+0x110 = 0):
    /// a pivot with `|d| ≤ 1e-16·|M_kk|` (or NaN) is dropped (d = 0, 1/d = 0); any other pivot is kept however small
    /// (oracle: degen2 keeps d = −3.6e-15 against M_kk = 12). Returns F+0x100 = min |d/M_kk| before dropping (0 on NaN).
    pub fn numeric(&mut self, ap: &[usize], ai: &[usize], ax: &[f64]) -> f64 {
        let n = self.n;
        let (mut y, mut pattern, mut flag, mut lnz) = (vec![0.0; n], vec![0usize; n], vec![NONE; n], vec![0usize; n]);
        self.fixed = 0;
        let mut min_ratio = 1e100f64;
        for k in 0..n {
            let mut top = n;
            flag[k] = k;
            let mut diag = 0.0;
            for p in ap[k]..ap[k + 1] {
                let i = ai[p];
                if i == k {
                    diag += ax[p];
                    continue;
                }
                y[i] += ax[p];
                let mut len = 0;
                let mut j = i;
                while flag[j] != k {
                    pattern[len] = j;
                    len += 1;
                    flag[j] = k;
                    j = self.parent[j];
                }
                while len > 0 {
                    top -= 1;
                    len -= 1;
                    pattern[top] = pattern[len];
                }
            }
            let mut dk = diag;
            for &i in &pattern[top..n] {
                let yi = y[i];
                y[i] = 0.0;
                let p2 = self.lp[i] + lnz[i];
                for p in self.lp[i]..p2 {
                    y[self.li[p]] -= self.lx[p] * yi;
                }
                let l_ki = yi * self.e[i];
                dk -= l_ki * yi;
                self.li[p2] = k;
                self.lx[p2] = l_ki;
                lnz[i] += 1;
            }
            if dk.is_nan() {
                min_ratio = 0.0;
            } else if diag != 0.0 {
                min_ratio = min_ratio.min((dk / diag).abs());
            }
            if diag.abs() * 1e-16 < dk.abs() {
                self.e[k] = 1.0 / dk;
            } else {
                dk = 0.0;
                self.e[k] = 0.0;
                self.fixed += 1;
            }
            self.d[k] = dk;
        }
        min_ratio
    }

    /// Solve `L D Lᵀ x = b` in place.
    pub fn solve(&self, x: &mut [f64]) {
        for j in 0..self.n {
            for p in self.lp[j]..self.lp[j + 1] {
                x[self.li[p]] -= self.lx[p] * x[j];
            }
        }
        for (xj, ej) in x.iter_mut().zip(&self.e) {
            *xj *= ej;
        }
        for j in (0..self.n).rev() {
            for p in self.lp[j]..self.lp[j + 1] {
                x[j] -= self.lx[p] * x[self.li[p]];
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::needless_range_loop)] // dense test matrices read clearer with indices
mod tests {
    use super::*;

    /// Upper triangle of a dense symmetric matrix as CSC.
    fn upper(a: &[Vec<f64>]) -> (Vec<usize>, Vec<usize>, Vec<f64>) {
        let (mut ap, mut ai, mut ax) = (vec![0], vec![], vec![]);
        for k in 0..a.len() {
            for i in 0..=k {
                if a[i][k] != 0.0 {
                    ai.push(i);
                    ax.push(a[i][k]);
                }
            }
            ap.push(ai.len());
        }
        (ap, ai, ax)
    }

    #[test]
    fn spd_and_semidefinite() {
        // Sparse SPD: M = G Gᵀ + I from a deterministic sparse G; then a singular one (rank 2 in 3x3).
        let n = 12;
        let mut g = vec![vec![0.0; n]; n];
        let mut s = 7u64;
        for i in 0..n {
            for j in 0..n {
                s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                if (s >> 60) < 3 {
                    g[i][j] = ((s >> 33) % 100) as f64 / 10.0 - 5.0;
                }
            }
        }
        let mut a = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                a[i][j] = (0..n).map(|k| g[i][k] * g[j][k]).sum::<f64>() + if i == j { 1.0 } else { 0.0 };
            }
        }
        let (ap, ai, ax) = upper(&a);
        let mut f = Ldl::symbolic(n, &ap, &ai);
        f.numeric(&ap, &ai, &ax);
        assert_eq!(f.fixed, 0);
        let b: Vec<f64> = (0..n).map(|i| i as f64 - 3.0).collect();
        let mut x = b.clone();
        f.solve(&mut x);
        for i in 0..n {
            let r: f64 = (0..n).map(|j| a[i][j] * x[j]).sum::<f64>() - b[i];
            assert!(r.abs() < 1e-9, "residual {r}");
        }
        let sing = vec![vec![1.0, 1.0, 0.0], vec![1.0, 1.0, 0.0], vec![0.0, 0.0, 2.0]];
        let (ap, ai, ax) = upper(&sing);
        let mut f = Ldl::symbolic(3, &ap, &ai);
        f.numeric(&ap, &ai, &ax);
        assert_eq!(f.fixed, 1);
        assert_eq!(f.d[1], 0.0);
    }
}
