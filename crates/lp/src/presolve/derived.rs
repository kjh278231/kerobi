//! `5b488d` (GURO_PAR_PREDERIVED): derived bounds.
use super::reduce::ColTols;
use super::state::State;

impl State {
    /// `5b488d`. shortcut: not ported yet; no-op (Netlib results do not change with GURO_PAR_PREDERIVED=0).
    /// Port before claiming the default reduction set.
    pub fn derived_bounds(&mut self, _t: &ColTols) -> i32 {
        0
    }
}
