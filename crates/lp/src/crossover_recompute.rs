//! Reclassification after primal-push refactorization; numeric FTRAN remains external.

/// New factor header in basis order, bounds in column order, and fixed push frame.
/// All values and intermediate arithmetic must be finite, bounds ordered, indices valid,
/// and infinity > tolerance > 0. Arrays have exactly header.len() basis entries.
pub struct RecomputeInput<'a> {
    pub header: &'a [i32],
    pub lower: &'a [f64],
    pub upper: &'a [f64],
    pub infinity: f64,
    pub tolerance: f64,
    pub model_status: i32,
    pub attempt: i32,
}

/// FTRAN has already returned. Its output either is the basic buffer or must be copied.
/// These variants represent aliasing without overlapping mutable Rust references.
pub enum RecomputeValues<'a> {
    InPlace(&'a mut [f64]),
    Separate { raw: &'a [f64], basic: &'a mut [f64] },
}

/// PInf and violations are replaced; retry is written only when the gate fires.
/// Request is retained for the caller to clear after this boundary.
pub struct RecomputeState {
    pub pinf: f64,
    pub violations: i32,
    pub retry: i32,
    pub request: i32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RecomputeExit {
    Normal,
    Retry,
}

/// Shared by the pivot and round boundaries, stopping before either caller clears request.
/// The caller supplies the new factor header and raw FTRAN output, not the old basis.
pub fn push_recompute(
    input: &RecomputeInput<'_>,
    values: RecomputeValues<'_>,
    width: &mut [f64],
    marks: &mut [u8],
    state: &mut RecomputeState,
) -> RecomputeExit {
    let basic = match values {
        RecomputeValues::InPlace(basic) => basic,
        RecomputeValues::Separate { raw, basic } => {
            basic.copy_from_slice(raw);
            basic
        }
    };
    let inf = input.infinity;
    let h = input.tolerance;
    let mut pinf = 0.0;
    let mut violations = 0;
    for (p, &column) in input.header.iter().enumerate() {
        let j = column as usize;
        let lower = input.lower[j];
        let upper = input.upper[j];
        let w = if lower <= -inf {
            inf
        } else if upper >= inf {
            inf
        } else {
            upper - lower
        };
        width[p] = w;
        if lower > -inf {
            basic[p] -= lower;
        }
        let v = basic[p];
        marks[p] = if lower <= -inf && upper >= inf {
            b'F'
        } else if v < -h {
            violations += 1;
            pinf -= v;
            b'B'
        } else if v < h {
            if w == 0.0 { b'X' } else { b'L' }
        } else if v < w - h || w >= inf {
            b'N'
        } else if w + h <= v {
            violations += 1;
            pinf += v - w;
            b'A'
        } else {
            b'U'
        };
    }
    state.pinf = pinf;
    state.violations = violations;
    if input.model_status == 2 && pinf > 1e10 && input.attempt == 0 {
        state.retry = 1;
        return RecomputeExit::Retry;
    }
    RecomputeExit::Normal
}
