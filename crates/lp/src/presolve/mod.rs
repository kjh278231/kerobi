//! 03 LP presolve (`390989`), ported stage by stage against the presolve oracle (tools/presolve_oracle.py,
//! docs/m1_presolve.md). Entry from `Model.presolve()` is `999702`, which names the result `<name>_pre`.
//! Every reduction call counts toward GURO_PAR_PRELIMIT; reaching it jumps straight to model generation, which
//! is what makes each reduction checkable on its own.
mod reduce;
mod sort;
mod state;
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
}

impl Default for Params {
    fn default() -> Self {
        Params { presolve: -1, feastol: 1e-6, infbound: 1e30, fixtol: -1.0, prelimit: 2_000_000_000, dual_reductions: true, prebound: 1,
                 opttol: 1e-6, numeric_focus: 0, agg_mult_limit: 1e14, precrush: 0, prepasses: -1 }
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
                _ => {}
            }
        }
        p
    }
}

pub fn run(md: &Model, p: &Params) -> Model {
    let mut st = State::new(md);
    if p.presolve != 0 {
        reduce(&mut st, p);
    }
    st.to_model(md)
}

/// The reduction sequence; returns when a reduction reports a status or the call limit is reached.
fn reduce(st: &mut State, p: &Params) {
    let limit = p.prelimit;
    let mut calls = 0;
    if st.empty_and_free_rows(p.feastol) != 0 {
        return;
    }
    calls += 1;
    if limit <= calls {
        return;
    }
    // `503985` fixes columns with 1e-10 < u - l <= FIXTOL; with FIXTOL < 0 (default) only for n >= 10000.
    // shortcut: not ported; no Netlib model reaches it. Port it before running models with >= 10000 columns.
    assert!(p.fixtol < 0.0 && st.n < 10000, "503985 (near-fixed columns) is not ported");
    calls += 1;
    if limit <= calls {
        return;
    }
    st.activity(None, p.infbound);
    if p.prebound != 0 {
        st.dual_singleton_fix(p.dual_reductions, p.infbound);
        calls += 1;
        if limit <= calls {
            return;
        }
    }
    let tols = ColTols {
        infbound: p.infbound,
        feastol: p.feastol,
        opttol: p.opttol,
        numeric_focus: p.numeric_focus,
        agg_mult_limit: p.agg_mult_limit,
        dual_reductions: p.dual_reductions,
        precrush: p.precrush,
    };
    let passes = if p.prepasses < 0 { if p.presolve == 1 { 5 } else { 100 } } else { p.prepasses };
    let mut pass_snap = st.snapshot();
    for k in 0..passes {
        let mut i = 0;
        let mut snap;
        loop {
            snap = st.snapshot();
            st.tracker.level = 2;
            // shortcut: 3ba9de's work-budget stop (budget = nnz*1000 + 1e7) is not checked; Netlib never hits
            // it. Port it with the constructor's work terms (P2).
            st.reset_tracker();
            if st.singleton_cols(&tols) != 0 {
                return;
            }
            calls += 1;
            if limit <= calls {
                return;
            }
            if st.singleton_rows(p.feastol, p.infbound) != 0 {
                return;
            }
            calls += 1;
            if limit <= calls {
                return;
            }
            // state+0x278 (set by aggregation 517fa8, unported) would skip 4ff158
            st.forcing_rows(p.feastol, p.infbound);
            calls += 1;
            if limit <= calls {
                return;
            }
            if st.redundant_rows(0, p.feastol, p.opttol, p.infbound) != 0 {
                return;
            }
            calls += 1;
            if limit <= calls {
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
        if limit <= calls {
            return;
        }
        if p.dual_reductions {
            if st.empty_cols(p.opttol, p.feastol, p.infbound) != 0 {
                return;
            }
            calls += 1;
            if limit <= calls {
                return;
            }
            if st.cost_point_fix(p.feastol, p.opttol, p.infbound) != 0 {
                return;
            }
            calls += 1;
            if limit <= calls {
                return;
            }
        }
        // state+0x27c (quadratic constraints) is 0 for LP: 50495e, not 451195
        if st.implied_redundant_rows(p.feastol, p.infbound, p.prebound) != 0 {
            return;
        }
        calls += 1;
        if limit <= calls {
            return;
        }
        // shortcut: the rest of the outer pass (505077, 3ff744, 50fe2b, 519a52, 5b488d, 47c445) and the finish
        // stage are not ported yet; stop here like a reached PRELIMIT.
        let _ = &snap;
        return;
    }
}
