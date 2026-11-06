//! Reductions of 3cd406 not ported yet: each changes nothing and returns 0 (the driver still counts it where Gurobi
//! does). Porting one = delete its stub here and add the method in its own file (docs/m2_presolve.md §0).
// shortcut: every stub below skips a reduction Gurobi runs at defaults; port them (docs/m2_presolve.md §2). On the
// 38 models 419ffc changes only the work counter (docs/m2_red_roundend_b.md §7).
#![allow(unused_variables)]
use crate::state::State;
use crate::Params;

impl State {
    /// `419ffc` (PRECOVERS)
    pub fn covers(&mut self, p: &Params) -> i32 { 0 }
}
