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
    rows: Vec<Vec<(usize, f64)>>,
    pub iter: u64,
    pub iter_limit: u64,
    lu_interval: usize,
    perturbed: bool,
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
    /// Pivot path: (iteration, entering column, leaving column); bound flips of the entering column leave −1.
    pub path: Vec<(u64, i64, i64)>,
}

impl<'a> Engine<'a> {
    pub fn new(spx: &'a Spx) -> Engine<'a> {
        let (m, n) = (spx.m, spx.n);
        let mut rows: Vec<Vec<(usize, f64)>> = vec![Vec::new(); m];
        for j in 0..n {
            for (i, a) in spx.col(j) {
                rows[i].push((j, a));
            }
        }
        let interval = ((m as f64).powf(0.7) as usize).max(20).min(((m as f64).powf(0.65) as usize).max(20));
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
            rows,
            iter: 0,
            iter_limit: u64::MAX,
            lu_interval: interval,
            perturbed: false,
            markowitz_tol: crate::lu::MARKOWITZ_TOL,
            in_dual: false,
            rejected: vec![false; n + m],
            trace: Vec::new(),
            pr: Pricing::new(m, n, &spx.len),
            big_m: false,
            fac_nnz: (m, m),
            upd_cols: 0,
            col_nnz: 0,
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
                break;
            }
            let mut pivoted_pos = vec![false; self.m];
            for &(_, p, _) in &self.lu.steps {
                pivoted_pos[p] = true;
            }
            let free_rows: Vec<usize> = (0..self.m).filter(|&i| self.lu.prow[i] < 0).collect();
            let free_pos: Vec<usize> = (0..self.m).filter(|&p| !pivoted_pos[p]).collect();
            for (&p, &i) in free_pos.iter().zip(&free_rows) {
                let out = self.head[p];
                self.vstat[out] = self.bound_status(out, self.xb.get(p).copied().unwrap_or(0.0));
                self.head[p] = self.n + i;
                self.vstat[self.n + i] = p as i32;
            }
        }
    }

    /// Refactor and recompute x_B and d; in the dual, boxed nonbasics that turned dual infeasible flip (048d5f).
    fn reinvert(&mut self) {
        self.refactor();
        self.compute_xb();
        self.compute_duals();
        if self.in_dual && self.make_dual_feasible() {
            self.compute_xb();
        }
        if self.in_dual {
            self.price_full();
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
        self.xb = self.lu.ftran_dense(rhs);
    }

    /// y = c_Bᵀ B⁻¹ (dense BTRAN, as for the iteration-0 d) and d = c − Aᵀy.
    pub fn compute_duals(&mut self) {
        let cb: Vec<(usize, f64)> = (0..self.m).map(|p| (p, self.c[self.head[p]])).collect();
        let mut y = vec![0.0; self.m];
        for (r, v) in self.lu.btran(&cb) {
            y[r] = v;
        }
        for j in 0..self.n + self.m {
            self.d[j] = if self.vstat[j] >= 0 {
                0.0
            } else if self.vstat[j] == -4 && j < self.n {
                self.c[j]
            } else {
                let t = self.spx.col(j).fold(self.c[j], |t, (i, a)| t - a * y[i]);
                if self.vstat[j] == -2 { -t } else { t }
            };
        }
    }

    fn exact_weights(&mut self) {
        self.w = (0..self.m).map(|p| sum_squares(&self.lu.btran(&[(p, 1.0)]).iter().map(|e| e.1).collect::<Vec<_>>())).collect();
    }

    /// α_j = ρᵀa_j scattered over the row-wise copy in BTRAN output order (`0a4109` sparse path); basic entries 0
    /// except the leaving position's own column is not needed here.
    fn pivot_row_ordered(&self, rho: &[(usize, f64)]) -> Vec<f64> {
        let mut alpha = vec![0.0; self.n + self.m];
        for &(i, r) in rho {
            if r.abs() > ZERO {
                for &(j, a) in &self.rows[i] {
                    alpha[j] += r * a;
                }
                alpha[self.n + i] = r;
            }
        }
        for (j, a) in alpha.iter_mut().enumerate() {
            if self.vstat[j] >= 0 {
                *a = 0.0;
            }
        }
        alpha
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
        // 6268b LAB_001f6eb5: decided before this pivot's update with k updates done; refactor at the interval
        // (reason 2) or when the update file costs more than a fresh factor (reason 10)
        let k = self.lu.n_updates;
        let m = self.m as f64;
        let c46 = m.powf(0.45) as usize; // spx+0x46c (2f7e1 L882)
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
        let Ok(path) = std::env::var("RSOLVER_SNAP") else { return };
        use std::io::Write;
        let hex = |v: &[f64]| v.iter().map(|x| format!("\"{}\"", hexf(*x))).collect::<Vec<_>>().join(",");
        let dn: Vec<f64> = (0..self.n + self.m)
            .map(|j| self.d[j])
            .collect();
        let line = format!(
            "{{\"iter\":{},\"vstat\":{:?},\"xB\":[{}],\"d\":[{}],\"w\":[{}],\"mode\":{},\"list\":{:?},\"cursor\":{},\"thr\":\"{}\"}}\n",
            self.iter, self.vstat, hex(&self.xb), hex(&dn), hex(&self.w), self.pr.mode, self.pr.list, self.pr.cursor, hexf(self.pr.thr)
        );
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = f.write_all(line.as_bytes());
        }
    }

    fn log(&mut self) {
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

    fn price_rebuild(&mut self) {
        let viol: Vec<Option<f64>> = (0..self.m).map(|p| self.viol(p)).collect();
        let score: Vec<f64> =
            (0..self.m).map(|p| viol[p].map_or(0.0, |v| self.pr.score(p, self.head[p], v, self.w[p]))).collect();
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
        if self.pr.mode == 0 && self.iter > 0 && ((self.m as f64) < 15.0 * self.pr.avg || (self.m as u64) < self.iter) {
            self.pr.mode = 1;
        }
        if self.pr.mode == 0 {
            self.price_rebuild();
        } else {
            let viol: Vec<Option<f64>> = (0..self.m).map(|p| self.viol(p)).collect();
            let score: Vec<f64> =
                (0..self.m).map(|p| viol[p].map_or(0.0, |v| self.pr.score(p, self.head[p], v, self.w[p]))).collect();
            self.pr.full_init(&viol, &score);
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
            // reason 3: refactor; 1bae9c adds m to P+0x48, which makes the full path switch to mode 1
            self.pr.avg += self.m as f64;
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
    fn price_touch(&mut self, changed: &[usize], leaving: Option<usize>) {
        if self.pr.mode != 0 {
            for &p in changed {
                let sc = self.viol(p).map(|v| self.pr.score(p, self.head[p], v, self.w[p]));
                if Some(p) == leaving {
                    // the leaving position is listed: refresh its score or mark it listed-feasible (3)
                    match sc {
                        Some(x) => self.pr.score[p] = x,
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
        if self.pr.allow < 0 {
            self.pr.cursor = 0;
            self.pr.budget -= 1;
            if self.pr.budget < 0 {
                self.pr.avg += self.m as f64;
                self.reinvert();
            } else {
                self.price_rebuild();
            }
        }
    }

    /// `118725` at the end of the artificial-bound phase: a nonbasic on an artificial bound with a strictly
    /// favourable reduced cost means unbounded; otherwise all of them move to their finite bound.
    fn end_big_m(&mut self) -> Option<Status> {
        for j in 0..self.n + self.m {
            let dg = self.d[j];
            let art_up = self.vstat[j] == -2 && self.spx.u[j] >= INFINITY;
            let art_lo = j < self.n && self.vstat[j] == -1 && self.spx.l[j] <= -INFINITY;
            if (art_up || art_lo) && dg > OPT_TOL {
                return Some(Status::Unbounded);
            }
        }
        for j in 0..self.n + self.m {
            if self.vstat[j] == -2 && self.spx.u[j] >= INFINITY {
                self.vstat[j] = -1;
                self.d[j] = -self.d[j];
            }
        }
        for j in 0..self.n {
            if self.vstat[j] == -1 && self.spx.l[j] <= -INFINITY {
                self.vstat[j] = -2;
                self.d[j] = -self.d[j];
            }
        }
        self.big_m = false;
        self.reinvert();
        None
    }

    /// Dual start: crash basis, free columns pivoted in, first factorization, x_B, d, exact DSE norms.
    pub fn dual_start(&mut self) {
        self.set_basis(start_basis(self.spx));
        self.refactor();
        // free-column loop (0534f7 L700-900): each free nonbasic enters at the non-free basic with the largest |α|
        for q in 0..self.n {
            if self.vstat[q] != -3 {
                continue;
            }
            let (col, spike) = self.ftran_col(q);
            let best = (0..self.m)
                .filter(|&p| self.spx.l[self.head[p]] > -INFINITY)
                .max_by(|&a, &b| col[a].abs().total_cmp(&col[b].abs()));
            if let Some(p) = best.filter(|&p| col[p].abs() >= 3e-8) {
                let out = self.head[p];
                let st = if self.spx.u[out] - self.spx.l[out] <= 0.0 { -4 } else { -1 };
                self.col_nnz = col.iter().filter(|&&a| a.abs() > ZERO).count();
                self.pivot(p, q, st, &spike);
            }
        }
        self.compute_xb();
        self.compute_duals();
        self.make_dual_feasible();
        self.compute_xb();
        self.exact_weights();
        self.big_m = (0..self.n + self.m).any(|j| self.vstat[j] < 0 && self.x_nonbasic(j).abs() >= INFINITY);
        self.in_dual = true;
        self.price_full();
    }

    /// Put each boxed nonbasic on the bound its reduced cost asks for (048d5f L2025-2048).
    fn make_dual_feasible(&mut self) -> bool {
        let mut flipped = false;
        for j in 0..self.n + self.m {
            match self.vstat[j] {
                -1 | -2 if self.d[j] < -OPT_TOL => {
                    self.vstat[j] = -3 - self.vstat[j];
                    self.d[j] = -self.d[j];
                    flipped = true;
                }
                _ => {}
            }
        }
        flipped
    }

    /// Cost perturbation (048d5f): nonbasic costs move away from zero in their feasible direction.
    fn perturb(&mut self) {
        let mut seed: u64 = 1;
        for j in 0..self.n + self.m {
            let s = self.vstat[j];
            if s >= 0 || s == -4 {
                continue;
            }
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let nnz = if j < self.n { self.spx.len[j] as f64 } else { 1.0 };
            let delta = ((seed >> 33) % 250 + 1) as f64 * (2e-4 / 500.0) * (1.0 + nnz).log2();
            match s {
                -1 => {
                    self.c[j] += delta;
                    self.d[j] += delta;
                }
                -2 => {
                    self.c[j] -= delta;
                    self.d[j] += delta;
                }
                _ => {}
            }
        }
        self.perturbed = true;
    }

    /// Dual simplex until primal feasible. Returns Optimal (for the working costs), Infeasible, or a status that
    /// asks the driver to switch to primal (Unbounded stands for "artificial bounds are in the way").
    pub fn dual(&mut self) -> Status {
        self.in_dual = true;
        let mut degenerate = 0usize;
        let mut retried = false;
        loop {
            if self.iter >= self.iter_limit {
                return Status::IterLimit;
            }
            // weights past 1e30 or NaN restart from 1 (06268b L200-415, "simplex pricing changes to steepest edge")
            if self.w.iter().any(|w| w.is_nan() || w.abs() > 1e30) {
                self.w.iter_mut().for_each(|w| *w = 1.0);
            }
            self.snap();
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
                    self.reinvert();
                    if (0..self.m).any(|p| self.primal_infeas(p) != 0.0) {
                        continue;
                    }
                }
                self.compute_duals();
                return Status::Optimal;
            }
            let delta = self.primal_infeas(r);
            let rho_sp = self.lu.btran(&[(r, 1.0)]);
            let mut rho = vec![0.0; self.m];
            for &(i, v) in &rho_sp {
                rho[i] = v;
            }
            let alpha = self.pivot_row_ordered(&rho_sp);
            // bound flipping ratio test (1c35c5): walk the breakpoints d̃_j/|α_j| in Harris groups; boxed columns
            // whose flip keeps the slope |δ| − Σ|α|(u−l) positive flip instead of entering (m1_notes §8.1)
            let sgn = if delta < 0.0 { -1.0 } else { 1.0 };
            // 1c35c5 in the original sign convention (candidates α < 0, normalized d ≥ 0), structural columns
            // first, then slacks
            let mut cands: Vec<Cand> = Vec::new();
            for j in 0..self.n + self.m {
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
                let w = if j < self.n { self.spx.len[j] as f64 * (1.0 * 0.3 + 0.7) } else { 0.5 * (1.0 * 0.3 + 0.7) };
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
                let changed: Vec<usize> = if huge && !self.big_m {
                    // outside the artificial-bound phase a 1e30-scale move would leave residue: recompute
                    self.compute_xb();
                    (0..self.m).collect()
                } else {
                    let dxb = self.lu.ftran_dense(dx);
                    for p in 0..self.m {
                        self.xb[p] -= dxb[p];
                    }
                    (0..self.m).filter(|&p| dxb[p] != 0.0).collect()
                };
                self.price_touch(&changed, None);
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
            if theta_d.abs() <= 1e-12 {
                degenerate += 1;
            } else {
                degenerate = 0;
            }
            if theta_d != 0.0 {
                for j in 0..self.n + self.m {
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
            let mut tau = rho.clone();
            self.lu.ftran_lower(&mut tau);
            self.lu.ftran_upper(&mut tau);
            let mut taup = vec![0.0; self.m];
            for &(rr, pos, _) in &self.lu.steps {
                taup[pos] = tau[rr];
            }
            // 1bae9c L420-470: β = ‖ρ_r‖² (summed in BTRAN output order), w_r = (β/α_r)/α_r, then
            // w_i = max(1e-8, (β·t·t − (t+t)·τ_i) + w_i) with t = α_i/α_r
            let beta = sum_squares(&self.lu.btran(&[(r, 1.0)]).iter().map(|e| e.1).collect::<Vec<_>>());
            let ar = if arq.abs() >= 1e-8 { arq } else if arq < 0.0 { -1e-8 } else { 1e-8 };
            self.w[r] = if beta == 0.0 { 1e-8 } else { (beta / ar) / ar };
            for p in 0..self.m {
                if p != r && col[p] != 0.0 {
                    let t = col[p] / ar;
                    self.w[p] = ((beta * t * t - (t + t) * taup[p]) + self.w[p]).max(1e-8);
                }
            }
            // primal step
            let theta_p = delta / arq;
            for p in 0..self.m {
                if col[p] != 0.0 {
                    self.xb[p] -= theta_p * col[p];
                }
            }
            let xq = self.x_nonbasic(q) + theta_p;
            let leave = if delta < 0.0 { -1 } else { -2 };
            let leave = if self.spx.u[out] - self.spx.l[out] <= 0.0 { -4 } else { leave };
            let col_nnz = col.iter().filter(|&&a| a.abs() > ZERO).count();
            self.col_nnz = col_nnz;
            let updates_before = self.lu.n_updates;
            self.pivot(r, q, leave, &spike);
            if self.lu.n_updates > 0 {
                self.xb[r] = xq;
            }
            if !self.big_m && (theta_p.abs() >= 1e20 || delta.abs() >= 1e20) {
                // outside the artificial-bound phase a 1e30-scale step would leave residue in x_B: recompute
                self.compute_xb();
            }
            self.iter += 1;
            // P+0x48 = (2·nnz(α_q) + (k − 2)·P+0x48)/k with k the iteration count
            let k = self.iter as f64;
            self.pr.avg = (2.0 * col_nnz as f64 + (k - 2.0) * self.pr.avg) / k;
            if self.pr.mode == 1 && self.pr.list.len() as f64 > self.pr.avg.max(4.0) {
                // shortcut: spx+0x46c read as 4 (afiro); 1bae9c raises the threshold by 10% on a long list
                self.pr.thr *= 1.1;
            }
            if self.lu.n_updates > updates_before {
                let changed: Vec<usize> = (0..self.m).filter(|&p| p == r || col[p] != 0.0).collect();
                self.price_touch(&changed, Some(r));
            }
            if !self.perturbed && degenerate > 50 {
                self.perturb();
            }
            self.log();
        }
    }

    /// Primal simplex (phase 1 on the bound violations of the basics, then phase 2), Dantzig pricing with
    /// fresh duals each iteration and a textbook ratio test.
    pub fn primal(&mut self) -> Status {
        self.in_dual = false;
        // nonbasics may not sit on an infinite bound in the primal
        for j in 0..self.n + self.m {
            let s = self.vstat[j];
            if s < 0 && self.x_nonbasic(j).abs() >= INFINITY {
                self.vstat[j] = self.bound_status(j, 0.0);
            }
        }
        self.c = self.spx.c.clone();
        self.perturbed = false;
        self.compute_xb();
        loop {
            if self.iter >= self.iter_limit {
                return Status::IterLimit;
            }
            let phase1 = (0..self.m).any(|p| self.primal_infeas(p) != 0.0);
            let cb: Vec<(usize, f64)> = (0..self.m)
                .map(|p| {
                    let v = if phase1 { self.primal_infeas(p).signum() } else { self.c[self.head[p]] };
                    (p, v)
                })
                .collect();
            let mut y = vec![0.0; self.m];
            for (r, v) in self.lu.btran(&cb) {
                y[r] = v;
            }
            let cost = |j: usize| if phase1 { 0.0 } else { self.c[j] };
            let mut q = usize::MAX;
            let mut best = 0.0;
            for j in 0..self.n + self.m {
                let s = self.vstat[j];
                if s >= 0 || s == -4 {
                    continue;
                }
                let dj = self.spx.col(j).fold(cost(j), |t, (i, a)| t - a * y[i]);
                self.d[j] = dj;
                let score = match s {
                    -1 if dj < -OPT_TOL => -dj,
                    -2 if dj > OPT_TOL => dj,
                    -3 if dj.abs() > OPT_TOL => dj.abs(),
                    _ => 0.0,
                };
                if score > best {
                    best = score;
                    q = j;
                }
            }
            if q == usize::MAX {
                if phase1 {
                    return Status::Infeasible;
                }
                self.compute_duals();
                return Status::Optimal;
            }
            let dir = if self.d[q] < 0.0 { 1.0 } else { -1.0 }; // x_q increases when d_q < 0
            let (col, spike) = self.ftran_col(q);
            // ratio test (Harris two passes): x_B(t) = x_B − t·dir·col; phase 1 basics stop where they turn feasible
            let range = self.spx.u[q] - self.spx.l[q];
            let mut tmax = f64::INFINITY;
            let mut r = usize::MAX;
            let mut leave_upper = false;
            let mut amax = 0.0;
            let mut tr = f64::INFINITY;
            for pass in 0..2 {
                for p in 0..self.m {
                    let a = dir * col[p];
                    if a.abs() <= PIVOT_TOL {
                        continue;
                    }
                    let j = self.head[p];
                    let (l, u, x) = (self.spx.l[j], self.spx.u[j], self.xb[p]);
                    let (bound, upper) = if a > 0.0 {
                        if x > u + FEAS_TOL {
                            (u, true)
                        } else if x < l - FEAS_TOL || l <= -INFINITY {
                            continue;
                        } else {
                            (l, false)
                        }
                    } else if x < l - FEAS_TOL {
                        (l, false)
                    } else if x > u + FEAS_TOL || u >= INFINITY {
                        continue;
                    } else {
                        (u, true)
                    };
                    let dist = ((x - bound) * a.signum()).max(0.0);
                    if pass == 0 {
                        tmax = tmax.min((dist + FEAS_TOL) / a.abs());
                    } else if dist / a.abs() <= tmax && a.abs() > amax {
                        amax = a.abs();
                        r = p;
                        leave_upper = upper;
                        tr = dist / a.abs();
                    }
                }
            }
            if r == usize::MAX && range >= INFINITY {
                return if phase1 { Status::Numeric } else { Status::Unbounded };
            }
            self.iter += 1;
            if range < INFINITY && range <= tr {
                // the entering variable reaches its other bound first: flip it
                for p in 0..self.m {
                    self.xb[p] -= range * dir * col[p];
                }
                self.vstat[q] = if self.vstat[q] == -1 { -2 } else { -1 };
                self.log();
                continue;
            }
            let j = self.head[r];
            let t = tr;
            for p in 0..self.m {
                if col[p] != 0.0 {
                    self.xb[p] -= t * dir * col[p];
                }
            }
            let xq = self.x_nonbasic(q) + t * dir;
            let st = if self.spx.u[j] - self.spx.l[j] <= 0.0 { -4 } else if leave_upper { -2 } else { -1 };
            self.col_nnz = col.iter().filter(|&&a| a.abs() > ZERO).count();
            self.pivot(r, q, st, &spike);
            if self.lu.n_updates > 0 {
                self.xb[r] = xq;
            }
            self.log();
        }
    }

    /// Outer driver (`cda5b`): dual (or primal) simplex, removing perturbations and cleaning up with primal.
    pub fn solve(&mut self, method: i32) -> Status {
        self.dual_start();
        self.log();
        if method == 0 {
            return self.primal();
        }
        let st = self.dual();
        if std::env::var("RSOLVER_DEBUG").is_ok() {
            eprintln!("dual -> {st:?} at it {} perturbed {}", self.iter, self.perturbed);
        }
        match st {
            Status::Optimal => {
                if self.perturbed {
                    self.c = self.spx.c.clone();
                    self.perturbed = false;
                    self.compute_duals();
                }
                let dual_infeasible = (0..self.n + self.m).any(|j| match self.vstat[j] {
                    -1 | -2 => self.d[j] < -OPT_TOL,
                    -3 => self.d[j].abs() > OPT_TOL,
                    _ => false,
                });
                let artificial = (0..self.n + self.m).any(|j| self.vstat[j] < 0 && self.x_nonbasic(j).abs() >= INFINITY);
                if dual_infeasible || artificial {
                    self.primal()
                } else {
                    Status::Optimal
                }
            }
            Status::Unbounded => self.primal(),
            s => s,
        }
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
}

/// LP driver (`0c3d0d` without presolve): scaled workspace, simplex, unscaled solution and objective.
/// shortcut: presolve, warm starts, network/sifting/concurrent branches and the unscaled-violation re-solve
/// (`2e9ae`) are not ported.
pub fn solve_model(md: &rsolver_model::Model, method: i32, iter_limit: u64) -> LpResult {
    let mut spx = Spx::new(md);
    if md.sense < 0 {
        for c in &mut spx.c {
            *c = -*c;
        }
    }
    let mut e = Engine::new(&spx);
    e.iter_limit = iter_limit;
    let status = e.solve(method);
    if std::env::var("RSOLVER_DEBUG").is_ok() {
        let xs = e.x();
        for j in 0..spx.n + spx.m {
            let v = (spx.l[j] - xs[j]).max(xs[j] - spx.u[j]);
            if v > 1e-6 {
                eprintln!("viol j {j} vstat {} x {} l {} u {} d {}", e.vstat[j], xs[j], spx.l[j], spx.u[j], e.d[j]);
            }
        }
    }
    let xs = e.x();
    let x: Vec<f64> = (0..spx.n).map(|j| xs[j] * spx.scaling.col[j]).collect();
    let obj = md.obj.iter().zip(&x).map(|(c, x)| c * x).sum::<f64>() + md.objcon;
    let scale = spx.scaling.obj * md.sense as f64;
    let trace = e.trace.iter().map(|&(k, z, p, d, pt)| (k, z / scale + md.objcon, p, d, pt)).collect();
    LpResult { status, obj, x, iter: e.iter, trace, path: std::mem::take(&mut e.path) }
}
