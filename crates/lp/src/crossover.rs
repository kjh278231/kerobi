//! Crossover primal push pieces: the ratio tests on a feasible (docs/m3/primal_push_ratio_rust.md) and a violated basis
//! (docs/m3/primal_push_violated.md), and the step decisions around them (docs/m3/primal_push_decision_rust.md). No push
//! loop calls them yet.

/// The small-pivot second pass with a tightened pivot tolerance was never observed in the original; it is refused
/// instead of guessed.
#[derive(Debug, PartialEq, Eq)]
pub struct Reentry;

/// Feasible-basis ratio test of the primal push on the FTRAN column `(idx, val)`: `idx = None` is the dense form with
/// `val.len() == x.len()`. `x` is the lower-shifted basic value, `width` the bound width, `marks` the basic class.
/// Returns the position in `val` of the leaving entry (-1 for none) and writes `theta`/`pivot` only where the
/// original does; `work += units * work_coef`.
/// shortcut: no column-length weighting of the second pass (the push passes 0); add it for the other callers.
#[allow(clippy::too_many_arguments)]
pub fn feasible_ratio(harris: f64, piv_tol: f64, inf: f64, cap: f64, idx: Option<&[i32]>, val: &[f64], x: &[f64],
                      width: &[f64], marks: &[u8], work_coef: f64, work: &mut f64, theta: &mut f64, pivot: &mut f64)
                      -> Result<i32, Reentry> {
    let count = idx.map_or(x.len(), |i| i.len());
    let row = |k: usize| idx.map_or(k, |i| i[k] as usize);
    let (mut ratios, mut cands) = (Vec::new(), Vec::new());
    let mut chosen = -1i32;
    let mut tested = 0i32;
    let mut bound = cap;
    for k in 0..count {
        let (a, r) = (val[k], row(k));
        let num = if a <= piv_tol || marks[r] == b'F' {
            if !(a < -piv_tol && width[r] < inf) {
                continue;
            }
            x[r] - width[r]
        } else {
            x[r]
        };
        tested += 1;
        let ratio = num / a;
        if ratio <= bound {
            ratios.push(ratio);
            cands.push(k);
            chosen = k as i32;
            let relaxed = if a > piv_tol && marks[r] != b'F' { (num + harris) / a } else { (num - harris) / a };
            if relaxed < bound {
                bound = relaxed;
            }
        }
    }
    let units = if idx.is_none() {
        tested as f64 + tested as f64 + count as f64 + cands.len() as f64 * 3.0
    } else {
        count as f64 * 4.0 + cands.len() as f64 * 3.0
    };
    *work += units * work_coef;
    ratio_tail(piv_tol, inf, cap, &val[..count], bound, &ratios, &cands, chosen, theta, pivot)
}

/// Violated-basis ratio test (`054eef`): like [`feasible_ratio`] but each basic row's class decides which bound it
/// hits, so `B` rows block a decreasing and `A` rows an increasing basic value. The dense form tests `U`/`X` rows on a
/// negative entry with a strict `<`, as the original does; the sparse form uses `<=` there like everywhere else.
/// shortcut: the push sends a result with position < 0 or |pivot| < 0.001 to the `056973` retry ratio before any step
/// decision; that retry is not ported (M3-04c-7), so a caller must not feed such a result to [`push_step`].
#[allow(clippy::too_many_arguments)]
pub fn infeasible_ratio(harris: f64, piv_tol: f64, inf: f64, cap: f64, idx: Option<&[i32]>, val: &[f64], x: &[f64],
                        width: &[f64], marks: &[u8], work_coef: f64, work: &mut f64, theta: &mut f64, pivot: &mut f64)
                        -> Result<i32, Reentry> {
    let count = idx.map_or(x.len(), |i| i.len());
    let row = |k: usize| idx.map_or(k, |i| i[k] as usize);
    let (mut ratios, mut cands) = (Vec::new(), Vec::new());
    let mut chosen = -1i32;
    let mut tested = 0i32;
    let mut bound = cap;
    for k in 0..count {
        let (a, r) = (val[k], row(k));
        let (up, down) = (a > piv_tol, a < -piv_tol);
        if !up && !down {
            continue;
        }
        tested += 1;
        let (v, w) = (x[r], width[r]);
        let num = match (up, marks[r]) {
            (true, b'N' | b'U' | b'L' | b'X') | (false, b'B') => v,
            (true, b'A') | (false, b'U' | b'X') => v - w,
            (false, b'L' | b'N') if inf > w => v - w,
            _ => continue,
        };
        let ratio = num / a;
        let strict = idx.is_none() && !up && matches!(marks[r], b'U' | b'X');
        if if strict { ratio < bound } else { ratio <= bound } {
            ratios.push(ratio);
            cands.push(k);
            chosen = k as i32;
            let relaxed = (if up { num + harris } else { num - harris }) / a;
            if relaxed < bound {
                bound = relaxed;
            }
        }
    }
    let (c, t, n) = (cands.len() as f64, tested as f64, count as f64);
    let units = if idx.is_none() { c + c + t + t + n } else { c + c + n * 4.0 };
    *work += units * work_coef;
    ratio_tail(piv_tol, inf, cap, &val[..count], bound, &ratios, &cands, chosen, theta, pivot)
}

/// Shared end of both ratio tests: the finite cap, no candidate, the largest-pivot pick among ratios within `bound`,
/// and the small-pivot check that would re-run the first pass with a tightened tolerance.
#[allow(clippy::too_many_arguments)]
fn ratio_tail(piv_tol: f64, inf: f64, cap: f64, val: &[f64], bound: f64, ratios: &[f64], cands: &[usize],
              mut chosen: i32, theta: &mut f64, pivot: &mut f64) -> Result<i32, Reentry> {
    if cap < inf && cap <= bound {
        *pivot = 0.0;
        *theta = cap;
        return Ok(-1);
    }
    if bound == inf || cands.is_empty() {
        *pivot = 0.0;
        *theta = inf;
        return Ok(-1);
    }
    let mut best = 0.0;
    for (&ratio, &k) in ratios.iter().zip(cands) {
        if ratio <= bound && best < val[k].abs() {
            *theta = ratio;
            chosen = k as i32;
            best = val[k].abs();
        }
    }
    *pivot = val[chosen as usize].abs();
    if *pivot >= 0.1 {
        return Ok(chosen);
    }
    let big = val.iter().fold(0.0f64, |m, v| m.max(v.abs()));
    let scale = match big {
        b if b < 10.0 => 0.1,
        b if b < 1e4 => 0.01,
        b if b < 1e7 => 0.001,
        b if b < 1e10 => 1e-4,
        b if b < 1e13 => 1e-5,
        _ => 1e-6,
    };
    if piv_tol * big * scale <= *pivot { Ok(chosen) } else { Err(Reentry) }
}

/// After the ratio test the original re-runs it in the opposite direction when it found nothing (`theta == I`); that
/// path is outside the step decision and is refused instead of guessed.
#[derive(Debug, PartialEq, Eq)]
pub struct Recall;

/// Step-loop constants written once from `h` (the Harris slack the ratio test also gets).
#[derive(Debug, Clone, Copy)]
pub struct PushTol {
    pub e: f64,
    pub reject: f64,
    pub reject_neg: f64,
    pub leave: f64,
}

impl PushTol {
    pub fn new(h: f64) -> Self {
        Self { e: h * 0.01, reject: 100.0 * h, reject_neg: -100.0 * h, leave: h + h }
    }
}

/// Entering column `j` before the ratio test: reduced cost, distance to the lower and upper bound, and whether it
/// moves down (the original's direction 1).
#[derive(Debug, Clone, Copy)]
pub struct PushEntry {
    pub r: f64,
    pub dl: f64,
    pub du: f64,
    pub decrease: bool,
}

/// What the step does with the ratio result. `Reject` is a flip with `theta = I`; `Defer` skips the pivot.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PushStep {
    Flip,
    Reject,
    Pivot,
    Defer,
}

/// Direction of entering column `j`. `x` holds the n structural values and `slack` the row slacks; `cost`/`lb`/`ub`
/// span all n+m columns. `(idx, val)` is the sparse FTRAN column; its signs are flipped when the column decreases, as
/// the ratio test then expects. `violated` carries the basic row classes when some basic value is out of its bounds;
/// the original then weighs the column's pull on the `B`/`A` rows too.
#[allow(clippy::too_many_arguments)]
pub fn push_direction(j: usize, x: &[f64], slack: &[f64], lb: &[f64], ub: &[f64], cost: &[f64], header: &[i32],
                      idx: &[i32], val: &mut [f64], tol: &PushTol, inf: f64, violated: Option<&[u8]>) -> PushEntry {
    let n = x.len();
    let (dl, du) = if j < n { (x[j] - lb[j], ub[j] - x[j]) } else { (slack[j - n], inf) };
    let mut r = cost[j];
    for (&i, &a) in idx.iter().zip(val.iter()) {
        r -= a * cost[header[i as usize] as usize];
    }
    let e = tol.e;
    let decrease = match violated {
        // The second clause is branched as `!(..)` in the original, so an unordered compare falls through to `0 < r`.
        None => (10.0 * dl < du && -(r * dl) < e) || (!(dl > 10.0 * du && e > du * r) && 0.0 < r),
        Some(marks) => {
            let mut s = 0.0;
            for (&i, &a) in idx.iter().zip(&*val) {
                match marks[i as usize] {
                    b'B' => s += a,
                    b'A' => s -= a,
                    _ => {}
                }
            }
            violated_decrease(r, dl, du, s, tol)
        }
    };
    if decrease {
        val.iter_mut().for_each(|v| *v = -*v);
    }
    PushEntry { r, dl, du, decrease }
}

/// Violated-basis direction with `s` the sum of the column over `B` rows minus over `A` rows. Each test is a taken
/// `ja` in the original, so an unordered compare never takes it.
fn violated_decrease(r: f64, dl: f64, du: f64, s: f64, tol: &PushTol) -> bool {
    let (k, e) = (tol.reject, tol.e);
    let (tl, tu) = (10.0 * dl, 10.0 * du); // computed once into xmm0/xmm6 at 0x1c22c1/0x1c2311
    if r > k && k > dl {
        return true;
    }
    if tol.reject_neg > r && k > du {
        return false;
    }
    if du > tl && e > -(dl * s) && e > -(r * dl) {
        return true;
    }
    if dl > tu && e > du * s && e > du * r {
        return false;
    }
    if e > s.abs() {
        if du > tl && e > -(r * dl) {
            return true;
        }
        if dl > tu && e > du * r {
            return false;
        }
        return r > 0.0;
    }
    s > 0.0
}

/// Step kind after the ratio test returned position `pos` with `theta` and `|pivot|`. A rejection writes
/// `theta = inf`.
pub fn push_step(entry: &PushEntry, tol: &PushTol, inf: f64, pos: i32, theta: &mut f64, pivot: f64)
                 -> Result<PushStep, Recall> {
    if pos < 0 || *theta == inf {
        return Err(Recall);
    }
    let gap = if entry.decrease { entry.dl } else { entry.du };
    if gap - 1e-10 <= *theta {
        return Ok(PushStep::Flip);
    }
    let reject = if entry.decrease {
        entry.r > tol.reject && entry.dl < tol.reject
    } else {
        entry.r < tol.reject_neg && entry.du < tol.reject
    };
    if reject {
        *theta = inf;
        return Ok(PushStep::Reject);
    }
    Ok(if 1e-6 > pivot || -1e-5 > *theta * pivot { PushStep::Defer } else { PushStep::Pivot })
}

/// Nonbasic status (-1 lower, -2 upper, -4 fixed) of the variable leaving on an accepted pivot. `range` is its
/// `ub - lb`; `a` the signed pivot entry after the direction flip; `mark`/`basic`/`width` describe the pivot row.
#[allow(clippy::too_many_arguments)]
pub fn push_leave_state(range: f64, fixed: f64, a: f64, theta: f64, mark: u8, basic: f64, width: f64, tol: &PushTol)
                        -> i32 {
    let (t, k) = (theta, tol.leave);
    if range <= fixed {
        return -4;
    }
    if 0.0 > a {
        match mark {
            b'N' | b'U' => -2,
            b'B' => if (t + 1.0) * k > (basic / a - t).abs() { -1 } else { -2 },
            b'X' => -4,
            b'L' => if k > a.abs() * t { -1 } else { -2 },
            _ => -1,
        }
    } else {
        match mark {
            b'L' | b'N' => -1,
            b'X' => -4,
            b'A' if (t + 1.0) * k > ((basic - width) / a - t).abs() => -2,
            b'U' if k > a.abs() * t => -2,
            _ => -1,
        }
    }
}
