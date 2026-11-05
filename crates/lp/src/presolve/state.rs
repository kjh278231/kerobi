//! LP presolve state (`3a484d` with CSR storage, 12 §4.1). Field comments give the original state offset.
//! Removal is lazy, as in the original: a deleted row-CSR entry gets column index -1, removed rows and columns
//! get a negative live count, and the column CSR keeps entries of removed rows until the model is rebuilt.
use super::tracker::Tracker;
use rsolver_model::Model;

/// Postsolve records (12 §4.2), newest last. Only what presolve pushes so far; uncrush is not ported yet.
#[derive(Clone, Debug)]
pub enum PostRec {
    /// type 1: column `j` fixed at `v`; `obj` its cost then, `status` the bound it sits at (-1 lower, -2 upper,
    /// -3 between, -4 fixed), `entries` its live (row, value) pairs for the dual.
    FixCol { j: i32, v: f64, obj: f64, status: i32, entries: Vec<(i32, f64)> },
    /// type 2 (`4f4d0a`): column `j`, singleton in row `r`, is `x_j = c0 + sum(row[k] * x_k)`; `mode` 0 free
    /// column, 1/2 implied upper/lower bound (row kept as inequality); `col` the column's other entries.
    Subst { j: i32, r: i32, c0: f64, a: f64, obj: f64, mode: i32, row: Vec<(i32, f64)>, col: Vec<(i32, f64)> },
    /// type 3 (`4f39e3`): column `j` with its rows dropped; it moves toward `dir` (1 up, 0 down) as far as the
    /// rows allow. `rows[t]` = (row, rhs/a), `terms[t]` = the row's other (column, -a_k/a).
    DualFix { j: i32, dir: i32, rows: Vec<(i32, f64)>, terms: Vec<Vec<(i32, f64)>> },
    /// type 7 (`4ffe11`): singleton row `r` became bound(s) of column `j` (`code` 1 lower, 2 upper, 3 both)
    /// at `x`; its dual is recovered from `j`'s reduced cost and `a`.
    SingletonRow { j: i32, r: i32, code: i32, a: f64, x: f64 },
    /// type 8 (`4ff158`): forcing row `r` (`dir` 1 at min activity, -1 at max) with its live entries.
    Forcing { r: i32, dir: f64, entries: Vec<(i32, f64)> },
    /// type 4 (`509fd0`): column `y` = `r` times column `x` merged into `x` (x' = x + r y); the bounds both had.
    ParallelCol { x: i32, y: i32, lx: f64, ly: f64, ux: f64, uy: f64, r: f64 },
    /// type 5 (`50b4e5`): `<` rows `a` (kept, now `=`) and `b` (dropped) bound the same line from both sides;
    /// row a = `ratio` times row b.
    ParallelRow { a: i32, b: i32, ratio: f64 },
    /// type 14 (`50c736`): like type 5 with room between the rows: row `a` (`=` at `rhs` now, with range column
    /// number `k` in [0, rhs - lo]) and dropped row `b`, row a = `ratio` times row b; `entries` row a's live entries.
    SplitRow { a: i32, b: i32, k: i32, rhs: f64, lo: f64, ratio: f64, entries: Vec<(i32, f64)> },
    /// type 6 (`39d3d9`, aggregation stage D): before x_e = (b - a_k x_k) / a_e went away through row `r`, e's
    /// bounds moved onto k (`code` bit 1 lower, bit 2 upper).
    AggBound { code: i32, e: i32, k: i32, r: i32, b: f64, ae: f64, ak: f64 },
    /// type 11 (`3a0e1b`, aggregation stage B): `<` row `r` was substituted out with column `j` becoming its
    /// slack (pivot |a|).
    SlackCol { r: i32, j: i32, a: f64 },
    /// type 21 (`39d14c`, aggregation stage C): in `=` row `r`, x_e1 = f1 x_k1 and x_e2 = f2 x_k2 were folded
    /// away; `a1`/`a2` their row-r coefficients, `obj`/`ub` theirs at push, `n1`/`n2` their other entry counts and
    /// `entries` those entries (e1's, then e2's).
    PairAgg {
        r: i32, e1: i32, e2: i32, k1: i32, k2: i32, n1: i32, n2: i32, f1: f64, f2: f64, a1: f64, a2: f64, obj1: f64,
        obj2: f64, ub1: f64, ub2: f64, entries: Vec<(i32, f64)>,
    },
}

#[derive(Clone, Debug)]
pub struct State {
    /// +0x0c / +0x10
    pub n: usize,
    pub m: usize,
    /// +0x20 / +0x28 / +0x30 / +0x38: row CSR start, slot count, column index (-1 deleted), value
    pub row_beg: Vec<i64>,
    pub row_len: Vec<i32>,
    pub row_ind: Vec<i32>,
    pub row_val: Vec<f64>,
    /// +0x40 / +0x48 / +0x50 / +0x58: column CSR
    pub col_beg: Vec<i64>,
    pub col_len: Vec<i32>,
    pub col_ind: Vec<i32>,
    pub col_val: Vec<f64>,
    /// +0x60 / +0x68: live nonzeros per row / column; negative = removed (-1 empty, -2 free/redundant, -3 merged)
    pub row_cnt: Vec<i32>,
    pub col_cnt: Vec<i32>,
    /// +0x70 / +0x90: infinite contributions to, and finite part of, max activity - rhs
    pub inf_max: Vec<i32>,
    pub act_max: Vec<f64>,
    /// +0x78 / +0x98: same for min activity - rhs
    pub inf_min: Vec<i32>,
    pub act_min: Vec<f64>,
    /// +0xa0 / +0xa8 / +0xb0
    pub lb: Vec<f64>,
    pub ub: Vec<f64>,
    pub obj: Vec<f64>,
    /// +0xb8 / +0xc0 ('<' or '='; '>' rows are stored negated)
    pub rhs: Vec<f64>,
    pub sense: Vec<u8>,
    /// +0xc8
    pub objcon: f64,
    /// +0xd8
    pub nnz: i64,
    /// +0x80 / +0x88: per-column growth bounds of substitution multipliers (start at 1.0)
    pub mult_lo: Vec<f64>,
    pub mult_hi: Vec<f64>,
    /// +0x170 per-column flags (nonzero = special column, e.g. PWL objective)
    pub col_flag: Vec<i32>,
    /// +0x3f8 / +0x3fc / +0x400 / +0x404 / +0x408
    pub bounds_tightened: i32,
    pub coefs_changed: i32,
    pub ineq_to_eq: i32,
    pub rows_removed: i32,
    pub cols_removed: i32,
    /// +0x42c: opposite-sign parallel `<` row pairs left as they are (`50b4e5`), then split (`50c736`)
    pub antipar: i32,
    /// +0x3e0 / +0x3e8: per split row, the row and s_a * lo (its range column gets bounds [0, rhs - that])
    pub splits: Vec<(i32, f64)>,
    /// +0x470 / +0x478: column / row proving infeasibility or unboundedness
    pub bad_col: i32,
    pub bad_row: i32,
    /// *(+0x458): deterministic work; +0x448 multiplier
    pub work: f64,
    pub work_mult: f64,
    /// +0x3d8
    pub tracker: Tracker,
    /// +0x370 record list
    pub post: Vec<PostRec>,
    /// +0x3a8 scratch marks for the tracker list getters (and aggregation)
    pub(super) mark: Vec<bool>,
    /// +0x158 / +0x198: aggregation node capacity and free count (`3a484d`: spare + nnz and spare);
    /// `node_extra` = +0x1b8, nodes added by `3b85ea`
    pub node_cap: i64,
    pub node_free: i64,
    pub node_extra: bool,
    /// +0x488: aggregation prescan state (0 not run, -1 no pattern)
    pub agg_scan: i32,
}

/// Snapshot of the progress counters (`3c546c`); LP uses the first nine.
#[derive(Clone, Copy, Debug, Default)]
pub struct Snapshot {
    n: i32,
    m: i32,
    cols_removed: i32,
    rows_removed: i32,
    ineq_to_eq: i32,
    coefs_changed: i32,
    bounds_tightened: i32,
}

/// Rounding guard of `4f79a1`: if `s = old + t` was inexact, push `s` one ulp in the safe direction
/// (`up`: max activity, larger; otherwise min activity, smaller).
pub(super) fn guarded_add(old: f64, t: f64, up: bool) -> f64 {
    let s = old + t;
    let inexact = if old.abs() < t.abs() { s - t != old } else { s - old != t };
    if !inexact {
        return s;
    }
    let grow = if up { s > 0.0 } else { s < 0.0 };
    s * if grow { 1.0000000000000002 } else { 0.9999999999999998 }
}

impl State {
    pub fn new(md: &Model) -> State {
        let (n, m) = (md.num_vars(), md.num_constrs());
        let neg: Vec<bool> = md.row_sense.iter().map(|&s| s == b'>').collect();
        // `5118e6`: column segments keep the model's starts; entries with |a| <= 1e-13 (or NaN) are squeezed out,
        // leaving a gap at the end of the segment.
        let raw_nnz: usize = md.col_len.iter().map(|&l| l as usize).sum();
        let (mut col_ind, mut col_val) = (vec![-1i32; raw_nnz], vec![0f64; raw_nnz]);
        let mut col_beg = Vec::with_capacity(n);
        let mut col_len = vec![0i32; n];
        let mut row_len = vec![0i32; m];
        let mut packed = 0usize;
        let contiguous = n > 0 && md.col_beg[n - 1] as usize + md.col_len[n - 1] as usize == raw_nnz;
        for j in 0..n {
            let b = md.col_beg[j] as usize;
            let start = if contiguous { b } else { packed };
            col_beg.push(start as i64);
            let mut kept = 0;
            for k in b..b + md.col_len[j] as usize {
                let (i, v) = (md.col_ind[k], md.col_val[k]);
                if !(1e-13 < v.abs()) {
                    continue;
                }
                col_ind[start + kept] = i;
                col_val[start + kept] = if neg[i as usize] { -v } else { v };
                kept += 1;
                row_len[i as usize] += 1;
            }
            col_len[j] = kept as i32;
            packed += md.col_len[j] as usize;
        }
        let mut row_beg = vec![0i64; m];
        let mut s = 0i64;
        for i in 0..m {
            row_beg[i] = s;
            s += row_len[i] as i64;
        }
        let mut fill = row_beg.clone();
        let mut row_ind = vec![-1i32; raw_nnz];
        let mut row_val = vec![0f64; raw_nnz];
        for j in 0..n {
            for k in col_beg[j] as usize..col_beg[j] as usize + col_len[j] as usize {
                let i = col_ind[k] as usize;
                row_ind[fill[i] as usize] = j as i32;
                row_val[fill[i] as usize] = col_val[k];
                fill[i] += 1;
            }
        }
        // `3a484d`: spare list nodes for aggregation (model+0x18 and the QC counts are 0 for LP)
        let spare = if raw_nnz / 10 <= 2_000_000_000 { (raw_nnz as i64 / 10).max(n.max(m).max(10000) as i64) } else { 2_000_000_000 };
        let rhs = md.rhs.iter().zip(&neg).map(|(&r, &g)| if g { -r } else { r }).collect();
        let sense = md.row_sense.iter().map(|&c| if c == b'>' { b'<' } else { c }).collect();
        State {
            n,
            m,
            row_cnt: row_len.clone(),
            row_beg,
            row_len,
            row_ind,
            row_val,
            col_cnt: col_len.clone(),
            col_beg,
            col_len,
            col_ind,
            col_val,
            inf_max: vec![0; m],
            act_max: vec![0.0; m],
            inf_min: vec![0; m],
            act_min: vec![0.0; m],
            lb: md.lb.clone(),
            ub: md.ub.clone(),
            obj: md.obj.clone(),
            rhs,
            sense,
            objcon: md.objcon,
            nnz: raw_nnz as i64,
            mult_lo: vec![1.0; n],
            mult_hi: vec![1.0; n],
            col_flag: vec![0; n],
            bounds_tightened: 0,
            coefs_changed: 0,
            ineq_to_eq: 0,
            rows_removed: 0,
            cols_removed: 0,
            antipar: 0,
            splits: Vec::new(),
            bad_col: -1,
            bad_row: -1,
            // shortcut: the constructor's own work terms (3a484d/5118e6/534a8b) are not added yet; add them for P2.
            work: 0.0,
            work_mult: if raw_nnz <= 100_000 { 1.0 } else { (raw_nnz as f64 / 1e4).log10().sqrt() },
            tracker: Tracker::new(m, n),
            post: Vec::new(),
            mark: vec![false; n.max(m)],
            node_cap: spare + raw_nnz as i64,
            node_free: spare,
            node_extra: false,
            agg_scan: 0,
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            n: self.n as i32,
            m: self.m as i32,
            cols_removed: self.cols_removed,
            rows_removed: self.rows_removed,
            ineq_to_eq: self.ineq_to_eq,
            coefs_changed: self.coefs_changed,
            bounds_tightened: self.bounds_tightened,
        }
    }

    /// `3c5595`: progress since a snapshot (LP: no SOS, QC or type changes). Same evaluation order as the original.
    pub fn progress(&self, s: &Snapshot) -> f64 {
        let live_cols = (s.n - s.cols_removed).max(1) as f64;
        let live_rows = (s.m - s.rows_removed).max(1) as f64;
        let nnz = self.nnz.max(1) as f64;
        let d_cols = ((self.cols_removed - s.cols_removed) - (self.n as i32 - s.n)).max(0);
        let d_rows = ((self.rows_removed - s.rows_removed) - (self.m as i32 - s.m)).max(0);
        ((self.coefs_changed - s.coefs_changed) as f64 * 0.1) / nnz
            + d_cols as f64 / live_cols
            + ((self.bounds_tightened - s.bounds_tightened) as f64 * 0.0001) / live_cols
            + d_rows as f64 / live_rows
            + ((self.ineq_to_eq - s.ineq_to_eq) as f64 * 0.5) / live_rows
            + 0.0 / live_rows
    }

    /// `4f79a1`: recompute activities of the given rows (all rows when `rows` is None).
    pub fn activity(&mut self, rows: Option<&[i32]>, infbound: f64) {
        let all: Vec<i32>;
        let rows = match rows {
            Some(r) => r,
            None => {
                all = (0..self.m as i32).collect();
                &all
            }
        };
        for &r in rows {
            let r = r as usize;
            if self.row_cnt[r] < 0 {
                continue;
            }
            self.inf_min[r] = 0;
            self.inf_max[r] = 0;
            self.act_min[r] = -self.rhs[r];
            self.act_max[r] = -self.rhs[r];
            let b = self.row_beg[r] as usize;
            for k in b..b + self.row_len[r] as usize {
                let j = self.row_ind[k];
                if j < 0 || self.col_cnt[j as usize] < 0 {
                    continue;
                }
                let (j, a) = (j as usize, self.row_val[k]);
                let (for_max, for_min) = if a <= 0.0 { (self.lb[j], self.ub[j]) } else { (self.ub[j], self.lb[j]) };
                let max_finite = if a <= 0.0 { for_max > -infbound } else { for_max < infbound };
                if max_finite {
                    self.act_max[r] = guarded_add(self.act_max[r], for_max * a, true);
                } else {
                    self.inf_max[r] += 1;
                }
                let min_finite = if a <= 0.0 { for_min < infbound } else { for_min > -infbound };
                if min_finite {
                    self.act_min[r] = guarded_add(self.act_min[r], a * for_min, false);
                } else {
                    self.inf_min[r] += 1;
                }
            }
            self.work += self.row_len[r] as f64 * 6.0;
        }
    }

    /// `5034e9`: drop empty rows (infeasible if their rhs is violated) and `<` rows with rhs >= 1e20.
    /// Returns 3 (infeasible) with `bad_row` set, else 0.
    pub fn empty_and_free_rows(&mut self, feastol: f64) -> i32 {
        let mut removed = 0;
        let mut rc = 0;
        for i in 0..self.m {
            if self.row_cnt[i] == 0 {
                let r = self.rhs[i];
                if (self.sense[i] == b'=' && feastol < r.abs()) || (self.sense[i] != b'=' && r < -feastol) {
                    self.bad_row = i as i32;
                    rc = 3;
                    break;
                }
                self.row_cnt[i] = -1;
                removed += 1;
            } else if self.row_cnt[i] > 0 && self.sense[i] != b'=' && 1e20 <= self.rhs[i] {
                removed += 1;
                let b = self.row_beg[i] as usize;
                for k in b..b + self.row_len[i] as usize {
                    let j = self.row_ind[k];
                    if j >= 0 {
                        self.col_cnt[j as usize] -= 1;
                        self.row_ind[k] = -1;
                    }
                }
                self.work += self.row_len[i] as f64 * 3.0;
                self.row_cnt[i] = -2;
            }
        }
        if rc == 0 {
            self.work += self.m as f64 * 5.0;
        }
        self.rows_removed += removed;
        rc
    }

    /// Presolved model from the state (`3b0917`). Surviving rows and columns keep their original order; column
    /// entries keep column-CSR order, skipping removed rows.
    pub fn to_model(&self, md: &Model) -> Model {
        let mut row_map = vec![-1i32; self.m];
        let mut q = Model { name: format!("{}_pre", md.name), sense: md.sense, objcon: self.objcon, ..Model::default() };
        for i in 0..self.m {
            if self.row_cnt[i] >= 0 {
                row_map[i] = q.rhs.len() as i32;
                q.rhs.push(self.rhs[i]);
                q.row_sense.push(self.sense[i]);
                q.constr_names.push(md.constr_names[i].clone());
            }
        }
        for j in 0..self.n {
            if self.col_cnt[j] < 0 {
                continue;
            }
            q.col_beg.push(q.col_ind.len() as i64);
            let b = self.col_beg[j] as usize;
            for k in b..b + self.col_len[j] as usize {
                let i = self.col_ind[k];
                if i >= 0 && row_map[i as usize] >= 0 {
                    q.col_ind.push(row_map[i as usize]);
                    q.col_val.push(self.col_val[k]);
                }
            }
            // `3b0917` takes the length from the live count; a mismatch means the state is inconsistent.
            debug_assert_eq!(q.col_ind.len() as i64 - q.col_beg.last().unwrap(), self.col_cnt[j] as i64, "col {j}");
            q.col_len.push(self.col_cnt[j]);
            q.obj.push(self.obj[j]);
            q.lb.push(self.lb[j]);
            q.ub.push(self.ub[j]);
            q.vtype.push(md.vtype[j]);
            q.var_names.push(md.var_names[j].clone());
        }
        // range columns of split rows, after the live columns; Gurobi leaves them unnamed and writes `C<j>`
        for &(i, lo) in &self.splits {
            q.var_names.push(format!("C{}", q.obj.len()));
            q.col_beg.push(q.col_ind.len() as i64);
            q.col_ind.push(row_map[i as usize]);
            q.col_val.push(1.0);
            q.col_len.push(1);
            q.obj.push(0.0);
            q.lb.push(0.0);
            q.ub.push(self.rhs[i as usize] - lo);
            q.vtype.push(b'C');
        }
        q
    }

    /// `538f30`: rows to visit at the tracker's level: marked rows, then rows of marked columns in column-CSR
    /// order, live ones only; all live rows in index order when that would be most of them.
    pub fn rows_list(&mut self) -> Vec<i32> {
        let lv = self.tracker.level;
        if lv == 0 {
            let q = &self.tracker.rows;
            return q.list[0][..q.a[0]].to_vec();
        }
        if let Some(c) = &self.tracker.rows.cache[lv] {
            return c.clone();
        }
        let (nr, nc) = (self.tracker.rows.a[lv], self.tracker.cols.a[lv]);
        let mut dense = (self.n as f64) <= nc as f64 * 5.0 || (self.m as f64) <= nr as f64 * 4.0;
        if !dense {
            let mut sum = 0i64;
            for &j in &self.tracker.cols.list[lv][..nc] {
                sum += self.col_cnt[j as usize].max(0) as i64;
            }
            self.work += (nc as f64 + nc as f64) * self.work_mult;
            dense = (self.m as f64) < nr as f64 * 3.0 + sum as f64;
        }
        let out: Vec<i32> = if dense {
            self.work += (self.m as f64 + self.m as f64) * self.work_mult;
            (0..self.m as i32).filter(|&i| self.row_cnt[i as usize] >= 0).collect()
        } else {
            let mut l: Vec<i32> = self.tracker.rows.list[lv][..nr].to_vec();
            for &i in &l {
                self.mark[i as usize] = true;
            }
            self.work += self.work_mult * 3.0 * nr as f64;
            for t in 0..nc {
                let j = self.tracker.cols.list[lv][t] as usize;
                let (b, len) = (self.col_beg[j] as usize, self.col_len[j].max(0) as usize);
                for k in b..b + len {
                    let i = self.col_ind[k];
                    if i >= 0 && !self.mark[i as usize] {
                        self.mark[i as usize] = true;
                        l.push(i);
                    }
                }
                self.work += self.work_mult * 3.0 * len as f64;
            }
            self.work += (nc as f64 + nc as f64) * self.work_mult;
            for &i in &l {
                self.mark[i as usize] = false;
            }
            self.work += self.work_mult * 3.0 * l.len() as f64;
            l.retain(|&i| self.row_cnt[i as usize] >= 0);
            l
        };
        self.tracker.rows.cache[lv] = Some(out.clone());
        out
    }

    /// `5399fc`: the column counterpart of `rows_list`.
    pub fn cols_list(&mut self) -> Vec<i32> {
        let lv = self.tracker.level;
        if lv == 0 {
            let q = &self.tracker.cols;
            return q.list[0][..q.a[0]].to_vec();
        }
        if let Some(c) = &self.tracker.cols.cache[lv] {
            return c.clone();
        }
        let (nr, nc) = (self.tracker.rows.a[lv], self.tracker.cols.a[lv]);
        let mut dense = (self.m as f64) <= nr as f64 * 5.0 || (self.n as f64) <= nc as f64 * 4.0;
        if !dense {
            let mut sum = 0i64;
            for &i in &self.tracker.rows.list[lv][..nr] {
                sum += self.row_cnt[i as usize].max(0) as i64;
            }
            self.work += (nr as f64 + nr as f64) * self.work_mult;
            dense = (self.n as f64) < nc as f64 * 3.0 + sum as f64;
        }
        let out: Vec<i32> = if dense {
            self.work += (self.n as f64 + self.n as f64) * self.work_mult;
            (0..self.n as i32).filter(|&j| self.col_cnt[j as usize] >= 0).collect()
        } else {
            let mut l: Vec<i32> = self.tracker.cols.list[lv][..nc].to_vec();
            for &j in &l {
                self.mark[j as usize] = true;
            }
            self.work += self.work_mult * 3.0 * nc as f64;
            for t in 0..nr {
                let i = self.tracker.rows.list[lv][t] as usize;
                let (b, len) = (self.row_beg[i] as usize, self.row_len[i].max(0) as usize);
                for k in b..b + len {
                    let j = self.row_ind[k];
                    if j >= 0 && !self.mark[j as usize] {
                        self.mark[j as usize] = true;
                        l.push(j);
                    }
                }
                self.work += self.work_mult * 3.0 * len as f64;
            }
            self.work += (nr as f64 + nr as f64) * self.work_mult;
            for &j in &l {
                self.mark[j as usize] = false;
            }
            self.work += self.work_mult * 3.0 * l.len() as f64;
            l.retain(|&j| self.col_cnt[j as usize] >= 0);
            l
        };
        self.tracker.cols.cache[lv] = Some(out.clone());
        out
    }

    /// `53af0f` (columns) / `53a487` (rows): the items marked at the tracker's level plus every item sharing a
    /// row (column) with the other kind's list (`538f30`/`5399fc`), live ones only; all live items in index
    /// order when that would be most of them. Cached per level until the next reset.
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
                let i = i as usize;
                let (b, len, ind) = if cols {
                    (self.row_beg[i] as usize, self.row_len[i].max(0) as usize, &self.row_ind)
                } else {
                    (self.col_beg[i] as usize, self.col_len[i].max(0) as usize, &self.col_ind)
                };
                for &j in &ind[b..b + len] {
                    if j >= 0 && !self.mark[j as usize] {
                        self.mark[j as usize] = true;
                        l.push(j);
                    }
                }
                self.work += len as f64 * mult * 3.0;
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

    /// `5369b8` on the state's tracker.
    pub fn reset_tracker(&mut self) {
        let State { tracker, row_cnt, col_cnt, work, work_mult, .. } = self;
        tracker.reset(row_cnt, col_cnt, work, *work_mult);
    }

    /// `53802c`: mark every row of column `j` (column CSR, removed rows included).
    pub fn mark_rows_of_col(&mut self, j: usize) {
        let b = self.col_beg[j] as usize;
        let len = self.col_len[j].max(0) as usize;
        for k in b..b + len {
            let i = self.col_ind[k];
            if i >= 0 {
                self.tracker.mark_row(i);
            }
        }
        self.work += self.work_mult * 3.0 * len as f64;
    }

    /// Type-1 record of column `j` fixed at `v` (the push inlined in `51a144`, `4fe113`, `503985`).
    pub fn push_fix(&mut self, j: usize, v: f64) {
        let (l, u) = (self.lb[j], self.ub[j]);
        let status = if u - l < 1e-10 {
            -4
        } else if l < v {
            if v < u { -3 } else { -2 }
        } else {
            -1
        };
        let mut entries = Vec::new();
        if self.col_cnt[j] > 0 {
            let b = self.col_beg[j] as usize;
            for k in b..b + self.col_len[j] as usize {
                let i = self.col_ind[k];
                if i >= 0 && self.row_cnt[i as usize] >= 0 {
                    entries.push((i, self.col_val[k]));
                }
            }
            self.work += self.col_len[j] as f64 * 6.0;
        }
        self.cols_removed += 1;
        self.post.push(PostRec::FixCol { j: j as i32, v, obj: self.obj[j], status, entries });
    }

    /// `4fa83a`: fix column `j` at `v` and take it out of its rows: record (if `rec`), objective constant, row
    /// marks, rhs and both activities. With `rec` the column is gone (-2); without, it stays as an empty column
    /// fixed at `v` (removed later as an empty column).
    pub fn fix_at(&mut self, j: usize, v: f64, infbound: f64, rec: bool) {
        if rec {
            self.push_fix(j, v);
        }
        self.objcon += self.obj[j] * v;
        self.obj[j] = 0.0;
        self.mark_rows_of_col(j);
        let (l, u) = (self.lb[j], self.ub[j]);
        let b = self.col_beg[j] as usize;
        let mut touched = Vec::new();
        for k in b..b + self.col_len[j].max(0) as usize {
            let i = self.col_ind[k];
            if i < 0 || self.row_cnt[i as usize] < 0 {
                continue;
            }
            let i = i as usize;
            self.col_ind[k] = -1;
            self.row_cnt[i] -= 1;
            let a = self.col_val[k];
            if !rec && !touched.contains(&i) {
                touched.push(i);
            }
            let t = v * a;
            self.rhs[i] -= t;
            self.act_max[i] += t;
            let s = self.act_min[i] + t;
            if a <= 0.0 {
                if u < infbound {
                    self.act_min[i] = s - a * u;
                } else {
                    self.act_min[i] = s;
                    self.inf_min[i] -= 1;
                }
                if -infbound < l {
                    self.act_max[i] -= a * l;
                } else {
                    self.inf_max[i] -= 1;
                }
            } else {
                self.act_min[i] = s;
                if u < infbound {
                    self.act_max[i] -= a * u;
                } else {
                    self.inf_max[i] -= 1;
                }
                if -infbound < l {
                    self.act_min[i] -= a * l;
                } else {
                    self.inf_min[i] -= 1;
                }
            }
        }
        self.work += self.col_len[j].max(0) as f64 * 10.0;
        if rec {
            self.col_len[j] = -1;
            self.col_cnt[j] = -2;
            return;
        }
        self.col_len[j] = 0;
        self.lb[j] = v;
        self.ub[j] = v;
        if self.col_cnt[j] > 0 {
            self.col_cnt[j] = 0;
            self.tracker.mark_col(j as i32);
        }
        // drop the column from the row CSR of the rows it was in (state+0x390 list, +0x3b0 marks)
        for &i in &touched {
            if self.row_cnt[i] >= 0 {
                let rb = self.row_beg[i] as usize;
                for k in rb..rb + self.row_len[i].max(0) as usize {
                    if self.row_ind[k] == j as i32 {
                        self.row_ind[k] = -1;
                    }
                }
                self.work += self.row_len[i] as f64 * 2.0;
            }
        }
        self.work += touched.len() as f64 * 4.0;
    }

    /// `4f6740`: fix a column whose bounds are within tolerance, unless its coefficients make the gap matter.
    /// The value is the bound its cost prefers, or the midpoint (0 if the range contains 0) for a tiny cost.
    /// Rows lose the column and their rhs; activities are not updated here. Returns 3 if lb > ub + 2 feastol.
    pub fn fix_near(&mut self, j: usize, feastol: f64) -> i32 {
        let d = self.ub[j] - self.lb[j];
        if feastol + feastol < -d {
            self.bad_col = j as i32;
            return 3;
        }
        let b = self.col_beg[j] as usize;
        let len = self.col_len[j].max(0) as usize;
        if 1e-13 < d {
            let mut amax = 0.0f64;
            for k in b..b + len {
                let i = self.col_ind[k];
                if i >= 0 && self.row_cnt[i as usize] >= 0 && amax <= self.col_val[k].abs() {
                    amax = self.col_val[k].abs();
                }
            }
            self.work += len as f64 * 3.0;
            if feastol < amax * d {
                return 0;
            }
        }
        let c = self.obj[j];
        let v = if feastol * 0.01 <= (d * c).abs() {
            if c <= 0.0 { self.ub[j] } else { self.lb[j] }
        } else if 0.0 < self.lb[j] || self.ub[j] < 0.0 {
            (self.ub[j] + self.lb[j]) * 0.5
        } else {
            0.0
        };
        self.push_fix(j, v);
        self.objcon += c * v;
        self.obj[j] = 0.0;
        self.mark_rows_of_col(j);
        for k in b..b + len {
            let i = self.col_ind[k];
            if i >= 0 && self.row_cnt[i as usize] >= 0 {
                self.row_cnt[i as usize] -= 1;
                self.rhs[i as usize] -= v * self.col_val[k];
                self.col_ind[k] = -1;
            }
        }
        self.work += len as f64 * 6.0;
        self.col_cnt[j] = -2;
        self.col_len[j] = -1;
        0
    }

    /// `4f9fc2`: bounds on column `j` implied by its rows' activities (rows with at most one infinite
    /// contribution, that one being `j`'s own). Returns (lo, hi), starting from (-infbound, infbound).
    pub fn implied_bounds(&mut self, j: usize, infbound: f64) -> (f64, f64) {
        let (l, u) = (self.lb[j], self.ub[j]);
        let (mut lo, mut hi) = (-infbound, infbound);
        let b = self.col_beg[j] as usize;
        for k in b..b + self.col_len[j].max(0) as usize {
            let i = self.col_ind[k];
            if i < 0 || self.row_cnt[i as usize] < 0 {
                continue;
            }
            let i = i as usize;
            let a = self.col_val[k];
            let eq = self.sense[i] == b'=';
            let min_hi = |t: f64, hi: &mut f64| {
                if t <= *hi {
                    *hi = t;
                }
            };
            let max_lo = |t: f64, lo: &mut f64| {
                if *lo <= t {
                    *lo = t;
                }
            };
            if a > 1e-9 {
                if self.inf_min[i] == 0 {
                    min_hi(l - self.act_min[i] / a, &mut hi);
                } else if self.inf_min[i] == 1 && l <= -infbound {
                    min_hi(-self.act_min[i] / a, &mut hi);
                }
                if eq {
                    if self.inf_max[i] == 0 {
                        max_lo(u - self.act_max[i] / a, &mut lo);
                    } else if self.inf_max[i] == 1 && infbound <= u {
                        max_lo(-self.act_max[i] / a, &mut lo);
                    }
                }
            } else if a < -1e-9 {
                if self.inf_min[i] == 0 {
                    max_lo(u - self.act_min[i] / a, &mut lo);
                } else if self.inf_min[i] == 1 && infbound <= u {
                    max_lo(-self.act_min[i] / a, &mut lo);
                }
                if eq {
                    if self.inf_max[i] == 0 {
                        min_hi(l - self.act_max[i] / a, &mut hi);
                    } else if self.inf_max[i] == 1 && l <= -infbound {
                        min_hi(-self.act_max[i] / a, &mut hi);
                    }
                }
            }
        }
        self.work += self.col_len[j] as f64 * 8.0;
        (lo, hi)
    }

    /// `4fe49b`: move column `j`'s bounds from (ol, ou) to (nl, nu) in the activities of its rows
    /// (remove the old contributions, add the new, with the rounding guard).
    pub fn bound_change(&mut self, j: usize, ol: f64, ou: f64, nl: f64, nu: f64, inf: f64) {
        let lb_changed = nl != ol;
        let ub_changed = nu != ou;
        if !lb_changed && !ub_changed {
            return;
        }
        let b = self.col_beg[j] as usize;
        for k in b..b + self.col_len[j].max(0) as usize {
            let i = self.col_ind[k];
            if i < 0 || self.row_cnt[i as usize] < 0 {
                continue;
            }
            let i = i as usize;
            let a = self.col_val[k];
            let lb_on_max = a <= 0.0; // a <= 0: lb feeds the max activity, ub the min; a > 0 the other way
            if lb_changed && ub_changed {
                self.act_side(i, !lb_on_max, ou, a, -1, ou < inf);
                self.act_side(i, lb_on_max, ol, a, -1, -inf < ol);
                self.act_side(i, !lb_on_max, nu, a, 1, nu < inf);
                self.act_side(i, lb_on_max, nl, a, 1, -inf < nl);
            } else if lb_changed {
                self.act_side(i, lb_on_max, ol, a, -1, -inf < ol);
                self.act_side(i, lb_on_max, nl, a, 1, -inf < nl);
            } else {
                self.act_side(i, !lb_on_max, ou, a, -1, ou < inf);
                self.act_side(i, !lb_on_max, nu, a, 1, nu < inf);
            }
        }
        self.work += self.col_len[j] as f64 * 7.0;
    }

    /// One activity update of `4fe49b`: add (`sign` 1) or remove (-1) the contribution `v * a` on the max
    /// (`on_max`) or min side of row `i`, or adjust the infinite count when `v` is infinite.
    fn act_side(&mut self, i: usize, on_max: bool, v: f64, a: f64, sign: i32, finite: bool) {
        if !finite {
            if on_max {
                self.inf_max[i] += sign;
            } else {
                self.inf_min[i] += sign;
            }
            return;
        }
        let t = if sign < 0 { -(v * a) } else { v * a };
        if on_max {
            self.act_max[i] = guarded_add(self.act_max[i], t, true);
        } else {
            self.act_min[i] = guarded_add(self.act_min[i], t, false);
        }
    }
}
