//! Primal push solution install: what the push does after its loop ends (or straight from an initialization with
//! no candidate) until it returns. The cleanup that consumes the result is not part of it.
use crate::crossover::{PushLp, PushParams, PushState};

/// The install's inputs besides the push state.
pub struct InstallEntry<'a> {
    /// param_7: write the basic values back into x and the slacks.
    pub first: bool,
    /// param_4 and param_3.
    pub attempt: i32,
    pub model_status: i32,
    /// The candidates still in the list, in list order (their count is the remaining count).
    pub candidates: &'a [i32],
    /// S+0x46c.
    pub interval: i32,
    /// The environment value at (S+0x4b8)+0x100 → +0x2ba0.
    pub env_2ba0: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Installed {
    /// S+0x120 and the violation count of the same scan.
    pub pinf: f64,
    pub violations: i32,
    /// `*param_8` after the condition call.
    pub condition: f64,
    /// `*param_9 = 1`.
    pub retry: bool,
    /// `*param_10`: the remaining count, negative when the caller must leave the candidates to the cleanup.
    pub remaining: i32,
}

/// `comisd b, a` followed by `jb`: taken when a < b or the operands are unordered.
fn below_or_unordered(a: f64, b: f64) -> bool {
    !matches!(a.partial_cmp(&b), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal))
}

/// Free in the install's tests: lower bound not above -I (`jb` not taken) and upper bound at least I (`jae`).
fn free(lb: f64, ub: f64, inf: f64) -> bool {
    !below_or_unordered(-inf, lb) && ub >= inf
}

/// Write the basic values back (first push only): a column gets `lb + b` when its lower bound is finite, a slack
/// gets `b`. `st.xs` holds x followed by the slacks (the original's contiguous alias).
fn write_back(lp: &PushLp, par: &PushParams, st: &mut PushState) {
    let neg_inf = -par.inf;
    for (i, &v) in st.header.iter().enumerate() {
        let (v, b) = (v as usize, st.basic[i]);
        st.xs[v] = if v < lp.n && below_or_unordered(neg_inf, lp.lb[v]) { lp.lb[v] + b } else { b };
    }
}

/// Violations and PInf in position order over the basic rows that are not free (the loop's own classification
/// with the strict comparisons of the install scan).
fn infeasibility(lp: &PushLp, par: &PushParams, st: &PushState) -> (f64, i32) {
    let (tau, inf) = (par.h, par.inf);
    let neg_tau = -tau;
    let (mut p, mut count) = (0.0, 0);
    for (i, &v) in st.header.iter().enumerate() {
        let v = v as usize;
        if free(lp.lb[v], lp.ub[v], inf) {
            continue;
        }
        let b = st.basic[i];
        if neg_tau > b {
            count += 1;
            p -= b;
            continue;
        }
        let w = st.width[i];
        if tau > b || w - tau > b || w >= inf || tau + w > b {
            continue;
        }
        count += 1;
        p += b - w;
    }
    (p, count)
}

/// Sign of the remaining count: negative if a remaining candidate is free; otherwise, when the environment value
/// is -1, negative below `max(5, S+0x46c / 10)`.
fn remaining(lp: &PushLp, par: &PushParams, e: &InstallEntry) -> i32 {
    let c = e.candidates.len() as i32;
    if c == 0 {
        return 0;
    }
    if e.candidates.iter().any(|&v| free(lp.lb[v as usize], lp.ub[v as usize], par.inf)) {
        return -c;
    }
    if e.env_2ba0 != -1 {
        return c;
    }
    let threshold = (e.interval / 10).max(5);
    if c >= threshold {
        c
    } else {
        -c
    }
}

/// Run the install. `condition` is the `12f282` call on the push's current factor; it runs after S+0x120 is set.
pub fn push_install(lp: &PushLp, par: &PushParams, st: &mut PushState, e: &InstallEntry,
                    condition: &mut dyn FnMut() -> f64) -> Installed {
    if e.first {
        write_back(lp, par, st);
    }
    let (pinf, violations) = infeasibility(lp, par, st);
    let cond = condition();
    let p = if e.model_status == 2 { pinf } else { 1.0 };
    let retry = (e.attempt == 0 && (cond > 1e14 || p > 1e10 || p * cond > 1e16)) || cond > 1e30;
    let mut out = Installed { pinf, violations, condition: cond, retry, remaining: e.candidates.len() as i32 };
    if retry {
        return out;
    }
    out.remaining = remaining(lp, par, e);
    if out.remaining > 0 {
        for &v in &e.candidates[..out.remaining as usize] {
            let v = v as usize;
            st.vstat[v] = if lp.lb[v] + lp.ub[v] > st.xs[v] * 2.0 { -1 } else { -2 };
        }
    } else if out.remaining < 0 {
        for (j, s) in st.vstat.iter().enumerate() {
            match *s {
                -1 => st.xs[j] = lp.lb[j],
                -2 => st.xs[j] = lp.ub[j],
                _ => {}
            }
        }
    }
    out
}
