//! Reductions of 3cd406 not ported yet: each changes nothing and returns 0 (the driver still counts it where Gurobi
//! does). Porting one = delete its stub here and add the method in its own file (docs/m2_presolve.md §0).
// shortcut: every stub below skips a reduction Gurobi runs at defaults; port them (docs/m2_presolve.md §2).
#![allow(unused_variables)]
use crate::state::State;
use crate::Params;

impl State {
    /// `5a9282` (before the loop and at round end; PREBOUND && DualReductions)
    pub fn r5a9282(&mut self, p: &Params) -> i32 { 0 }
    /// `3e5363` XOR / parity rows
    pub fn xor_rows(&mut self, p: &Params) -> i32 { 0 }
    /// `3ff744(st, env, T6, mode, flag, 0)` parallel rows / columns
    pub fn parallel(&mut self, p: &Params, mode: i32, flag: i32) -> i32 { 0 }
    /// `57627e` implied integers, integer equations
    pub fn implied_int(&mut self, p: &Params) -> i32 { 0 }
    /// `578fd2` knapsack coefficient lifting
    pub fn knapsack(&mut self, p: &Params) -> i32 { 0 }
    /// `374521` dual bounds
    pub fn dual_bounds(&mut self, p: &Params) -> i32 { 0 }
    /// `5431ee` binary implication graph SCC
    pub fn trans_red(&mut self, p: &Params) -> i32 { 0 }
    /// `4030c9` dominated columns
    pub fn dominated(&mut self, p: &Params) -> i32 { 0 }
    /// `414c03(st, env, mode, T15)` row nonzero cancellation (`40d2bc` mode 0)
    pub fn cancel_rows(&mut self, p: &Params, mode: i32) -> i32 { 0 }
    /// `36ee92` compression (no count)
    pub fn compress(&mut self, p: &Params) -> i32 { 0 }
    /// `3e40d1` inequalities to equalities (return value ignored)
    pub fn dual_ineq_to_eq(&mut self, p: &Params) -> i32 { 0 }
    /// `37f484` aggregation
    pub fn aggregate(&mut self, p: &Params) -> i32 { 0 }
    /// `414c53(st, env, mode, T16)` column nonzero cancellation (`40d2bc` mode 1)
    pub fn cancel_cols(&mut self, p: &Params, mode: i32) -> i32 { 0 }
    /// `418542` dual aggregation
    pub fn dual_agg(&mut self, p: &Params) -> i32 { 0 }
    /// 3cd406 inline dual binarization candidates (m2_spine_b.md §5.4; `3c2f36` never runs on the 38 models)
    pub fn dual_conv_bin(&mut self, p: &Params) -> i32 { 0 }
    /// `5466a7` VUB merging
    pub fn merge_vubs(&mut self, p: &Params) -> i32 { 0 }
    /// `45feaa(st, env, T7, mode)` two-row reductions
    pub fn two_rows(&mut self, p: &Params, mode: i32) -> i32 { 0 }
    /// `47c445(mode)` linear dependent rows (2: dual side)
    pub fn dep_rows(&mut self, p: &Params, mode: i32) -> i32 { 0 }
    /// `58d4ce` (Q diagonal columns; nothing without Q)
    pub fn q_diag(&mut self, p: &Params) -> i32 { 0 }
    /// `5577ba(env, st, nodes, 1, &f4; 1.01, limit)` symmetry
    pub fn symmetry(&mut self, p: &Params, nodes: i32, limit: f64) -> i32 { 0 }
    /// `5ab429`
    pub fn r5ab429(&mut self, p: &Params) -> i32 { 0 }
    /// `59c193` VUB lifting (PREVUBLIFT)
    pub fn vub_lift(&mut self, p: &Params) -> i32 { 0 }
    /// `55a779` aggregation pass
    pub fn agg_pass(&mut self, p: &Params) -> i32 { 0 }
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
    /// 3cd406 inline final dual fixing of empty columns (m2_spine_d.md §5.2)
    pub fn dual_fix_empty_cols(&mut self, p: &Params) -> i32 { 0 }
    /// 3cd406 inline implied-integer scaling (m2_spine_d.md §5.3)
    pub fn implied_int_scale(&mut self, p: &Params) -> i32 { 0 }
}
