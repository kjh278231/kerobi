//! `47c445` (PreDepRow): linearly dependent rows, as presolve calls it (mode 1, then mode 2).
use super::reduce::ColTols;
use super::state::State;

impl State {
    /// `47c445(work_mult, state, 0, env, 0, mode, work)`. shortcut: not ported yet; no-op (only degen2 and degen3
    /// change on Netlib). Port before claiming the default reduction set.
    pub fn dependent_rows(&mut self, _mode: i32, _t: &ColTols) -> i32 {
        0
    }
}
