//! 04 §5 Dual pricing state (P = spx+0x478) and CHUZR: the ordered candidate list of mode 0 (`2896a3`, rebuilt by
//! `15d2d`), the infeasibility marks spx+0x280 and the BTRAN-cost score weights (docs/m1_notes.md §8.2–8.4).
//! shortcut: mode 1 (Full pricing, threshold list P+0x30) picks the best score over all positions; its list and
//! threshold rules (`1bae9c` L560–726) are not decoded yet.
use crate::common::sort_desc;

/// Feasibility tolerance of CHUZR during the artificial-bound phase (spx+0x2cc): FeasTol·1e30·1e-5.
pub const BIG_M_TOL: f64 = 1e-6 * 1e30 * 1e-5;

pub struct Pricing {
    /// P+0x40: 0 ordered list, 1 Full pricing.
    pub mode: i32,
    /// P+0x44: 0 plain viol²/w, 1 × P+0x70[j]/P+0x68[p], 2 / P+0x68[p].
    pub cost_mode: i32,
    pub list: Vec<usize>,
    pub cursor: usize,
    /// P+0x38: list rebuilds left before a refactor (reason 3).
    pub budget: i32,
    /// P+0x28 in mode 0: new infeasibilities allowed before the list is rebuilt.
    pub allow: i64,
    /// spx+0x280: 0 in the list and untouched, 1 infeasible not listed, 2 feasible.
    pub mark: Vec<u8>,
    /// P+0x68 by position, P+0x70 by variable.
    pub row_cost: Vec<f64>,
    pub var_cost: Vec<f64>,
    /// P+0x48: running average of 2·nnz(α_q).
    pub avg: f64,
}

impl Pricing {
    /// `534f7`: cost mode for SimplexPricing auto on a non-network LP; P+0x70[j] = nnz/n + len_j.
    pub fn new(m: usize, n: usize, len: &[i32]) -> Pricing {
        let nnz: i64 = len.iter().map(|&l| l as i64).sum();
        let (mf, nf, zf) = (m as f64, n as f64, nnz as f64);
        let cost_mode = if n < 100 || m < 100 || (nf * 20.0 < zf && nf * 10.0 * nf < mf * zf) { 0 } else { 1 };
        let base = zf / nf;
        let var_cost = (0..n + m).map(|j| base + if j < n { len[j] as f64 } else { 1.0 }).collect();
        Pricing {
            mode: 0,
            cost_mode,
            list: Vec::new(),
            cursor: 0,
            budget: 40,
            allow: 0,
            mark: vec![2; m],
            row_cost: vec![1.0; m],
            var_cost,
            avg: 0.0,
        }
    }

    /// Score of position p holding variable j with violation v (15d2d); the LP structural/slack factor is 1.
    pub fn score(&self, p: usize, j: usize, v: f64, w: f64) -> f64 {
        let s = v * v / w;
        match self.cost_mode {
            1 => s * (self.var_cost[j] / self.row_cost[p]),
            2 => s / self.row_cost[p],
            _ => s,
        }
    }

    /// `15d2d`: keep the untouched listed positions that stay in non-increasing score order, sort the other
    /// infeasible positions by score (`aabbe3`) and merge (ties keep the old entry first). `viol[p]` is the
    /// violation of position p (None if feasible), `score[p]` its score.
    pub fn rebuild(&mut self, viol: &[Option<f64>], score: &[f64]) {
        for (p, v) in viol.iter().enumerate() {
            if v.is_some() {
                if self.mark[p] != 0 {
                    self.mark[p] = 1;
                }
            } else {
                self.mark[p] = 2;
            }
        }
        let mut kept: Vec<usize> = Vec::new();
        for &p in &self.list {
            if self.mark[p] != 0 {
                continue;
            }
            if let Some(&last) = kept.last() {
                if score[last] < score[p] {
                    self.mark[p] = 1;
                    continue;
                }
            }
            kept.push(p);
        }
        let mut key = Vec::new();
        let mut idx = Vec::new();
        for (p, m) in self.mark.iter_mut().enumerate() {
            if *m == 1 {
                key.push(score[p]);
                idx.push(p as i32);
                *m = 0;
            }
        }
        sort_desc(&mut key, &mut idx);
        let mut out = Vec::with_capacity(kept.len() + idx.len());
        let (mut a, mut b) = (0, 0);
        while a < kept.len() && b < idx.len() {
            if score[kept[a]] < key[b] {
                out.push(idx[b] as usize);
                b += 1;
            } else {
                out.push(kept[a]);
                a += 1;
            }
        }
        out.extend_from_slice(&kept[a..]);
        out.extend(idx[b..].iter().map(|&p| p as usize));
        self.list = out;
        self.cursor = 0;
    }

    /// Mode-0 walk (`2896a3`): next listed position with mark 0, or None when the list is used up.
    pub fn next_listed(&mut self) -> Option<usize> {
        while self.cursor < self.list.len() {
            let p = self.list[self.cursor];
            self.cursor += 1;
            if self.mark[p] == 0 {
                return Some(p);
            }
        }
        None
    }

    /// Mark a position whose x_B changed (1c8477 / 1bae9c mode 0): infeasible 1 and one allowance used, else 2.
    pub fn touch(&mut self, p: usize, infeasible: bool) {
        if infeasible {
            self.mark[p] = 1;
            self.allow -= 1;
        } else {
            self.mark[p] = 2;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rebuild_keeps_untouched_order_and_merges() {
        let mut pr = Pricing::new(5, 2, &[1, 1]);
        pr.mark = vec![0, 0, 0, 2, 2];
        pr.list = vec![2, 0, 1];
        let viol = [Some(1.0), Some(1.0), Some(1.0), Some(1.0), None];
        // 0 now outscores 2: ejected and re-sorted; 3 is new
        let score = [5.0, 1.0, 3.0, 4.0, 0.0];
        pr.rebuild(&viol, &score);
        assert_eq!(pr.list, vec![0, 3, 2, 1]);
        assert_eq!(pr.mark, vec![0, 0, 0, 0, 2]);
    }
}
