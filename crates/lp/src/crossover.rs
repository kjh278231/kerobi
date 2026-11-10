//! Crossover primal push pieces: the ratio tests on a feasible (docs/m3/primal_push_ratio_rust.md) and a violated basis
//! (docs/m3/primal_push_violated.md) with its retry (docs/m3/primal_push_retry.md), and the step decisions around them
//! (docs/m3/primal_push_decision_rust.md). No push loop calls them yet.

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
/// The push sends a result with position < 0 or |pivot| < 0.001 on to [`retry_ratio`]; use [`violated_ratio`].
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
    small_pivot(piv_tol, val, *pivot, chosen)
}

/// Accepts `chosen` unless its pivot is below 0.1 and below the tolerance scaled by the column's largest entry; the
/// original then re-runs its first pass with that tightened tolerance.
fn small_pivot(piv_tol: f64, val: &[f64], pivot: f64, chosen: i32) -> Result<i32, Reentry> {
    if pivot >= 0.1 {
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
    if piv_tol * big * scale <= pivot { Ok(chosen) } else { Err(Reentry) }
}

/// The push's ratio test on a violated basis: [`infeasible_ratio`], then [`retry_ratio`] on the same column when that
/// found no row or only a pivot below 0.001.
#[allow(clippy::too_many_arguments)]
pub fn violated_ratio(harris: f64, piv_tol: f64, inf: f64, cap: f64, idx: Option<&[i32]>, val: &[f64], x: &[f64],
                      width: &[f64], marks: &[u8], work_coef: f64, work: &mut f64, theta: &mut f64, pivot: &mut f64)
                      -> Result<i32, Reentry> {
    let pos = infeasible_ratio(harris, piv_tol, inf, cap, idx, val, x, width, marks, work_coef, work, theta, pivot)?;
    if pos < 0 || 0.001 > pivot.abs() {
        return retry_ratio(harris, piv_tol, inf, cap, idx, val, x, width, marks, work_coef, work, theta, pivot);
    }
    Ok(pos)
}

/// Piecewise-linear retry ratio test (`056973`) with the arguments of [`infeasible_ratio`]. The breakpoints are where a
/// basic row enters or leaves its bounds; a quickselect finds the one where the total infeasibility slope stops
/// falling (0 when it never falls), and the row nearest to it within `harris` with the largest |entry| leaves, else the
/// smallest breakpoint.
#[allow(clippy::too_many_arguments)]
pub fn retry_ratio(harris: f64, piv_tol: f64, inf: f64, cap: f64, idx: Option<&[i32]>, val: &[f64], x: &[f64],
                   width: &[f64], marks: &[u8], work_coef: f64, work: &mut f64, theta: &mut f64, pivot: &mut f64)
                   -> Result<i32, Reentry> {
    let count = idx.map_or(x.len(), |i| i.len());
    let row = |k: usize| idx.map_or(k, |i| i[k] as usize);
    let (mut pos, mut rat) = (Vec::new(), Vec::new());
    let (mut ab, mut at_bound, mut tested) = (0.0, 0.0, 0i32);
    for k in 0..count {
        let (a, r) = (val[k], row(k));
        let (v, w) = (x[r], width[r]);
        let mut cand = |q: f64, filter: bool| {
            if !filter || cap > q {
                pos.push(k);
                rat.push(q);
            }
        };
        if a > piv_tol {
            tested += 1;
            match marks[r] {
                b'A' => {
                    cand((v - w) / a, true);
                    cand(v / a, true);
                    ab -= a;
                }
                b'B' => ab += a,
                b'L' | b'X' => at_bound += a,
                b'N' | b'U' => cand(v / a, idx.is_none()), // the sparse form skips the cap test here
                _ => {}
            }
        } else if -piv_tol > a {
            tested += 1;
            match marks[r] {
                b'A' => ab -= a,
                b'B' => {
                    cand(v / a, true);
                    if inf > w {
                        cand((v - w) / a, true);
                    }
                    ab += a;
                }
                b'L' | b'N' if inf > w => cand((v - w) / a, true),
                b'U' | b'X' => at_bound -= a,
                _ => {}
            }
        }
    }
    let (c, n) = (pos.len() as f64, count as f64);
    *work += (if idx.is_none() { c * 4.0 + tested as f64 + n } else { c * 4.0 + n + n }) * work_coef;
    let mut slope = at_bound + ab;
    if pos.is_empty() && 0.0 > slope {
        *theta = if cap < inf { cap } else { inf };
        *pivot = 0.0;
        return Ok(-1);
    }
    let mut t = 0.0;
    if 0.0 > slope {
        let (mut lo, mut hi) = (0usize, pos.len() - 1);
        while lo < hi {
            let p = rat[lo];
            let (mut i, mut j) = (lo as isize - 1, hi as isize + 1);
            loop {
                j -= 1;
                while rat[j as usize] > p {
                    j -= 1;
                }
                i += 1;
                while p > rat[i as usize] {
                    i += 1;
                }
                if i >= j {
                    break;
                }
                rat.swap(i as usize, j as usize);
                pos.swap(i as usize, j as usize);
            }
            let j = j as usize;
            if 0.0 > slope {
                slope = lane_sum(slope, &pos[lo..=j], val, false);
            } else if j < hi {
                slope = lane_sum(slope, &pos[j + 1..=hi], val, true);
            }
            if 0.0 > slope { lo = j + 1 } else { hi = j }
        }
        t = rat[lo];
        if inf > cap && cap <= t {
            *theta = cap;
            *pivot = 0.0;
            return Ok(-1);
        }
    }
    let (mut best, mut chosen) = (0.0, -1i32);
    for k in 0..count {
        let (a, r) = (val[k], row(k));
        let (v, w) = (x[r], width[r]);
        let near = |num: f64| (num - t * a).abs() * 2.0 <= harris;
        let at = |num: f64| (num - t * a).abs() <= harris; // rows already at a bound get no factor 2
        let (num, size) = if a > piv_tol {
            let num = match marks[r] {
                b'N' | b'U' if near(v) => Some(v),
                b'A' if near(v - w) => Some(v - w),
                b'A' if near(v) => Some(v),
                b'L' | b'X' if at(v) => Some(v),
                _ => None,
            };
            (num, a)
        } else if -piv_tol > a {
            let num = match marks[r] {
                b'L' | b'N' if inf > w && near(v - w) => Some(v - w),
                b'B' if near(v) => Some(v),
                b'B' if inf > w && near(v - w) => Some(v - w),
                b'U' | b'X' if at(v - w) => Some(v - w),
                _ => None,
            };
            (num, -a)
        } else {
            (None, 0.0)
        };
        if let Some(num) = num {
            if size > best {
                *theta = num / a;
                best = size;
                chosen = k as i32;
            }
        }
    }
    *work += (if idx.is_none() { n * 3.0 } else { n * 4.0 }) * work_coef;
    if chosen < 0 {
        *theta = inf;
        let mut low = inf;
        for (&q, &k) in rat.iter().zip(&pos) {
            if low > q {
                *theta = q;
                low = q;
                chosen = k as i32;
            }
        }
        if chosen < 0 {
            *pivot = 0.0;
            return Ok(-1);
        }
    }
    *pivot = val[chosen as usize].abs();
    small_pivot(piv_tol, &val[..count], *pivot, chosen)
}

/// `seed` plus (or minus) the |entries| of the column at `pos`, summed the way the original vectorizes it: eight
/// elements per block into four two-lane accumulators, combined as (A+B)+(C+D) and then across lanes.
fn lane_sum(seed: f64, pos: &[usize], val: &[f64], sub: bool) -> f64 {
    let step = |s: f64, k: usize| if sub { s - val[pos[k]].abs() } else { s + val[pos[k]].abs() };
    let blocks = if pos.len() >= 8 { pos.len() & !7 } else { 0 };
    let mut s = seed;
    if blocks > 0 {
        let mut acc = [[seed, 0.0], [0.0; 2], [0.0; 2], [0.0; 2]];
        for b in (0..blocks).step_by(8) {
            for (q, lanes) in acc.iter_mut().enumerate() {
                for (l, lane) in lanes.iter_mut().enumerate() {
                    *lane = step(*lane, b + 2 * q + l);
                }
            }
        }
        let lane = |l: usize| (acc[0][l] + acc[1][l]) + (acc[2][l] + acc[3][l]);
        s = lane(0) + lane(1);
    }
    (blocks..pos.len()).fold(s, step)
}

/// After the ratio test the original re-runs it in the opposite direction when it found nothing (`theta == I`) and the
/// column is free or increases without an upper bound; that path (M3-04c-8) is refused instead of guessed.
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
    /// `theta == I` sends the step to the opposite-direction recall instead of a distance flip.
    pub recall: bool,
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
    let unbounded_up = inf <= ub[j];
    let recall = (lb[j] <= -inf && unbounded_up) || (!decrease && unbounded_up);
    PushEntry { r, dl, du, decrease, recall }
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
/// `theta = inf`. Without a leaving row (`theta == I` and no recall) the step is a flip.
pub fn push_step(entry: &PushEntry, tol: &PushTol, inf: f64, pos: i32, theta: &mut f64, pivot: f64)
                 -> Result<PushStep, Recall> {
    if *theta == inf && entry.recall {
        return Err(Recall);
    }
    let gap = if entry.decrease { entry.dl } else { entry.du };
    if gap - 1e-10 <= *theta || pos < 0 {
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
