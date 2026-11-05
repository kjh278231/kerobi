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
        let mut lu = Lu { steps: Vec::with_capacity(m), prow: vec![-1; m], head: head.to_vec() };
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
        lu
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
    let mut head = vec![usize::MAX; spx.m];
    for (j, &s) in vstat.iter().enumerate() {
        if s >= 0 {
            head[s as usize] = j;
        }
    }
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
