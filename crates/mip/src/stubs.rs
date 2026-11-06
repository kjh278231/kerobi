//! Reductions of 3cd406 not ported yet: each changes nothing and returns 0 (the driver still counts it where Gurobi
//! does). Porting one = delete its stub here and add the method in its own file (docs/m2_presolve.md §0).
// shortcut: every stub below skips a reduction Gurobi runs at defaults; port them (docs/m2_presolve.md §2).
#![allow(unused_variables)]
use crate::state::State;
use crate::Params;

impl State {
    /// `594c0c` (PRESS2MIP)
    pub fn ss2mip(&mut self, p: &Params) -> i32 { 0 }
    /// `423e35(st, env, T10, probe mode)` (PREPROBE0)
    pub fn probe0(&mut self, p: &Params, probe: i32) -> i32 { 0 }
    /// `419ffc` (PRECOVERS)
    pub fn covers(&mut self, p: &Params) -> i32 { 0 }
    /// `5a1293` (PREDUALREDUNDANT)
    pub fn dual_redundant(&mut self, p: &Params) -> i32 { 0 }
    /// `5a4d8b` (PREDERIVED)
    pub fn derived_a(&mut self, p: &Params) -> i32 { 0 }
    /// `5a5eff` (PREDERIVED)
    pub fn derived_b(&mut self, p: &Params) -> i32 { 0 }
    /// `5aaaf7` (PREDUALREDUNDANT)
    pub fn r5aaaf7(&mut self, p: &Params) -> i32 { 0 }
    /// `838543(env, st, 1, k, W)` mod-k (k = 2, 3, 5)
    pub fn mod_k(&mut self, p: &Params, k: i32) -> i32 { 0 }
    /// `579e67`
    pub fn r579e67(&mut self, p: &Params) -> i32 { 0 }
    /// `5a04fc` smart disaggregation (PRESMARTDISAGG >= 2)
    pub fn smart_disagg(&mut self, p: &Params) -> i32 { 0 }
    /// `52cba4(st, env, T21, &ncomp)` connected components
    pub fn components(&mut self, p: &Params, ncomp: &mut i32) -> i32 { 0 }
    /// `521497(st, env, T22)` (PREBICONNECTED)
    pub fn biconnected(&mut self, p: &Params) -> i32 { 0 }
    /// `470936(st, env, T20, 0, 0)` sparsify
    pub fn sparsify(&mut self, p: &Params) -> i32 { 0 }
    /// `57999f` (return value ignored)
    pub fn r57999f(&mut self, p: &Params) {}
}
