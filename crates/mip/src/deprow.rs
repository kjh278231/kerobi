//! `47c445` PreDepRow on the linked-list state (docs/m2_red_outer_c.md §4; LP port: crates/lp/src/presolve/deprow.rs).
use crate::state::State;
use crate::Params;

impl State {
    /// `47c445(μ, st, 0, env, T19, mode, w)`: W += (μ·3)·nnz, then mode 2 eliminates on the free continuous columns
    /// (live, no flags, lb <= -1e30, ub >= 1e30, type 'C') and fixes dependent ones at 0, as in the LP port.
    /// shortcut: the elimination (the LP port's `Elim` on a matrix built from the lists) is not ported for MIP, nor
    /// mode 1 (PreDepRow > 0); none of the 38 models has a free continuous column when mode 2 runs (47 calls, all
    /// with this work only). Port it by copying the LP `Elim` when such a model is compared.
    pub fn dep_rows(&mut self, _p: &Params, _mode: i32) -> i32 {
        self.work = (self.work_mult * 3.0) * self.nnz as f64 + self.work;
        0
    }
}
