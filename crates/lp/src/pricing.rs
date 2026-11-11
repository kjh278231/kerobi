//! 04 §5 Dual pricing state (P = spx+0x478) and CHUZR: the ordered candidate list of mode 0 (`2896a3`, rebuilt by
//! `15d2d`), the infeasibility marks spx+0x280 and the BTRAN-cost score weights (docs/m1/m1_notes.md §8.2–8.4).
//! Mode 1 (Full pricing) keeps an unordered list of positions whose stored score P+0x18 is at least the threshold
//! P+0x30; marks then mean bit 0 = listed, bit 1 = feasible.
use crate::common::sort_desc;
use crate::work::{Work, X148};

/// Feasibility tolerance of CHUZR during the artificial-bound phase (spx+0x2cc): FeasTol·1e30·1e-5.
pub const BIG_M_TOL: f64 = 1e-6 * 1e30 * 1e-5;

/// Counts of one `rebuild` for the work stores (`1bae9c` C:799–1260).
pub struct Rebuilt {
    /// Feasible positions (marked 2).
    pub feasible: usize,
    /// Old list entries scanned (= the old length P+0x3c) and kept.
    pub scanned: usize,
    pub kept: usize,
    /// Violated positions appended (sorted, then merged).
    pub added: usize,
}

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
    /// Mode 1: stored scores by position (P+0x18, refreshed only when x_B changes) and the threshold P+0x30.
    pub score: Vec<f64>,
    pub thr: f64,
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
            score: vec![0.0; m],
            thr: 0.0,
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
    pub fn rebuild(&mut self, viol: &[Option<f64>], score: &[f64]) -> Rebuilt {
        let feasible = viol.iter().filter(|v| v.is_none()).count();
        let scanned = self.list.len();
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
        Rebuilt { feasible, scanned, kept: kept.len(), added: idx.len() }
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

    /// Mode-1 start after a factorization (`1bae9c` L2266–2471): t = the largest score / `div`; the threshold is t,
    /// or max(t, old) when the largest score passed a nonzero old threshold; list = the violated positions at or
    /// above it in position order.
    pub fn full_init(&mut self, viol: &[Option<f64>], score: &[f64], div: f64) {
        let mx = viol.iter().zip(score).filter(|(v, _)| v.is_some()).map(|(_, &s)| s).fold(0.0, |a, s| if s > a { s } else { a });
        let t = mx / div;
        self.thr = if self.thr != 0.0 && mx > self.thr && t <= self.thr { self.thr } else { t };
        self.list.clear();
        for (p, v) in viol.iter().enumerate() {
            if v.is_some() {
                self.score[p] = score[p];
                if score[p] >= self.thr {
                    self.list.push(p);
                    self.mark[p] = 1;
                } else {
                    self.mark[p] = 0;
                }
            } else {
                self.mark[p] = 2;
            }
        }
    }

    /// Mode-1 CHUZR (`2896a3` P+0x40 ≠ 0): drop listed positions that turned feasible or fell under the threshold,
    /// take the best stored score; if none, lower the threshold tenfold and scan the unlisted violated positions.
    pub fn full_pick(&mut self, work: &mut Work) -> Option<usize> {
        let mut best = self.thr * 0.5;
        let mut r = None;
        let scanned = self.list.len();
        let feasible = self.list.iter().filter(|&&p| self.mark[p] >= 2).count();
        let mut kept = Vec::with_capacity(scanned);
        for &p in &self.list {
            if self.mark[p] < 2 {
                if self.score[p] >= self.thr {
                    kept.push(p);
                    if best < self.score[p] {
                        best = self.score[p];
                        r = Some(p);
                    }
                } else {
                    self.mark[p] = 0;
                }
            } else if self.mark[p] < 5 {
                self.mark[p] = 2;
            }
        }
        self.list = kept;
        let a = ((self.list.len() as f64 * 0.5 + feasible as f64) + (scanned - feasible) as f64 * 2.0) * X148;
        if r.is_some() {
            work.add(0xcce72, a);
            return r;
        }
        if !self.mark.is_empty() {
            work.add(0xcd052, a);
        } else {
            work.carry(a);
        }
        let lower = self.thr / 10.0;
        let mut mx = 0.0;
        for p in 0..self.mark.len() {
            if self.mark[p] == 0 {
                if mx < self.score[p] {
                    mx = self.score[p];
                    r = Some(p);
                }
                if lower <= self.score[p] {
                    self.list.push(p);
                    self.mark[p] = 1;
                }
            }
        }
        let m = self.mark.len() as f64;
        work.add(0xcd191, ((self.list.len() as f64 + m) + m) * X148);
        if !self.list.is_empty() {
            self.thr = lower;
        } else if let Some(p) = r {
            self.list.push(p);
            self.mark[p] = 1;
            self.thr = mx;
        } else {
            self.thr = 1e-13;
        }
        r
    }

    /// Mode-1 update of a position whose x_B changed (`1bae9c` L556–726, `1c8477`): a violated position gets a fresh
    /// score and joins the list when at or above the threshold; a feasible one gets bit 1.
    pub fn full_touch(&mut self, p: usize, score: Option<f64>) {
        match score {
            Some(sc) => {
                self.score[p] = sc;
                if self.mark[p] == 3 {
                    self.mark[p] = 1;
                } else if self.mark[p] & 1 == 0 {
                    if sc < self.thr {
                        self.mark[p] = 0;
                    } else {
                        self.list.push(p);
                        self.mark[p] = 1;
                    }
                }
            }
            None => self.mark[p] |= 2,
        }
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
