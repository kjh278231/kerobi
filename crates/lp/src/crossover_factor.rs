//! Serial primal-push factor control. Numeric factor/quality work is an external boundary.
use crate::crossover_supplier::PushColumn;

/// Selector input at an approved pivot; batch capacity/fill/position must all be zero.
/// Arithmetic must remain finite and the signed L+U sum must fit in i64.
pub struct FactorSelection {
    pub m: i32,
    pub request: i32,
    pub numeric: i32,
    pub updates: i32,
    pub limit: i32,
    pub interval: i32,
    pub column_work: i64,
    pub eta_count: i32,
    pub l_nnz: i64,
    pub u_nnz: i64,
    pub support: i32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FactorDecision {
    pub request: i32,
    pub call: bool,
}

/// Select a request without changing the input. A deferred support update returns call=false.
pub fn factor_select(input: &FactorSelection) -> FactorDecision {
    let mut request = input.request;
    if request == 0 {
        if input.updates == input.limit {
            request = 2;
        } else {
            let m = f64::from(input.m);
            let a = (input.l_nnz + input.u_nnz) as f64 - m * 1.5;
            let mut cost = 0.0;
            if input.updates >= input.interval {
                let interval_cost = f64::from(input.interval) * ((m * 5.0) * m);
                let fill_cost = a * a;
                let minimum = if interval_cost < fill_cost { interval_cost } else { fill_cost };
                cost = ((input.column_work as f64 + f64::from(input.eta_count)) * 2.0) * m - minimum;
            }
            if cost > 0.0 || (input.numeric != 0 && input.updates > input.interval) {
                request = 10;
            }
        }
    }
    FactorDecision { request, call: request != 0 || input.support == 0 }
}

/// Mutable caller state; factor/quality backends may update these fields.
pub struct FactorState {
    pub request: i32,
    pub numeric: i32,
    pub failure: i32,
    pub status: i32,
    pub updates: i32,
}

/// Persistent quality period plus fixed frame values for the current push attempt.
pub struct FactorFrame {
    pub period: i32,
    pub attempt: i32,
    pub numeric_limit: i32,
    pub enter: i32,
}

/// Meaningful pointer aliases. The wrapper temporarily installs current as workspace;
/// spike stays unchanged on this serial path. Numeric buffer mutation belongs to the backend.
pub struct FactorVectors<'a> {
    pub workspace: &'a PushColumn,
    pub current: &'a PushColumn,
    pub spike: &'a PushColumn,
}

pub trait PushFactorBackend {
    /// Return 1 to repeat, 12 to retry, >1 for an error, or <=0 to continue.
    fn factor(&mut self, state: &mut FactorState, vectors: &FactorVectors<'_>) -> i32;
    /// Return the next period increment; request/status changes are consumed by the caller.
    /// The workspace has already been restored when this method is called.
    fn quality(&mut self, state: &mut FactorState, enter: i32, period: i32,
               vectors: &FactorVectors<'_>) -> i32;
}

#[derive(Debug, PartialEq, Eq)]
pub enum FactorExit {
    Continue,
    /// Stop before RHS FTRAN/reclassification. Caller must perform that work before resuming.
    Recompute,
    Retry,
    Error(i32),
}

enum Tail {
    Again,
    Exit(FactorExit),
}

/// Quality period and request clear after a successful call (C 1982–1996). `Again` re-enters the wrapper.
fn quality_tail(state: &mut FactorState, frame: &mut FactorFrame, vectors: &FactorVectors<'_>,
                backend: &mut impl PushFactorBackend) -> Tail {
    if state.updates >= frame.period {
        let increment = backend.quality(state, frame.enter, frame.period, vectors);
        if state.status == 12 {
            state.status = 1;
            return Tail::Exit(FactorExit::Retry);
        }
        if state.request != 0 {
            frame.period = 20;
            return Tail::Again;
        }
        frame.period += increment;
    } else if state.updates != 0 && frame.period > 50 {
        frame.period = 50;
    }
    state.request = 0;
    Tail::Exit(FactorExit::Continue)
}

/// Run only the serial caller. Backends must eventually return a value other than 1 and
/// resolve quality requests; no arbitrary iteration cap changes the original control flow.
/// On every returned exit workspace and spike aliases have their original values.
pub fn factor_run(state: &mut FactorState, frame: &mut FactorFrame, vectors: &mut FactorVectors<'_>,
                  backend: &mut impl PushFactorBackend) -> FactorExit {
    loop {
        let saved = vectors.workspace;
        vectors.workspace = vectors.current;
        let ret = loop {
            let ret = backend.factor(state, vectors);
            if ret != 1 { break ret; }
        };
        vectors.workspace = saved;
        if ret == 12 { return FactorExit::Retry; }
        if ret > 1 { return FactorExit::Error(ret); }
        if state.status < -999 {
            state.status = 1;
            return FactorExit::Retry;
        }
        if frame.attempt == 0 && (state.failure != 0 || state.numeric >= frame.numeric_limit) {
            return FactorExit::Retry;
        }
        if state.request != 0 { return FactorExit::Recompute; }
        match quality_tail(state, frame, vectors, backend) {
            Tail::Again => continue,
            Tail::Exit(exit) => return exit,
        }
    }
}

/// Resume the same wrapper after the caller finished the `Recompute` exit's RHS FTRAN and reclassification:
/// the request is still set and the quality period check follows (C 1980–1982).
pub fn factor_resume(state: &mut FactorState, frame: &mut FactorFrame, vectors: &mut FactorVectors<'_>,
                     backend: &mut impl PushFactorBackend) -> FactorExit {
    match quality_tail(state, frame, vectors, backend) {
        Tail::Again => factor_run(state, frame, vectors, backend),
        Tail::Exit(exit) => exit,
    }
}
