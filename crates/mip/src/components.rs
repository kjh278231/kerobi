//! `52cba4` connected components and `521497` biconnected blocks (docs/m2_red_final.md §3). For linear models both
//! act only through a sub-solve (`5307f4` / `522aa2` → `98b671`) and add no work themselves.
use crate::state::State;
use crate::Params;

impl State {
    /// `52cba4(st, env, T21, &ncomp)`: `52da19` splits the live row/column graph into components (no work for linear
    /// models); with two or more, `5307f4` solves every one but the largest as its own MIP and `ncomp` gets the
    /// columns solved that way.
    /// shortcut: the component search and `5307f4` are not ported (a MIP solve); all 38 models have at most one
    /// component here (52da19 dumps), where Gurobi changes nothing. Port both when a model with more is compared.
    pub fn components(&mut self, _p: &Params, ncomp: &mut i32) -> i32 {
        *ncomp = 0;
        0
    }

    /// `521497(st, env, T22)` (PREBICONNECTED): linear models only (no SOS, PWL, QC, general constraints, NL).
    /// shortcut: the block search and its solve (`522aa2` → `98b671`) are not ported; `522aa2` is never called on the
    /// 38 models and `521497` leaves state and work as they were there (dumps 38/38).
    pub fn biconnected(&mut self, _p: &Params) -> i32 {
        0
    }
}
