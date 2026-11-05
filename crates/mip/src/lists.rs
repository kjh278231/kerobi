//! Tracker list getters on the linked-list matrix (state+0xe0 == 0 branch): `538f30` rows, `5399fc` columns,
//! `53af0f` / `53a487` adjacent columns / rows. Same rules as the LP port's CSR versions
//! (crates/lp/src/presolve/state.rs); the list walk visits every node, dead ones (col -1) included, and its work per
//! node is 5 (`538f30`, `53af0f`, `53a487`) or 4 (`5399fc`) times the multiplier instead of the CSR's 3.
use crate::state::{State, NIL};

impl State {
    /// Rows (`cols` false) or columns of the nodes in list `k` of the other kind, appended to `l` when live and
    /// unmarked; returns the number of nodes walked.
    fn gather(&mut self, k: usize, of_col: bool, l: &mut Vec<i32>) -> usize {
        let mut q = if of_col { self.col_head[k] } else { self.row_head[k] };
        let mut walked = 0;
        while q != NIL {
            let nd = self.nd[q as usize];
            if nd.col >= 0 {
                let x = if of_col { nd.row } else { nd.col } as usize;
                if !self.mark[x] {
                    self.mark[x] = true;
                    l.push(x as i32);
                }
            }
            walked += 1;
            q = if of_col { nd.nc } else { nd.nr };
        }
        walked
    }

    /// `538f30`: rows to visit at the tracker's level: marked rows, then rows of marked columns in list order, live
    /// ones only; all live rows in index order when that would be most of them. Cached per level until a reset.
    pub fn rows_list(&mut self) -> Vec<i32> {
        self.side_list(false)
    }

    /// `5399fc`: the column counterpart of `rows_list`.
    pub fn cols_list(&mut self) -> Vec<i32> {
        self.side_list(true)
    }

    fn side_list(&mut self, cols: bool) -> Vec<i32> {
        let lv = self.tracker.level;
        let (mine, other) = if cols { (&self.tracker.cols, &self.tracker.rows) } else { (&self.tracker.rows, &self.tracker.cols) };
        if lv == 0 {
            return mine.list[0][..mine.a[0]].to_vec();
        }
        if let Some(c) = &mine.cache[lv] {
            return c.clone();
        }
        let (na, no) = (mine.a[lv], other.a[lv]);
        let (dim, odim) = if cols { (self.n, self.m) } else { (self.m, self.n) };
        let mut dense = (odim as f64) <= no as f64 * 5.0 || (dim as f64) <= na as f64 * 4.0;
        if !dense {
            let ocnt = if cols { &self.row_cnt } else { &self.col_cnt };
            let sum: i64 = other.list[lv][..no].iter().map(|&k| ocnt[k as usize].max(0) as i64).sum();
            self.work += (no as f64 + no as f64) * self.work_mult;
            dense = (dim as f64) < na as f64 * 3.0 + sum as f64;
        }
        let out: Vec<i32> = if dense {
            self.work += (dim as f64 + dim as f64) * self.work_mult;
            let cnt = if cols { &self.col_cnt } else { &self.row_cnt };
            (0..dim as i32).filter(|&k| cnt[k as usize] >= 0).collect()
        } else {
            let q = if cols { &self.tracker.cols } else { &self.tracker.rows };
            let mut l: Vec<i32> = q.list[lv][..na].to_vec();
            let olist: Vec<i32> = (if cols { &self.tracker.rows } else { &self.tracker.cols }).list[lv][..no].to_vec();
            for &k in &l {
                self.mark[k as usize] = true;
            }
            self.work += self.work_mult * 3.0 * na as f64;
            let per = if cols { 4.0 } else { 5.0 };
            for &k in &olist {
                let walked = self.gather(k as usize, !cols, &mut l);
                self.work += self.work_mult * per * walked as f64;
            }
            self.work += (no as f64 + no as f64) * self.work_mult;
            for &k in &l {
                self.mark[k as usize] = false;
            }
            self.work += self.work_mult * 3.0 * l.len() as f64;
            let cnt = if cols { &self.col_cnt } else { &self.row_cnt };
            l.retain(|&k| cnt[k as usize] >= 0);
            l
        };
        let q = if cols { &mut self.tracker.cols } else { &mut self.tracker.rows };
        q.cache[lv] = Some(out.clone());
        out
    }

    /// `53af0f` (columns) / `53a487` (rows): the items marked at the tracker's level plus every item sharing a row
    /// (column) with the other kind's list (`538f30` / `5399fc`), live ones only; all live items in index order when
    /// that would be most of them. Cached per level until the next reset.
    pub fn adjacent_list(&mut self, cols: bool) -> Vec<i32> {
        let lv = self.tracker.level;
        let q = if cols { &self.tracker.cols } else { &self.tracker.rows };
        if lv == 0 {
            return q.list[0][..q.a[0]].to_vec();
        }
        if let Some(c) = &q.cache2[lv] {
            return c.clone();
        }
        let na = q.a[lv];
        let dim = if cols { self.n } else { self.m };
        let mult = self.work_mult;
        let mut dense = !((na as f64) * 5.0 < dim as f64);
        let mut other = Vec::new();
        if !dense {
            other = if cols { self.rows_list() } else { self.cols_list() };
            let ocnt = if cols { &self.row_cnt } else { &self.col_cnt };
            let sum: i64 = other.iter().map(|&i| ocnt[i as usize].max(0) as i64).sum();
            self.work += (other.len() as f64 + other.len() as f64) * mult;
            dense = dim as f64 <= na as f64 + na as f64 + (sum * 2) as f64;
        }
        let out: Vec<i32> = if dense {
            self.work += (dim as f64 + dim as f64) * mult;
            let cnt = if cols { &self.col_cnt } else { &self.row_cnt };
            (0..dim as i32).filter(|&j| cnt[j as usize] >= 0).collect()
        } else {
            let q = if cols { &self.tracker.cols } else { &self.tracker.rows };
            let mut l: Vec<i32> = q.list[lv][..na].to_vec();
            for &j in &l {
                self.mark[j as usize] = true;
            }
            self.work += mult * 3.0 * na as f64;
            for &i in &other {
                let walked = self.gather(i as usize, !cols, &mut l);
                self.work += walked as f64 * mult * 5.0;
            }
            self.work += (other.len() as f64 + other.len() as f64) * mult;
            for &j in &l {
                self.mark[j as usize] = false;
            }
            self.work += mult * 3.0 * l.len() as f64;
            let cnt = if cols { &self.col_cnt } else { &self.row_cnt };
            l.retain(|&j| cnt[j as usize] >= 0);
            l
        };
        let q = if cols { &mut self.tracker.cols } else { &mut self.tracker.rows };
        q.cache2[lv] = Some(out.clone());
        out
    }
}
