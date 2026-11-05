//! 03 LP presolve (`390989`), ported stage by stage against the presolve oracle (tools/presolve_oracle.py,
//! docs/m1_presolve.md). Entry from `Model.presolve()` is `999702`, which names the result `<name>_pre`.
//! Every reduction call counts toward GURO_PAR_PRELIMIT; reaching it jumps straight to model generation, which
//! is what makes each reduction checkable on its own.
// Ported comparisons keep the original's form (`!(a < b)` differs from `a >= b` on NaN), and loops index several
// parallel arrays at once.
#![allow(clippy::neg_cmp_op_on_partial_ord, clippy::needless_range_loop)]
mod agg;
mod deprow;
mod derived;
mod parallel;
pub mod postsolve;
mod reduce;
mod sort;
mod state;
mod symmetry;
mod tracker;

use reduce::ColTols;
use rsolver_model::Model;
use state::State;

/// The Gurobi parameters LP presolve reads. Values as gurobipy takes them; names match case-insensitively.
#[derive(Clone, Debug)]
pub struct Params {
    /// env+0x2424 Presolve (-1 auto, 0 off, 1 conservative, 2 aggressive)
    pub presolve: i32,
    /// env+0x2460 FeasibilityTol
    pub feastol: f64,
    /// env+0x2458 GURO_PAR_INFBOUND
    pub infbound: f64,
    /// env+0x24d0 GURO_PAR_FIXTOL
    pub fixtol: f64,
    /// env+0x26ec GURO_PAR_PRELIMIT: number of reduction calls before presolve stops
    pub prelimit: i32,
    /// env+0x2430 DualReductions
    pub dual_reductions: bool,
    /// env+0x2544 GURO_PAR_PREBOUND
    pub prebound: i32,
    /// env+0x2470 OptimalityTol
    pub opttol: f64,
    /// env+0x2718 NumericFocus
    pub numeric_focus: i32,
    /// env+0x26c0 GURO_PAR_AGGMULTLIMIT
    pub agg_mult_limit: f64,
    /// env+0x26f4 PreCrush
    pub precrush: i32,
    /// env+0x2540 PrePasses
    pub prepasses: i32,
    /// env+0x242c Aggregate
    pub aggregate: i32,
    /// env+0x2428 AggFill (-1 auto)
    pub agg_fill: i32,
    /// env+0x2594 GURO_PAR_PREPAIRAGG
    pub pre_pair_agg: i32,
    /// env+0x2534 PreDepRow (-1 auto)
    pub pre_dep_row: i32,
    /// env+0x2864 Symmetry (-1 auto)
    pub symmetry: i32,
    /// env+0x2c28 GURO_PAR_PRESYMMPASSES (-1)
    pub presym_passes: i32,
}

impl Default for Params {
    fn default() -> Self {
        Params { presolve: -1, feastol: 1e-6, infbound: 1e30, fixtol: -1.0, prelimit: 2_000_000_000, dual_reductions: true, prebound: 1,
                 opttol: 1e-6, numeric_focus: 0, agg_mult_limit: 1e14, precrush: 0, prepasses: -1, aggregate: 1, agg_fill: -1,
                 pre_pair_agg: 3, pre_dep_row: -1, symmetry: -1, presym_passes: -1 }
    }
}

impl Params {
    pub fn from_pairs(pairs: &[(String, f64)]) -> Params {
        let mut p = Params::default();
        for (k, v) in pairs {
            match k.to_ascii_lowercase().as_str() {
                "presolve" => p.presolve = *v as i32,
                "feasibilitytol" => p.feastol = *v,
                "guro_par_infbound" => p.infbound = *v,
                "guro_par_fixtol" => p.fixtol = *v,
                "guro_par_prelimit" => p.prelimit = *v as i32,
                "dualreductions" => p.dual_reductions = *v != 0.0,
                "guro_par_prebound" => p.prebound = *v as i32,
                "optimalitytol" => p.opttol = *v,
                "numericfocus" => p.numeric_focus = *v as i32,
                "guro_par_aggmultlimit" => p.agg_mult_limit = *v,
                "precrush" => p.precrush = *v as i32,
                "prepasses" => p.prepasses = *v as i32,
                "aggregate" => p.aggregate = *v as i32,
                "aggfill" => p.agg_fill = *v as i32,
                "guro_par_prepairagg" => p.pre_pair_agg = *v as i32,
                "predeprow" => p.pre_dep_row = *v as i32,
                "symmetry" => p.symmetry = *v as i32,
                "guro_par_presymmpasses" => p.presym_passes = *v as i32,
                _ => {}
            }
        }
        p
    }
}

pub fn run(md: &Model, p: &Params) -> Model {
    run_post(md, p).0
}

/// `run` plus what uncrush needs (`postsolve.rs`).
pub fn run_post(md: &Model, p: &Params) -> (Model, postsolve::Post) {
    let mut st = State::new(md);
    let nnz0 = st.nnz; // aggregation changes `st.nnz`; the arena's first chunk size uses the constructor's value
    let mut work = st.work;
    // `390989` L211-244: work budget from the constructor's nnz (+0xd8); the base (+0x450) is 0 here
    st.prelimit = p.prelimit;
    let size = if 1 < p.prepasses || p.presolve == 2 { 1e100 } else if p.presolve == 1 { st.nnz as f64 * 200.0 } else { st.nnz as f64 * 1000.0 };
    st.budget = size + 1e7;
    if p.presolve != 0 {
        let tols = ColTols {
            infbound: p.infbound,
            feastol: p.feastol,
            opttol: p.opttol,
            numeric_focus: p.numeric_focus,
            agg_mult_limit: p.agg_mult_limit,
            dual_reductions: p.dual_reductions,
            precrush: p.precrush,
        };
        reduce(&mut st, p, &tols);
        work = st.work;
        // `3b0917`: opposite parallel rows 3ff744 left (state+0x42c) become `=` rows with a range column, unless
        // PreCrush > 1. shortcut: PRESPLITROWS is not in Params and assumed at its default (-1, on); add it
        // before running with GURO_PAR_PRESPLITROWS=0.
        if st.antipar != 0 && p.precrush <= 1 {
            st.parallel(2, &tols);
        }
    }
    let q = st.to_model(md);
    let post = postsolve::Post::new(&st, p.infbound, nnz0, work);
    (q, post)
}

/// The reduction sequence; returns when a reduction reports a status or the call limit is reached.
fn reduce(st: &mut State, p: &Params, tols: &ColTols) {
    let mut calls = 0;
    if st.empty_and_free_rows(p.feastol) != 0 {
        return;
    }
    calls += 1;
    if st.prelimit <= calls {
        return;
    }
    // `503985` fixes columns with 1e-10 < u - l <= FIXTOL; with FIXTOL < 0 (default) only for n >= 10000.
    // shortcut: not ported; no Netlib model reaches it. Port it before running models with >= 10000 columns.
    assert!(p.fixtol < 0.0 && st.n < 10000, "503985 (near-fixed columns) is not ported");
    calls += 1;
    if st.prelimit <= calls {
        return;
    }
    st.activity(None, p.infbound);
    if p.prebound != 0 {
        st.dual_singleton_fix(p.dual_reductions, p.infbound);
        calls += 1;
        if st.prelimit <= calls {
            return;
        }
    }
    // PreDepRow -1 (`390989` L251): 3 when nnz > 2.5 n (Presolve 1 gives 0; Method, INTBASIS, PRESYMMPASSES at
    // their defaults)
    let mut dep_rows = if p.pre_dep_row != -1 {
        p.pre_dep_row
    } else if p.presolve == 2 || (p.presolve != 1 && (st.n as f64) * 2.5 < st.nnz as f64) {
        3
    } else {
        0
    };
    let passes = if p.prepasses < 0 { if p.presolve == 1 { 5 } else { 100 } } else { p.prepasses };
    let mut pass_snap = st.snapshot();
    for k in 0..passes {
        let mut i = 0;
        let mut snap;
        loop {
            snap = st.snapshot();
            st.tracker.level = 2;
            // `3ba9de` here runs with a 1e100 budget (390989 swaps +0x1f0 around the call): no work stop
            st.reset_tracker();
            if st.singleton_cols(tols) != 0 {
                return;
            }
            calls += 1;
            if st.prelimit <= calls {
                return;
            }
            if st.singleton_rows(p.feastol, p.infbound) != 0 {
                return;
            }
            calls += 1;
            if st.prelimit <= calls {
                return;
            }
            // state+0x278 (set by aggregation 517fa8, unported) would skip 4ff158
            st.forcing_rows(p.feastol, p.infbound);
            calls += 1;
            if st.prelimit <= calls {
                return;
            }
            if st.redundant_rows(0, p.feastol, p.opttol, p.infbound) != 0 {
                return;
            }
            calls += 1;
            if st.prelimit <= calls {
                return;
            }
            let rho = st.progress(&snap);
            let slack = (i - 5).max(0);
            i += 1;
            if !((slack as f64) * 1e-11 < rho) {
                break;
            }
        }
        st.tracker.level = 1;
        if st.n == 0 || st.m == 0 {
            break;
        }
        if 0 < k && st.progress(&pass_snap) <= ((k - 2).max(0) as f64) * 0.0001 {
            break;
        }
        pass_snap = st.snapshot();
        st.reset_tracker();
        if st.redundant_rows(1, p.feastol, p.opttol, p.infbound) != 0 {
            return;
        }
        calls += 1;
        if st.prelimit <= calls {
            return;
        }
        if p.dual_reductions {
            if st.empty_cols(p.opttol, p.feastol, p.infbound) != 0 {
                return;
            }
            calls += 1;
            if st.prelimit <= calls {
                return;
            }
            if st.cost_point_fix(p.feastol, p.opttol, p.infbound) != 0 {
                return;
            }
            calls += 1;
            if st.prelimit <= calls {
                return;
            }
        }
        // state+0x27c (quadratic constraints) is 0 for LP: 50495e, not 451195
        let early = p.dual_reductions && p.precrush < 2 && (k < 5 || p.presolve == 2);
        if st.implied_redundant_rows(p.feastol, p.infbound, p.prebound) != 0 {
            return;
        }
        calls += 1;
        if st.prelimit <= calls {
            return;
        }
        if early {
            if st.dual_bound_rows(tols, p.presolve == 2) != 0 {
                return;
            }
            calls += 1;
            if st.prelimit <= calls {
                return;
            }
        }
        if p.presolve == 2 {
            if st.singleton_cols(tols) != 0 {
                return;
            }
            calls += 1;
            if st.prelimit <= calls {
                return;
            }
        }
        if early {
            if st.singleton_rows(p.feastol, p.infbound) != 0 {
                return;
            }
            calls += 1;
            if st.prelimit <= calls {
                return;
            }
            st.forcing_rows(p.feastol, p.infbound);
            calls += 1;
            if st.prelimit <= calls {
                return;
            }
        }
        // parallel columns and rows (the 0.8 timer test is always true). shortcut: PREPARALLEL is not in Params
        // and assumed at its default (1, on); add it before running with GURO_PAR_PREPARALLEL=0.
        if p.dual_reductions {
            // shortcut: Presolve=2 runs 51f5ab here, not ported; stop like a reached PRELIMIT. Port it with
            // the other Presolve=2 reductions.
            if p.presolve == 2 {
                return;
            }
            if st.parallel(1, tols) != 0 {
                return;
            }
            calls += 1;
            if st.prelimit <= calls {
                return;
            }
        }
        if st.parallel(0, tols) != 0 {
            return;
        }
        calls += 1;
        if st.prelimit <= calls {
            return;
        }
        // no PWL columns in LP; `50fe2b` needs DualReductions but the call counts either way
        if p.dual_reductions {
            st.cost_tight_rows(p.infbound);
        }
        calls += 1;
        if st.prelimit <= calls {
            return;
        }
        // aggregation; state+8 is not 4 for a general LP, so the reduced-mode guard (local_38) is off
        if p.aggregate != 0 && (k == 0 || p.aggregate > 1) {
            // `390989` resolves AggFill -1 to 10 (20 when model+0xf0 is set, which Model.presolve() does not do)
            let ap = agg::AggPar { fill: if p.agg_fill == -1 { 10 } else { p.agg_fill }, pair_agg: p.pre_pair_agg, prebound: p.prebound };
            if st.aggregate(tols, &ap) != 0 {
                return;
            }
            calls += 1;
            if st.prelimit <= calls {
                return;
            }
        }
        // `5b488d` (GURO_PAR_PREDERIVED, default 1)
        if p.dual_reductions && p.precrush == 0 {
            if st.derived_bounds(tols) != 0 {
                return;
            }
            calls += 1;
            if st.prelimit <= calls {
                return;
            }
        }
        let rows = st.rows_list();
        st.activity(Some(&rows), p.infbound);
        // PRETWOROWS (-1) runs `51aba6` only with Presolve 2.
        // shortcut: Presolve=2 reductions are not ported; stop like a reached PRELIMIT.
        if p.presolve == 2 {
            return;
        }
        // `5577ba` (symmetry): pass 0 with >= 500 columns and >= 100 rows left (state+0x330, +0x258, +0x27c are 0
        // for LP, local_38 is off). Not a counted call. A verified symmetry turns automatic PreDepRow off.
        if k == 0 && p.presym_passes < 0 && 500 <= st.n as i32 - st.cols_removed && 100 <= st.m as i32 - st.rows_removed {
            let (nodes, work) = if 1 < p.symmetry { (5000, (st.budget / 5.0 + 1e7) * 10.0) } else { (5, st.budget / 5.0 + 1e7) };
            if 0 < st.symmetry_count(nodes, work) && p.pre_dep_row == -1 {
                dep_rows = 0;
            }
        }
        if k == 0 && dep_rows & 1 != 0 {
            if st.dependent_rows(1, tols, p.pre_dep_row == -1) != 0 {
                return;
            }
            calls += 1;
            if st.prelimit <= calls {
                return;
            }
        }
        if k == 0 && dep_rows & 2 != 0 && p.dual_reductions {
            if st.dependent_rows(2, tols, p.pre_dep_row == -1) != 0 {
                return;
            }
            calls += 1;
            if st.prelimit <= calls {
                return;
            }
        }
        if st.progress(&snap) <= 1e-6 {
            break;
        }
    }
    finish(st, p, tols, calls);
}

/// The finish stage (`390989` LAB_0048dff3): level 0, `51a144`, {`50cc90(1)`, `50d878`} until `50d878` removes no
/// more columns, then `50de61` (and `50d878` once more if it removed rows).
fn finish(st: &mut State, p: &Params, tols: &ColTols, mut calls: i32) {
    st.tracker.level = 0;
    st.reset_tracker();
    if p.prebound != 0 {
        st.dual_singleton_fix(p.dual_reductions, p.infbound);
        calls += 1; // counted without a limit check
    }
    if st.prelimit <= calls {
        return;
    }
    // PreSparsify (-1) would run `470936` only at 2; no QP/QC parts for LP
    loop {
        if st.redundant_rows(1, p.feastol, p.opttol, p.infbound) != 0 {
            return;
        }
        calls += 1;
        if st.prelimit <= calls {
            return;
        }
        let before = st.cols_removed;
        if p.dual_reductions {
            if st.empty_cols(p.opttol, p.feastol, p.infbound) != 0 {
                return;
            }
            calls += 1;
            if st.prelimit <= calls {
                return;
            }
        }
        if st.cols_removed <= before {
            break;
        }
    }
    if p.prebound != 0 {
        let before = st.rows_removed;
        if st.bound_rows(tols) != 0 {
            return;
        }
        calls += 1;
        if st.prelimit <= calls {
            return;
        }
        if before < st.rows_removed && p.dual_reductions {
            st.empty_cols(p.opttol, p.feastol, p.infbound);
        }
    }
}

