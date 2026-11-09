//! Initial linear MIP incumbent construction, feasibility repair and objective improvement.
use rsolver_lp::common::Rng;
use rsolver_model::Model;

use crate::{root::incumbent_objective, Params};

fn column(model: &Model, j: usize) -> std::ops::Range<usize> {
    let start = model.col_beg[j] as usize;
    start..start + model.col_len[j] as usize
}

fn slacks(model: &Model, x: &[f64]) -> Vec<f64> {
    let mut slack = model.rhs.clone();
    for (j, &v) in x.iter().enumerate() {
        for k in column(model, j) {
            slack[model.col_ind[k] as usize] -= model.col_val[k] * v;
        }
    }
    slack
}

fn violated(model: &Model, i: usize, slack: f64) -> bool {
    slack < -1e-6 || (model.row_sense[i] == b'=' && slack > 1e-6)
}

/// Improve a feasible point by moving in column order along objective-improving directions within their row slacks.
fn improve(model: &Model, x: &mut [f64]) {
    let mut slack = slacks(model, x);
    for j in 0..x.len() {
        let c = model.obj[j];
        let mut up = if c < -1e-10 { model.ub[j] - x[j] } else { 0.0 };
        let mut down = if c > 1e-10 { x[j] - model.lb[j] } else { 0.0 };
        for k in column(model, j) {
            let i = model.col_ind[k] as usize;
            let a = model.col_val[k];
            if model.row_sense[i] == b'=' {
                up = 0.0;
                down = 0.0;
                break;
            }
            let d = slack[i] / a.abs();
            if a > 0.0 { up = up.min(d); }
            else if a < 0.0 { down = down.min(d); }
        }
        let delta = if up > 0.0 {
            if model.vtype[j] == b'C' { up } else { (up + 1e-10).floor() }
        } else if down > 0.0 {
            if model.vtype[j] == b'C' { -down } else { -(down + 1e-10).floor() }
        } else { 0.0 };
        if delta != 0.0 && delta.is_finite() && delta.abs() < 1e30 {
            x[j] += delta;
            for k in column(model, j) {
                slack[model.col_ind[k] as usize] -= model.col_val[k] * delta;
            }
        }
    }
}

fn repair_delta(model: &Model, x: &[f64], j: usize, delta: f64) -> f64 {
    let delta = delta.min(model.ub[j] - x[j]).max(model.lb[j] - x[j]).clamp(-1e10, 1e10);
    if model.vtype[j] == b'C' { delta }
    else {
        let delta = if delta <= 0.0 { (delta + 1e-6).floor() } else { (delta - 1e-6).ceil() };
        delta.min(2e9 - x[j]).max(-2e9 - x[j])
    }
}

/// Restore sampled binary columns to a reference point, flipping the reference on the first pass.
fn perturb(model: &Model, reference: &mut [f64], x: &mut [f64], pass: usize, rng: &mut Rng) -> usize {
    let integer_count = model.vtype.iter().filter(|&&t| t == b'B' || t == b'I').count();
    let target = (integer_count as f64 * 0.125 * ((pass as f64 + 1.0) / 200.0)).max(1.0) as usize;
    let mut selected = 0;
    let mut work = 0;
    for _ in 0..x.len() {
        let j = rng.next_int() as usize % x.len();
        if (model.vtype[j] == b'B' || (model.vtype[j] == b'I' && model.lb[j] > -0.5 && model.ub[j] < 1.5))
            // Fixed columns must remain in their domain, including on the first-pass flip.
            && model.lb[j] <= 0.0 && model.ub[j] >= 1.0 {
            if pass == 0 { reference[j] = 1.0 - reference[j]; }
            x[j] = reference[j];
            work += 2;
            selected += 1;
            if selected >= target { break; }
        }
        work += 3;
    }
    work
}

// shortcut: semi variables, quadratic repair and sub-MIP fallbacks need the full heuristic scheduler and MIP state.
fn repair(model: &Model, rows: &[Vec<(usize, f64)>], x: &mut [f64], rng: &mut Rng) {
    let budget = model.num_nz().saturating_mul(100).saturating_add(100_000);
    repair_with_budget(model, rows, x, rng, budget, 0);
}

fn repair_with_budget(model: &Model, rows: &[Vec<(usize, f64)>], x: &mut [f64], rng: &mut Rng, budget: usize, initial_work: usize) -> usize {
    let m = rows.len();
    let mut slack = slacks(model, x);
    let mut weight = vec![1.0; m];
    let mut seen_row = vec![usize::MAX; m];
    let mut seen_col = vec![usize::MAX; x.len()];
    let mut work = initial_work.saturating_add(x.len().saturating_mul(2))
        .saturating_add(model.num_nz().saturating_mul(4)).saturating_add(m.saturating_mul(5));
    let mut timer = crate::driver::Timer::default();
    timer.limit = budget as f64;
    timer.start(work as f64);
    let mut queue = Vec::new();
    let mut candidates = Vec::new();
    let mut reference = x.to_vec();
    let mut stalled = 0;
    let mut last_failed = None;
    let mut restarted = false;
    for pass in 0..200 {
        if timer.usage(work as f64) >= 1.0 { return work; }
        queue.clear();
        for i in 0..m {
            if violated(model, i, slack[i]) {
                queue.push(i);
                seen_row[i] = pass;
                if restarted && i > 0 && last_failed == Some(i) {
                    let last = queue.len() - 1;
                    queue.swap(0, last);
                }
            }
        }
        work = work.saturating_add(m.saturating_mul(3));
        if queue.is_empty() { return work; }
        let mut pos = 0;
        let mut changed = false;
        let mut candidate_count = 0;
        while pos < queue.len() {
            if timer.usage(work as f64) >= 1.0 { break; }
            let i = queue[pos];
            pos += 1;
            if !violated(model, i, slack[i]) { continue; }
            last_failed = Some(i);
            candidates.clear();
            let mut singletons = 0;
            for &(j, a) in &rows[i] {
                work = work.saturating_add(8);
                if seen_col[j] == pass || a == 0.0 { continue; }
                let delta = repair_delta(model, x, j, slack[i] / a);
                if delta.abs() >= 9.999999999999999e-6 {
                    if model.col_len[j] == 1 {
                        candidates.push((j, a));
                        let last = candidates.len() - 1;
                        candidates.swap(singletons, last);
                        singletons += 1;
                    } else { candidates.push((j, a)); }
                }
            }
            candidate_count += candidates.len();
            for k in (singletons..candidates.len()).rev() {
                let other = singletons + rng.next_int() as usize % (k - singletons + 1);
                candidates.swap(k, other);
            }
            work = work.saturating_add((candidates.len() - singletons).saturating_mul(2));
            for &(j, a) in &candidates {
                work = work.saturating_add(9);
                if !violated(model, i, slack[i]) { continue; }
                let delta = repair_delta(model, x, j, slack[i] / a);
                if delta.abs() < 9.999999999999999e-6 { continue; }
                let mut count = 0i32;
                let mut penalty = 0.0;
                for k in column(model, j) {
                    let r = model.col_ind[k] as usize;
                    let before = slack[r];
                    let after = before - delta * model.col_val[k];
                    if violated(model, r, before) {
                        count -= 1;
                        penalty -= before.abs() * weight[r];
                    }
                    if violated(model, r, after) {
                        count += 1;
                        penalty += after.abs() * weight[r];
                    }
                    work = work.saturating_add(5);
                }
                seen_col[j] = pass;
                if count < 0 || (count == 0 && penalty < 0.0) || (pass % 3 == 2 && penalty < 0.0) {
                    x[j] += delta;
                    changed = true;
                    work = work.saturating_add(8);
                    for k in column(model, j) {
                        let r = model.col_ind[k] as usize;
                        slack[r] -= delta * model.col_val[k];
                        if seen_row[r] != pass && violated(model, r, slack[r]) {
                            queue.push(r);
                            seen_row[r] = pass;
                        }
                        work = work.saturating_add(5);
                    }
                }
            }
        }
        work = work.saturating_add(pos.saturating_mul(5));
        let feasible = !(0..m).any(|i| violated(model, i, slack[i]));
        stalled = if changed { 0 } else { stalled + 1 };
        if !feasible && !changed && (candidate_count == 0 || stalled > 2) && timer.usage(work as f64) < 1.0 {
            work = work.saturating_add(perturb(model, &mut reference, x, pass, rng));
            slack = slacks(model, x);
            work = work.saturating_add(model.num_nz().saturating_mul(4))
                .saturating_add(x.len().saturating_mul(4)).saturating_add(m.saturating_mul(5));
            restarted = true;
        } else {
            restarted = false;
            for i in 0..m {
                if violated(model, i, slack[i]) { weight[i] *= 2.0; }
            }
            work = work.saturating_add(m.saturating_mul(4));
        }
        if feasible { return work; }
    }
    work
}

/// Try two sweeps of finite bounds when the main construction has found no incumbent.
fn bound_flip(model: &Model) -> Option<Vec<f64>> {
    let n = model.obj.len();
    let mut x = model.lb.clone();
    for (j, v) in x.iter_mut().enumerate() {
        if v.abs() > 1e6 {
            let lo = model.lb[j];
            let hi = model.ub[j];
            *v = if lo > 0.0 || hi < 0.0 {
                if hi.abs() <= lo.abs() { hi } else { lo }
            } else { 0.0 };
        }
    }
    if let Some(j) = (0..n).find(|&j| model.vtype[j] == b'B' && model.ub[j] > 0.5) {
        x[j] = 1.0;
    }
    let mut slack = slacks(model, &x);
    let mut remaining = slack.iter().enumerate().filter(|&(i, &v)| violated(model, i, v)).count();
    if remaining == 0 { return Some(x); }
    let budget = model.num_nz().saturating_mul(100).saturating_add(100_000);
    let mut work = n.saturating_mul(3).saturating_add(model.num_nz().saturating_mul(4));
    for _ in 0..2 {
        let mut j = 0;
        while j < n {
            if work >= budget { return None; }
            let delta = if x[j] == model.lb[j] && model.ub[j] < 1e6 {
                model.ub[j] - x[j]
            } else if x[j] == model.ub[j] && model.lb[j] > -1e6 {
                model.lb[j] - x[j]
            } else { 0.0 };
            work = work.saturating_add(5);
            if delta != 0.0 {
                x[j] += delta;
                for k in column(model, j) {
                    let i = model.col_ind[k] as usize;
                    if violated(model, i, slack[i]) { remaining -= 1; }
                    slack[i] -= model.col_val[k] * delta;
                    if violated(model, i, slack[i]) { remaining += 1; }
                    work = work.saturating_add(4);
                }
                if remaining == 0 { return Some(x); }
                if model.vtype[j] == b'B' && x[j] > 0.5 { continue; }
            }
            j += 1;
        }
    }
    None
}

pub(crate) fn initial_incumbent(domain: &Model, p: &Params) -> Option<Vec<f64>> {
    if p.heuristics <= 0.0 || p.prepreheur == 0 || domain.obj.is_empty()
        || !domain.vtype.iter().any(|t| matches!(t, b'B' | b'I'))
        || domain.vtype.iter().any(|t| !matches!(t, b'C' | b'B' | b'I')) {
        return None;
    }
    let mut model = domain.clone();
    for j in 0..model.obj.len() {
        model.obj[j] *= model.sense as f64;
        if model.vtype[j] != b'C' {
            model.lb[j] = (model.lb[j] - 1e-5).ceil();
            model.ub[j] = (model.ub[j] + 1e-5).floor();
        }
        if !model.lb[j].is_finite() || !model.ub[j].is_finite() || model.lb[j] > model.ub[j] { return None; }
        for k in column(&model, j) {
            if model.row_sense[model.col_ind[k] as usize] == b'>' { model.col_val[k] = -model.col_val[k]; }
        }
    }
    model.objcon *= model.sense as f64;
    model.sense = 1;
    for i in 0..model.rhs.len() {
        if model.row_sense[i] == b'>' {
            model.row_sense[i] = b'<';
            model.rhs[i] = -model.rhs[i];
        }
    }
    let mut rows = vec![Vec::new(); model.rhs.len()];
    for j in 0..model.obj.len() {
        for k in column(&model, j) { rows[model.col_ind[k] as usize].push((j, model.col_val[k])); }
    }
    let mut best = None;
    let mut best_obj = f64::INFINITY;
    let mut rng = Rng((p.seed as u64).wrapping_add(20));
    for mode in 0..4 {
        let mut x = if mode == 3 {
            if best.is_some() { break; }
            match bound_flip(&model) { Some(x) => x, None => break }
        } else { Vec::with_capacity(model.obj.len()) };
        if mode != 3 {
            for j in 0..model.obj.len() {
                let lo = model.lb[j];
                let hi = model.ub[j];
                let v = match mode {
                    0 => if lo > -1e8 { lo } else if hi < 1e8 { hi } else { 0.0 },
                    1 => if hi < 1e8 { hi } else if lo > -1e8 { lo } else { 0.0 },
                    _ => {
                        let down = column(&model, j).all(|k| model.row_sense[model.col_ind[k] as usize] != b'=' && model.col_val[k] > 0.0);
                        let up = column(&model, j).all(|k| model.row_sense[model.col_ind[k] as usize] != b'=' && model.col_val[k] <= 0.0);
                        if up { hi.min(1e8) } else if down { lo.max(-1e8) }
                        else if lo > -1e8 { lo } else if hi < 1e8 { hi } else { 0.0 }
                    }
                };
                x.push(if model.vtype[j] == b'C' { v } else { (v + 0.5).floor() });
            }
        }
        if mode == 2 && p.heuristics > 0.005 && incumbent_objective(domain, p, &x).is_err() {
            repair(&model, &rows, &mut x, &mut rng);
        }
        if let Ok(mut objective) = incumbent_objective(domain, p, &x) {
            let mut improved = x.clone();
            improve(&model, &mut improved);
            if let Ok(z) = incumbent_objective(domain, p, &improved) {
                if z < objective { x = improved; objective = z; }
            }
            if objective < best_obj { best_obj = objective; best = Some(x); }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::root::{initial_incumbent, root_domain, solve_root};

    #[test]
    fn work_limit_stops_between_rows_not_between_candidates() {
        let m = Model {
            obj: vec![0.0; 3], lb: vec![0.0; 3], ub: vec![1.0; 3], vtype: b"BBB".to_vec(),
            col_beg: vec![0, 1, 2], col_len: vec![1, 1, 1], col_ind: vec![0, 0, 1],
            col_val: vec![-1.0; 3], row_sense: b"<<".to_vec(), rhs: vec![-2.0, -1.0],
            ..Model::default()
        };
        let rows = vec![vec![(0, -1.0), (1, -1.0)], vec![(2, -1.0)]];
        for initial_work in [0, 1000] {
            let mut x = vec![0.0; 3];
            let work = repair_with_budget(&m, &rows, &mut x, &mut Rng(20), 20, initial_work);
            assert_eq!(x, vec![1.0, 1.0, 0.0]);
            assert_eq!(work, initial_work + 117);
        }
    }

    fn mixed_cover() -> Model {
        Model {
            sense: 1, objcon: 7.0, obj: vec![1.0, 1.0],
            lb: vec![0.0, 0.0], ub: vec![1.0, 10.0], vtype: b"BC".to_vec(),
            col_beg: vec![0, 1], col_len: vec![1, 1], col_ind: vec![0, 0], col_val: vec![1.0, 1.0],
            row_sense: vec![b'>'], rhs: vec![2.5], ..Model::default()
        }
    }

    #[test]
    fn repairs_mixed_cover_and_automatically_tightens_root() {
        let m = mixed_cover();
        let p = Params::default();
        let x = initial_incumbent(&m, &p).expect("repair finds continuous completion");
        assert!(incumbent_objective(&m, &p, &x).unwrap() <= 9.5);
        assert_eq!(x[0], x[0].round());
        let domain = root_domain(&m, &p, None).unwrap();
        assert!(domain.ub[1] < m.ub[1]);
        let root = solve_root(&m, &p).unwrap();
        assert_eq!(root.incumbent.unwrap(), x);
        assert_eq!(m.ub, vec![1.0, 10.0]);
        assert_eq!(initial_incumbent(&m, &p).unwrap(), x);
    }

    #[test]
    fn disabled_search_preserves_root_and_explicit_incumbents() {
        let m = mixed_cover();
        for p in [Params { heuristics: 0.0, ..Params::default() }, Params { prepreheur: 0, ..Params::default() }] {
            assert!(initial_incumbent(&m, &p).is_none());
            assert_eq!(root_domain(&m, &p, None).unwrap().ub, m.ub);
            assert!(root_domain(&m, &p, Some(&[0.0, 2.5])).unwrap().ub[1] < 10.0);
            assert!(root_domain(&m, &p, Some(&[0.5, 2.0])).is_err());
        }
    }

    #[test]
    fn maximization_improves_with_negative_coefficients_and_fixed_integers() {
        let m = Model {
            sense: -1, objcon: -3.0, obj: vec![2.0, 1.0],
            lb: vec![1.0, -4.0], ub: vec![1.0, 3.0], vtype: b"BI".to_vec(),
            col_beg: vec![0, 1], col_len: vec![1, 1], col_ind: vec![0, 0], col_val: vec![-1.0, 1.0],
            row_sense: vec![b'<'], rhs: vec![1.0], ..Model::default()
        };
        let p = Params::default();
        let x = initial_incumbent(&m, &p).unwrap();
        assert_eq!(x, vec![1.0, 2.0]);
        assert_eq!(incumbent_objective(&m, &p, &x), Ok(-1.0));
    }

    #[test]
    fn repairs_equalities_but_never_submits_infeasible_or_fractional_points() {
        let mut m = mixed_cover();
        m.row_sense[0] = b'=';
        let p = Params::default();
        let x = initial_incumbent(&m, &p).unwrap();
        assert!(incumbent_objective(&m, &p, &x).is_ok());
        m.vtype[1] = b'I';
        assert!(initial_incumbent(&m, &p).is_none());
        m.rhs[0] = 50.0;
        assert!(initial_incumbent(&m, &p).is_none());
        m.ub[0] = -1.0;
        assert!(initial_incumbent(&m, &p).is_none());
    }

    #[test]
    fn restart_recovers_from_a_stalled_binary_repair() {
        let m = Model {
            sense: 1, obj: vec![0.0, 2.0, -3.0, 3.0],
            lb: vec![0.0; 4], ub: vec![1.0; 4], vtype: b"BBBB".to_vec(),
            col_beg: vec![0, 2, 4, 5], col_len: vec![2, 2, 1, 3],
            col_ind: vec![0, 3, 1, 2, 2, 1, 2, 3],
            col_val: vec![1.0, -2.0, 3.0, 1.0, -2.0, 3.0, -3.0, -1.0],
            row_sense: b"<<<=".to_vec(), rhs: vec![1.0, 3.0, 1.0, -2.0], ..Model::default()
        };
        let p = Params::default();
        let x = initial_incumbent(&m, &p).expect("restart escapes the stalled candidate");
        assert!(incumbent_objective(&m, &p, &x).is_ok());
        assert_eq!(x[0], 1.0);
        assert_eq!(x[3], 0.0);
        assert_eq!(initial_incumbent(&m, &p), Some(x));
    }

    #[test]
    fn restart_repairs_the_last_failed_row_first() {
        let m = Model {
            sense: 1, obj: vec![-3.0, 3.0, 2.0, -3.0, -3.0, 0.0],
            lb: vec![0.0; 6], ub: vec![1.0; 6], vtype: b"BBBBBB".to_vec(),
            col_beg: vec![0, 2, 5, 7, 8, 10], col_len: vec![2, 3, 2, 1, 2, 0],
            col_ind: vec![1, 2, 0, 2, 3, 2, 3, 1, 0, 2],
            col_val: vec![-2.0, 2.0, 1.0, 2.0, 3.0, -1.0, 2.0, 1.0, 2.0, -1.0],
            row_sense: b"<<=<".to_vec(), rhs: vec![4.0, -2.0, 3.0, 4.0], ..Model::default()
        };
        let p = Params::default();
        let x = initial_incumbent(&m, &p).expect("failed-row priority escapes the stalled candidate");
        assert_eq!(incumbent_objective(&m, &p, &x), Ok(-3.0));
        assert_eq!(initial_incumbent(&m, &p), Some(x));
    }

    #[test]
    fn bound_sweeps_find_an_incumbent_after_row_repair_fails() {
        let mut m = Model {
            sense: 1, objcon: 4.25, obj: vec![1.0, 0.0, 1.0, -3.0],
            lb: vec![0.0; 4], ub: vec![1.0; 4], vtype: b"BBBB".to_vec(),
            col_beg: vec![0, 2, 4, 5], col_len: vec![2, 2, 1, 1],
            col_ind: vec![1, 2, 0, 2, 2, 1], col_val: vec![3.0, 1.0, 2.0, -2.0, -1.0, 3.0],
            row_sense: b"<<=".to_vec(), rhs: vec![3.0, 0.0, -2.0], ..Model::default()
        };
        for sense in [1, -1] {
            m.sense = sense;
            for h in [0.05, 0.001] {
                let p = Params { heuristics: h, ..Params::default() };
                let x = initial_incumbent(&m, &p).expect("bound sweeps recover the missed binary assignment");
                assert_eq!(incumbent_objective(&m, &p, &x), Ok(sense as f64 * 4.25));
                assert_eq!(x, vec![0.0, 1.0, 0.0, 0.0]);
                assert_eq!(initial_incumbent(&m, &p), Some(x));
            }
        }
    }

    #[test]
    fn bound_sweeps_preserve_fixed_and_free_columns_and_skip_infinite_bounds() {
        let m = Model {
            sense: 1, obj: vec![0.0; 3], lb: vec![1.0, -1e30, 0.0], ub: vec![1.0, 1e30, 1.0],
            vtype: b"BCI".to_vec(), col_beg: vec![0, 0, 0], col_len: vec![0, 0, 1],
            col_ind: vec![0], col_val: vec![1.0], row_sense: vec![b'='], rhs: vec![1.0], ..Model::default()
        };
        assert_eq!(bound_flip(&m), Some(vec![1.0, 0.0, 1.0]));
        let mut large = Model {
            sense: 1, obj: vec![0.0], lb: vec![0.0], ub: vec![1e6], vtype: vec![b'I'],
            col_beg: vec![0], col_len: vec![1], col_ind: vec![0], col_val: vec![1.0],
            row_sense: vec![b'='], rhs: vec![1e6], ..Model::default()
        };
        assert!(bound_flip(&large).is_none());
        large.ub[0] -= 1.0;
        large.rhs[0] -= 1.0;
        assert_eq!(bound_flip(&large), Some(vec![1e6 - 1.0]));
        let mut integers = Model {
            sense: 1, obj: vec![0.0; 2], lb: vec![0.0; 2], ub: vec![1.0; 2], vtype: b"II".to_vec(),
            col_beg: vec![0, 1], col_len: vec![1, 1], col_ind: vec![0, 0], col_val: vec![1.0, 1.0],
            row_sense: vec![b'='], rhs: vec![2.0], ..Model::default()
        };
        assert_eq!(bound_flip(&integers), Some(vec![1.0, 1.0]));
        integers.vtype = b"BB".to_vec();
        assert!(bound_flip(&integers).is_none());
    }

    #[test]
    fn perturb_flips_first_pass_and_restores_later_passes() {
        let m = Model {
            obj: vec![0.0], lb: vec![0.0], ub: vec![1.0], vtype: vec![b'I'], ..Model::default()
        };
        let mut reference = vec![0.0];
        let mut x = vec![0.0];
        let mut rng = Rng(20);
        perturb(&m, &mut reference, &mut x, 0, &mut rng);
        assert_eq!(x, vec![1.0]);
        assert_eq!(reference, vec![1.0]);
        x[0] = 0.0;
        perturb(&m, &mut reference, &mut x, 4, &mut rng);
        assert_eq!(x, reference);
    }

    #[test]
    fn perturb_counts_general_integers_when_setting_restart_size() {
        for vtype in [b"IIIIIIIIIIIIIIII".to_vec(), b"BBBBBBBBBBBBBBBB".to_vec()] {
            let m = Model { obj: vec![0.0; 16], lb: vec![0.0; 16], ub: vec![1.0; 16], vtype, ..Model::default() };
            let mut reference = vec![0.0; 16];
            let mut x = vec![1.0; 16];
            let mut rng = Rng(20);
            perturb(&m, &mut reference, &mut x, 199, &mut rng);
            assert_eq!(x.iter().filter(|&&v| v == 0.0).count(), 2);
            assert_eq!(rng.0, 4720012900338762598);
        }
    }

    #[test]
    fn perturb_preserves_fixed_and_general_integer_columns() {
        let m = Model {
            obj: vec![0.0; 4], lb: vec![1.0, 0.0, 0.0, -2.0], ub: vec![1.0, 0.0, 1.0, 5.0],
            vtype: b"BBBI".to_vec(), ..Model::default()
        };
        let mut reference = vec![1.0, 0.0, 0.0, 3.0];
        let mut x = reference.clone();
        let mut rng = Rng(20);
        for pass in 0..200 {
            perturb(&m, &mut reference, &mut x, pass, &mut rng);
            assert_eq!(x[0], 1.0);
            assert_eq!(x[1], 0.0);
            assert_eq!(x[3], 3.0);
            assert!(x[2] == 0.0 || x[2] == 1.0);
        }
    }

    #[test]
    fn free_and_unbounded_columns_do_not_create_nonfinite_incumbents() {
        let m = Model {
            sense: 1, obj: vec![-1.0, 0.0], lb: vec![0.0, -1e30], ub: vec![1e30, 1e30],
            vtype: b"IC".to_vec(), col_beg: vec![0, 0], col_len: vec![0, 0], ..Model::default()
        };
        let p = Params::default();
        let x = initial_incumbent(&m, &p).unwrap();
        assert!(x.iter().all(|v| v.is_finite() && v.abs() < 1e30));
        assert!(incumbent_objective(&m, &p, &x).is_ok());
    }
}
