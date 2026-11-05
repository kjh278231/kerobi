//! 05 §2.1 Normal-equation matrix `M = A Θ Aᵀ + diag(r)` on the fixed pattern of `AAᵀ` (upper triangle, CSC).

/// Pattern of the upper triangle of `AAᵀ` (diagonal always present) and, per structural column, the positions its
/// outer product `a_j a_jᵀ` adds into.
#[derive(Clone, Debug)]
pub struct Normal {
    pub m: usize,
    pub ap: Vec<usize>,
    pub ai: Vec<usize>,
    pub ax: Vec<f64>,
    diag: Vec<usize>,
    /// For column j: (entry p, entry q, position in ax) for every pair p ≤ q of the column's entries.
    pairs: Vec<Vec<(usize, usize, usize)>>,
}

impl Normal {
    /// `beg/ind` give the columns of `A` (m rows); `beg` has n+1 entries.
    pub fn new(m: usize, beg: &[usize], ind: &[usize]) -> Normal {
        let n = beg.len() - 1;
        let mut cols: Vec<Vec<usize>> = (0..m).map(|k| vec![k]).collect();
        for j in 0..n {
            for &r in &ind[beg[j]..beg[j + 1]] {
                for &s in &ind[beg[j]..beg[j + 1]] {
                    if r < s {
                        cols[s].push(r);
                    }
                }
            }
        }
        let (mut ap, mut ai) = (vec![0], vec![]);
        for c in &mut cols {
            c.sort_unstable();
            c.dedup();
            ai.extend_from_slice(c);
            ap.push(ai.len());
        }
        let pos = |r: usize, s: usize| ap[s] + ai[ap[s]..ap[s + 1]].binary_search(&r).unwrap();
        let diag = (0..m).map(|k| pos(k, k)).collect();
        let pairs = (0..n)
            .map(|j| {
                let mut v = vec![];
                for p in beg[j]..beg[j + 1] {
                    for q in beg[j]..beg[j + 1] {
                        if ind[p] <= ind[q] {
                            v.push((p, q, pos(ind[p], ind[q])));
                        }
                    }
                }
                v
            })
            .collect();
        Normal { m, ax: vec![0.0; ai.len()], ap, ai, diag, pairs }
    }

    /// Off-diagonal plus diagonal entries of the upper triangle (the "AA' NZ" statistic counts this pattern).
    pub fn nnz(&self) -> usize {
        self.ai.len()
    }

    /// Fill `ax` with `A Θ Aᵀ + diag(r)`.
    pub fn assemble(&mut self, val: &[f64], theta: &[f64], r: &[f64]) {
        self.ax.iter_mut().for_each(|x| *x = 0.0);
        for (k, &d) in self.diag.iter().enumerate() {
            self.ax[d] = r[k];
        }
        for (j, pr) in self.pairs.iter().enumerate() {
            for &(p, q, at) in pr {
                self.ax[at] += val[p] * val[q] * theta[j];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small() {
        // A = [[1, 0, 2], [0, 3, 4]]: AΘAᵀ with Θ = (1, 2, 0.5) is [[3, 4], [4, 26]].
        let (beg, ind, val) = (vec![0, 1, 2, 4], vec![0, 1, 0, 1], vec![1.0, 3.0, 2.0, 4.0]);
        let mut nm = Normal::new(2, &beg, &ind);
        assert_eq!((nm.ap.clone(), nm.ai.clone()), (vec![0, 1, 3], vec![0, 0, 1]));
        nm.assemble(&val, &[1.0, 2.0, 0.5], &[0.0, 1.0]);
        assert_eq!(nm.ax, vec![3.0, 4.0, 27.0]);
    }
}
