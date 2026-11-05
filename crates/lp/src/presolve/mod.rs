//! 03 LP presolve (`390989`), ported stage by stage against the presolve oracle (tools/presolve_oracle.py,
//! docs/m1_presolve.md). Entry from `Model.presolve()` is `999702`, which names the result `<name>_pre`.
//! Every reduction call counts toward GURO_PAR_PRELIMIT; reaching it jumps straight to model generation, which
//! is what makes each reduction checkable on its own.
mod reduce;
mod sort;
mod state;
mod tracker;

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
}

impl Default for Params {
    fn default() -> Self {
        Params { presolve: -1, feastol: 1e-6, infbound: 1e30, fixtol: -1.0, prelimit: 2_000_000_000, dual_reductions: true, prebound: 1 }
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
}
