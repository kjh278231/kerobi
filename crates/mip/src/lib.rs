//! 03 MIP presolve (`3cd406`), entered from `Model.presolve()` through `999702` -> `3db19d` with probe mode
//! GURO_PAR_PREPROBE, no MIP context and forSolve 0 (docs/m2_presolve.md §1, docs/m2_presolve_state.md).
//! Ported so far: the state (`3a484d` MIP branch), the prelude up to the first reduction, finishing and model
//! generation. Every reduction call counts toward GURO_PAR_PRELIMIT; reaching it jumps to finishing, which makes each
//! reduction checkable on its own against `tools/presolve_oracle.py --param GURO_PAR_PRELIMIT=k`.
// Ported comparisons keep the original's form (`!(a < b)` differs from `a >= b` on NaN), and loops index several
// parallel arrays at once.
#![allow(clippy::neg_cmp_op_on_partial_ord, clippy::needless_range_loop)]
pub mod state;
pub mod tracker;

use rsolver_model::Model;
use state::State;

/// The Gurobi parameters MIP presolve reads so far. Values as gurobipy takes them; names match case-insensitively.
/// Defaults are hard-coded here until the shared parameter table (M1 LP driver stream) replaces this struct.
#[derive(Clone, Debug)]
pub struct Params {
    /// env+0x2424 Presolve (-1 auto, 0 off, 1 conservative, 2 aggressive)
    pub presolve: i32,
    /// env+0x26ec GURO_PAR_PRELIMIT: number of reduction calls before presolve stops
    pub prelimit: i32,
    /// env+0x2430 DualReductions
    pub dual_reductions: bool,
    /// env+0x2544 GURO_PAR_PREBOUND
    pub prebound: i32,
    /// env+0x2540 PrePasses (-1 auto)
    pub prepasses: i32,
    /// env+0x2828 GURO_PAR_PREPROBE (also 3cd406's probe mode, from `999702`)
    pub preprobe: i32,
    /// env+0x2680 PreSparsify
    pub presparsify: i32,
    /// env+0x26f4 PreCrush
    pub precrush: i32,
    /// env+0x2460 FeasibilityTol
    pub feastol: f64,
    /// env+0x2468 IntFeasTol
    pub intfeastol: f64,
}

impl Default for Params {
    fn default() -> Self {
        Params { presolve: -1, prelimit: 2_000_000_000, dual_reductions: true, prebound: 1, prepasses: -1, preprobe: 1,
                 presparsify: -1, precrush: 0, feastol: 1e-6, intfeastol: 1e-5 }
    }
}

impl Params {
    pub fn from_pairs(pairs: &[(String, f64)]) -> Params {
        let mut p = Params::default();
        for (k, v) in pairs {
            match k.to_ascii_lowercase().as_str() {
                "presolve" => p.presolve = *v as i32,
                "guro_par_prelimit" => p.prelimit = *v as i32,
                "dualreductions" => p.dual_reductions = *v != 0.0,
                "guro_par_prebound" => p.prebound = *v as i32,
                "prepasses" => p.prepasses = *v as i32,
                "guro_par_preprobe" => p.preprobe = *v as i32,
                "presparsify" => p.presparsify = *v as i32,
                "precrush" => p.precrush = *v as i32,
                "feasibilitytol" => p.feastol = *v,
                "intfeastol" => p.intfeastol = *v,
                _ => {}
            }
        }
        p
    }
}

/// 3cd406's locals that the reduction loop reads.
#[derive(Clone, Copy, Debug)]
pub struct Run {
    /// `local_68`: reduction calls so far, compared with state+0x1dc (GURO_PAR_PRELIMIT) after each
    pub calls: i32,
    /// `local_928`: passes (PrePasses; auto 2e9 with Presolve 2, 20 with Presolve 1, else 100)
    pub passes: i32,
    /// `local_58`: probe mode (GURO_PAR_PREPROBE; 3 for large models whose binaries carry most nonzeros)
    pub probe: i32,
    /// `local_c90`: work limit the reduction timers are set from (L1522-1602)
    pub limit: f64,
}

impl Run {
    /// One reduction call has finished: count it. True = PRELIMIT reached, go to finishing.
    pub fn counted(&mut self, st: &State) -> bool {
        self.calls += 1;
        st.prelimit <= self.calls
    }
}

/// MIP presolve: the presolved model, or Err(status) when presolve proves the model infeasible (3); 3cd406 then
/// skips model generation and `Model.presolve()` fails with "Unable to create presolved model" (checked on a model
/// with an integer column fixed at 2.5).
pub fn presolve(md: &Model, p: &Params) -> Result<Model, i32> {
    let mut st = State::new(md, p);
    prelude(&mut st, p)?;
    match finish(&mut st, p) {
        0 => Ok(st.to_model(md)),
        rc => Err(rc),
    }
}

/// 3cd406 from the constructor to the first reduction (decompiled L756-1745). Ok = go to finishing (PRELIMIT
/// reached, Presolve 0, or the first reduction that is not ported yet); Err = status (3 infeasible).
pub fn prelude(st: &mut State, p: &Params) -> Result<Run, i32> {
    // `5b84b1` (state+0x3f0 scratch), `3ba5ac` (callback, log) and `3ba9de` (time/work limits) change nothing here.
    // GURO_PAR_DISTRIBUTED (env+0x2ca0) > 1 would go straight to finishing; it is 0 by default.
    let passes = if p.prepasses >= 0 { p.prepasses } else if p.presolve == 2 { 2_000_000_000 } else if p.presolve == 1 { 20 } else { 100 };
    let mut run = Run { calls: 0, passes, probe: p.preprobe, limit: 0.0 };
    if let Err(j) = st.round_integers() {
        st.bad_col = j as i32;
        return Err(3);
    }
    if run.counted(st) {
        return Ok(run);
    }
    if p.presolve != 0 {
        // env+0x2728 (inside a sub-MIP) is 0 and there is no MIP context, so the restart block (L1022) is skipped
        let (n, m) = (st.n as f64, st.m as f64);
        if n * 0.99 <= st.nbin as f64 && n * 1.2 < m {
            let eq = st.sense.iter().filter(|&&c| c == b'=').count();
            st.work += m * st.work_mult;
            st.eq_rows = m < eq as f64 * 1000.0;
        }
        run.probe = large_probe(st, p, run.probe);
    }
    // L1522: the work limit, from nnz (+0xd8; Q, SOS, QC and general constraint sizes are 0 here)
    let size = st.nnz as f64;
    let base = if p.presolve == 2 || 1 < p.prepasses || 1 < p.preprobe || 0 < p.presparsify {
        1e100
    } else if p.presolve == 1 {
        size * 500.0
    } else {
        size * 2500.0
    };
    run.limit = base + 1e7;
    if st.empty_and_free_rows(p.feastol) != 0 {
        return Err(3);
    }
    if run.counted(st) {
        return Ok(run);
    }
    // `9f566a` (PWL functions, state+0x240) is not ported: linear models only
    st.activity(None);
    if p.presolve == 0 {
        return Ok(run);
    }
    // L1638 `3ff744` mode 0 runs when 0.2 n <= local_91c, which stays 0 here: only for n = 0.
    // L1668: problem type 11 goes back to 0 without a MIP solution (`5b58f4` not reached); never set here.
    // shortcut: no reduction is ported yet; the first one stops presolve as if PRELIMIT were reached there.
    // L1694: with PREBOUND and dual reductions `5a9282` is count 3.
    if p.prebound != 0 && p.dual_reductions {
        return Ok(run);
    }
    // L1697-1714: the Q (`42872b`, `429e1b`), QC (`451195`) and NL (`5b1161`) blocks are for nonlinear models only.
    // L1730 round 0, pass 0, inner loop i = -5: snapshot (`3c546c`), tracker level 2 (`536971`), `5369b8`, then
    // `3e39a6` is the next count (checked: work and tracker level at its call match Gurobi with PREBOUND 0).
    st.tracker.level = 2;
    st.reset_tracker();
    Ok(run)
}

/// L1214/L3582 large-model check (n > 10000, 2-20% binaries, probe mode 1, PREPROBE 1): when the binaries hold
/// most nonzeros and few carry objective, probing runs in mode 3. A continuous/integer objective column stops it.
fn large_probe(st: &mut State, p: &Params, probe: i32) -> i32 {
    let n = st.n as i32;
    if !(10000 < n && st.nbin < n / 5 && n / 50 + 1 < st.nbin && probe == 1 && p.preprobe == 1) {
        return probe;
    }
    let (mut len, mut costly, mut long) = (0i64, 0, 0);
    let mut j = 0;
    while j < st.n {
        let has_cost = 1e-10 < st.obj[j].abs();
        if st.vtype[j] == b'B' {
            len += st.col_cnt[j] as i64;
            if has_cost {
                if 20 < st.col_cnt[j] {
                    long += 1;
                }
                costly += 1;
            }
        } else if has_cost {
            break;
        }
        j += 1;
    }
    st.work += (st.work_mult * 3.0) * j as f64;
    if j < st.n || 20 <= long || 50 <= costly {
        return probe;
    }
    let share = (st.nbin as f64 * 5.0 / n as f64).max(0.5);
    if share * (st.nnz as f64) < len as f64 { 3 } else { probe }
}

/// Finishing (`LAB_004b2415`, decompiled L9601-9766) up to model generation. Returns a status.
/// shortcut: the PWL, NL, QC and general-constraint blocks before it and the Q/QC/PSD/PreQLinearize/MIQCP work after
/// `3b0917` are not ported (linear models only); `4ca023` (OptimalityTarget 1 with QC) likewise.
pub fn finish(st: &mut State, p: &Params) -> i32 {
    let near = |x: &mut f64| {
        let r = (*x + 0.5).floor();
        if (r - *x).abs() <= 1e-13 {
            *x = r;
        }
    };
    // `538353` / `5383d0`: the raw row and column lists at the tracker's level (removed ones included)
    let lv = st.tracker.level;
    let rows = st.tracker.rows.list[lv][..st.tracker.rows.a[lv]].to_vec();
    for &i in &rows {
        near(&mut st.rhs[i as usize]);
    }
    st.work += (rows.len() as f64 + rows.len() as f64) * st.work_mult;
    let cols = st.tracker.cols.list[lv][..st.tracker.cols.a[lv]].to_vec();
    for &j in &cols {
        let j = j as usize;
        near(&mut st.lb[j]);
        near(&mut st.ub[j]);
        near(&mut st.obj[j]);
    }
    st.work += (st.work_mult * 4.0) * cols.len() as f64;
    st.empty_and_free_rows(p.feastol)
}

impl State {
    /// `5369b8` on the state's tracker.
    pub fn reset_tracker(&mut self) {
        let State { tracker, row_cnt, col_cnt, work, work_mult, .. } = self;
        tracker.reset(row_cnt, col_cnt, work, *work_mult);
    }

    /// 3cd406 L800-868: integer bounds rounded inward (IntFeasTol), binaries clipped to [0, 1], [0, 1] integers
    /// become binaries, a fixed integer at a fractional value within tolerance becomes continuous. Counts +0xf4
    /// (non-continuous) and +0xf0 (binary). Err(j) = column j is a binary fixed outside [0, 1] or an integer fixed
    /// at a fractional value (infeasible).
    pub fn round_integers(&mut self) -> Result<(), usize> {
        let tol = self.intfeastol;
        let mut done = 0;
        let mut bad = None;
        for j in 0..self.n {
            if self.vtype[j] == b'C' {
                done += 1;
                continue;
            }
            self.nint += 1;
            if self.vtype[j] == b'B' {
                self.nbin += 1;
            }
            let l = self.lb[j];
            if l == self.ub[j] {
                if (self.vtype[j] == b'B' && (l < -tol || 1.0 + tol < l)) || tol < l - (tol + l).floor() {
                    bad = Some(j);
                    break;
                }
                if l.floor() != l {
                    self.vtype[j] = b'C';
                }
            } else {
                self.lb[j] = (l - tol).ceil();
                self.ub[j] = (self.ub[j] + tol).floor();
                if self.vtype[j] == b'B' {
                    self.lb[j] = if self.lb[j] > 0.0 { self.lb[j] } else { 0.0 };
                    self.ub[j] = if self.ub[j] < 1.0 { self.ub[j] } else { 1.0 };
                } else if self.lb[j] == 0.0 && self.ub[j] == 1.0 {
                    self.nbin += 1;
                    self.vtype[j] = b'B';
                }
            }
            done += 1;
        }
        self.work += (done as f64 * 3.0) * self.work_mult;
        bad.map_or(Ok(()), Err)
    }
}
