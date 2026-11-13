//! Crossover cleanup entry: what the crossover does with the push result before the cleanup simplex runs. The
//! retry reset (both push attempts asked for a retry), the choice of the cleanup algorithm and its basis
//! arguments, the dual check of the automatic setting, and the state install the cleanup performs when it is
//! handed the workspace's own status array. The cleanup iterations are not part of it.
// The negated comparisons keep the original's unordered (NaN) outcomes.
#![allow(clippy::neg_cmp_op_on_partial_ord)]

/// The retry reset's inputs: the model bounds of the columns, the column values, and the tolerances.
pub struct ResetInput<'a> {
    pub n: usize,
    pub m: usize,
    pub lb: &'a [f64],
    pub ub: &'a [f64],
    pub x: &'a [f64],
    /// FIXTOL as the crossover read it (may be negative), FeasibilityTol, and the infinity S+0x370.
    pub fixtol: f64,
    pub feastol: f64,
    pub inf: f64,
}

/// The state the retry reset leaves.
#[derive(Debug, Clone, PartialEq)]
pub struct Reset {
    /// Columns −1/−2/−3/−4, rows the basis position of their slack.
    pub vstat: Vec<i32>,
    /// The basis header of the slack basis: position i holds n + i.
    pub header: Vec<i32>,
}

/// Retry reset with env+0x2340 = 0: a slack basis, each column at the bound nearer its value. The caller also
/// sets the remaining count, S+0x184 and S+0x454 to 0 and puts the entry PerturbValue back.
pub fn retry_reset(r: &ResetInput) -> Reset {
    let fix = if 0.0 <= r.fixtol { if r.feastol <= r.fixtol { r.feastol } else { r.fixtol } } else { 0.0 };
    let mut vstat = Vec::with_capacity(r.n + r.m);
    for j in 0..r.n {
        let (lb, ub) = (r.lb[j], r.ub[j]);
        vstat.push(if !(fix < ub - lb) {
            -4
        } else if -r.inf < lb || ub < r.inf {
            if lb + ub <= r.x[j] + r.x[j] {
                -2
            } else {
                -1
            }
        } else {
            -3
        });
    }
    vstat.extend(0..r.m as i32);
    Reset { vstat, header: (0..r.m as i32).map(|i| r.n as i32 + i).collect() }
}

/// The automatic setting's dual check over the recomputed reduced costs: the largest |d| of a basic position plus
/// the largest dual violation of a nonbasic one (skipped below `tol`) must stay below `tol`.
pub fn dual_check(d: &[f64], vstat: &[i32], tol: f64) -> bool {
    let (mut bmax, mut nmax) = (0.0f64, 0.0f64);
    for (&dj, &s) in d.iter().zip(vstat) {
        if s >= 0 {
            let a = dj.abs();
            bmax = if a > bmax { a } else { bmax };
            continue;
        }
        if s == -3 {
            continue;
        }
        let v = if s < -3 || 0.0 > -dj { 0.0 } else { -dj };
        if tol > v {
            continue;
        }
        nmax = if v > nmax { v } else { nmax };
    }
    nmax + bmax < tol
}

/// What the crossover reads to choose the cleanup.
pub struct SelectInput {
    /// The Crossover parameter (env+0x2ba0).
    pub crossover: i32,
    /// The push results after the retry reset.
    pub retry: i32,
    pub remaining: i32,
    /// S+0x2d8, S+0x120 (PInf), S+0x28 (precision).
    pub s2d8: f64,
    pub pinf: f64,
    pub precision: i32,
    /// OptimalityTol.
    pub opt_tol: f64,
    /// Restart count and S+0x178.
    pub restarts: i32,
    pub s178: i32,
}

/// The three `0cda5b` call sites.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cleanup {
    Dual,
    Primal,
    /// Primal with the workspace's status array, x and the slacks as the starting basis.
    PrimalBasis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    /// The choice before the call site (r14d).
    pub dual: bool,
    /// The dual check ran (it recomputes the reduced costs).
    pub checked: bool,
    pub call: Cleanup,
    /// S+0x180 and S+0x178 after the choice.
    pub s180: i32,
    pub s178: i32,
}

/// Choose the cleanup. `reduced_costs` recomputes d (n+m) for the dual check; it is called only on that path.
pub fn select(s: &SelectInput, vstat: &[i32], reduced_costs: &mut dyn FnMut() -> Vec<f64>) -> Selection {
    let mut checked = false;
    let dual = if s.crossover == -1 {
        if s.retry != 0 {
            false
        } else if s.remaining > 0 {
            true
        } else if s.s2d8 == 0.0 && 0.0 < s.pinf && s.precision == 0 {
            checked = true;
            dual_check(&reduced_costs(), vstat, s.opt_tol)
        } else {
            false
        }
    } else {
        !((s.crossover & !2) == 1 || s.remaining < 0)
    };
    let call = if dual {
        Cleanup::Dual
    } else if s.remaining == 0 {
        Cleanup::Primal
    } else {
        Cleanup::PrimalBasis
    };
    Selection { dual, checked, call, s180: 1, s178: s.s178 + if s.restarts == 2 { 10 } else { 0 } }
}

/// The install's inputs (the unscaled path, S+0x4d8 ≠ 0).
pub struct InstallInput<'a> {
    pub n: usize,
    pub m: usize,
    /// vbasis (n) and cbasis (m); `None` when the cleanup was given no basis.
    pub vbasis: Option<&'a [i32]>,
    pub cbasis: Option<&'a [i32]>,
    pub x: Option<&'a [f64]>,
    pub slack: Option<&'a [f64]>,
    /// Workspace bounds (n+m), costs and column lengths (n).
    pub lb: &'a [f64],
    pub ub: &'a [f64],
    pub cost: &'a [f64],
    pub lengths: &'a [i32],
    /// S+0x380, S+0x3b0 (FIXTOL), S+0x388 (OptimalityTol).
    pub tau: f64,
    pub fixtol: f64,
    pub opt_tol: f64,
    /// D+0x3c = 1 with the per-column flags at D+0x408.
    pub flipped: Option<&'a [u8]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Installed {
    /// The workspace status array after the install (n+m).
    pub vstat: Vec<i32>,
    /// The superbasic list (S+0x440 positions, S+0x448 values; S+0x450 is its length).
    pub list: Vec<(i32, f64)>,
    /// The current factor was marked stale.
    pub factor_reset: bool,
}

/// `0325b4`: copy the given basis into the workspace status array and reclassify its nonbasic entries against
/// x and the slacks; values away from both bounds become superbasic (−3) and enter the list. `vstat` is the
/// workspace array (left alone when either basis argument is missing).
pub fn install_basis(e: &InstallInput, vstat: &mut [i32]) -> Option<Installed> {
    let (vb, cb) = (e.vbasis?, e.cbasis?);
    let n = e.n;
    vstat[..n].copy_from_slice(vb);
    vstat[n..n + e.m].copy_from_slice(cb);
    let mut xs: Option<Vec<f64>> = e.x.map(|x| x.to_vec());
    if let Some(flags) = e.flipped {
        for j in 0..n {
            if flags[j] == 1 && vstat[j] == -2 {
                vstat[j] = -1;
            }
        }
        if let Some(x) = xs.as_mut() {
            for j in 0..n {
                if flags[j] == 1 {
                    x[j] = -x[j];
                }
            }
        }
    }
    let mut list = Vec::new();
    if let Some(x) = xs.as_deref() {
        for j in 0..n {
            if vb[j] >= 0 {
                continue;
            }
            let (lb, ub) = (e.lb[j], e.ub[j]);
            if !(e.fixtol < ub - lb) {
                vstat[j] = -4;
            } else if !(e.tau <= (x[j] - lb).abs()) {
                vstat[j] = -1;
            } else if !(e.tau <= (x[j] - ub).abs()) {
                vstat[j] = -2;
            } else if e.lengths[j] != 0 || (e.opt_tol <= e.cost[j].abs() && e.cost[j].abs() != e.opt_tol) {
                vstat[j] = -3;
                if 1e-13 < x[j].abs() {
                    list.push((j as i32, x[j]));
                }
            }
        }
    }
    if let Some(s) = e.slack {
        for i in 0..e.m {
            if cb[i] >= 0 {
                continue;
            }
            let j = n + i;
            let lb = e.lb[j];
            if !(e.fixtol < e.ub[j] - lb) {
                vstat[j] = -4;
            } else if e.tau <= (s[i] - lb).abs() {
                vstat[j] = -3;
                list.push((j as i32, s[i]));
            } else {
                vstat[j] = -1;
            }
        }
    }
    Some(Installed { vstat: vstat.to_vec(), list, factor_reset: true })
}

/// Boundaries read from the C and asm that the original captures of M3-04f-2 never reached (ties, fixed and
/// free columns, small costs, tiny values, rows at their upper bound). They pin the reading, not the original.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_reset_boundaries() {
        let inf = 1e100;
        let (lb, ub, x) = ([0.0, 0.0, -inf, 2.0, 0.0], [2.0, 2.0, inf, 2.0, 2.0], [1.0, 0.9, 0.0, 2.0, 0.0]);
        let r = ResetInput { n: 5, m: 2, lb: &lb, ub: &ub, x: &x, fixtol: -1.0, feastol: 1e-6, inf };
        // tie lb + ub = 2x → upper; free → −3; ub − lb = 0 is fixed with FIXTOL clamped to 0
        let got = retry_reset(&r);
        assert_eq!(got.vstat, vec![-2, -1, -3, -4, -1, 0, 1]);
        assert_eq!(got.header, vec![5, 6]);
        // FIXTOL above FeasibilityTol uses FeasibilityTol
        let r = ResetInput { fixtol: 1.0, feastol: 1e-6, lb: &[0.0], ub: &[0.5], x: &[0.0], n: 1, m: 0, inf };
        assert_eq!(retry_reset(&r).vstat, vec![-1]);
    }

    #[test]
    fn dual_check_boundaries() {
        let tol = 1e-6;
        // sum equal to tol is not below it
        assert!(!dual_check(&[tol], &[0], tol));
        // a nonbasic violation under tol is skipped; −3 and −4 never count
        assert!(dual_check(&[5e-7, -1.0, -1.0, 5e-7], &[0, -3, -4, -1], tol));
        assert!(!dual_check(&[-2e-6], &[-2], tol));
        // NaN: a basic NaN keeps the maximum, a nonbasic NaN violation is not skipped and not taken
        assert!(dual_check(&[f64::NAN, f64::NAN], &[0, -1], tol));
    }

    #[test]
    fn select_boundaries() {
        let base = SelectInput { crossover: -1, retry: 0, remaining: 0, s2d8: 0.0, pinf: 1.0, precision: 0, opt_tol: 1e-6,
                                 restarts: 2, s178: 3 };
        let mut no_d = || -> Vec<f64> { panic!("no dual check expected") };
        let s = select(&SelectInput { precision: 1, ..base }, &[], &mut no_d);
        assert_eq!((s.dual, s.checked, s.call, s.s180, s.s178), (false, false, Cleanup::Primal, 1, 13));
        let s = select(&SelectInput { s2d8: -0.0, remaining: -2, ..base }, &[0], &mut || vec![0.0]);
        assert_eq!((s.dual, s.checked), (true, true));
        let s = select(&SelectInput { s2d8: f64::NAN, ..base }, &[], &mut no_d);
        assert!(!s.checked);
        for (c, rem, dual) in [(1, 3, false), (3, 0, false), (2, 0, true), (4, -1, false), (2, 5, true)] {
            let s = select(&SelectInput { crossover: c, remaining: rem, ..base }, &[], &mut no_d);
            assert_eq!(s.dual, dual, "crossover {c} remaining {rem}");
        }
    }

    #[test]
    fn install_boundaries() {
        let (lb, ub) = ([0.0, 0.0, 0.0, 1.0, 0.0, 0.0], [4.0, 4.0, 4.0, 1.0, 4.0, 4.0]);
        let cost = [0.0, 1e-6, 0.0, 0.0];
        let lengths = [0, 0, 1, 1];
        let (vb, cb) = ([-1, -1, -1, -2], [-2, -1]);
        let x = [2.0, 2.0, 1e-14 + 1e-6, 1.0];
        let slack = [4.0, 5e-7];
        let e = InstallInput { n: 4, m: 2, vbasis: Some(&vb), cbasis: Some(&cb), x: Some(&x), slack: Some(&slack), lb: &lb,
                               ub: &ub, cost: &cost, lengths: &lengths, tau: 1e-6, fixtol: 0.0, opt_tol: 1e-6,
                               flipped: None };
        let mut vstat = vec![9; 6];
        let r = install_basis(&e, &mut vstat).unwrap();
        // empty column with |c| = OptTol keeps its status; |x − lb| = τ is away from the bound; fixed → −4;
        // a row at its upper bound is superbasic; a row within τ of its lower bound is −1
        assert_eq!(r.vstat, vec![-1, -1, -3, -4, -3, -1]);
        assert_eq!(r.list.len(), 2);
        assert_eq!(r.list[0].0, 2);
        assert_eq!(r.list[1], (4, 4.0));
        // a missing basis argument leaves the array alone
        let mut keep = vec![7; 6];
        assert!(install_basis(&InstallInput { cbasis: None, ..e }, &mut keep).is_none());
        assert_eq!(keep, vec![7; 6]);
    }
}
