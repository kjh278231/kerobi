//! 04 §2, §5 Simplex engine on the scaled workspace: the outer driver (`cda5b`), dual simplex (`06268b`/`028c93`)
//! and primal simplex (`0478d3`) sharing one basis, LU and solution state.
//!
//! Conventions: vstat −1 at lower, −2 at upper, −3 free at zero, −4 fixed, ≥ 0 basis position. Infinite bounds are
//! ±1e30 and the dual simplex uses them as real (artificial) bounds, as the original does (m1_notes §3). `d` holds
//! raw reduced costs c_j − a_jᵀy (basic 0); dual feasible means d ≥ 0 at lower, ≤ 0 at upper, = 0 free.
//! shortcut: pricing (DSE max over all rows, not the `15d2d` candidate list), Harris 2-pass instead of BFRT, the
//! perturbation trigger and the refactor schedule are textbook stand-ins; iteration paths differ from Gurobi's.
use crate::lu::{sum_squares, Lu, ZERO};
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
        loop {
            self.lu = Lu::factor_tol(self.spx, &self.head, self.markowitz_tol);
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
                self.spx.col(j).fold(self.c[j], |t, (i, a)| t - a * y[i])
            };
        }
    }

    fn exact_weights(&mut self) {
        self.w = (0..self.m).map(|p| sum_squares(&self.lu.btran(&[(p, 1.0)]).iter().map(|e| e.1).collect::<Vec<_>>())).collect();
    }

    /// ρ = e_pᵀB⁻¹ by row.
    fn btran_unit(&self, p: usize) -> Vec<f64> {
        let mut rho = vec![0.0; self.m];
        for (r, v) in self.lu.btran(&[(p, 1.0)]) {
            rho[r] = v;
        }
        rho
    }

    /// α_j = ρᵀa_j for all nonbasic j (basic entries stay 0).
    fn pivot_row(&self, rho: &[f64]) -> Vec<f64> {
        let mut alpha = vec![0.0; self.n + self.m];
        for (i, &r) in rho.iter().enumerate() {
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
        self.vstat[q] = r as i32;
        self.head[r] = q;
        if !self.lu.update(r, q, spike) || self.lu.n_updates >= self.lu_interval {
            self.reinvert();
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

    fn sums(&self) -> (f64, f64) {
        let pinf: f64 = (0..self.m).map(|p| self.primal_infeas(p).abs()).sum();
        let dinf: f64 = (0..self.n + self.m)
            .map(|j| match self.vstat[j] {
                -1 => (-self.d[j]).max(0.0),
                -2 => self.d[j].max(0.0),
                -3 => self.d[j].abs(),
                _ => 0.0,
            })
            .sum();
        (pinf, dinf)
    }

    fn log(&mut self) {
        let (p, d) = self.sums();
        let z = self.objective();
        self.trace.push((self.iter, z, p, d, self.perturbed));
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
                self.pivot(p, q, st, &spike);
            }
        }
        self.compute_xb();
        self.compute_duals();
        self.make_dual_feasible();
        self.compute_xb();
        self.exact_weights();
    }

    /// Put each boxed nonbasic on the bound its reduced cost asks for (048d5f L2025-2048).
    fn make_dual_feasible(&mut self) -> bool {
        let mut flipped = false;
        for j in 0..self.n + self.m {
            match self.vstat[j] {
                -1 if self.d[j] < -OPT_TOL => {
                    self.vstat[j] = -2;
                    flipped = true;
                }
                -2 if self.d[j] > OPT_TOL => {
                    self.vstat[j] = -1;
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
                    self.d[j] -= delta;
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
            // CHUZR: largest infeasibility² / weight
            let mut r = usize::MAX;
            let mut best = 0.0;
            for p in 0..self.m {
                let v = self.primal_infeas(p);
                if v != 0.0 {
                    let s = v * v / self.w[p];
                    if s > best {
                        best = s;
                        r = p;
                    }
                }
            }
            if r == usize::MAX {
                if self.lu.n_updates > 0 {
                    self.refactor();
                    self.compute_xb();
                    if (0..self.m).any(|p| self.primal_infeas(p) != 0.0) {
                        continue;
                    }
                }
                self.compute_duals();
                return Status::Optimal;
            }
            let delta = self.primal_infeas(r);
            let rho = self.btran_unit(r);
            let alpha = self.pivot_row(&rho);
            // bound flipping ratio test (1c35c5): walk the breakpoints d̃_j/|α_j| in Harris groups; boxed columns
            // whose flip keeps the slope |δ| − Σ|α|(u−l) positive flip instead of entering (m1_notes §8.1)
            let sgn = if delta < 0.0 { -1.0 } else { 1.0 };
            let mut cand: Vec<(usize, f64)> = Vec::new(); // (column, d̃)
            for j in 0..self.n + self.m {
                let a = alpha[j];
                if a.abs() <= PIVOT_TOL || self.rejected[j] {
                    continue;
                }
                let t = sgn * a;
                let dj = match self.vstat[j] {
                    -1 if t > 0.0 => self.d[j],
                    -2 if t < 0.0 => -self.d[j],
                    -3 => self.d[j].abs(),
                    _ => continue,
                };
                cand.push((j, dj));
            }
            cand.sort_by(|x, y| (x.1.max(0.0) / alpha[x.0].abs()).total_cmp(&(y.1.max(0.0) / alpha[y.0].abs())));
            let mut slope = delta.abs();
            let mut flips: Vec<usize> = Vec::new();
            let mut q = usize::MAX;
            // cand is sorted by ratio, so each Harris group is a prefix of what is left
            while !cand.is_empty() {
                let theta_max = cand.iter().map(|&(j, dj)| (dj.max(0.0) + OPT_TOL) / alpha[j].abs()).fold(f64::INFINITY, f64::min);
                let len = cand.iter().take_while(|&&(j, dj)| dj.max(0.0) / alpha[j].abs() <= theta_max).count();
                let group = &cand[..len];
                let cap: f64 = group.iter().map(|&(j, _)| alpha[j].abs() * (self.spx.u[j] - self.spx.l[j])).sum();
                if group.iter().all(|&(j, _)| self.vstat[j] != -3) && slope - cap > FEAS_TOL {
                    slope -= cap;
                    flips.extend(group.iter().map(|&(j, _)| j));
                    cand.drain(..len);
                    continue;
                }
                q = group.iter().map(|&(j, _)| j).max_by(|&x, &y| alpha[x].abs().total_cmp(&alpha[y].abs())).unwrap();
                break;
            }
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
                    let step = self.x_nonbasic(j) - old;
                    huge |= step.abs() >= 1e20;
                    for (i, a) in self.spx.col(j) {
                        dx[i] += a * step;
                    }
                }
                if huge {
                    // same as the pivot step below: artificial-bound-scale moves leave residue, recompute
                    self.compute_xb();
                } else {
                    let dxb = self.lu.ftran_dense(dx);
                    for p in 0..self.m {
                        self.xb[p] -= dxb[p];
                    }
                }
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
            let dq_t = if self.vstat[q] == -2 { -self.d[q] } else { self.d[q] };
            if self.vstat[q] != -3 && dq_t < 0.0 {
                self.c[q] -= self.d[q];
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
            let theta_d = self.d[q] / arq;
            if theta_d.abs() <= 1e-12 {
                degenerate += 1;
            } else {
                degenerate = 0;
            }
            for j in 0..self.n + self.m {
                if alpha[j] != 0.0 && self.vstat[j] < 0 {
                    self.d[j] -= theta_d * alpha[j];
                }
            }
            let out = self.head[r];
            self.d[out] = -theta_d;
            self.d[q] = 0.0;
            // DSE weights with τ = B⁻¹ρ
            let mut tau = rho.clone();
            self.lu.ftran_lower(&mut tau);
            self.lu.ftran_upper(&mut tau);
            let mut taup = vec![0.0; self.m];
            for &(rr, pos, _) in &self.lu.steps {
                taup[pos] = tau[rr];
            }
            let wr = self.w[r];
            for p in 0..self.m {
                if p != r && col[p] != 0.0 {
                    let ratio = col[p] / arq;
                    self.w[p] = (self.w[p] - 2.0 * ratio * taup[p] + ratio * ratio * wr).max(1e-4);
                }
            }
            self.w[r] = (wr / (arq * arq)).max(1e-4);
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
            self.pivot(r, q, leave, &spike);
            if self.lu.n_updates > 0 {
                self.xb[r] = xq;
            }
            if theta_p.abs() >= 1e20 || delta.abs() >= 1e20 {
                // a step at artificial-bound scale leaves rounding residue of that scale in x_B: recompute
                self.compute_xb();
            }
            self.iter += 1;
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
                    -1 => self.d[j] < -OPT_TOL,
                    -2 => self.d[j] > OPT_TOL,
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
