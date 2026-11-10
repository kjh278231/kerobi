//! Crossover primal push pieces. Only the feasible-basis ratio test exists so far (docs/m3/primal_push_ratio_rust.md).

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
    for (&ratio, &k) in ratios.iter().zip(&cands) {
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
    let big = val[..count].iter().fold(0.0f64, |m, v| m.max(v.abs()));
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
