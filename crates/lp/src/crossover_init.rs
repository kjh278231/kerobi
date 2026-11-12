//! Initialization of the double serial primal push (`134a87` before the first pass preparation 0x1c1ff8): the first
//! push's factor call, the RHS and candidate scan, the candidate sort, the basic solve with its residual check and
//! Markowitz tightening, the basic classification, the quality gate, the one-time tolerance adjustment and the loop
//! state. The factor, its RHS FTRAN and the Markowitz setting stay behind [`PushInitBackend`].
//!
//! Values the captures do not hold (`param_1`, S+0x120, S+0x3c8 and the environment flag of `061bfe`, the condition
//! estimate) are `Option`s: a branch that needs a missing one panics, except the condition, whose terms of the quality
//! gate count as false when it is missing ([`InitResult::condition_known`]). Contract and C references:
//! docs/m3/primal_push_init_rust.md.
use crate::common::sort_desc;
use crate::crossover::{PushLp, PushParams, PushState};
use crate::crossover_loop::{renumber, FactorCtl, PushLoop, Renumber};
use crate::crossover_round::RoundState;

/// Numeric boundary of the initialization.
pub trait PushInitBackend {
    /// `05b257` with S+0x294 = 10 on the push state (header, S+0x2f8, vstat). Returns its value and, after a
    /// successful refactorization, the renumbering applied to the basis.
    fn initial_factor(&mut self, ctl: &mut FactorCtl, st: &PushState) -> (i32, Option<Renumber>);
    /// `09cf9b` with count -1: m raw values of B⁻¹ rhs in header order.
    fn rhs_ftran(&mut self, rhs: &[f64], raw: &mut [f64]);
    /// `12f5d1` has run: the Markowitz threshold S+0x390 for later refactorizations and, when `061bfe` lowered it,
    /// the update limit F+0x188.
    fn tighten(&mut self, markowitz: f64, limit: Option<i32>);
}

/// `061bfe` inputs: S+0x3c8 and the environment value at (S+0x4b8)+0x100 → +0x2c54.
#[derive(Clone, Copy, Debug)]
pub struct TightenInput {
    pub update_limit: i32,
    pub env_2c54: i32,
}

/// The push state at `134a87` entry that the initialization reads.
pub struct InitEntry<'a> {
    /// Constraint right-hand side S+0xa0 (m).
    pub rhs: &'a [f64],
    pub vstat: Vec<i32>,
    pub header: Vec<i32>,
    /// x (param_5) followed by the row slacks (param_6, the contiguous alias).
    pub xs: Vec<f64>,
    /// S+0x2a0.
    pub iterations: f64,
    /// S+0x294, S+0x178, S+0x184, S+0xf0.
    pub request: i32,
    pub numeric: i32,
    pub failure: i32,
    pub status: i32,
    /// S+0x2fc, S+0x2f8, S+0x328.
    pub enter: i32,
    pub leave: i32,
    pub direction: i32,
    /// S+0x390.
    pub markowitz: f64,
    /// param_7, param_4, param_3.
    pub first: bool,
    pub attempt: i32,
    pub model_status: i32,
    /// param_1 (FeasibilityTol argument of the threshold counts and the adjustment).
    pub tau_in: Option<f64>,
    /// S+0x120 at entry (PInf of the previous push).
    pub prev_pinf: Option<f64>,
    /// `*param_8` after `12f282` (first push) or as the caller left it.
    pub condition: Option<f64>,
    pub tighten: Option<TightenInput>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InitExit {
    /// Candidates exist: run the loop from the returned state.
    Loop,
    /// No candidate: the push goes straight to the solution install (`LAB_002c4050`).
    End,
    /// `*param_9 = 1`.
    Retry,
    /// A factor error code passed to the cleanup.
    Error(i32),
}

/// Values the initialization computed but the loop does not keep (the last pass through C 646–796).
#[derive(Debug, Clone, Copy, Default)]
pub struct InitDiag {
    pub rhs_max: f64,
    pub basic_max: f64,
    pub residual: f64,
    pub markowitz_candidate: f64,
}

pub struct InitResult {
    pub exit: InitExit,
    /// The loop state at 0x1c1ff8 (meaningful for `Loop`/`End`; for `Retry`/`Error` as far as it was built).
    pub state: PushLoop,
    /// local_f8 + 10 (entry S+0x178 plus 10).
    pub numeric_limit: i32,
    /// S+0x390 and S+0x3c8 after the initialization (the limit only when `061bfe` lowered it).
    pub markowitz: f64,
    pub update_limit: Option<i32>,
    pub adjusted: bool,
    /// False when the quality gate ran without a condition value.
    pub condition_known: bool,
    pub diag: InitDiag,
}

/// The rows `r -= v·a_j` over column j in CSC order (C 471–482, 1123–1133).
fn subtract_column(lp: &PushLp, j: usize, v: f64, r: &mut [f64]) {
    let s = lp.starts[j] as usize;
    for k in s..s + lp.lengths[j] as usize {
        let i = lp.rows[k] as usize;
        r[i] -= v * lp.values[k];
    }
}

/// Distance of a candidate column (C 459–465; asm `comisd 2x, ub+lb; jbe` takes `ub - x` when unordered).
fn distance(x: f64, lb: f64, ub: f64) -> f64 {
    if ub + lb > x + x {
        x - lb
    } else {
        ub - x
    }
}

/// The threshold counts of C 432–452 / 573–583: gaps above τ·{0.001, 0.01, 0.1}.
fn count(counts: &mut Option<(f64, [i32; 3])>, gap: f64) {
    if let Some((tau, c)) = counts {
        for (k, f) in [0.001, 0.01, 0.1].into_iter().enumerate() {
            if *tau * f < gap {
                c[k] += 1;
            }
        }
    }
}

/// `061bfe` (double): returns the lowered S+0x3c8/F+0x188 if any.
fn tightened_limit(m: usize, t: TightenInput) -> Option<i32> {
    if t.env_2c54 >= 1 {
        return None;
    }
    let root = (m as i32 as f64).powf(0.5);
    let scaled = t.update_limit as f64 * 0.9;
    let mut d = (root as i32) as f64;
    if scaled <= d {
        d = scaled;
    }
    let mut limit = 50;
    if 50 < d as i32 {
        limit = d as i32;
    }
    (limit < t.update_limit).then_some(limit)
}

/// Run the initialization. `lp.cost` is not read.
pub fn push_init<B: PushInitBackend>(lp: &PushLp, par: &PushParams, e: InitEntry<'_>, backend: &mut B) -> InitResult {
    let (n, m) = (lp.n, lp.m);
    let (inf, h) = (par.inf, par.h);
    let numeric_entry = e.numeric; // local_f8 (C 178)
    let mut st = PushState { vstat: e.vstat, header: e.header, xs: e.xs, basic: vec![0.0; m], width: vec![0.0; m],
                             marks: vec![0; m], rhs: vec![0.0; m], violations: 0, iterations: e.iterations,
                             enter: e.enter, leave: e.leave, direction: e.direction, leave_col: -1,
                             request: e.request, theta: f64::NAN, pivot: f64::NAN };
    let mut ctl = FactorCtl { request: e.request, numeric: e.numeric, failure: e.failure, status: e.status };
    let mut out = InitResult {
        exit: InitExit::Loop,
        state: PushLoop { st: st.clone(), round: empty_round(n, m), pinf: 0.0, period: 50,
                          buffers: Default::default() },
        numeric_limit: numeric_entry + 10,
        markowitz: e.markowitz,
        update_limit: None,
        adjusted: false,
        condition_known: e.condition.is_some(),
        diag: InitDiag::default(),
    };
    let finish = |mut out: InitResult, exit, st: PushState, ctl: FactorCtl, pinf: f64| {
        out.exit = exit;
        out.state.st = st;
        out.state.pinf = pinf;
        out.state.round.request = ctl.request;
        out.state.round.numeric = ctl.numeric;
        out.state.round.failure = ctl.failure;
        out.state.round.status = ctl.status;
        out
    };

    // C 1017–1051: the first push refactors (request 10), repeating while the call returns 1.
    if e.first {
        ctl.request = 10;
        let ret = loop {
            let (ret, r) = backend.initial_factor(&mut ctl, &st);
            if let Some(r) = r {
                renumber(&mut st, &r);
            }
            if ret != 1 {
                break ret;
            }
        };
        if ret == 12 {
            return finish(out, InitExit::Retry, st, ctl, 0.0);
        }
        if ret >= 2 {
            return finish(out, InitExit::Error(ret), st, ctl, 0.0);
        }
        if ctl.status <= -1000 {
            ctl.status = 1;
            return finish(out, InitExit::Retry, st, ctl, 0.0);
        }
        if !(e.attempt != 0 || (ctl.failure == 0 && ctl.numeric < numeric_entry + 10)) {
            return finish(out, InitExit::Retry, st, ctl, 0.0);
        }
        ctl.request = 0;
    }
    st.request = ctl.request;

    // C 401–596: RHS scratch and candidates with their distances.
    let mut r = e.rhs.to_vec();
    let mut cand: Vec<i32> = Vec::new();
    let mut dist: Vec<f64> = Vec::new();
    let mut counts = e.tau_in.map(|t| (t, [0i32; 3]));
    let mut xmm0 = 0.0; // asm 0x1c0b68
    for j in 0..n {
        let s = st.vstat[j];
        if s >= 0 {
            continue;
        }
        let x = st.xs[j];
        let v = match s {
            -1 => {
                count(&mut counts, x - lp.lb[j]);
                lp.lb[j]
            }
            -2 => {
                count(&mut counts, lp.ub[j] - x);
                lp.ub[j]
            }
            -3 => {
                dist.push(distance(x, lp.lb[j], lp.ub[j]));
                cand.push(j as i32);
                x
            }
            -4 => lp.lb[j],
            -5 => lp.ub[j],
            _ => 0.0,
        };
        xmm0 = v;
        if v != 0.0 {
            subtract_column(lp, j, v, &mut r);
        }
    }
    for i in 0..m {
        let s = st.vstat[n + i];
        if !(-3..0).contains(&s) {
            continue;
        }
        let slack = st.xs[n + i];
        let v = match s {
            -1 => {
                count(&mut counts, slack);
                0.0
            }
            -3 => {
                cand.push((n + i) as i32);
                dist.push(slack);
                slack
            }
            // -2: asm 0x1c4f3d jumps to the subtraction without loading a value (the previous xmm0)
            _ => xmm0,
        };
        xmm0 = v;
        r[i] -= v;
    }
    let counts = counts.map(|(_, c)| c);
    let mut tau_in = e.tau_in;
    let mut markowitz = e.markowitz;
    let mut raw = vec![0.0; m];
    loop {
        // C 598–645: keys, aabbe3 descending, then the scratch becomes S+0x200.
        let mut key: Vec<f64> = cand
            .iter()
            .zip(&dist)
            .map(|(&j, &d)| {
                let c = if d < 0.99 { d } else { 0.99 }; // minsd d, 0.99
                if (j as usize) < n {
                    -(lp.lengths[j as usize] as f64 + c)
                } else {
                    -1.0 - c
                }
            })
            .collect();
        sort_desc(&mut key, &mut cand);
        st.rhs.copy_from_slice(&r);

        // C 646–785: x_B = B⁻¹ rhs, the residual of B·x_B against rhs (`089d8f` on the factor header).
        backend.rhs_ftran(&r, &mut raw);
        let amax = |v: &mut f64, x: f64| {
            if *v <= x.abs() {
                *v = x.abs();
            }
        };
        let mut rhs_max = 0.0;
        st.rhs.iter().for_each(|&x| amax(&mut rhs_max, x));
        let mut acc = vec![0.0; m]; // S+0x1f8, zero between uses
        for (p, &col) in st.header.iter().enumerate() {
            let v = raw[p];
            let j = col as usize;
            if j < n {
                let s = lp.starts[j] as usize;
                for k in s..s + lp.lengths[j] as usize {
                    let i = lp.rows[k] as usize;
                    acc[i] += lp.values[k] * v;
                }
            } else {
                acc[j - n] += v;
            }
        }
        let mut basic_max = 0.0;
        raw.iter().for_each(|&x| amax(&mut basic_max, x));
        let mut residual = 0.0;
        st.rhs.iter().zip(&acc).for_each(|(&b, &a)| amax(&mut residual, b - a));

        // C 786–796: Markowitz tightening and the numeric counter.
        let candidate = if rhs_max * 1e10 < basic_max || 1e-8 < residual {
            0.5
        } else if rhs_max * 1e9 < basic_max || 4e-9 < residual {
            0.25
        } else {
            0.0001
        };
        out.diag = InitDiag { rhs_max, basic_max, residual, markowitz_candidate: candidate };
        if markowitz <= candidate && candidate != markowitz {
            let t = e.tighten.expect("Markowitz tightening (061bfe) without S+0x3c8 and the environment flag");
            let limit = tightened_limit(m, t);
            if limit.is_some() {
                out.update_limit = limit;
            }
            backend.tighten(candidate, limit);
            markowitz = candidate;
        }
        if 1e-6 < residual {
            ctl.numeric += 1;
        }

        // C 797–866: basic copy, lower shift, widths, marks, violations and PInf on the factor header.
        st.basic.copy_from_slice(&raw);
        for p in 0..m {
            let j = st.header[p] as usize;
            let lower = lp.lb[j];
            if lower <= -inf {
                st.width[p] = inf;
            } else {
                st.basic[p] -= lower;
                let upper = lp.ub[j];
                st.width[p] = if inf <= upper { inf } else { upper - lower };
            }
        }
        let mut violations = 0;
        let mut pinf = 0.0;
        for p in 0..m {
            let j = st.header[p] as usize;
            st.marks[p] = if -inf < lp.lb[j] || lp.ub[j] < inf {
                let v = st.basic[p];
                if -h <= v {
                    if h <= v {
                        let w = st.width[p];
                        if v < w - h || inf <= w {
                            b'N'
                        } else if w + h <= v {
                            violations += 1;
                            pinf += st.basic[p] - st.width[p];
                            b'A'
                        } else {
                            b'U'
                        }
                    } else if st.width[p] == 0.0 {
                        b'X'
                    } else {
                        b'L'
                    }
                } else {
                    violations += 1;
                    pinf -= st.basic[p];
                    b'B'
                }
            } else {
                b'F'
            };
        }
        st.violations = violations;

        // C 867–896: quality gate (asm 0x1c1800–0x1c1847).
        let p = if e.model_status == 2 { pinf } else { 1.0 };
        let cond_above = |t: f64| e.condition.is_some_and(|c| c > t);
        if e.attempt == 0 && (cond_above(1e14) || 1e10 < p || e.condition.is_some_and(|c| 1e16 < p * c)) {
            return finish(out, InitExit::Retry, st, ctl, pinf);
        }
        if cond_above(1e30) {
            return finish(out, InitExit::Retry, st, ctl, pinf);
        }
        if 1e15 < pinf {
            ctl.numeric += 1;
            return finish(out, InitExit::Retry, st, ctl, pinf);
        }

        // asm 0x1c184d–0x1c1890: one adjustment when the previous push ended below τ and many candidates qualify.
        let adjust = !out.adjusted
            && pinf > 1.0
            && h > e.prev_pinf.expect("tolerance adjustment test without S+0x120")
            && counts.expect("tolerance adjustment without param_1")[2] < cand.len() as i32;
        if !adjust {
            // C 897–930: loop preparation.
            let total = cand.len();
            st.iterations += 1.0;
            let support = ((m as i32 as f64).sqrt() <= total as f64) as i32;
            out.markowitz = markowitz;
            out.state.round = RoundState { support, candidates: cand, ..empty_round(n, m) };
            let exit = if total == 0 { InitExit::End } else { InitExit::Loop };
            return finish(out, exit, st, ctl, pinf);
        }
        out.adjusted = true;
        let c = counts.unwrap();
        let total = cand.len() as i32;
        let tau = tau_in.unwrap()
            * if c[0] < total {
                0.001
            } else if c[1] < total {
                0.01
            } else {
                0.1
            };
        tau_in = Some(tau);
        // C 1063–1141: rebuild; -1/-2 columns inside τ' become candidates, -4/-5 are not subtracted (asm 0x1c1cae).
        r.copy_from_slice(e.rhs);
        cand.clear();
        dist.clear();
        for j in 0..n {
            let s = st.vstat[j];
            if !(s < 0 && s > -4) {
                continue;
            }
            let x = st.xs[j];
            let (lower, upper) = (lp.lb[j], lp.ub[j]);
            let promote = match s {
                -1 => tau + lower <= x,
                // C 1101: stays at ub when ub - τ' < x (an unordered compare promotes)
                -2 => (upper - tau).partial_cmp(&x) != Some(std::cmp::Ordering::Less),
                _ => true,
            };
            let v = if promote {
                st.vstat[j] = -3;
                dist.push(distance(x, lower, upper));
                cand.push(j as i32);
                x
            } else if s == -1 {
                lower
            } else {
                upper
            };
            if v != 0.0 {
                subtract_column(lp, j, v, &mut r);
            }
        }
        for i in 0..m {
            let s = st.vstat[n + i];
            if !(s < 0 && s > -4) {
                continue;
            }
            let slack = st.xs[n + i];
            let v = if s != -1 || tau <= slack {
                st.vstat[n + i] = -3;
                cand.push((n + i) as i32);
                dist.push(slack);
                slack
            } else {
                0.0
            };
            r[i] -= v;
        }
    }
}

/// Round state at the first pass: position/deferred 0, failure counts zero (n+m, `ac64e6`), no support marks.
fn empty_round(n: usize, m: usize) -> RoundState {
    RoundState { m, candidates: Vec::new(), position: 0, deferred: 0, failures: vec![0; n + m], marks: None,
                 support: 0, approved: None, count_sum: 0.0, successes: 0, request: 0, status: 0, numeric: 0,
                 failure: 0, retry: 0 }
}
