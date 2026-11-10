//! Single-threaded branch-and-bound for a linear MIP in input coordinates.
use rsolver_lp::simplex::{solve_model_deadline, Status as LpStatus};
use rsolver_model::Model;
use crate::{root, termination::{self, State, Status}, Params};

pub struct SearchResult {
    pub status: Status,
    pub bound: f64,
    pub incumbent_objective: Option<f64>,
    pub incumbent: Option<Vec<f64>>,
    pub nodes: u64,
    pub iterations: f64,
    pub path: &'static str,
}

struct Node { model: Model, bound: f64 }

/// Solve in reduced coordinates and restore the incumbent. Unsupported reductions use the original model.
pub fn solve_original(model: &Model, p: &Params) -> Result<SearchResult, &'static str> {
    let started = std::time::Instant::now();
    if p.presolve == 0 { return solve_started(model, p, None, started); }
    let (reduced, state) = match crate::presolve_with_state(model, p) {
        Ok(result) => result,
        Err(_) => return fallback(model, p, started, "presolve_error"),
    };
    if !supported_records(&state.post) {
        return fallback(model, p, started, "unsupported_records");
    }
    let mut result = solve_started(&reduced, p, None, started)?;
    if let Some(ref reduced_x) = result.incumbent {
        let Ok((x, objective)) = restore_candidate(model, p, &state, reduced_x) else { return fallback(model, p, started, "invalid_restoration"); };
        if result.incumbent_objective.is_some_and(|v| (v - objective).abs() > p.feastol * (1.0 + objective.abs())) {
            return fallback(model, p, started, "invalid_restoration");
        }
        result.incumbent = Some(x);
        result.incumbent_objective = Some(objective);
    }
    result.path = "reduced";
    Ok(result)
}

fn supported_records(records: &[crate::post::PostRec]) -> bool {
    records.iter().all(|r| matches!(r, crate::post::PostRec::Fix { .. } | crate::post::PostRec::Subst { .. } | crate::post::PostRec::T9 { .. } | crate::post::PostRec::T10 { .. } | crate::post::PostRec::T11 { .. } | crate::post::PostRec::Par { flag: 0..=2, .. }))
}

/// Restore a reduced incumbent and validate it in the original model.
pub fn restore_candidate(model: &Model, p: &Params, state: &crate::state::State, reduced_x: &[f64]) -> Result<(Vec<f64>, f64), &'static str> {
    if !supported_records(&state.post) { return Err("unsupported records"); }
    if reduced_x.len() != state.col_cnt.iter().take(state.n).filter(|&&c| c >= 0).count() { return Err("wrong reduced solution length"); }
    let mut x = vec![0.0; state.n];
    let mut values = reduced_x.iter();
    for (j, value) in x.iter_mut().enumerate() {
        if state.col_cnt[j] >= 0 { *value = *values.next().unwrap(); }
    }
    restore_records(&mut x, &state.post);
    x.truncate(model.num_vars());
    let objective = root::incumbent_objective(model, p, &x)? * model.sense as f64;
    Ok((x, objective))
}

fn restore_records(x: &mut [f64], records: &[crate::post::PostRec]) {
    for record in records.iter().rev() {
        match record {
            crate::post::PostRec::Fix { j, v } => x[*j as usize] = *v,
            crate::post::PostRec::T9 { i, d1, d2 } => x[*i as usize] = x[*i as usize] * d2 + d1,
            crate::post::PostRec::T10 { j, k, d1, d2, d3 } => {
                x[*j as usize] = d2 * x[*k as usize] + d3 + x[*j as usize] * d1;
            }
            crate::post::PostRec::T11 { .. } => {},
            crate::post::PostRec::Par { x: xc, y: yc, lx, ly, ux, uy, r, flag } if (0..=2).contains(flag) => {
                let (xc, yc) = (*xc as usize, *yc as usize);
                let combined = x[xc];
                rsolver_lp::presolve::postsolve::parallel_col_primal(x, (xc, yc), [*lx, *ly, *ux, *uy, *r], crate::state::INF);
                if *flag == 1 && combined - (combined + 1e-4).floor() <= 1e-4 {
                    let y = x[yc];
                    if y - (y + 1e-5).floor() > 1e-5 {
                        let near = if (if *r > 0.0 { *ly } else { *uy }) * r + lx > combined {
                            combined.ceil()
                        } else if combined > (if *r > 0.0 { *uy } else { *ly }) * r + ux
                            || (combined - combined.floor()).abs() < (combined - combined.ceil()).abs() {
                            combined.floor()
                        } else { combined.ceil() };
                        let y = y.floor().max(*ly);
                        let value = combined - r * y;
                        if value < lx - 1e-5 || ux + 1e-5 < value {
                            x[xc] = value - r;
                            x[yc] = y + 1.0;
                        } else {
                            x[xc] = value;
                            x[yc] = y;
                        }
                        if combined - near != 0.0 {
                            let delta = (combined - near) / (r.abs() + 1.0);
                            x[xc] = (x[xc] + 0.5).floor() + delta;
                            x[yc] += if *r <= 0.0 { -delta } else { delta };
                        }
                    }
                } else if *flag == 2 {
                    let y = x[yc];
                    if y - (y + 1e-5).floor() > 1e-5 {
                        x[yc] = (y + 0.5).floor();
                        x[xc] -= (x[yc] - y) * r;
                        if x[xc] > ux + 1e-4 || x[xc] < lx - 1e-4 {
                            let down = (x[xc] > ux + 1e-4 && *r <= 0.0) || (x[xc] < lx - 1e-4 && *r > 0.0);
                            x[yc] += if down { -1.0 } else { 1.0 };
                            x[xc] += if down { *r } else { -*r };
                        }
                    }
                }
            }
            crate::post::PostRec::Subst { j, c0, ind, val } => {
                x[*j as usize] = *c0 + ind.iter().zip(val).map(|(&i, a)| a * x[i as usize]).sum::<f64>();
            }
            _ => unreachable!(),
        }
    }
}

fn fallback(model: &Model, p: &Params, started: std::time::Instant, reason: &'static str) -> Result<SearchResult, &'static str> {
    let mut result = solve_started(model, p, None, started)?;
    result.path = reason;
    Ok(result)
}

/// Solve a linear MIP without presolving it. Solutions use the supplied model's coordinates.
pub fn solve(model: &Model, p: &Params, incumbent: Option<&[f64]>) -> Result<SearchResult, &'static str> {
    solve_started(model, p, incumbent, std::time::Instant::now())
}

pub(crate) fn deadline(started: std::time::Instant, p: &Params) -> Option<std::time::Instant> {
    std::time::Duration::try_from_secs_f64(p.time_limit).ok().and_then(|d| started.checked_add(d))
}

fn solve_started(model: &Model, p: &Params, incumbent: Option<&[f64]>, started: std::time::Instant) -> Result<SearchResult, &'static str> {
    let root = root::solve_root_progress_started(model, p, incumbent, started)?;
    let sense = model.sense as f64;
    let mut best = root.incumbent;
    let mut state = State { incumbent: root.incumbent_objective.map_or(1e100, |z| sense * z),
        bound: sense * root.bound, root_processed: root.root.is_some(),
        iterations: root.root.as_ref().map_or(0.0, |r| r.lp.iter as f64),
        solutions: i32::from(best.is_some()), ..State::default() };
    let user_cutoff = if p.cutoff < 1e100 { sense * p.cutoff } else { 1e100 };
    let step = crate::cutoff::objective_step(model, p);
    let offset = sense * model.objcon;
    let mut limits = p.clone();
    if model.sense == -1 {
        if p.best_obj_stop != -1e100 { limits.best_obj_stop = -p.best_obj_stop; }
        if p.best_bd_stop != 1e100 { limits.best_bd_stop = -p.best_bd_stop; }
    }
    let mut status = root.status;
    let mut nodes = Vec::new();
    if status == Status::Running {
        let lp = root.root.as_ref().ok_or("missing root relaxation")?;
        if lp.lp.status != LpStatus::Optimal || !branch(&mut nodes, root.domain, state.bound, &lp.lp.x, p.intfeastol) {
            status = Status::Numeric;
        }
    }
    while status == Status::Running {
        state.seconds = started.elapsed().as_secs_f64();
        state.cutoff = crate::cutoff::improvement_cutoff(state.incumbent, user_cutoff, step, offset, p);
        nodes.retain(|n| n.bound < state.cutoff || (n.bound == state.cutoff && state.incumbent >= 1e100));
        state.exhausted = nodes.is_empty();
        if !state.exhausted { state.bound = nodes.iter().map(|n| n.bound).fold(1e100, f64::min); }
        else if state.incumbent < 1e100 { state.bound = state.incumbent; }
        status = termination::check(&state, &limits);
        if status != Status::Running { break; }
        let index = nodes.iter().enumerate().min_by(|a, b| a.1.bound.total_cmp(&b.1.bound)).unwrap().0;
        let node = nodes.swap_remove(index);
        let remaining = (p.iteration_limit - state.iterations).max(0.0).ceil() as u64;
        let lp = solve_model_deadline(&node.model, 1, remaining, deadline(started, p));
        state.nodes += 1.0;
        state.iterations += lp.iter as f64;
        match lp.status {
            LpStatus::Infeasible => {},
            LpStatus::IterLimit => { nodes.push(node); status = if started.elapsed().as_secs_f64() >= p.time_limit { Status::TimeLimit } else { Status::IterationLimit }; },
            LpStatus::Numeric | LpStatus::Unbounded => { nodes.push(node); status = Status::Numeric; },
            LpStatus::Optimal => {
                let bound = node.bound.max(termination::lattice_bound(sense * lp.obj, step, offset, p));
                if bound < state.cutoff || (bound == state.cutoff && state.incumbent >= 1e100) {
                    if let Ok(z) = root::incumbent_objective(model, p, &lp.x) {
                        if z <= user_cutoff && z < state.incumbent {
                            state.incumbent = z; state.solutions += 1; best = Some(lp.x);
                        }
                    } else if !branch(&mut nodes, node.model, bound, &lp.x, p.intfeastol) {
                        nodes.push(Node { model: model.clone(), bound });
                        status = Status::Numeric;
                    }
                }
            }
        }
    }
    if !nodes.is_empty() { state.bound = nodes.iter().map(|n| n.bound).fold(1e100, f64::min); }
    status = termination::finish(status, best.is_some(), p);
    Ok(SearchResult { status, bound: sense * state.bound, incumbent_objective: best.as_ref().map(|_| sense * state.incumbent),
        incumbent: best, nodes: state.nodes as u64, iterations: state.iterations, path: "original" })
}

fn branch(nodes: &mut Vec<Node>, model: Model, bound: f64, x: &[f64], tol: f64) -> bool {
    let j = (0..model.obj.len()).filter(|&j| matches!(model.vtype[j], b'B' | b'I') && (x[j] - x[j].round()).abs() > tol)
        .max_by(|&a, &b| (x[a] - x[a].round()).abs().total_cmp(&(x[b] - x[b].round()).abs()));
    let Some(j) = j else { return false };
    let down = x[j].floor();
    let up = x[j].ceil();
    if down >= model.lb[j] {
        let mut child = model.clone(); child.ub[j] = child.ub[j].min(down);
        nodes.push(Node { model: child, bound });
    }
    if up <= model.ub[j] {
        let mut child = model; child.lb[j] = child.lb[j].max(up);
        nodes.push(Node { model: child, bound });
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restores_mixed_records_in_reverse_order() {
        use crate::post::PostRec;
        let records = [PostRec::T9 { i: 2, d1: 3.0, d2: 4.0 }, PostRec::Fix { j: 1, v: -7.0 },
            PostRec::T9 { i: 2, d1: -2.0, d2: 5.0 }];
        let mut x = [0.0, 0.0, 2.0];
        restore_records(&mut x, &records);
        assert_eq!(x, [0.0, -7.0, 35.0]);
    }
    #[test]
    fn restores_parallel_columns_like_native() {
        use crate::post::PostRec;
        let cases = [
            (0.0, 0.0, 3.0, 5.0, 2.0, 7.0, (0.0, 3.5)),
            (-3.0, -2.0, 4.0, 5.0, -2.0, -7.0, (3.0, 5.0)),
            (-1e30, -1e30, 4.0, 3.0, 2.0, 3.0, (-3.0, 3.0)),
            (1.0, 2.0, 6.0, 5.0, 2.0, 20.0, (6.0, 7.0)),
        ];
        for (lx, ly, ux, uy, r, value, expected) in cases {
            let mut values = [0.0, 0.0, value];
            restore_records(&mut values, &[PostRec::Par { x: 2, y: 1, lx, ly, ux, uy, r, flag: 0 }]);
            assert_eq!((values[2], values[1]), expected);
        }
    }
    #[test]
    fn restores_integer_parallel_columns_like_native() {
        use crate::post::PostRec;
        for line in include_str!("../../../bench/oracle/mip_postsolve_parallel.txt").lines() {
            let v: Vec<f64> = line.split_whitespace().map(|x| x.parse().unwrap()).collect();
            let mut x = [0.0, 0.0, v[6]];
            restore_records(&mut x, &[PostRec::Par { x: 2, y: 1, lx: v[1], ly: v[2], ux: v[3], uy: v[4], r: v[5], flag: v[0] as i32 }]);
            assert_eq!((x[2], x[1]), (v[7], v[8]), "{line}");
        }
    }
    #[test]
    fn restores_affine_and_slack_records_like_native() {
        use crate::post::PostRec;
        let records = [PostRec::T10 { j: 2, k: 1, d1: 2.0, d2: 3.0, d3: 4.0 },
            PostRec::T11 { r: 0, j: 2, a: 7.0 }, PostRec::Fix { j: 1, v: -7.0 }];
        let mut x = [0.0, 0.0, 2.0];
        restore_records(&mut x, &records);
        assert_eq!(x, [0.0, -7.0, -13.0]);
    }
    #[test]
    fn solves_and_restores_an_empty_reduced_model() {
        let model = Model { sense: 1, obj: vec![3.0], lb: vec![2.0], ub: vec![2.0],
            vtype: b"I".to_vec(), col_beg: vec![0], col_len: vec![0], ..Model::default() };
        let result = solve_original(&model, &Params::default()).unwrap();
        assert_eq!(result.path, "reduced");
        assert_eq!(result.status, Status::Optimal);
        assert_eq!(result.incumbent_objective, Some(6.0));
        assert_eq!(result.incumbent, Some(vec![2.0]));
    }
    #[test]
    fn original_pipeline_restores_eliminated_variable() {
        let model = Model { sense: 1, obj: vec![3.0, -1.0, -1.0, -1.0],
            lb: vec![2.0, 0.0, 0.0, 0.0], ub: vec![2.0, 1.0, 1.0, 1.0], vtype: b"IBBB".to_vec(),
            col_beg: vec![0, 0, 2, 4], col_len: vec![0, 2, 2, 2],
            col_ind: vec![0, 2, 0, 1, 1, 2], col_val: vec![1.0; 6],
            row_sense: vec![b'<'; 3], rhs: vec![1.0; 3], ..Model::default() };
        let p = (1..100).map(|prelimit| Params { prelimit, mip_gap: 0.0, mip_gap_abs: 0.0, ..Params::default() })
            .find(|p| {
                let (q, s) = crate::presolve_with_state(&model, p).unwrap();
                !q.obj.is_empty() && s.post.iter().any(|r| matches!(r, crate::post::PostRec::Fix { j: 0, .. }))
                    && s.post.iter().all(|r| matches!(r, crate::post::PostRec::Fix { .. } | crate::post::PostRec::Subst { .. }))
            }).expect("a presolve limit that eliminates the fixed variable");
        let result = solve_original(&model, &p).unwrap();
        assert_eq!(result.status, Status::Optimal);
        assert_eq!(result.incumbent_objective, Some(5.0));
        let x = result.incumbent.unwrap();
        assert_eq!(x[0], 2.0);
        assert_eq!(x[1..].iter().sum::<f64>(), 1.0);
    }
    fn triangle(sense: i32) -> Model {
        Model { sense, obj: vec![1.0; 3], lb: vec![0.0; 3], ub: vec![1.0; 3], vtype: b"BBB".to_vec(),
            col_beg: vec![0, 2, 4], col_len: vec![2; 3], col_ind: vec![0, 2, 0, 1, 1, 2],
            col_val: vec![1.0; 6], row_sense: vec![b'<'; 3], rhs: vec![1.0; 3], ..Model::default() }
    }
    #[test]
    fn searches_fractional_root_and_respects_node_limit() {
        let m = triangle(-1);
        let p = Params { heuristics: 0.0, objgcd: 0, mip_gap: 0.0, mip_gap_abs: 0.0, ..Params::default() };
        let r = solve(&m, &p, None).unwrap();
        assert_eq!(r.status, Status::Optimal);
        assert_eq!(r.incumbent_objective, Some(1.0));
        assert!(r.nodes > 0);
        assert_eq!(r.bound, 1.0);
        let limited = solve(&m, &Params { node_limit: 1.0, ..p }, None).unwrap();
        assert_eq!(limited.status, Status::NodeLimit);
        assert_eq!(limited.nodes, 1);
        assert!(limited.bound >= 1.0);
    }
}
