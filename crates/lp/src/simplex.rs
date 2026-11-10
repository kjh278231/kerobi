//! 04 §2, §5 Simplex engine on the scaled workspace: the outer driver (`cda5b`), dual simplex (`06268b`/`028c93`)
//! and primal simplex (`0478d3`) sharing one basis, LU and solution state.
//!
//! Conventions: vstat −1 at lower, −2 at upper, −3 free at zero, −4 fixed, ≥ 0 basis position. Infinite bounds are
//! ±1e30 and the dual simplex uses them as real (artificial) bounds, as the original does (m1_notes §3). `d` holds
//! raw reduced costs c_j − a_jᵀy (basic 0); dual feasible means d ≥ 0 at lower, ≤ 0 at upper, = 0 free.
//! shortcut: pricing (DSE max over all rows, not the `15d2d` candidate list), Harris 2-pass instead of BFRT, the
//! perturbation trigger and the refactor schedule are textbook stand-ins; iteration paths differ from Gurobi's.
use crate::lu::{sum_squares, Lu, ZERO};
use crate::bfrt::{bfrt, Cand};
use crate::common::Rng;
use crate::pricing::{Pricing, BIG_M_TOL};
use crate::spx::Spx;
use crate::start::start_basis;
use rsolver_model::INFINITY;

pub const FEAS_TOL: f64 = 1e-6; // spx+0x380
pub const OPT_TOL: f64 = 1e-6; // spx+0x388
const PIVOT_TOL: f64 = 1e-7;
const BIG_TOL: f64 = 3e-6; // spx+0x3a0: allowed relative gap between the row and column pivot

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Optimal,
    Infeasible,
    Unbounded,
    IterLimit,
    Numeric,
}

/// One SIMPLEX callback row: iteration count, objective, primal and dual infeasibility, perturbed flag.
pub type TraceRow = (u64, f64, f64, f64, bool);

pub struct Engine<'a> {
    pub spx: &'a Spx,
    pub m: usize,
    pub n: usize,
    /// Working costs (perturbations land here; `spx.c` stays the original).
    pub c: Vec<f64>,
    pub vstat: Vec<i32>,
    pub head: Vec<usize>,
    pub lu: Lu,
    pub xb: Vec<f64>,
    pub d: Vec<f64>,
    pub w: Vec<f64>,
    y: Vec<f64>,
    /// Row copy (see `rc_build`): per row (column, value, nonzero index), and each nonzero's slot in its row.
    rc: Vec<Vec<(usize, f64, usize)>>,
    /// Row-wise copy of the structural columns (column, value), rows ascending; the d recomputation of `a4268`
    /// accumulates row by row.
    arow_start: Vec<usize>,
    arow: Vec<(usize, f64)>,
    rc_pos: Vec<usize>,
    rc_on: bool,
    pub iter: u64,
    pub iter_limit: u64,
    deadline: Option<std::time::Instant>,
    /// Record the per-iteration trace (objective and infeasibility sums, O(n+m) each); off when nobody reads it.
    pub trace_on: bool,
    lu_interval: usize,
    perturbed: bool,
    /// Perturbation size spx+0x348 (PerturbValue 2e-4 at the start).
    pert_size: f64,
    /// 48d5f trigger state: level 0x338 (2 shifted, 3 perturbed), refactor count 0x2c0, degenerate pivots 0x330 and
    /// the marks 0x334/0x2b0, objective record 0x140, ratio 0.9 or 0.97 (min |c| ≥ 1e-6, spx+0x3e0).
    pt_lvl: u32,
    pt_rc: u64,
    pt_deg: u64,
    pt_deg0: u64,
    pt_it0: u64,
    pt_rec: f64,
    pt_ratio: f64,
    /// COSTSCALE factor 0x350 applied to the working costs (spx+0xd8 grows with it).
    cost_scale: f64,
    /// Primal bound shift s (P+0x30) added to b by row; empty when there is none.
    rhs_shift: Vec<f64>,
    /// The primal may hand a shifted optimum over to the dual (PRIMDUALSWITCH, round budget left).
    primal_switch: bool,
    markowitz_tol: f64,
    in_dual: bool,
    /// Columns refused as entering after a pivot mismatch on a fresh factorization; cleared after a pivot.
    rejected: Vec<bool>,
    pub trace: Vec<TraceRow>,
    /// Dual pricing state (P = spx+0x478) and the artificial-bound phase flag spx+0x2cc.
    pub pr: Pricing,
    pub big_m: bool,
    /// Refactor bookkeeping: (nnz L, nnz U) of the last factor, Σ nnz of the update columns, and the current
    /// entering column's nonzeros (set by the iteration before `pivot`).
    fac_nnz: (usize, usize),
    upd_cols: usize,
    col_nnz: usize,
    /// max|b − N·x_N| of the last x_B computation (spx+0x2c8 test of `1bae9c` L1691–1743).
    rhs_max: f64,
    /// Last dual pivot for snapshots: τ = B⁻¹ρ by position, α_r, β = ‖ρ‖².
    last_dse: (Vec<f64>, f64, f64),
    /// Whether the last pivot's BTRAN output was dense (snapshot `bt_dense`, tools/m1_beta_check.py).
    bt_dense: bool,
    last_aq_row: f64,
    /// Row-wise U (by position, storage order) and prefix t at the last DSE τ, as snapshot JSON.
    last_urow: String,
    /// `RSOLVER_SNAP` / `RSOLVER_BFRT` dump files, read once.
    snap_path: Option<String>,
    bfrt_path: Option<String>,
    /// spx+0x2c8: CHUZR tests violations with the infinite-bound guard (artificial-bound phase or x_B past 1e20).
    c8: bool,
    /// Pivot path: (iteration, entering column, leaving column); bound flips of the entering column leave −1.
    pub path: Vec<(u64, i64, i64)>,
}

impl<'a> Engine<'a> {
    pub fn new(spx: &'a Spx) -> Engine<'a> {
        let (m, n) = (spx.m, spx.n);
        // spx+0x3c8 = max(20, ⌊m^0.7⌋); the switch to Full pricing lowers it to max(20, ⌊m^0.65⌋)
        let interval = ((m as f64).powf(0.7) as usize).max(20);
        Engine {
            spx,
            m,
            n,
            c: spx.c.clone(),
            vstat: Vec::new(),
            head: Vec::new(),
            lu: Lu::factor(spx, &(n..n + m).collect::<Vec<_>>()),
            xb: vec![0.0; m],
            d: vec![0.0; n + m],
            w: vec![1.0; m],
            y: vec![0.0; m],
            rc: Vec::new(),
            arow_start: Vec::new(),
            arow: Vec::new(),
            rc_pos: Vec::new(),
            rc_on: false,
            iter: 0,
            iter_limit: u64::MAX,
            deadline: None,
            trace_on: true,
            lu_interval: interval,
            perturbed: false,
            pert_size: 2e-4,
            pt_lvl: 0,
            pt_rc: 0,
            pt_deg: 0,
            pt_deg0: 0,
            pt_it0: 0,
            pt_rec: -1e100,
            cost_scale: 1.0,
            pt_ratio: if spx.c[..spx.n].iter().filter(|&&c| c != 0.0).fold(f64::INFINITY, |a, &c| a.min(c.abs())) >= 1e-6 {
                0.9
            } else {
                0.97
            },
            rhs_shift: Vec::new(),
            primal_switch: true,
            markowitz_tol: crate::lu::MARKOWITZ_TOL,
            in_dual: false,
            rejected: vec![false; n + m],
            trace: Vec::new(),
            pr: Pricing::new(m, n, &spx.len),
            big_m: false,
            fac_nnz: (m, m),
            upd_cols: 0,
            col_nnz: 0,
            rhs_max: 0.0,
            last_dse: (Vec::new(), 0.0, 0.0),
            bt_dense: false,
            last_aq_row: 0.0,
            last_urow: String::new(),
            snap_path: std::env::var("RSOLVER_SNAP").ok(),
            bfrt_path: std::env::var("RSOLVER_BFRT").ok(),
            c8: false,
            path: Vec::new(),
        }
    }

    pub fn x_nonbasic(&self, j: usize) -> f64 {
        match self.vstat[j] {
            -1 | -4 => self.spx.l[j],
            -2 => self.spx.u[j],
            _ => 0.0,
        }
    }

    fn set_basis(&mut self, vstat: Vec<i32>) {
        self.head = vec![usize::MAX; self.m];
        for (j, &s) in vstat.iter().enumerate() {
            if s >= 0 {
                self.head[s as usize] = j;
            }
        }
        self.vstat = vstat;
    }

    /// Factor the current basis; a singular basis gets slacks of the unpivoted rows in its unpivoted positions.
    fn refactor(&mut self) {
        self.upd_cols = 0;
        loop {
            self.lu = Lu::factor_tol(self.spx, &self.head, self.markowitz_tol);
            self.fac_nnz = self.lu.factor_nnz();
            if self.lu.complete() {
                {
                    // renumber positions as the original does after every factorization; position-indexed
                    // state (DSE weights, pricing) moves along
                    let old_of_new = self.lu.renumbered();
                    self.lu.relabel(&old_of_new);
                    self.head = self.lu.head.clone();
                    for (p, &j) in self.head.iter().enumerate() {
                        self.vstat[j] = p as i32;
                    }
                    let perm = |v: &[f64]| old_of_new.iter().map(|&op| v[op]).collect::<Vec<f64>>();
                    self.w = perm(&self.w);
                    self.pr.score = perm(&self.pr.score);
                    self.pr.mark = old_of_new.iter().map(|&op| self.pr.mark[op]).collect();
                    let mut new_of_old = vec![0; self.m];
                    for (np, &op) in old_of_new.iter().enumerate() {
                        new_of_old[op] = np;
                    }
                    self.pr.list = self.pr.list.iter().map(|&op| new_of_old[op]).collect();
                }
                break;
            }
            // 5b257 L1517–1600: each singular tail position takes the slack of its paired row
            for &(p, i) in &self.lu.singular.clone() {
                let out = self.head[p];
                self.vstat[out] = self.bound_status(out, self.xb.get(p).copied().unwrap_or(0.0));
                if self.rc_on && out < self.n && self.vstat[out] != -4 {
                    self.rc_add(out);
                }
                self.head[p] = self.n + i;
                self.vstat[self.n + i] = p as i32;
            }
        }
    }

    /// Refactor and recompute x_B and d; in the dual, boxed nonbasics that turned dual infeasible flip (048d5f).
    fn reinvert(&mut self) {
        self.reinvert_with(false);
    }

    /// Refactor, then recompute x_B and d. In the dual, `48d5f` absorbs dual infeasibilities by shifting costs after
    /// an ordinary refactor (SHIFT) and by moving boxed columns to the other bound after reason 4 (FLIP, `flip`).
    fn reinvert_with(&mut self, flip: bool) {
        self.refactor();
        self.compute_xb();
        self.compute_duals();
        if self.in_dual {
            if flip {
                if self.make_dual_feasible() {
                    self.compute_xb();
                }
            } else {
                self.shift_costs();
            }
            self.pt_rc += 1;
            self.perturb_trigger();
            self.price_full();
        }
    }

    /// TRIGGER of `48d5f` (m1_notes §8.9) after a refactor: every K-th factorization (K = 2, 5 once perturbed), perturb
    /// when more than 90% (97%) of the pivots since the last mark were degenerate or the objective has not moved past
    /// its record; then mark.
    /// shortcut: flag 0x110 taken as 1, the artificial-bound count test and COSTSCALE are left out.
    fn perturb_trigger(&mut self) {
        let k = if self.pt_lvl <= 2 { 2 } else { 5 };
        if self.pt_rc % k != 1 || self.pert_size <= 0.0 {
            return;
        }
        // spx+0x118: objective under the working costs
        let x = self.x();
        let obj: f64 = (0..self.n + self.m).map(|j| self.c[j] * x[j]).sum();
        let degenerate = (self.pt_deg - self.pt_deg0) as f64 > self.pt_ratio * (self.iter - self.pt_it0) as f64;
        if degenerate || ZERO * (obj.abs().min(1e10) + 1.0) + self.pt_rec > obj {
            self.perturb();
            self.pt_rec = -1e100;
        } else {
            self.pt_rec = self.pt_rec.max(obj);
        }
        self.pt_deg0 = self.pt_deg;
        self.pt_it0 = self.iter;
    }

    /// SHIFT of `48d5f` (m1_notes §8.9): a dual-infeasible nonbasic gets its working cost moved so that d = 0.
    fn shift_costs(&mut self) {
        for j in 0..self.n + self.m {
            let d = self.d[j];
            let shift = match self.vstat[j] {
                -1 if d < -OPT_TOL => d,
                -2 if d < -OPT_TOL => -d,
                -3 if d.abs() > OPT_TOL => d,
                _ => continue,
            };
            self.c[j] -= shift; // raw reduced cost −shift·(sign) becomes 0
            self.d[j] = 0.0;
            self.perturbed = true;
            self.pt_lvl = self.pt_lvl.max(2);
        }
    }

    /// Nonbasic status nearest to x.
    fn bound_status(&self, j: usize, x: f64) -> i32 {
        let (l, u) = (self.spx.l[j], self.spx.u[j]);
        if u - l <= 0.0 {
            -4
        } else if l <= -INFINITY && u >= INFINITY {
            -3
        } else if l <= -INFINITY || (u < INFINITY && (x - u).abs() < (x - l).abs()) {
            -2
        } else {
            -1
        }
    }

    pub fn compute_xb(&mut self) {
        let mut rhs = self.spx.b.clone();
        for (r, s) in rhs.iter_mut().zip(&self.rhs_shift) {
            *r += s;
        }
        for j in 0..self.n + self.m {
            if self.vstat[j] < 0 {
                let xj = self.x_nonbasic(j);
                if xj != 0.0 {
                    for (i, a) in self.spx.col(j) {
                        rhs[i] -= a * xj;
                    }
                }
            }
        }
        self.rhs_max = rhs.iter().fold(0.0, |a: f64, v| a.max(v.abs()));
        self.xb = self.lu.ftran_dense(rhs);
    }

    /// y = c_Bᵀ B⁻¹ (dense BTRAN, as for the iteration-0 d) and d = c − Aᵀy.
    pub fn compute_duals(&mut self) {
        self.compute_duals_only();
        let (n, m) = (self.n, self.m);
        if self.arow_start.is_empty() {
            let mut start = vec![0usize; m + 1];
            for &i in &self.spx.ind {
                start[i as usize + 1] += 1;
            }
            for i in 0..m {
                start[i + 1] += start[i];
            }
            let mut fill = start.clone();
            let mut arow = vec![(0usize, 0.0f64); start[m]];
            for j in 0..n {
                for (i, a) in self.spx.col(j) {
                    arow[fill[i]] = (j, a);
                    fill[i] += 1;
                }
            }
            self.arow_start = start;
            self.arow = arow;
        }
        // a4268: d = c over the nonbasics, then row by row (ascending, rows with |y_i| ≤ ZERO skipped) d_j −= y_i·a_ij
        // for the nonbasic non-fixed structurals and the slack; fixed structurals keep c_j; upper-bounded ones are
        // negated at the end (the original adds to −c with the row copy's flipped sign, the same bits)
        for j in 0..n + m {
            self.d[j] = if self.vstat[j] >= 0 { 0.0 } else { self.c[j] };
        }
        for i in 0..m {
            let yi = self.y[i];
            if yi.abs() <= ZERO {
                continue;
            }
            if self.vstat[n + i] < 0 {
                self.d[n + i] -= yi;
            }
            for &(j, a) in &self.arow[self.arow_start[i]..self.arow_start[i + 1]] {
                if self.vstat[j] < 0 && self.vstat[j] != -4 {
                    self.d[j] -= a * yi;
                }
            }
        }
        for j in 0..n + m {
            if self.vstat[j] == -2 {
                self.d[j] = -self.d[j];
            }
        }
    }

    fn exact_weights(&mut self) {
        self.w = (0..self.m).map(|p| sum_squares(&self.lu.btran(&[(p, 1.0)]).iter().map(|e| e.1).collect::<Vec<_>>())).collect();
    }

    /// Row-wise copy of the nonbasic structural columns (spx+0x1d0, built by `a075e` once at the end of the dual
    /// start in column order). A column leaving the basis is appended to its rows (`a104a`); an entering column is
    /// removed by moving each row's last entry into its slot (`a114f`). The entry order is the α-vector order.
    /// shortcut: the equal-magnitude row partition of `9edd1` (rows with ≥ 128 entries of one |a|) is not kept.
    fn rc_build(&mut self) {
        self.rc = vec![Vec::new(); self.m];
        self.rc_pos = vec![0; self.spx.ind.len()];
        for j in 0..self.n {
            if matches!(self.vstat[j], -3..=-1) {
                self.rc_add(j);
            }
        }
        self.rc_on = true;
    }

    fn rc_add(&mut self, j: usize) {
        let b = self.spx.beg[j] as usize;
        for k in b..b + self.spx.len[j] as usize {
            let i = self.spx.ind[k] as usize;
            self.rc_pos[k] = self.rc[i].len();
            self.rc[i].push((j, self.spx.val[k], k));
        }
    }

    fn rc_remove(&mut self, j: usize) {
        let b = self.spx.beg[j] as usize;
        for k in b..b + self.spx.len[j] as usize {
            let i = self.spx.ind[k] as usize;
            let p = self.rc_pos[k];
            let last = self.rc[i].pop().expect("row copy entry");
            if p < self.rc[i].len() {
                self.rc[i][p] = last;
                self.rc_pos[last.2] = p;
            }
        }
    }

    /// Pivot row α = ρᵀA_N laid out as `0a4109` does: the structural part scattered over the row copy with ρ's rows
    /// in BTRAN output order, kept where |α| > ZERO, in ascending columns when n < 1.8·Σ row counts (dense
    /// scatter) and in first-touch order otherwise (one ρ row: row order); then the slacks of ρ's rows in ρ order.
    /// Returns the dense α (0 off the vector) and the vector order.
    fn pivot_row(&self, rho: &[(usize, f64)]) -> (Vec<f64>, Vec<usize>) {
        let mut alpha = vec![0.0; self.n + self.m];
        let mut touched = vec![false; self.n];
        let mut order = Vec::new();
        let work: usize = rho.iter().map(|&(i, _)| self.rc[i].len()).sum();
        if rho.len() > 1 && (self.n as f64) < work as f64 * 1.8 {
            // dense accumulation, output in column order
            for &(i, r) in rho {
                for &(j, a, _) in &self.rc[i] {
                    touched[j] = true;
                    alpha[j] += r * a;
                }
            }
            order.extend((0..self.n).filter(|&j| touched[j]));
        } else {
            for &(i, r) in rho {
                for &(j, a, _) in &self.rc[i] {
                    if !touched[j] {
                        touched[j] = true;
                        order.push(j);
                    }
                    alpha[j] += r * a;
                }
            }
        }
        order.retain(|&j| {
            let keep = alpha[j].abs() > ZERO;
            if !keep {
                alpha[j] = 0.0;
            }
            keep
        });
        for &(i, r) in rho {
            alpha[self.n + i] = r;
            order.push(self.n + i);
        }
        (alpha, order)
    }

    /// FTRAN of column j: (B⁻¹a_j by position, spike by row).
    fn ftran_col(&self, j: usize) -> (Vec<f64>, Vec<f64>) {
        let mut x = vec![0.0; self.m];
        for (i, a) in self.spx.col(j) {
            x[i] = a;
        }
        self.lu.ftran_lower(&mut x);
        let spike = x.clone();
        self.lu.ftran_upper(&mut x);
        let mut col = vec![0.0; self.m];
        for &(r, pos, _) in &self.lu.steps {
            col[pos] = x[r];
        }
        (col, spike)
    }

    /// Replace basis position r by column q (status of the leaving variable given), update the LU or refactor.
    fn pivot(&mut self, r: usize, q: usize, leave_status: i32, spike: &[f64]) {
        let out = self.head[r];
        self.path.push((self.iter, q as i64, out as i64));
        self.vstat[out] = leave_status;
        if leave_status == -2 {
            self.d[out] = -self.d[out];
        }
        self.vstat[q] = r as i32;
        self.head[r] = q;
        // 28c93 L845-880: the leaving structural joins the row copy unless fixed, then the entering one leaves it
        if self.rc_on {
            if out < self.n && leave_status != -4 {
                self.rc_add(out);
            }
            if q < self.n {
                self.rc_remove(q);
            }
        }
        // 6268b LAB_001f6eb5: decided before this pivot's update with k updates done; refactor at the interval
        // (reason 2) or when the update file costs more than a fresh factor (reason 10)
        let k = self.lu.n_updates;
        let m = self.m as f64;
        let c46 = self.c46();
        let refactor = k >= self.lu_interval || {
            let (ln, un) = self.fac_nnz;
            let t = (ln + un) as f64 - 1.5 * m;
            k >= c46 && {
                let f = (t * t).min(c46 as f64 * (4.0 * m * m));
                (self.upd_cols as f64 * 1.5 + self.lu.eta_nnz() as f64 * 2.0) * m - f > 0.0
            }
        };
        if refactor || !self.lu.update(r, q, spike) {
            self.reinvert();
        } else {
            // LU+0x190: nonzeros of the FTRAN'd entering column (pivot included)
            self.upd_cols += self.col_nnz;
        }
    }

    /// spx+0x46c = ⌊m^0.45⌋ (2f7e1 L882).
    fn c46(&self) -> usize {
        (self.m as f64).powf(0.45) as usize
    }

    fn primal_infeas(&self, p: usize) -> f64 {
        let j = self.head[p];
        let x = self.xb[p];
        if x < self.spx.l[j] - FEAS_TOL {
            x - self.spx.l[j]
        } else if x > self.spx.u[j] + FEAS_TOL {
            x - self.spx.u[j]
        } else {
            0.0
        }
    }

    pub fn objective(&self) -> f64 {
        let mut z = 0.0;
        for j in 0..self.n + self.m {
            let x = if self.vstat[j] >= 0 { self.xb[self.vstat[j] as usize] } else { self.x_nonbasic(j) };
            z += self.spx.c[j] * x;
        }
        z
    }

    /// Reduced cost c_j − a_jᵀy of a nonbasic column; `d` stores it negated at the upper bound (spx+0x1e0).
    fn raw_d(&self, j: usize) -> f64 {
        if self.vstat[j] == -2 {
            -self.d[j]
        } else {
            self.d[j]
        }
    }

    fn sums(&self) -> (f64, f64) {
        let pinf: f64 = (0..self.m).map(|p| self.primal_infeas(p).abs()).sum();
        let dinf: f64 = (0..self.n + self.m)
            .map(|j| match self.vstat[j] {
                -1 | -2 => (-self.d[j]).max(0.0),
                -3 => self.d[j].abs(),
                _ => 0.0,
            })
            .sum();
        (pinf, dinf)
    }

    /// `RSOLVER_SNAP=<file>`: one JSON line per iteration at the SIMPLEX-callback point (vstat, x_B, normalized d,
    /// DSE weights as hex bits, pricing state), for bit comparison with tools/spx_oracle.py.
    fn snap(&self) {
        let Some(path) = &self.snap_path else { return };
        use std::io::Write;
        let hex = |v: &[f64]| v.iter().map(|x| format!("\"{}\"", hexf(*x))).collect::<Vec<_>>().join(",");
        let dn: Vec<f64> = (0..self.n + self.m)
            .map(|j| self.d[j])
            .collect();
        let line = format!(
            "{{\"iter\":{},\"vstat\":{:?},\"xB\":[{}],\"d\":[{}],\"w\":[{}],\"mode\":{},\"list\":{:?},\"cursor\":{},\"thr\":\"{}\",\"allow\":{},\"k\":{},\"avg\":{},\"score\":[{}],\"cost_mode\":{},\"row_cost\":[{}],\"var_cost\":[{}],\"ns\":{},\"eta\":{},\"cols\":{},\"tau\":[{}],\"ar\":\"{}\",\"beta\":\"{}\",{}\"bt_dense\":{},\"aqrow\":\"{}\",\"pt\":{{\"rc\":{},\"deg\":{},\"deg0\":{},\"it0\":{}}}}}\n",
            self.iter, self.vstat, hex(&self.xb), hex(&dn), hex(&self.w), self.pr.mode, self.pr.list, self.pr.cursor, hexf(self.pr.thr),
            self.pr.allow, self.lu.n_updates, self.pr.avg, hex(&self.pr.score), self.pr.cost_mode, hex(&self.pr.row_cost),
            hex(&self.pr.var_cost), self.lu.n_singleton, self.lu.eta_nnz(), self.upd_cols,
            hex(&self.last_dse.0), hexf(self.last_dse.1), hexf(self.last_dse.2), self.last_urow, self.bt_dense, hexf(self.last_aq_row), self.pt_rc, self.pt_deg, self.pt_deg0,
            self.pt_it0
        );
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = f.write_all(line.as_bytes());
        }
    }

    fn log(&mut self) {
        if !self.trace_on {
            return;
        }
        let (p, d) = self.sums();
        let z = self.objective();
        self.trace.push((self.iter, z, p, d, self.perturbed));
    }

    /// CHUZR violation of position p: a finite bound missed by more than the tolerance (1e19 in the
    /// artificial-bound phase).
    fn viol(&self, p: usize) -> Option<f64> {
        let j = self.head[p];
        let tol = if self.big_m { BIG_M_TOL } else { FEAS_TOL };
        let (l, u, x) = (self.spx.l[j], self.spx.u[j], self.xb[p]);
        if l - x > tol && l > -INFINITY {
            Some(l - x)
        } else if x - u > tol && u < INFINITY {
            Some(x - u)
        } else {
            None
        }
    }

    /// Mode-1 violation tests of `1bae9c`/`1c8477` (forms VB/VC/VD of the original, which differ in rounding at the
    /// boundary): `full` for the full-path list (VD, or VC when c8), else an updated element (VB, or VC when c8).
    /// The violation is x − u when l ≤ x, else l − x.
    fn viol1(&self, p: usize, full: bool) -> Option<f64> {
        let j = self.head[p];
        let tol = if self.big_m { BIG_M_TOL } else { FEAS_TOL };
        let (l, u, x) = (self.spx.l[j], self.spx.u[j], self.xb[p]);
        let hit = if self.c8 {
            (-INFINITY < l && x < l - tol) || (u < INFINITY && u + tol < x)
        } else if full {
            x < l - tol || u + tol < x
        } else {
            tol < l - x || tol < x - u
        };
        hit.then_some(if l <= x { x - u } else { l - x })
    }

    fn price_score(&self, p: usize, v: f64) -> f64 {
        let j = self.head[p];
        self.spx.var_factor(j) * self.pr.score(p, j, v, self.w[p])
    }

    fn price_rebuild(&mut self) {
        let viol: Vec<Option<f64>> = (0..self.m).map(|p| self.viol(p)).collect();
        let score: Vec<f64> =
            (0..self.m).map(|p| viol[p].map_or(0.0, |v| self.price_score(p, v))).collect();
        self.pr.allow = self.lu_interval as i64;
        self.pr.rebuild(&viol, &score);
    }

    /// Full path of `1bae9c` after a factorization: BTRAN-cost weights from the new LU (`9e60b`), the switch to
    /// Full pricing (m < 15·P+0x48 or m < iterations) and a fresh candidate list.
    fn price_full(&mut self) {
        if self.pr.cost_mode >= 1 {
            let (c, ok) = self.lu.row_cost_estimate();
            for (k, &(_, pos, _)) in self.lu.steps.iter().enumerate() {
                self.pr.row_cost[pos] = c[k];
            }
            if !ok {
                self.pr.cost_mode = 2;
            }
        }
        let xmax = self.xb.iter().fold(0.0, |a: f64, v| a.max(v.abs()));
        self.c8 = self.big_m || self.rhs_max + xmax > 1e20;
        if self.pr.mode == 0 && ((self.m as f64) < 15.0 * self.pr.avg || (self.m as u64) < self.iter) {
            self.pr.mode = 1;
            self.lu_interval = self.lu_interval.min(((self.m as f64).powf(0.65) as usize).max(20));
        }
        if self.pr.mode == 0 {
            // after a factorization the list is built from scratch: no old entries kept, budget P+0x38 back to 40
            self.pr.mark.iter_mut().for_each(|k| *k = 1);
            self.pr.budget = 40;
            self.pr.cursor = 0;
            self.price_rebuild();
        } else {
            let viol: Vec<Option<f64>> = (0..self.m).map(|p| self.viol1(p, true)).collect();
            let score: Vec<f64> =
                (0..self.m).map(|p| viol[p].map_or(0.0, |v| self.price_score(p, v))).collect();
            // divisor 1000 when the violations are at least nnz(α_q) of the last pivot (or none yet) and not c8
            let few = self.path.is_empty() || viol.iter().flatten().count() >= self.col_nnz.saturating_sub(1);
            self.pr.full_init(&viol, &score, if few && !self.c8 { 1000.0 } else { 100000.0 });
        }
    }

    /// CHUZR (`2896a3`): mode 0 walks the ordered list and rebuilds it when used up (P+0x38 rebuilds, then a
    /// refactor); mode 1 takes the best score. None: no violated row.
    fn chuzr(&mut self) -> Option<usize> {
        if self.pr.mode == 1 {
            return self.pr.full_pick();
        }
        if let Some(p) = self.pr.next_listed() {
            return Some(p);
        }
        self.pr.budget -= 1;
        if self.pr.budget < 0 {
            // reason 3: refactor; with fewer than spx+0x46c updates 2896a3 adds m to P+0x48, which makes the full path
            // switch to mode 1
            if self.lu.n_updates < self.c46() {
                self.pr.avg += self.m as f64;
            }
            self.reinvert();
            return self.chuzr();
        }
        self.price_rebuild();
        if self.pr.list.is_empty() {
            return None;
        }
        self.pr.cursor = 1;
        Some(self.pr.list[0])
    }

    /// Mark positions whose x_B changed (mode 0) and rebuild the list once the allowance runs out.
    /// `rebuild`: the allowance check of `1bae9c` (after the pivot); the flip pass of `1c8477` only marks.
    fn price_touch(&mut self, changed: &[usize], leaving: Option<usize>, rebuild: bool) {
        if self.pr.mode != 0 {
            for &p in changed {
                let v = if Some(p) == leaving { self.viol(p) } else { self.viol1(p, leaving.is_none()) };
                let sc = v.map(|v| self.price_score(p, v));
                if Some(p) == leaving {
                    // 1bae9c L481–555: the leaving position (test VA) is listed again with a fresh score, or marked
                    // listed-feasible (3)
                    match sc {
                        Some(x) => {
                            self.pr.score[p] = x;
                            self.pr.mark[p] = 1;
                        }
                        None => self.pr.mark[p] = 3,
                    }
                } else {
                    self.pr.full_touch(p, sc);
                }
            }
            return;
        }
        for &p in changed {
            let inf = self.viol(p).is_some();
            self.pr.touch(p, inf);
        }
        if !rebuild {
            return;
        }
        if self.pr.allow < 0 {
            self.pr.cursor = 0;
            self.pr.budget -= 1;
            if self.pr.budget < 0 {
                if self.lu.n_updates < self.c46() {
                    self.pr.avg += self.m as f64; // 1bae9c L787
                }
                self.reinvert();
            } else {
                self.price_rebuild();
            }
        }
    }

    /// `118725` at the end of the artificial-bound phase: a nonbasic on an artificial bound with a strictly
    /// favourable reduced cost means unbounded; otherwise all of them move to their finite bound.
    fn end_big_m(&mut self) -> Option<Status> {
        let favourable = |e: &Self, j: usize| {
            let art_up = e.vstat[j] == -2 && e.spx.u[j] >= INFINITY;
            let art_lo = j < e.n && e.vstat[j] == -1 && e.spx.l[j] <= -INFINITY;
            (art_up || art_lo) && e.d[j] > OPT_TOL
        };
        let keep: Vec<bool> = (0..self.n + self.m).map(|j| favourable(self, j)).collect();
        // shortcut: the original stops with status 5 here; rows the 1e19 tolerance let through can make that a false
        // alarm on paths that already left Gurobi's, so columns with a favourable d stay on their artificial bound and
        // the dual goes on at the normal tolerance (the driver's primal check settles a real unboundedness)
        for j in 0..self.n + self.m {
            if keep[j] {
                continue;
            }
            if self.vstat[j] == -2 && self.spx.u[j] >= INFINITY {
                self.vstat[j] = -1;
                self.d[j] = -self.d[j];
            } else if j < self.n && self.vstat[j] == -1 && self.spx.l[j] <= -INFINITY {
                self.vstat[j] = -2;
                self.d[j] = -self.d[j];
            }
        }
        if std::env::var("RSOLVER_DEBUG").is_ok() && keep.iter().any(|&k| k) {
            eprintln!("end_big_m: {} favourable columns stay on artificial bounds at it {}", keep.iter().filter(|&&k| k).count(), self.iter);
        }
        self.big_m = false;
        self.reinvert_with(true);
        None
    }

    /// Dual start: crash basis, free columns pivoted in, first factorization, x_B, d, exact DSE norms.
    pub fn dual_start(&mut self) {
        self.set_basis(start_basis(self.spx));
        self.refactor();
        // free-column loop (0534f7 L700-1300): in column order each free nonbasic enters at the first position of
        // largest |α| whose basic has a finite lower bound; every pivot is an iteration. Columns left out (max |α| <
        // PIVOTTOL) get a second pass after a refactor: those with |d| > OptTol enter with PIVOTTOL/100, one at a time
        // with fresh duals.
        let free: Vec<usize> = (0..self.n).filter(|&q| self.vstat[q] == -3).collect();
        let pivots = self.iter;
        let mut factors = 1; // spx+0x2c0 counts every factorization of the start
        let mut left = Vec::new();
        for q in free {
            if !self.free_pivot(q, 3e-8) {
                left.push(q);
            }
        }
        if !left.is_empty() {
            self.refactor();
            factors += 1;
            for q in left {
                self.compute_duals_only();
                let t = self.spx.col(q).fold(self.c[q], |t, (i, a)| t - a * self.y[i]);
                // shortcut: a column that cannot enter is left free here; the original stops with status 5
                if t.abs() > OPT_TOL {
                    self.free_pivot(q, 3e-10);
                }
            }
        }
        if self.iter > pivots {
            self.refactor(); // spx+0x294 = 1: the dual starts from a fresh factorization
            factors += 1;
        }
        self.compute_xb();
        self.compute_duals();
        self.make_dual_feasible();
        self.compute_xb();
        self.exact_weights();
        self.big_m = (0..self.n + self.m).any(|j| self.vstat[j] < 0 && self.x_nonbasic(j).abs() >= INFINITY);
        self.rc_build();
        self.pt_rc = factors;
        self.in_dual = true;
        self.price_full();
    }

    /// One free-column pivot of the dual start; false when no position has |α| ≥ tol.
    fn free_pivot(&mut self, q: usize, tol: f64) -> bool {
        let (col, spike) = self.ftran_col(q);
        let mut best = (usize::MAX, -1.0);
        for p in 0..self.m {
            if self.spx.l[self.head[p]] > -INFINITY && col[p].abs() > best.1 {
                best = (p, col[p].abs());
            }
        }
        if best.1 < tol {
            return false;
        }
        let p = best.0;
        let out = self.head[p];
        let st = if self.spx.u[out] - self.spx.l[out] <= 0.0 { -4 } else { -1 };
        self.col_nnz = col.iter().filter(|&&a| a.abs() > ZERO).count();
        self.iter += 1;
        self.pivot(p, q, st, &spike);
        self.log();
        true
    }

    /// y = c_Bᵀ B⁻¹ (dense BTRAN) into `self.y`.
    fn compute_duals_only(&mut self) {
        let cb: Vec<(usize, f64)> = (0..self.m).map(|p| (p, self.c[self.head[p]])).collect();
        self.y = vec![0.0; self.m];
        for (r, v) in self.lu.btran(&cb) {
            self.y[r] = v;
        }
    }

    /// Put each boxed nonbasic on the bound its reduced cost asks for (048d5f L2025-2048).
    fn make_dual_feasible(&mut self) -> bool {
        let mut flipped = false;
        for j in 0..self.n + self.m {
            match self.vstat[j] {
                -1 | -2 if self.d[j] < -OPT_TOL && self.spx.u[j] - self.spx.l[j] < INFINITY => {
                    self.vstat[j] = -3 - self.vstat[j];
                    self.d[j] = -self.d[j];
                    flipped = true;
                }
                _ => {}
            }
        }
        flipped
    }

    /// GLOBAL cost perturbation of `48d5f` (m1_notes §8.9): the stream is reseeded with the iteration count (Seed
    /// offset 0) and every nonbasic draws one number; δ = ((k % 250 + 1)·(size/500))·log2(w + 1) with w the column
    /// count (slack 1) moves the cost away from its bound; a column on an artificial upper bound with d ≤ OptTol
    /// moves to its lower bound. The size then triples up to 0.01.
    /// shortcut: the trigger is still "50 degenerate pivots in a row" instead of the original's counters.
    fn perturb(&mut self) {
        if self.pt_lvl <= 1 && self.spx.scaling.obj * self.cost_scale >= 1.0 {
            // COSTSCALE (48d5f L1503-1957): s = 128 when the largest dual infeasibility is below OptTol/128, 64 when
            // it reaches OptTol, else 1; working costs and d are multiplied by s
            let mi = (0..self.n + self.m).fold(0.0f64, |mi, j| match self.vstat[j] {
                -1 | -2 if self.d[j] < -mi => -self.d[j],
                -3 => mi.max(self.d[j].abs()),
                _ => mi,
            });
            let s = if OPT_TOL > 128.0 * mi {
                128.0
            } else {
                let mut s = 1.0;
                loop {
                    if OPT_TOL > mi * s {
                        break;
                    }
                    s += s;
                    if s >= 128.0 {
                        break;
                    }
                }
                if s > 2.0 { 0.5 * s } else { 1.0 }
            };
            if s > 1.0 {
                self.c.iter_mut().for_each(|c| *c *= s);
                self.d.iter_mut().for_each(|d| *d *= s);
                self.cost_scale *= s;
            }
        }
        let mut rng = Rng(self.iter);
        let pp = self.pert_size / 500.0;
        let mut moved = false;
        for j in 0..self.n + self.m {
            if self.vstat[j] >= 0 {
                continue;
            }
            let k = rng.next_int();
            let w = if j < self.n { self.spx.len[j] as f64 * (self.spx.var_factor(j) * 0.3 + 0.7) } else { 1.0 };
            let delta = ((k % 250 + 1) as f64 * pp) * (w + 1.0).log2();
            match self.vstat[j] {
                -1 => {
                    self.c[j] += delta;
                    self.d[j] += delta;
                }
                -2 if self.spx.u[j] < INFINITY => {
                    self.c[j] -= delta;
                    self.d[j] += delta;
                }
                -2 if self.d[j] <= OPT_TOL => {
                    moved = true;
                    self.vstat[j] = -1;
                    self.c[j] += delta;
                    self.d[j] = -self.d[j] + delta;
                }
                _ => {}
            }
        }
        if moved {
            self.compute_xb();
        }
        self.pert_size = (3.0 * self.pert_size).min(0.01);
        self.perturbed = true;
        self.pt_lvl = 3;
    }

    /// Dual simplex until primal feasible. Returns Optimal (for the working costs), Infeasible, or a status that
    /// asks the driver to switch to primal (Unbounded stands for "artificial bounds are in the way").
    pub fn dual(&mut self) -> Status {
        self.in_dual = true;
        let mut retried = false;
        loop {
            if self.iter >= self.iter_limit || self.deadline.is_some_and(|d| std::time::Instant::now() >= d) {
                return Status::IterLimit;
            }
            // weights past 1e30 or NaN restart from 1 (06268b L200-415, "simplex pricing changes to steepest edge")
            if self.w.iter().any(|w| w.is_nan() || w.abs() > 1e30) {
                self.w.iter_mut().for_each(|w| *w = 1.0);
            }
            self.snap();
            if self.iter.is_multiple_of(2000) && std::env::var("RSOLVER_DEBUG").is_ok() {
                let (pi, di) = self.sums();
                eprintln!("dual it {} obj {:e} pinf {pi:e} dinf {di:e} pert {} big_m {} mode {}", self.iter, self.objective(), self.perturbed, self.big_m, self.pr.mode);
            }
            let r = match self.chuzr() {
                Some(r) => r,
                None if self.big_m => {
                    if let Some(st) = self.end_big_m() {
                        return st;
                    }
                    continue;
                }
                None => usize::MAX,
            };

            if r == usize::MAX {
                if self.lu.n_updates > 0 {
                    self.reinvert_with(true); // reason 4
                    if (0..self.m).any(|p| self.primal_infeas(p) != 0.0) {
                        continue;
                    }
                }
                self.compute_duals();
                return Status::Optimal;
            }
            let delta = self.primal_infeas(r);
            let (rho_sp, dense) = self.lu.btran_dense(&[(r, 1.0)]);
            // β = ‖ρ_r‖² (8b473 C:105–222) over what 28c93 C:149–163 hands it: a dense BTRAN output compacted to
            // |ρ_i| > ZERO in row order, a sparse one as it is
            let beta = sum_squares(&rho_sp.iter().map(|e| e.1).filter(|v| !dense || v.abs() > ZERO).collect::<Vec<_>>());
            let mut rho = vec![0.0; self.m];
            for &(i, v) in &rho_sp {
                rho[i] = v;
            }
            let rho_sp: Vec<(usize, f64)> = rho_sp.into_iter().filter(|e| e.1.abs() > ZERO).collect();

            let (alpha, order) = self.pivot_row(&rho_sp);

            if let Some(path) = &self.bfrt_path {
                // α-vector dump for tools/proto/alphacmp.py: order and raw values as hex
                use std::io::Write;
                let vals: Vec<String> = order.iter().map(|&j| format!("\"{}\"", hexf(alpha[j]))).collect();
                if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
                    let _ = writeln!(f, "{{\"iter\":{},\"leave\":{},\"idx\":{:?},\"val\":[{}]}}", self.iter, self.head[r], order, vals.join(","));
                }
            }
            // bound flipping ratio test (1c35c5): walk the breakpoints d̃_j/|α_j| in Harris groups; boxed columns
            // whose flip keeps the slope |δ| − Σ|α|(u−l) positive flip instead of entering (m1_notes §8.1)
            let sgn = if delta < 0.0 { -1.0 } else { 1.0 };
            // 1c35c5 in the original sign convention (candidates α < 0, normalized d ≥ 0), structural columns
            // first, then slacks
            let mut cands: Vec<Cand> = Vec::new();
            for &j in &order {
                let a = alpha[j];
                if a == 0.0 || self.rejected[j] {
                    continue;
                }
                let t = sgn * a;
                let (ag, dg) = match self.vstat[j] {
                    -1 => (-t, self.d[j]),
                    -2 => (t, self.d[j]),
                    -3 => (-a.abs(), self.d[j].abs()),
                    _ => continue,
                };
                if ag >= -1e-10 {
                    continue; // bfrt skips it first thing
                }
                let w = if j < self.n { self.spx.len[j] as f64 * (self.spx.var_factor(j) * 0.3 + 0.7) } else { 0.5 };
                cands.push(Cand { j, a: ag, d: dg, range: self.spx.u[j] - self.spx.l[j], back: j >= self.n, w });
            }
            let feas = if self.big_m { BIG_M_TOL } else { FEAS_TOL };
            let (qq, flips) = bfrt(&cands, -delta.abs(), feas);
            let q = qq.unwrap_or(usize::MAX);

            if q == usize::MAX && flips.is_empty() {
                if self.lu.n_updates > 0 && !retried {
                    retried = true;
                    self.reinvert();
                    continue;
                }
                // a row that only artificial bounds keep infeasible is no proof
                let artificial = (0..self.n + self.m).any(|j| {
                    alpha[j].abs() > PIVOT_TOL && self.vstat[j] < 0 && self.x_nonbasic(j).abs() >= INFINITY
                });
                return if artificial { Status::Unbounded } else { Status::Infeasible };
            }
            if !flips.is_empty() {
                // move the flipped columns to their other bound; x_B absorbs the change with one FTRAN (1c8477)
                let mut dx = vec![0.0; self.m];
                let mut huge = false;
                for &j in &flips {
                    let old = self.x_nonbasic(j);
                    self.vstat[j] = if self.vstat[j] == -1 { -2 } else { -1 };
                    self.d[j] = -self.d[j];
                    let step = self.x_nonbasic(j) - old;
                    huge |= step.abs() >= 1e20;
                    for (i, a) in self.spx.col(j) {
                        dx[i] += a * step;
                    }
                }
                let dxb = self.lu.ftran_dense(dx);
                let changed: Vec<usize> = (0..self.m).filter(|&p| dxb[p] != 0.0).collect();
                if huge && !self.big_m {
                    // outside the artificial-bound phase a 1e30-scale move would leave residue: recompute
                    self.compute_xb();
                } else {
                    for p in 0..self.m {
                        self.xb[p] -= dxb[p];
                    }
                }
                self.price_touch(&changed, None, q == usize::MAX);
                if q == usize::MAX {
                    self.iter += 1;
                    self.log();
                    continue;
                }
            }
            let delta = {
                let j = self.head[r];
                self.xb[r] - if sgn < 0.0 { self.spx.l[j] } else { self.spx.u[j] }
            };
            // cost shifting: an entering column that sits slightly on the wrong side gets d_q = 0
            let dq_t = self.d[q];
            if dq_t <= OPT_TOL {
                self.pt_deg += 1; // 28c93 L822-831
            }
            if self.vstat[q] != -3 && dq_t < 0.0 {
                self.c[q] -= self.raw_d(q);
                self.d[q] = 0.0;
                self.perturbed = true;
            }
            retried = false;

            let (col, spike) = self.ftran_col(q);

            let arq = col[r];
            if arq * alpha[q] <= 0.0 || (arq - alpha[q]).abs() > BIG_TOL * (1.0 + arq.abs()) || arq.abs() <= ZERO {
                // row and column pivots disagree (028c93 safety ladder): refactor, then MarkowitzTol 0.5, then reject
                if self.lu.n_updates == 0 && self.markowitz_tol >= 0.5 {
                    self.rejected[q] = true;
                    continue;
                }
                if self.lu.n_updates == 0 {
                    self.markowitz_tol = 0.5;
                }
                self.reinvert();
                continue;
            }
            self.rejected.iter_mut().for_each(|x| *x = false);
            // dual step
            // 48d5f L170-215: θ = d_q/α_q with the row value α_q, d_j −= α_j·θ over the pivot row (the entering
            // column keeps its rounding residue); the leaving column's row entry is 1 (structural) or ρ_i (slack)
            let theta_d = self.raw_d(q) / alpha[q];
            if theta_d != 0.0 {
                for &j in &order {
                    if alpha[j] != 0.0 && self.vstat[j] < 0 && !(self.vstat[j] == -4 && j < self.n) {
                        // d is stored normalized (negated at upper): the step changes sign with it
                        let t = alpha[j] * theta_d;
                        self.d[j] = if self.vstat[j] == -2 { self.d[j] + t } else { self.d[j] - t };
                    }
                }
            }
            let out = self.head[r];
            let a_out = if out < self.n { 1.0 } else { rho[out - self.n] };
            self.d[out] -= a_out * theta_d;

            // DSE weights with τ = B⁻¹ρ
            let supp: Vec<usize> = (0..self.m).filter(|&p| col[p] != 0.0).collect(); // α_q support, position order
            let mut tau = rho.clone();
            self.lu.ftran_lower(&mut tau);
            let taup = self.lu.ftran_tau(tau, &supp);
            // 1bae9c L420-470: β = ‖ρ_r‖² (above), w_r = (β/α_r)/α_r, then
            // w_i = max(1e-8, (β·t·t − (t+t)·τ_i) + w_i) with t = α_i/α_r

            let ar = if arq.abs() >= 1e-8 { arq } else if arq < 0.0 { -1e-8 } else { 1e-8 };
            if self.snap_path.is_some() {
                self.last_dse = (taup.clone(), arq, beta);
                self.bt_dense = dense;
                self.last_aq_row = alpha[q];
                let (rows, t) = self.lu.u_rows();
                let rows: Vec<String> = rows
                    .iter()
                    .map(|r| format!("[{}]", r.iter().map(|&(c, u)| format!("[{},\"{}\"]", c, hexf(u))).collect::<Vec<_>>().join(",")))
                    .collect();
                self.last_urow = format!("\"urow\":[{}],\"t\":{},", rows.join(","), t);
            }
            self.w[r] = if beta == 0.0 { 1e-8 } else { (beta / ar) / ar };
            for &p in &supp {
                if p != r {
                    let t = col[p] / ar;
                    self.w[p] = ((beta * t * t - (t + t) * taup[p]) + self.w[p]).max(1e-8);
                }
            }

            // primal step
            let theta_p = delta / arq;
            for &p in &supp {
                self.xb[p] -= theta_p * col[p];
            }
            let xq = self.x_nonbasic(q) + theta_p;
            let leave = if sgn < 0.0 { -1 } else { -2 }; // 28c93 L845: the direction of the selected violation
            let leave = if self.spx.u[out] - self.spx.l[out] <= 0.0 { -4 } else { leave };
            let col_nnz = supp.iter().filter(|&&p| col[p].abs() > ZERO).count();
            self.col_nnz = col_nnz;
            let updates_before = self.lu.n_updates;

            self.iter += 1; // 28c93 L820: counted before the LU update or refactor (seeds, marks)
            self.pivot(r, q, leave, &spike);

            if self.lu.n_updates > 0 {
                self.xb[r] = xq;
            }
            if !self.big_m && (theta_p.abs() >= 1e20 || delta.abs() >= 1e20) {
                // outside the artificial-bound phase a 1e30-scale step would leave residue in x_B: recompute
                self.compute_xb();
            }
            // P+0x48 = (2·nnz(α_q) + (k − 2)·P+0x48)/k with k the iteration count; α_q without its pivot (8b473
            // strips it), and only on the update path of 1bae9c (not after a refactor)
            if self.lu.n_updates > updates_before {
                let k = self.iter as f64;
                self.pr.avg = (2.0 * col_nnz.saturating_sub(1) as f64 + (k - 2.0) * self.pr.avg) / k;
            }
            if self.lu.n_updates > updates_before && self.pr.mode == 1 && self.pr.list.len() as f64 > self.pr.avg.max(self.c46() as f64)
            {
                // 1bae9c L501–511 (update path only): a list longer than max(P+0x48, spx+0x46c) raises the threshold
                self.pr.thr *= 1.1;
            }
            if self.lu.n_updates > updates_before && self.pr.cost_mode > 0 {
                // 9eb18 (5b257 L1330, update branch only): P+0x60 = nnz(ρ_r); rows of α_q gain it up to m/2, the
                // pivot position gets at least P+0x60 + 100
                let p60 = rho_sp.len() as f64;
                let half = self.m as f64 * 0.5;
                for &p in &supp {
                    if p != r && col[p].abs() >= ZERO {
                        self.pr.row_cost[p] = (self.pr.row_cost[p] + p60).min(half);
                    }
                }
                self.pr.row_cost[r] = self.pr.row_cost[r].max(p60 + 100.0);
            }
            if self.lu.n_updates > updates_before {
                let mut changed = supp.clone();
                if col[r] == 0.0 {
                    changed.insert(changed.partition_point(|&p| p < r), r);
                }
                self.price_touch(&changed, Some(r), true);
            }

            self.log();
        }
    }

    /// Primal simplex `0478d3` (docs/m1/m1_primal_spec.md): composite phase 1 (basic costs c ∓ M on rows below/above
    /// their bounds, M = BigM 1e6, dual tolerance τ1·M), partial-list CHUZC (mode 0, `06dfff`), Harris ratio test in
    /// phase 2 (`55f68`) and the phase-1 test `54eef`, bound flips of the entering column; with no candidate in phase 1
    /// the composite costs are dropped (pure phase 1, τ1 = 1e-9) before infeasibility is declared, and optimality is
    /// only declared on a fresh factorization.
    /// shortcut: x_B is kept as values (the original keeps offsets from the lower bound), no devex/SE, no piecewise
    /// phase-1 test (`56973`), no degeneracy bound shift (`b1da8`); the iteration path is not Gurobi's yet.
    pub fn primal(&mut self) -> Status {
        const BIG_M: f64 = 1e6;
        self.in_dual = false;
        // nonbasics may not sit on an infinite bound in the primal
        for j in 0..self.n + self.m {
            let s = self.vstat[j];
            if s < 0 && self.x_nonbasic(j).abs() >= INFINITY {
                self.vstat[j] = self.bound_status(j, 0.0);
            }
        }
        self.c = self.spx.c.clone();
        self.cost_scale = 1.0;
        self.perturbed = false;
        self.compute_xb();
        let (n, m, nm) = (self.n, self.m, self.n + self.m);
        let mut composite = true;
        let mut tau1 = 1e-7;
        let mut cursor = 0usize; // P+0x54
        let colw = m >= 500; // P+0x1c/+0x20 = n: column-length scaling of scores
        let tol_p = 1e-9; // min(PIVOTTOL, 1e-9)
        let delta = FEAS_TOL; // Harris tolerance (lvl ≤ 2)
        // b1da8 bookkeeping: degenerate pivots (0x330) and the marks it₀/deg₀, refactor count, level, size PV
        let (mut deg, mut deg0, mut it0, mut refactors, mut lvl, mut removals) = (0u64, 0u64, self.iter, 0u64, 0u32, 0u32);
        let mut pv = 2e-4;
        loop {
            if self.iter >= self.iter_limit || self.deadline.is_some_and(|d| std::time::Instant::now() >= d) {
                return Status::IterLimit;
            }
            // row marks: −1 'B' below the lower bound, +1 'A' above the upper bound
            let mark: Vec<i8> = (0..m)
                .map(|p| {
                    let j = self.head[p];
                    let x = self.xb[p];
                    if x < self.spx.l[j] - FEAS_TOL {
                        -1
                    } else if x > self.spx.u[j] + FEAS_TOL {
                        1
                    } else {
                        0
                    }
                })
                .collect();
            let ninf = mark.iter().filter(|&&k| k != 0).count();
            let phase1 = ninf > 0;
            let cost = |e: &Self, j: usize| if !phase1 || composite { e.c[j] } else { 0.0 };
            let cb: Vec<(usize, f64)> = (0..m)
                .map(|p| (p, cost(self, self.head[p]) + if phase1 { mark[p] as f64 * BIG_M } else { 0.0 }))
                .collect();
            let mut y = vec![0.0; m];
            for (r, v) in self.lu.btran(&cb) {
                y[r] = v;
            }
            let tol = if phase1 { tau1 * BIG_M } else { OPT_TOL };
            if self.iter.is_multiple_of(5000) && std::env::var("RSOLVER_DEBUG").is_ok() {
                let xs = self.x();
                let mut res = self.spx.b.clone();
                for j in 0..self.n + self.m {
                    for (i, a) in self.spx.col(j) {
                        res[i] -= a * xs[j];
                    }
                }
                let rmax = res.iter().fold(0.0f64, |a, &b| a.max(b.abs()));
                let bad = (0..self.n + self.m).filter(|&j| self.vstat[j] < 0 && (xs[j] < self.spx.l[j] - 1e-6 || xs[j] > self.spx.u[j] + 1e-6)).count();
                eprintln!("primal it {} ninf {ninf} composite {composite} obj {:e} shift {} deg {deg} resid {rmax:e} badnb {bad}", self.iter, self.objective(), !self.rhs_shift.is_empty());
            }
            // CHUZC: scan from the cursor, stop after more than 10 columns with a candidate or a full cycle
            let (mut q, mut best, mut scanned, mut found) = (usize::MAX, 0.0, 0usize, 0usize);
            let mut dq = 0.0;
            for t in 0..nm {
                let j = (cursor + t) % nm;
                let s = self.vstat[j];
                if !(-3..=-1).contains(&s) {
                    continue;
                }
                scanned += 1;
                let dj = self.spx.col(j).fold(cost(self, j), |acc, (i, a)| acc - a * y[i]);
                let dhat = match s {
                    -1 => dj,
                    -2 => -dj,
                    _ => -dj.abs(),
                };
                if s == -3 && dj.abs() > tol {
                    q = j;
                    dq = dj;
                    cursor = j;
                    break;
                }
                if dhat < -tol {
                    found += 1;
                    let score = if colw && j < n { dhat / (self.spx.len[j] as f64 + 0.01) } else { dhat };
                    if score < best {
                        best = score;
                        q = j;
                        dq = dj;
                    }
                }
                if scanned > 10 && found > 0 {
                    cursor = (j + 1) % nm;
                    break;
                }
            }
            if q == usize::MAX {
                if phase1 {
                    if composite {
                        composite = false;
                        tau1 = 1e-9;
                        continue;
                    }
                    if self.lu.n_updates > 0 {
                        self.refactor();
                        self.compute_xb();
                        continue;
                    }
                    return Status::Infeasible;
                }
                if self.lu.n_updates > 0 {
                    self.refactor();
                    self.compute_xb();
                    continue;
                }
                if !self.rhs_shift.is_empty() {
                    // b3a19: with PRIMDUALSWITCH ≠ 0 the dual takes over from the shifted optimum (−1004); after the
                    // last allowed switch the shift is dropped and the primal goes on from the true x_B
                    if self.primal_switch {
                        return Status::Optimal;
                    }
                    self.rhs_shift.clear();
                    removals += 1;
                    self.compute_xb();
                    continue;
                }
                self.compute_duals();
                return Status::Optimal;
            }
            // σ = +1 when x_q increases
            let sigma = if self.vstat[q] == -2 || (self.vstat[q] == -3 && dq > 0.0) { -1.0 } else { 1.0 };
            let (col, spike) = self.ftran_col(q);
            let range = self.spx.u[q] - self.spx.l[q];
            let theta0 = if range < INFINITY { range } else { f64::INFINITY };
            // ratio test: x_B − t·σ·col; pass 1 Harris bound, pass 2 largest |α| among ratios within it
            let mut tmax = theta0;
            let mut cand: Vec<(usize, f64, bool)> = Vec::new(); // (position, ratio, leaves at upper)
            for p in 0..m {
                let a = sigma * col[p];
                if a.abs() <= tol_p {
                    continue;
                }
                let j = self.head[p];
                let (l, u, x) = (self.spx.l[j], self.spx.u[j], self.xb[p]);
                // target bound by mark (54eef in phase 1; 55f68 marks are all feasible)
                let (ratio, lim, upper) = if a > 0.0 {
                    match mark[p] {
                        -1 => continue,
                        1 => ((x - u) / a, (x - u + delta) / a, true),
                        _ if l <= -INFINITY => continue,
                        _ => ((x - l) / a, (x - l + delta) / a, false),
                    }
                } else {
                    match mark[p] {
                        1 => continue,
                        -1 => ((x - l) / a, (x - l - delta) / a, false),
                        _ if u >= INFINITY => continue,
                        _ => ((x - u) / a, (x - u - delta) / a, true),
                    }
                };
                tmax = tmax.min(lim);
                if ratio <= tmax {
                    cand.push((p, ratio, upper));
                }
            }
            if theta0 < f64::INFINITY && theta0 <= tmax {
                // the entering column reaches its other bound first: flip
                for p in 0..m {
                    if col[p] != 0.0 {
                        self.xb[p] -= theta0 * sigma * col[p];
                    }
                }
                self.vstat[q] = if self.vstat[q] == -1 { -2 } else { -1 };
                self.iter += 1;
                self.log();
                continue;
            }
            let mut pick: Option<(usize, f64, bool)> = None;
            let mut amax = 0.0;
            for &(p, ratio, upper) in &cand {
                if ratio > tmax {
                    continue;
                }
                let j = self.head[p];
                let mut sc = col[p].abs();
                if colw && j < n && !phase1 {
                    sc *= 1.0 + ((self.spx.len[j] - 1).min(10) as f64) / 10.0;
                }
                if sc > amax {
                    amax = sc;
                    pick = Some((p, ratio, upper));
                }
            }
            let Some((r, theta, upper)) = pick else {
                if phase1 && composite {
                    composite = false;
                    tau1 = 1e-9;
                    continue;
                }
                if self.lu.n_updates > 0 {
                    self.refactor();
                    self.compute_xb();
                    continue;
                }
                return if phase1 { Status::Numeric } else { Status::Unbounded };
            };
            let theta = if theta < 0.0 && !phase1 {
                // b44ce L873-948: a negative phase-2 step moves the leaving basic's bound instead (shift into s)
                let j = self.head[r];
                let bound = if upper { self.spx.u[j] } else { self.spx.l[j] };
                let dx = bound - self.xb[r];
                if self.rhs_shift.is_empty() {
                    self.rhs_shift = vec![0.0; m];
                }
                for (i, a) in self.spx.col(j) {
                    self.rhs_shift[i] += dx * a;
                }
                self.xb[r] = bound;
                0.0
            } else {
                theta.max(0.0)
            };
            if theta <= FEAS_TOL {
                deg += 1;
            }
            for p in 0..m {
                if col[p] != 0.0 {
                    self.xb[p] -= theta * sigma * col[p];
                }
            }
            let xq = self.x_nonbasic(q) + sigma * theta;
            let j = self.head[r];
            let mut st = if self.spx.u[j] - self.spx.l[j] <= 0.0 { -4 } else if upper { -2 } else { -1 };
            if st == -2 && self.spx.u[j] >= INFINITY {
                st = -1;
            }
            self.col_nnz = col.iter().filter(|&&a| a.abs() > ZERO).count();
            self.iter += 1;
            self.pivot(r, q, st, &spike);
            if self.lu.n_updates > 0 {
                self.xb[r] = xq;
            } else {
                // just refactored: b1da8 when 90% of the pivots since the last mark were degenerate
                refactors += 1;
                if removals < 3 && refactors % (lvl as u64 + 2) == 1 && 0.9 * ((self.iter - it0) as f64) < ((deg - deg0) as f64) {
                    self.shift_bounds(pv);
                    lvl += 2;
                    pv = (pv * 3.0).min(1e6 * 2e-4);
                }
                it0 = self.iter;
                deg0 = deg;
            }
            self.log();
        }
    }

    /// `b1da8`: basics on a bound move into the interior by ((k % m) + m)·PV/m (k from the stream seeded with the
    /// iteration count), where that stays inside the box; the move goes into the rhs shift s.
    fn shift_bounds(&mut self, pv: f64) {
        let m = self.m;
        if self.rhs_shift.is_empty() {
            self.rhs_shift = vec![0.0; m];
        }
        let mut rng = Rng(self.iter);
        for p in 0..m {
            let j = self.head[p];
            let (l, u, x) = (self.spx.l[j], self.spx.u[j], self.xb[p]);
            let k = rng.next_int() as usize;
            let mag = ((k % m) + m) as f64 * pv / m as f64;
            let at_lower = l > -INFINITY && (x - l).abs() <= FEAS_TOL;
            let at_upper = u < INFINITY && (x - u).abs() <= FEAS_TOL;
            let d = if u - l <= 0.0 {
                continue;
            } else if at_upper {
                -mag
            } else if at_lower {
                mag
            } else {
                continue;
            };
            if x + d < l || x + d > u {
                continue;
            }
            self.xb[p] += d;
            for (i, a) in self.spx.col(j) {
                self.rhs_shift[i] += d * a;
            }
        }
    }

    /// Outer driver (`cda5b`): dual (or primal) simplex, removing perturbations and cleaning up with primal.
    pub fn solve(&mut self, method: i32) -> Status {
        self.dual_start();
        self.log();
        self.drive(method == 0)
    }

    /// `2e9ae` on the unscaled workspace: with the basis of a scaled optimum, recompute x_B and d; if a basic
    /// violates its bounds by more than FeasTol or a nonbasic d its sign by more than OptTol, go on with the dual from
    /// that basis. Returns None when the basis is already optimal here.
    /// Warm-start dual (`6268b` entered with a basis, `534f7` param_2 ≠ 0): the basis is taken from `vbasis` (n
    /// structurals) and `cbasis` (m rows) in Gurobi's VBasis/CBasis codes (0 basic, −1 at lower, −2 at upper, −3
    /// superbasic or free), the dual starts in Full pricing with the refactor interval max(20, ⌊m^0.65⌋), true costs
    /// with small dual infeasibilities shifted away, exact DSE norms, and runs the usual primal/dual hand-over.
    /// A singular basis gets slacks like any refactor.
    /// shortcut: superbasic and free nonbasics sit at 0 (no nonbasic values off their bounds), a basis without exactly
    /// m basics falls back to the crash start, and the first-factor handling of `48d5f` (RANDSMALL) is the SHIFT of
    /// a re-solve.
    pub fn solve_warm(&mut self, vbasis: &[i32], cbasis: &[i32]) -> Status {
        let (n, m) = (self.n, self.m);
        let mut vstat = vec![0i32; n + m];
        let mut pos = 0usize;
        for j in 0..n + m {
            let s = if j < n { vbasis[j] } else { cbasis[j - n] };
            let (l, u) = (self.spx.l[j], self.spx.u[j]);
            vstat[j] = if s == 0 {
                pos += 1;
                pos as i32 - 1
            } else if u - l <= 0.0 {
                -4 // FIXTOL = 0
            } else if s == -2 && u < INFINITY || s != -3 && l <= -INFINITY && u < INFINITY {
                -2
            } else if s == -3 || l <= -INFINITY {
                -3
            } else {
                -1
            };
        }
        if pos != m {
            return self.solve(1);
        }
        self.set_basis(vstat);
        self.pr.mode = 1;
        self.lu_interval = self.lu_interval.min(((m as f64).powf(0.65) as usize).max(20));
        self.dual_resume();
        self.drive(false)
    }

    pub fn resolve_unscaled(&mut self, vstat: Vec<i32>) -> Option<Status> {
        self.set_basis(vstat);
        self.refactor();
        self.compute_xb();
        self.compute_duals();
        let pinf = (0..self.m).any(|p| self.primal_infeas(p) != 0.0);
        let dinf = (0..self.n + self.m).any(|j| match self.vstat[j] {
            -1 | -2 => self.d[j] < -OPT_TOL,
            -3 => self.d[j].abs() > OPT_TOL,
            _ => false,
        });
        if !pinf && !dinf {
            return None;
        }
        self.pr.mode = 1; // shortcut: Full pricing from the start of the re-solve
        self.dual_resume();
        Some(self.drive(false))
    }

    /// cda5b loop from the current state, starting with the primal or the dual.
    fn drive(&mut self, start_primal: bool) -> Status {
        // cda5b: primal and dual hand over to each other (status −1004) up to 20 times; a primal that ends on
        // shifted bounds goes to the dual, a dual that ends on shifted costs or artificial bounds goes to the primal
        let mut primal = start_primal;
        let mut st = if primal { self.primal() } else { self.dual() };
        for round in 0..20 {
            if std::env::var("RSOLVER_DEBUG").is_ok() {
                eprintln!("{} -> {st:?} at it {} perturbed {} shift {}", if primal { "primal" } else { "dual" }, self.iter, self.perturbed, !self.rhs_shift.is_empty());
            }
            self.primal_switch = round < 19;
            if primal {
                if st == Status::Optimal && !self.rhs_shift.is_empty() {
                    self.dual_resume();
                    primal = false;
                    st = self.dual();
                    continue;
                }
                return st;
            }
            match st {
                Status::Optimal => {
                    if self.perturbed {
                        self.c = self.spx.c.clone();
        self.cost_scale = 1.0;
                        self.perturbed = false;
                        self.compute_duals();
                    }
                    let dual_infeasible = (0..self.n + self.m).any(|j| match self.vstat[j] {
                        -1 | -2 => self.d[j] < -OPT_TOL,
                        -3 => self.d[j].abs() > OPT_TOL,
                        _ => false,
                    });
                    let artificial = (0..self.n + self.m).any(|j| self.vstat[j] < 0 && self.x_nonbasic(j).abs() >= INFINITY);
                    if !(dual_infeasible || artificial) {
                        return Status::Optimal;
                    }
                }
                Status::Unbounded => {}
                s => return s,
            }
            primal = true;
            st = self.primal();
        }
        st
    }

    /// Dual simplex from the current basis after a primal round: true costs and bounds, fresh factorization, small
    /// dual infeasibilities shifted away, exact DSE norms, a new row copy and candidate list.
    fn dual_resume(&mut self) {
        self.rhs_shift.clear();
        // 534f7 on re-entry: threshold and average start over
        self.pr.thr = 0.0;
        self.pr.avg = 0.0;
        self.c = self.spx.c.clone();
        self.cost_scale = 1.0;
        self.perturbed = false;
        self.big_m = false;
        self.refactor();
        self.compute_xb();
        self.compute_duals();
        self.in_dual = true;
        self.shift_costs();
        self.exact_weights();
        self.rc_build();
        self.price_full();
    }

    /// Primal values of all n+m columns (scaled space).
    pub fn x(&self) -> Vec<f64> {
        (0..self.n + self.m).map(|j| if self.vstat[j] >= 0 { self.xb[self.vstat[j] as usize] } else { self.x_nonbasic(j) }).collect()
    }
}

/// Python `float.hex()` form of a double, for bit-exact snapshots.
fn hexf(x: f64) -> String {
    if x == 0.0 {
        return if x.is_sign_negative() { "-0x0.0p+0".into() } else { "0x0.0p+0".into() };
    }
    if !x.is_finite() {
        return format!("{x}");
    }
    let b = x.to_bits();
    let sign = if b >> 63 == 1 { "-" } else { "" };
    let e = ((b >> 52) & 0x7ff) as i64;
    let mut m = b & ((1u64 << 52) - 1);
    let (lead, exp) = if e == 0 { (0, -1022) } else { (1, e - 1023) };
    let mut digits = format!("{m:013x}");
    while digits.len() > 1 && digits.ends_with('0') {
        digits.pop();
    }
    if m == 0 {
        digits = "0".into();
    }
    m = 0;
    let _ = m;
    format!("{sign}0x{lead}.{digits}p{}{}", if exp >= 0 { "+" } else { "-" }, exp.abs())
}

/// Result of an LP solve in the original space.
pub struct LpResult {
    pub path: Vec<(u64, i64, i64)>,
    pub status: Status,
    pub obj: f64,
    pub x: Vec<f64>,
    pub iter: u64,
    pub trace: Vec<TraceRow>,
    /// Final basis in Gurobi's VBasis/CBasis codes: n structurals then m rows (0 basic, −1 lower or fixed, −2 upper,
    /// −3 superbasic/free).
    pub basis: Vec<i32>,
}

/// Gurobi basis codes of an engine status array (internal −4 fixed reads as −1, positions as 0).
fn gurobi_basis(vstat: &[i32]) -> Vec<i32> {
    vstat.iter().map(|&s| if s >= 0 { 0 } else if s == -4 { -1 } else { s }).collect()
}

/// LP driver (`0c3d0d` without presolve): scaled workspace, simplex, unscaled solution and objective.
/// shortcut: presolve, warm starts, network/sifting/concurrent branches and the unscaled-violation re-solve
/// (`2e9ae`) are not ported.
pub fn solve_model(md: &rsolver_model::Model, method: i32, iter_limit: u64) -> LpResult {
    solve_model_opts(md, method, iter_limit, true)
}

/// LP solve by the dual simplex from a given basis (`Engine::solve_warm`): `vbasis` n structurals, `cbasis` m rows
/// in Gurobi's VBasis/CBasis codes, on the scaled workspace of `md`, with the unscaled re-check of `solve_model`.
/// For the postsolve re-solve of the LP driver and for crossover.
pub fn solve_model_warm(
    md: &rsolver_model::Model,
    vbasis: &[i32],
    cbasis: &[i32],
    iter_limit: u64,
    want_trace: bool,
) -> LpResult {
    solve_model_run(md, -1, iter_limit, want_trace, Some((vbasis, cbasis)), None)
}

/// `solve_model` with the per-iteration trace optional (`want_trace` false: `LpResult::trace` stays empty).
pub fn solve_model_opts(md: &rsolver_model::Model, method: i32, iter_limit: u64, want_trace: bool) -> LpResult {
    solve_model_run(md, method, iter_limit, want_trace, None, None)
}

/// Solve while checking an optional wall-clock deadline between simplex iterations.
pub fn solve_model_deadline(md: &rsolver_model::Model, method: i32, iter_limit: u64, deadline: Option<std::time::Instant>) -> LpResult {
    solve_model_run(md, method, iter_limit, false, None, deadline)
}

fn solve_model_run(
    md: &rsolver_model::Model,
    method: i32,
    iter_limit: u64,
    want_trace: bool,
    warm: Option<(&[i32], &[i32])>,
    deadline: Option<std::time::Instant>,
) -> LpResult {
    let mut spx = Spx::new(md);
    if md.sense < 0 {
        for c in &mut spx.c {
            *c = -*c;
        }
    }
    let mut e = Engine::new(&spx);
    e.iter_limit = iter_limit;
    e.deadline = deadline;
    e.trace_on = want_trace;
    let status = match warm {
        Some((vb, cb)) => e.solve_warm(vb, cb),
        None => e.solve(method),
    };
    if std::env::var("RSOLVER_DEBUG").is_ok() {
        let xs = e.x();
        for j in 0..spx.n + spx.m {
            let v = (spx.l[j] - xs[j]).max(xs[j] - spx.u[j]);
            if v > 1e-6 {
                eprintln!("viol j {j} vstat {} x {} l {} u {} d {}", e.vstat[j], xs[j], spx.l[j], spx.u[j], e.d[j]);
            }
        }
    }
    let scale = spx.scaling.obj * md.sense as f64;
    let mut trace: Vec<TraceRow> = e.trace.iter().map(|&(k, z, p, d, pt)| (k, z / scale + md.objcon, p, d, pt)).collect();
    let mut path = std::mem::take(&mut e.path);
    // 2e9ae: an optimum that the unscaled problem does not accept is re-solved there from the same basis
    if status == Status::Optimal {
        let mut sp2 = Spx::unscaled(md);
        if md.sense < 0 {
            for c in &mut sp2.c {
                *c = -*c;
            }
        }
        let mut e2 = Engine::new(&sp2);
        e2.iter = e.iter;
        e2.iter_limit = iter_limit;
        e2.deadline = deadline;
        e2.trace_on = want_trace;
        if let Some(st) = e2.resolve_unscaled(e.vstat.clone()) {
            let xs = e2.x();
            let x: Vec<f64> = xs[..sp2.n].to_vec();
            let obj = md.obj.iter().zip(&x).map(|(c, x)| c * x).sum::<f64>() + md.objcon;
            let s2 = md.sense as f64;
            trace.extend(e2.trace.iter().map(|&(k, z, p, d, pt)| (k, z / s2 + md.objcon, p, d, pt)));
            path.extend(e2.path.iter().copied());
            return LpResult { status: st, obj, x, iter: e2.iter, trace, path, basis: gurobi_basis(&e2.vstat) };
        }
    }
    let xs = e.x();
    let x: Vec<f64> = (0..spx.n).map(|j| xs[j] * spx.scaling.col[j]).collect();
    let obj = md.obj.iter().zip(&x).map(|(c, x)| c * x).sum::<f64>() + md.objcon;
    LpResult { status, obj, x, iter: e.iter, trace, path, basis: gurobi_basis(&e.vstat) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mixed_workspace() -> Spx {
        Spx::unscaled(&rsolver_model::Model {
            sense: 1,
            obj: vec![0.0; 3], lb: vec![0.0; 3], ub: vec![10.0; 3],
            vtype: b"CIB".to_vec(),
            col_beg: vec![0, 1, 2], col_len: vec![1; 3],
            col_ind: vec![0, 1, 2], col_val: vec![1.0; 3],
            row_sense: vec![b'<'; 3], rhs: vec![0.0; 3],
            ..Default::default()
        })
    }

    #[test]
    fn integer_basics_get_preference_in_both_pricing_modes_and_updates() {
        let spx = mixed_workspace();
        for mode in [0, 1] {
            let mut e = Engine::new(&spx);
            // Basis position zero holds binary column 2, not continuous column 0.
            e.head = vec![2, 0, 1];
            e.xb = vec![-1.0, -1.1, -0.5];
            e.pr.mode = mode;
            e.price_full();
            assert_eq!(e.chuzr(), Some(0), "mode {mode}");
            // Refreshing an integer's score must keep the preference too.
            e.xb[2] = -1.0;
            e.xb[0] = 0.0;
            e.price_touch(&[0, 2], None, true);
            if mode == 0 { e.price_rebuild(); }
            assert_eq!(e.chuzr(), Some(2), "mode {mode} update");
        }
    }

    #[test]
    fn integer_cost_perturbation_preserves_continuous_columns_and_rng_order() {
        let spx = mixed_workspace();
        let mut continuous = spx.clone();
        continuous.vtype.fill(b'C');
        let perturb = |spx: &Spx| {
            let mut e = Engine::new(spx);
            e.head = vec![3, 4, 5];
            e.vstat = vec![-1, -2, -1, 0, 1, 2];
            e.pt_lvl = 3;
            e.perturb();
            e.c
        };
        let (mixed, plain) = (perturb(&spx), perturb(&continuous));
        assert_eq!(mixed[0], plain[0]);
        let integer_width = 1.5_f64 * 0.3 + 0.7;
        let ratio = (integer_width + 1.0).log2();
        for j in [1, 2] {
            assert!((mixed[j] / plain[j] - ratio).abs() < 1e-14);
        }
        assert!(mixed[1] < 0.0 && mixed[2] > 0.0);
        assert_eq!(&mixed[3..], &plain[3..]);
    }

    #[test]
    fn bfrt_accounts_for_integer_column_weight() {
        let mut md = rsolver_model::Model {
            sense: 1,
            obj: vec![0.0; 2], lb: vec![0.0; 2], ub: vec![INFINITY; 2],
            vtype: b"IC".to_vec(),
            col_beg: vec![0, 1], col_len: vec![1; 2],
            col_ind: vec![0, 0], col_val: vec![-1.1, -1.0],
            row_sense: vec![b'<'], rhs: vec![-1.0],
            ..Default::default()
        };
        for (integer, entering) in [(true, 1), (false, 0)] {
            md.vtype[0] = if integer { b'I' } else { b'C' };
            let spx = Spx::unscaled(&md);
            let mut e = Engine::new(&spx);
            assert_eq!(e.solve_warm(&[-1, -1], &[0]), Status::Optimal);
            assert_eq!(e.path[0].1, entering);
        }
    }

    #[test]
    fn reduced_costs_follow_row_order_with_upper_and_fixed_nonbasics() {
        // The column order 0, 2, 1 loses the final unit term if costs are summed column-wise.
        // An identity slack basis makes y equal to the chosen basic costs.
        let spx = Spx {
            m: 3, n: 3,
            beg: vec![0, 3, 6], len: vec![3; 3],
            ind: vec![0, 2, 1, 0, 2, 1, 0, 2, 1],
            val: vec![1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0, -1.0],
            sense: vec![b'<'; 3],
            vtype: vec![b'C'; 3],
            c: vec![0.0, 0.0, 7.0, 1e16, 1e16, 1.0],
            l: vec![0.0; 6], u: vec![INFINITY; 6], b: vec![0.0; 3],
            scaling: crate::scaling::Scaling { col: vec![1.0; 3], row: vec![1.0; 3], obj: 1.0 },
        };
        let mut e = Engine::new(&spx);
        e.head = vec![3, 4, 5];
        e.vstat = vec![-1, -2, -4, 0, 1, 2];
        e.compute_duals();
        assert_eq!(e.d, vec![-1.0, 1.0, 7.0, 0.0, 0.0, 0.0]);
        // Refactorizations reuse the row copy, but must read the current costs and statuses.
        e.c[0] = 4.0;
        e.vstat[1] = -1;
        e.compute_duals();
        assert_eq!(e.d, vec![3.0, -1.0, 7.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn warm_start_from_the_optimal_basis_takes_no_pivots() {
        for name in ["afiro", "sc50a", "adlittle"] {
            let path = format!("{}/../../../re/bench/data/netlib/{name}.mps.gz", env!("CARGO_MANIFEST_DIR"));
            let md = rsolver_model::mps::read_file(&path).unwrap();
            let cold = solve_model_opts(&md, 1, u64::MAX, false);
            assert_eq!(cold.status, Status::Optimal);
            let n = md.obj.len();
            let warm = solve_model_warm(&md, &cold.basis[..n], &cold.basis[n..], u64::MAX, false);
            assert_eq!(warm.status, Status::Optimal, "{name}");
            assert_eq!(warm.iter, 0, "{name}");
            assert!((warm.obj - cold.obj).abs() <= 1e-9 * cold.obj.abs().max(1.0), "{name} {} {}", warm.obj, cold.obj);
        }
    }

    #[test]
    fn warm_start_from_an_intermediate_basis_reaches_the_optimum() {
        for name in ["afiro", "sc50a", "adlittle", "kb2"] {
            let path = format!("{}/../../../re/bench/data/netlib/{name}.mps.gz", env!("CARGO_MANIFEST_DIR"));
            let md = rsolver_model::mps::read_file(&path).unwrap();
            let cold = solve_model_opts(&md, 1, u64::MAX, false);
            let part = solve_model_opts(&md, 1, 5, false);
            assert_eq!(part.status, Status::IterLimit, "{name}");
            let n = md.obj.len();
            let warm = solve_model_warm(&md, &part.basis[..n], &part.basis[n..], u64::MAX, false);
            assert_eq!(warm.status, Status::Optimal, "{name}");
            assert!(warm.iter > 0, "{name}");
            assert!((warm.obj - cold.obj).abs() <= 1e-6 * cold.obj.abs().max(1.0), "{name} {} {}", warm.obj, cold.obj);
        }
    }
}
