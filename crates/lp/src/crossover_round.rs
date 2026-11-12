//! Serial primal-push round control. Numeric BTRAN/factor and the solver loop are external.
use crate::crossover_supplier::PushColumn;

/// Initialized push context with batch capacity/fill/position all zero.
/// Candidates are valid column indices; deferred <= position <= candidates.len().
/// Marks may be absent until their first clear. Counters must fit in i32.
pub struct RoundState {
    pub m: usize,
    pub candidates: Vec<i32>,
    pub position: usize,
    pub deferred: usize,
    pub failures: Vec<i32>,
    pub marks: Option<Vec<i32>>,
    pub support: i32,
    /// Not initialized until the first successful supplier of a pass.
    pub approved: Option<i32>,
    pub count_sum: f64,
    pub successes: i32,
    pub request: i32,
    pub status: i32,
    pub numeric: i32,
    pub failure: i32,
    pub retry: i32,
}

pub struct RoundFrame {
    pub attempt: i32,
    pub numeric_limit: i32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RoundExit {
    /// Return to the supplier, or to the pivot factor selector after sparse BTRAN.
    Continue,
    End,
    /// Request 10 is set; the round factor wrapper must run next.
    Factor,
    /// Stop before RHS FTRAN/reclassification, even if the backend cleared the request.
    Recompute,
    Retry,
    Error(i32),
}

fn clear_marks(state: &mut RoundState) {
    let marks = state.marks.get_or_insert_with(|| vec![0; state.m]);
    assert_eq!(marks.len(), state.m);
    marks.fill(0);
}

fn compact(state: &mut RoundState) {
    let total = state.candidates.len();
    assert!(state.deferred <= state.position && state.position <= total);
    let retained = state.deferred + total - state.position;
    state.candidates.copy_within(state.position..total, state.deferred);
    state.candidates.truncate(retained);
    state.position = 0;
    state.deferred = 0;
}

/// Before the first supplier call. Approved is not reset at this earlier boundary.
pub fn round_pass(state: &mut RoundState) {
    if state.support != 0 { clear_marks(state); }
    compact(state);
}

/// Call after the first successful supply, before its decision (including a possible defer).
pub fn round_begin(state: &mut RoundState) { state.approved = Some(0); }

/// Only approved flips and pivots increase this pass-local count.
pub fn round_accept(state: &mut RoundState) {
    *state.approved.as_mut().expect("successful supply must initialize approved") += 1;
}

/// Decision-level defer only. Supplier skips preserve the failure count instead.
pub fn round_defer(state: &mut RoundState, enter: i32) {
    state.candidates[state.deferred] = enter;
    state.deferred += 1;
    state.failures[enter as usize] += 1;
}

/// Exhaustion gate, before support adjustment and compaction.
pub fn round_exhausted(state: &mut RoundState) -> RoundExit {
    if state.deferred == state.candidates.len() {
        if state.deferred == 0 || state.support == 0 { return RoundExit::End; }
        state.support = 0;
    }
    RoundExit::Continue
}

pub fn round_support(state: &mut RoundState) {
    let approved = state.approved.expect("round needs initialized approved");
    if (state.m as f64).sqrt() > f64::from(approved) {
        state.support = 0;
    } else if state.support != 0 {
        clear_marks(state);
    }
}

/// Enter after the exhaustion gate, or after a nonempty dense-BTRAN compaction.
pub fn round_restart(state: &mut RoundState) -> RoundExit {
    round_support(state);
    compact(state);
    state.request = 10;
    RoundExit::Factor
}

pub trait RoundBtran {
    /// Return a valid sparse vector in original constraint coordinates, or m dense values.
    fn btran(&mut self, input: &PushColumn, work: &mut f64) -> PushColumn;
}

/// Approved pivot with nonnegative leave. No numeric solve is implemented here.
pub fn round_btran(state: &mut RoundState, leave: i32, work: &mut f64,
                   backend: &mut impl RoundBtran) -> RoundExit {
    if state.support == 0 { return RoundExit::Continue; }
    assert!(leave >= 0 && (leave as usize) < state.m);
    let unit = PushColumn { count: 1, indices: vec![leave], values: vec![1.0] };
    let raw = backend.btran(&unit, work);
    if raw.count >= 0 {
        let count = raw.count as usize;
        assert_eq!(count, raw.indices.len());
        assert_eq!(count, raw.values.len());
        let marks = state.marks.as_mut().expect("BTRAN needs initialized row marks");
        for &row in &raw.indices { marks[row as usize] = 1; }
        RoundExit::Continue
    } else {
        assert_eq!(raw.values.len(), state.m);
        state.support = -1;
        clear_marks(state);
        compact(state);
        if state.candidates.is_empty() { return RoundExit::End; }
        round_restart(state)
    }
}

pub trait RoundFactor {
    /// Numeric backend may change request/status/numeric/failure; return 1 to repeat.
    fn factor(&mut self, state: &mut RoundState) -> i32;
}

/// Round caller: no alpha swap, quality check, or request-dependent recompute shortcut.
pub fn round_factor(state: &mut RoundState, frame: &RoundFrame,
                    backend: &mut impl RoundFactor) -> RoundExit {
    let ret = loop {
        let ret = backend.factor(state);
        if ret != 1 { break ret; }
    };
    if ret == 12 { state.retry = 1; return RoundExit::Retry; }
    if ret > 1 { return RoundExit::Error(ret); }
    if state.status < -999 {
        state.status = 1;
        state.retry = 1;
        return RoundExit::Retry;
    }
    if frame.attempt == 0 && (state.failure != 0 || state.numeric >= frame.numeric_limit) {
        state.retry = 1;
        return RoundExit::Retry;
    }
    RoundExit::Recompute
}

/// After successful round recomputation only. The pass count is preserved until round_begin.
pub fn round_clear(state: &mut RoundState) {
    state.request = 0;
}
