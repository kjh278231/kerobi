//! Crossover primal push pieces: the ratio tests on a feasible (docs/m3/primal_push_ratio_rust.md) and a violated basis
//! (docs/m3/primal_push_violated.md) with its retry (docs/m3/primal_push_retry.md), and the step decisions around them
//! (docs/m3/primal_push_decision_rust.md, docs/m3/primal_push_recall.md). All three ratio tests run their small-pivot
//! second pass (docs/m3/primal_push_reentry.md, docs/m3/primal_push_violated_reentry.md). `push_pick`/`push_apply` chain
//! them into one push step (docs/m3/primal_push_step_chain.md); no push loop calls those yet.

/// Feasible-basis ratio test (`055f68`) of the primal push on the FTRAN column `(idx, val)`: `idx = None` is the dense
/// form with `val.len() == x.len()`. `x` is the lower-shifted basic value, `width` the bound width, `marks` the basic
/// class. Returns the position in `val` of the leaving entry (-1 for none) and writes `theta`/`pivot` only where the
/// original does; `work += units * work_coef` once per pass.
///
/// A chosen pivot below 0.1 and below `(piv_tol * max|val|) * scale` re-runs the first pass once with the tolerance
/// `piv_tol * (max|val| * scale)` (the original's multiplication order). When that second pass finds no row or only a
/// bound of `inf`, `theta`/`pivot` keep the first pass's values, the position is the last row the second pass recorded
/// (else the first pass's pick), and `factor_updates()` (`12dd49`: the factor's update count, -1 without a factor) is
/// asked; above 16 `*refactor` is set: the caller then writes refactor request 2 (S+0x294) and leaving -1 (S+0x2f8).
/// shortcut: no column-length weighting of the second pass (the push passes 0); add it for the other callers.
#[allow(clippy::too_many_arguments)]
pub fn feasible_ratio(harris: f64, piv_tol: f64, inf: f64, cap: f64, idx: Option<&[i32]>, val: &[f64], x: &[f64],
                      width: &[f64], marks: &[u8], work_coef: f64, work: &mut f64, theta: &mut f64, pivot: &mut f64,
                      factor_updates: &mut dyn FnMut() -> i32, refactor: &mut bool) -> i32 {
    let count = idx.map_or(x.len(), |i| i.len());
    let row = |k: usize| idx.map_or(k, |i| i[k] as usize);
    let mut tol = piv_tol;
    let mut chosen = -1i32;
    let mut first = true;
    loop {
        let (mut ratios, mut cands) = (Vec::new(), Vec::new());
        let mut tested = 0i32;
        let mut bound = cap;
        for k in 0..count {
            let (a, r) = (val[k], row(k));
            let up = a > tol && marks[r] != b'F'; // NaN entries are neither up nor down
            let num = if up {
                x[r]
            } else {
                if !(-tol > a && inf > width[r]) {
                    continue;
                }
                x[r] - width[r]
            };
            tested += 1;
            let ratio = num / a;
            if ratio <= bound {
                ratios.push(ratio);
                cands.push(k);
                chosen = k as i32;
                let relaxed = if up { (num + harris) / a } else { (num - harris) / a };
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
            return -1;
        }
        if bound == inf || cands.is_empty() {
            if first {
                *pivot = 0.0;
                *theta = inf;
                return -1;
            }
            if factor_updates() > 16 {
                *refactor = true;
            }
            return chosen;
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
        match first.then(|| tightened(tol, &val[..count], *pivot)).flatten() {
            Some(t) => tol = t,
            None => return chosen,
        }
        first = false;
    }
}

/// The small-pivot check after a first pass: `Some(tolerance)` for the second pass when the pivot is below 0.1 and
/// below `(tol * max|val|) * scale`, the new tolerance being `tol * (max|val| * scale)` (the original's order). NaN
/// pivots and thresholds accept, as the original's `jbe`.
fn tightened(tol: f64, val: &[f64], pivot: f64) -> Option<f64> {
    if 0.1 > pivot {
        // maxsd: a NaN entry makes the running maximum NaN, and the next entry replaces a NaN maximum
        let big = val.iter().fold(0.0f64, |m, v| if m > v.abs() { m } else { v.abs() });
        let scale = pivot_scale(big);
        return (tol * big * scale > pivot).then_some(tol * (big * scale));
    }
    None
}

/// Tolerance factor of the small-pivot check by the column's largest |entry| (NaN takes the last one).
fn pivot_scale(big: f64) -> f64 {
    match big {
        b if 10.0 > b => 0.1,
        b if 1e4 > b => 0.01,
        b if 1e7 > b => 0.001,
        b if 1e10 > b => 1e-4,
        b if 1e13 > b => 1e-5,
        _ => 1e-6,
    }
}

/// Violated-basis ratio test (`054eef`): like [`feasible_ratio`] but each basic row's class decides which bound it
/// hits, so `B` rows block a decreasing and `A` rows an increasing basic value. The dense form tests `U`/`X` rows on a
/// negative entry with a strict `<`, as the original does; the sparse form uses `<=` there like everywhere else.
/// The push sends a result with position < 0 or |pivot| < 0.001 on to [`retry_ratio`]; use [`violated_ratio`].
/// The small-pivot second pass and its refactor request are those of [`feasible_ratio`].
#[allow(clippy::too_many_arguments)]
pub fn infeasible_ratio(harris: f64, piv_tol: f64, inf: f64, cap: f64, idx: Option<&[i32]>, val: &[f64], x: &[f64],
                        width: &[f64], marks: &[u8], work_coef: f64, work: &mut f64, theta: &mut f64, pivot: &mut f64,
                        factor_updates: &mut dyn FnMut() -> i32, refactor: &mut bool) -> i32 {
    let count = idx.map_or(x.len(), |i| i.len());
    let row = |k: usize| idx.map_or(k, |i| i[k] as usize);
    let (mut tol, mut first, mut chosen) = (piv_tol, true, -1i32);
    loop {
        let (mut ratios, mut cands) = (Vec::new(), Vec::new());
        let mut tested = 0i32;
        let mut bound = cap;
        for k in 0..count {
            let (a, r) = (val[k], row(k));
            let (up, down) = (a > tol, a < -tol);
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
        if cap < inf && cap <= bound {
            *pivot = 0.0;
            *theta = cap;
            return -1;
        }
        if bound == inf || cands.is_empty() {
            if first {
                *pivot = 0.0;
                *theta = inf;
                return -1;
            }
            if factor_updates() > 16 {
                *refactor = true;
            }
            return chosen; // the second pass's last recorded row, else the first pass's pick
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
        match first.then(|| tightened(tol, &val[..count], *pivot)).flatten() {
            Some(t) => tol = t,
            None => return chosen,
        }
        first = false;
    }
}

/// The push's ratio test on a violated basis: [`infeasible_ratio`], then [`retry_ratio`] on the same column when that
/// found no row or only a pivot below 0.001. Either may set `*refactor` (request 2); the push then writes the
/// returned position over leaving (S+0x2f8) anyway.
#[allow(clippy::too_many_arguments)]
pub fn violated_ratio(harris: f64, piv_tol: f64, inf: f64, cap: f64, idx: Option<&[i32]>, val: &[f64], x: &[f64],
                      width: &[f64], marks: &[u8], work_coef: f64, work: &mut f64, theta: &mut f64, pivot: &mut f64,
                      factor_updates: &mut dyn FnMut() -> i32, refactor: &mut bool) -> i32 {
    let pos = infeasible_ratio(harris, piv_tol, inf, cap, idx, val, x, width, marks, work_coef, work, theta, pivot,
                               factor_updates, refactor);
    if pos < 0 || 0.001 > pivot.abs() {
        return retry_ratio(harris, piv_tol, inf, cap, idx, val, x, width, marks, work_coef, work, theta, pivot,
                           factor_updates, refactor);
    }
    pos
}

/// Piecewise-linear retry ratio test (`056973`) with the arguments of [`infeasible_ratio`]. The breakpoints are where a
/// basic row enters or leaves its bounds; a quickselect finds the one where the total infeasibility slope stops
/// falling (0 when it never falls), and the row nearest to it within `harris` with the largest |entry| leaves, else the
/// smallest breakpoint.
///
/// A small pivot re-runs everything once with the tolerance of [`feasible_ratio`]. When that second run finds no
/// breakpoint while the slope falls, `theta`/`pivot` keep the first run's values and the first run's pick is returned,
/// unless `factor_updates()` exceeds 16: then `*refactor` is set and the result is -1.
#[allow(clippy::too_many_arguments)]
pub fn retry_ratio(harris: f64, piv_tol: f64, inf: f64, cap: f64, idx: Option<&[i32]>, val: &[f64], x: &[f64],
                   width: &[f64], marks: &[u8], work_coef: f64, work: &mut f64, theta: &mut f64, pivot: &mut f64,
                   factor_updates: &mut dyn FnMut() -> i32, refactor: &mut bool) -> i32 {
    let count = idx.map_or(x.len(), |i| i.len());
    let row = |k: usize| idx.map_or(k, |i| i[k] as usize);
    let (mut piv_tol, mut first, mut prev) = (piv_tol, true, -1i32);
    loop {
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
            if !first {
                if factor_updates() > 16 {
                    *refactor = true;
                    return -1;
                }
                return prev;
            }
            *theta = if cap < inf { cap } else { inf };
            *pivot = 0.0;
            return -1;
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
                return -1;
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
                return -1;
            }
        }
        *pivot = val[chosen as usize].abs();
        match first.then(|| tightened(piv_tol, &val[..count], *pivot)).flatten() {
            Some(t) => piv_tol = t,
            None => return chosen,
        }
        first = false;
        prev = chosen;
    }
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

/// The ratio test found nothing (`theta == I`) and the column is free or increases without an upper bound: the caller
/// flips the column's signs back, runs the ratio test in the opposite direction and finishes with [`push_recall_step`].
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
    /// `lb <= -I` and `ub >= I`.
    pub free: bool,
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
    let free = lb[j] <= -inf && unbounded_up;
    let recall = free || (!decrease && unbounded_up);
    PushEntry { r, dl, du, decrease, recall, free }
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
    Ok(step_tail(entry, entry.decrease, tol, inf, pos, theta, pivot))
}

/// Step kind after a [`Recall`]: `pos`/`theta`/`pivot` come from the ratio test in the opposite direction, with
/// `theta = I` when the feasible ratio test found nothing (the original writes it again). The distance check uses the
/// new direction's gap; the rejection still uses the direction formula's result.
pub fn push_recall_step(entry: &PushEntry, tol: &PushTol, inf: f64, pos: i32, theta: &mut f64, pivot: f64) -> PushStep {
    let pos = if *theta == inf && entry.free { -1 } else { pos };
    step_tail(entry, !entry.decrease, tol, inf, pos, theta, pivot)
}

/// Status of entering column `j` after a flip: -1/-2 at the bound it moved to (`decrease` is the current direction),
/// -3 when both bounds are infinite and it stays superbasic.
pub fn push_flip_state(lb: f64, ub: f64, inf: f64, decrease: bool) -> i32 {
    if -inf < lb || ub < inf {
        if decrease { -1 } else { -2 }
    } else {
        -3
    }
}

fn step_tail(entry: &PushEntry, decrease: bool, tol: &PushTol, inf: f64, pos: i32, theta: &mut f64, pivot: f64)
             -> PushStep {
    let gap = if decrease { entry.dl } else { entry.du };
    if gap - 1e-10 <= *theta || pos < 0 {
        return PushStep::Flip;
    }
    // The original picks the rejection arm from the direction formula's result (r13b), not the current direction.
    let reject = if entry.decrease {
        entry.r > tol.reject && entry.dl < tol.reject
    } else {
        entry.r < tol.reject_neg && entry.du < tol.reject
    };
    if reject {
        *theta = inf;
        return PushStep::Reject;
    }
    if 1e-6 > pivot || -1e-5 > *theta * pivot { PushStep::Defer } else { PushStep::Pivot }
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

/// Workspace LP a push step reads: CSC of the n structural columns, and cost/bounds over all n+m columns.
pub struct PushLp<'a> {
    pub n: usize,
    pub m: usize,
    pub starts: &'a [i64],
    pub lengths: &'a [i32],
    pub rows: &'a [i32],
    pub values: &'a [f64],
    pub cost: &'a [f64],
    pub lb: &'a [f64],
    pub ub: &'a [f64],
}

/// Push constants: `h` (S+0x380) for the Harris slack, step constants and basic row classes, the ratio pivot tolerance
/// (S+0x398), the internal infinity `inf` (S+0x370) and the fixed-width threshold (S+0x3b0).
#[derive(Debug, Clone, Copy)]
pub struct PushParams {
    pub h: f64,
    pub piv_tol: f64,
    pub inf: f64,
    pub fixed: f64,
}

/// State one push step reads and writes. `xs` holds the n structural values followed by the m row slacks (one
/// allocation in the original). `theta`/`pivot` are the step locals the ratio tests write into.
#[derive(Debug, Clone)]
pub struct PushState {
    pub vstat: Vec<i32>,
    pub header: Vec<i32>,
    pub xs: Vec<f64>,
    pub basic: Vec<f64>,
    pub width: Vec<f64>,
    pub marks: Vec<u8>,
    pub rhs: Vec<f64>,
    /// Number of basic rows marked `B` or `A`; nonzero sends the step down the violated path.
    pub violations: i32,
    /// S+0x2a0.
    pub iterations: f64,
    /// S+0x2fc, S+0x2f8, S+0x328, S+0x32c, S+0x294.
    pub enter: i32,
    pub leave: i32,
    pub direction: i32,
    pub leave_col: i32,
    pub request: i32,
    pub theta: f64,
    pub pivot: f64,
}

/// What [`push_pick`] decided: the direction formula's entry, the ratio results (the second one after a recall)
/// and the step kind.
#[derive(Debug, Clone, Copy)]
pub struct PushPick {
    pub entry: PushEntry,
    pub violated: bool,
    pub first: (i32, f64, f64),
    pub second: Option<(i32, f64, f64)>,
    pub step: PushStep,
    /// S+0x2f8, S+0x328 and S+0x294 as each ratio call saw them.
    pub seen: [(i32, i32, i32); 2],
}

/// Basic row class from the lower-shifted value `v` and width `w`; `B`/`A` also count as violations.
fn classify(v: f64, w: f64, h: f64, inf: f64, violations: &mut i32) -> u8 {
    if -h <= v {
        if h <= v {
            if v < w - h || inf <= w {
                b'N'
            } else if w + h <= v {
                *violations += 1;
                b'A'
            } else {
                b'U'
            }
        } else if w == 0.0 {
            b'X'
        } else {
            b'L'
        }
    } else {
        *violations += 1;
        b'B'
    }
}

/// Step selection of the double primal push for entering column `j` with its sparse FTRAN column `(idx, val)` as the
/// supplier returned it: direction, ratio test, the theta = I recall in the opposite direction and the step kind.
/// Writes S+0x2fc/0x328/0x2f8 (and S+0x294 on a ratio's refactor request) in the original's order and leaves `val`
/// with the signs of the final direction. The ratio tests run with a zero work coefficient.
pub fn push_pick(lp: &PushLp, st: &mut PushState, par: &PushParams, j: usize, idx: &[i32], val: &mut [f64],
                 factor_updates: &mut dyn FnMut() -> i32) -> PushPick {
    let (n, inf) = (lp.n, par.inf);
    let tol = PushTol::new(par.h);
    st.enter = j as i32;
    st.direction = 0;
    let violated = st.violations != 0;
    let entry = push_direction(j, &st.xs[..n], &st.xs[n..], lp.lb, lp.ub, lp.cost, &st.header, idx, val, &tol, inf,
                               violated.then_some(&st.marks[..]));
    if entry.decrease {
        st.direction = 1;
    }
    let mut seen = [(0, 0, 0); 2];
    let mut ratio = |st: &mut PushState, val: &[f64], call: usize| {
        seen[call] = (st.leave, st.direction, st.request);
        let (mut work, mut refactor) = (0.0, false);
        let f = if violated { violated_ratio } else { feasible_ratio };
        let pos = f(par.h, par.piv_tol, inf, inf, Some(idx), val, &st.basic, &st.width, &st.marks, 0.0, &mut work,
                    &mut st.theta, &mut st.pivot, factor_updates, &mut refactor);
        if refactor {
            st.request = 2;
            st.leave = -1;
        }
        if violated || pos != -1 {
            st.leave = pos; // 0x1c2655 / 0x1c4d0c
        } else {
            st.theta = inf; // feasible -1 leaves S+0x2f8 alone
        }
        (pos, st.theta, st.pivot)
    };
    let first = ratio(st, val, 0);
    let mut theta = st.theta;
    let (second, step) = match push_step(&entry, &tol, inf, st.leave, &mut theta, st.pivot) {
        Ok(step) => (None, step),
        Err(Recall) => {
            st.direction = 1 - st.direction;
            val.iter_mut().for_each(|v| *v = -*v); // 0x1c49e2
            let second = ratio(st, val, 1);
            if st.theta == inf && entry.free {
                st.leave = -1;
            }
            theta = st.theta;
            (Some(second), push_recall_step(&entry, &tol, inf, st.leave, &mut theta, st.pivot))
        }
    };
    st.theta = theta;
    if matches!(step, PushStep::Flip | PushStep::Reject) {
        st.leave = -1; // 0x1c293b; the other flip paths already hold a negative S+0x2f8
    }
    PushPick { entry, violated, first, second, step, seen }
}

/// Applies a flip or an accepted pivot of [`push_pick`]: statuses and header, the pivot row's width/value/class
/// (`applied` sees the state at the join point 0x1c3004), then the rhs of the moved columns and the remaining basic
/// rows of the column. A deferred step changes nothing. `(idx, val)` loses the pivot position like the original's
/// sparse column.
#[allow(clippy::too_many_arguments)]
pub fn push_apply(lp: &PushLp, st: &mut PushState, par: &PushParams, pick: &PushPick, idx: &mut Vec<i32>,
                  val: &mut Vec<f64>, applied: &mut dyn FnMut(&PushState)) {
    let (n, inf, h) = (lp.n, par.inf, par.h);
    let j = st.enter as usize;
    let down = st.direction != 0;
    let theta = st.theta;
    let shift; // dVar49: how far the basic rows move along the column
    match pick.step {
        PushStep::Defer => return,
        PushStep::Flip | PushStep::Reject => {
            st.iterations += 1.0;
            let s = push_flip_state(lp.lb[j], lp.ub[j], inf, down);
            st.vstat[j] = s;
            shift = if s != -3 {
                if down { pick.entry.dl } else { pick.entry.du }
            } else if down {
                st.xs[j]
            } else {
                -st.xs[j]
            };
        }
        PushStep::Pivot => {
            let p = st.leave as usize;
            st.iterations += 1.0;
            let mut a = val[p];
            let r = idx[p] as usize;
            st.leave = r as i32;
            idx.swap_remove(p);
            val.swap_remove(p);
            st.vstat[j] = r as i32;
            let lv = st.header[r] as usize;
            st.leave_col = lv as i32;
            st.header[r] = j as i32;
            st.vstat[lv] = push_leave_state(lp.ub[lv] - lp.lb[lv], par.fixed, a, theta, st.marks[r], st.basic[r],
                                            st.width[r], &PushTol::new(h));
            st.width[r] = if lp.ub[j] < inf { lp.ub[j] - lp.lb[j] } else { inf };
            if down {
                a = -a;
            }
            if matches!(st.marks[r], b'B' | b'A') {
                st.violations -= 1;
            }
            let b = if st.vstat[lv] == -1 { st.basic[r] } else { (st.basic[r] - lp.ub[lv]) + lp.lb[lv] };
            let mut b = b / a;
            shift = if down { -b } else { b };
            if j < n {
                b += st.xs[j];
                if -inf < lp.lb[j] {
                    b -= lp.lb[j];
                }
            } else {
                b += st.xs[j];
            }
            st.basic[r] = b;
            st.marks[r] = if -inf < lp.lb[j] { classify(b, st.width[r], h, inf, &mut st.violations) } else { b'F' };
        }
    }
    applied(st);
    let pivot = pick.step == PushStep::Pivot;
    let add_col = |rhs: &mut [f64], c: usize, d: f64| {
        let s = lp.starts[c] as usize;
        for q in s..s + lp.lengths[c] as usize {
            let i = lp.rows[q] as usize;
            rhs[i] += d * lp.values[q]; // the original adds the product to the old value; + commutes bitwise
        }
    };
    if j < n {
        let mut d = st.xs[j];
        if !pivot {
            match st.vstat[j] {
                -1 => d -= lp.lb[j],
                -3 => {}
                _ => d -= lp.ub[j],
            }
        }
        if d != 0.0 {
            add_col(&mut st.rhs, j, d);
        }
    } else {
        st.rhs[j - n] += st.xs[j];
    }
    if pivot && (st.leave_col as usize) < n {
        let lc = st.leave_col as usize;
        let bound = if st.vstat[lc] == -1 { lp.lb[lc] } else { lp.ub[lc] };
        if -bound != 0.0 {
            add_col(&mut st.rhs, lc, -bound);
        }
    }
    for (&i, &a) in idx.iter().zip(val.iter()) {
        let r = i as usize;
        let v = st.basic[r] - shift * a;
        st.basic[r] = v;
        match st.marks[r] {
            b'F' => {}
            m => {
                if matches!(m, b'B' | b'A') {
                    st.violations -= 1;
                }
                st.marks[r] = classify(v, st.width[r], h, inf, &mut st.violations);
            }
        }
    }
}
