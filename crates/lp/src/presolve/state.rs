//! LP presolve state (`3a484d` with CSR storage, 12 §4.1). Field comments give the original state offset.
//! Removal is lazy, as in the original: a deleted row-CSR entry gets column index -1, removed rows and columns
//! get a negative live count, and the column CSR keeps entries of removed rows until the model is rebuilt.
use super::tracker::Tracker;
use rsolver_model::Model;

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
    /// +0x170 per-column flags (nonzero = special column, e.g. PWL objective)
    pub col_flag: Vec<i32>,
    /// +0x3f8 / +0x3fc / +0x400 / +0x404 / +0x408
    pub bounds_tightened: i32,
    pub coefs_changed: i32,
    pub ineq_to_eq: i32,
    pub rows_removed: i32,
    pub cols_removed: i32,
    /// +0x470 / +0x478: column / row proving infeasibility or unboundedness
    pub bad_col: i32,
    pub bad_row: i32,
    /// *(+0x458): deterministic work; +0x448 multiplier
    pub work: f64,
    pub work_mult: f64,
    /// +0x3d8
    pub tracker: Tracker,
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
fn guarded_add(old: f64, t: f64, up: bool) -> f64 {
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
        let mut col_beg = Vec::with_capacity(n);
        let (mut col_ind, mut col_val) = (Vec::new(), Vec::new());
        let mut row_len = vec![0i32; m];
        for j in 0..n {
            col_beg.push(col_ind.len() as i64);
            let b = md.col_beg[j] as usize;
            for k in b..b + md.col_len[j] as usize {
                let i = md.col_ind[k];
                col_ind.push(i);
                col_val.push(if neg[i as usize] { -md.col_val[k] } else { md.col_val[k] });
                row_len[i as usize] += 1;
            }
        }
        let col_len = md.col_len.clone();
        let mut row_beg = vec![0i64; m];
        let mut s = 0i64;
        for i in 0..m {
            row_beg[i] = s;
            s += row_len[i] as i64;
        }
        let mut fill = row_beg.clone();
        let mut row_ind = vec![0i32; s as usize];
        let mut row_val = vec![0f64; s as usize];
        for j in 0..n {
            for k in col_beg[j] as usize..col_beg[j] as usize + col_len[j] as usize {
                let i = col_ind[k] as usize;
                row_ind[fill[i] as usize] = j as i32;
                row_val[fill[i] as usize] = col_val[k];
                fill[i] += 1;
            }
        }
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
            nnz: s,
            col_flag: vec![0; n],
            bounds_tightened: 0,
            coefs_changed: 0,
            ineq_to_eq: 0,
            rows_removed: 0,
            cols_removed: 0,
            bad_col: -1,
            bad_row: -1,
            work: 0.0,
            work_mult: 1.0,
            tracker: Tracker::new(m, n),
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
            q.col_len.push((q.col_ind.len() as i64 - q.col_beg.last().unwrap()) as i32);
            q.obj.push(self.obj[j]);
            q.lb.push(self.lb[j]);
            q.ub.push(self.ub[j]);
            q.vtype.push(md.vtype[j]);
            q.var_names.push(md.var_names[j].clone());
        }
        q
    }
}
