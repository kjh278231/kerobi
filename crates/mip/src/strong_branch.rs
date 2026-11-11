//! Single integer candidate control with an external LP and saved-state backend.
//! shortcut: only a fresh candidate whose propagation changes its own bound is supported;
//! extra propagation, contradiction, other candidate reuse and batch cleanup need M2-06h-3.
//! Integer (B/I) columns only: a continuous column fixes both bounds and skips the up solve.
//! The work counter the original accumulates in this span is not modelled.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side { Lower, Upper }

#[derive(Clone, Copy, Debug)]
pub struct Candidate {
    pub column: usize,
    pub x: f64,
    pub lower: f64,
    pub upper: f64,
    pub int_tol: f64,
    pub parent_obj: f64,
}

/// Only the original statuses observed in the first-candidate fixture are accepted.
#[derive(Clone, Copy, Debug)]
pub enum LpValue {
    Optimal(f64),
    Limited(f64),
}

#[derive(Clone, Copy, Debug)]
pub struct LpResult {
    pub value: LpValue,
    pub iterations: f64,
}

/// The caller owns a fixed parent snapshot. Bounds are unscaled requests;
/// flushing applies the requests to the model before the LP reads them.
pub trait Backend {
    type Error;
    fn bound(&mut self, column: usize, side: Side, value: f64) -> Result<(), Self::Error>;
    fn flush(&mut self) -> Result<(), Self::Error>;
    fn solve(&mut self) -> Result<LpResult, Self::Error>;
    fn reset(&mut self) -> Result<(), Self::Error>;
    fn restore(&mut self) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Debug)]
pub struct Evaluation {
    pub down: f64,
    pub up: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error<E> {
    Backend(E),
    /// Result reached the infeasible sentinel; the remaining candidate/batch control is unported.
    Unported,
}

/// Starts after parent snapshot/ObjVal/solution capture and ends with the final
/// upper-bound restoration queued. The caller flushes before any next LP solve.
/// Both direction values must stay below 1e100. Otherwise returns Unported after
/// restoring that direction, before any down solve. Backend errors propagate
/// immediately: an error does not guarantee restoration.
pub fn evaluate<B: Backend>(c: &Candidate, lp: &mut B, iterations: &mut f64)
    -> Result<Evaluation, Error<B::Error>>
{
    let split = (c.x + c.int_tol).floor();
    let mut values = [0.0; 2];
    for (k, side, bound, original) in [
        (0, Side::Lower, split + 1.0, c.lower),
        (1, Side::Upper, split, c.upper),
    ] {
        lp.bound(c.column, side, bound).map_err(Error::Backend)?;
        lp.flush().map_err(Error::Backend)?;
        let result = lp.solve().map_err(Error::Backend)?;
        *iterations += result.iterations;
        values[k] = match result.value {
            // maxsd operand order, so NaN and ±0 pick the same operand as the original.
            LpValue::Optimal(obj) => if obj > c.parent_obj { obj } else { c.parent_obj },
            LpValue::Limited(dual) => if c.parent_obj > dual { c.parent_obj } else { dual },
        };
        lp.reset().map_err(Error::Backend)?;
        lp.restore().map_err(Error::Backend)?;
        lp.bound(c.column, side, original).map_err(Error::Backend)?;
        // Up: the original skips the down solve. Down: it only resets the stall counter;
        // both continuations are unported.
        if values[k] >= 1e100 {
            return Err(Error::Unported);
        }
    }
    Ok(Evaluation { down: values[1], up: values[0] })
}
