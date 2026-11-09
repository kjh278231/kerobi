//! Single-threaded branch-and-bound for a linear MIP in input coordinates.
use rsolver_lp::simplex::{solve_model_opts, Status as LpStatus};
use rsolver_model::Model;
use crate::{root, termination::{self, State, Status}, Params};

pub struct SearchResult {
    pub status: Status,
    pub bound: f64,
    pub incumbent_objective: Option<f64>,
    pub incumbent: Option<Vec<f64>>,
    pub nodes: u64,
    pub iterations: f64,
}

struct Node { model: Model, bound: f64 }

/// Solve in reduced coordinates and restore the incumbent. Unsupported reductions use the original model.
pub fn solve_original(model: &Model, p: &Params) -> Result<SearchResult, &'static str> {
    if p.presolve == 0 { return solve(model, p, None); }
    let (reduced, state) = match crate::presolve_with_state(model, p) {
        Ok(result) => result,
        Err(_) => return solve(model, p, None),
    };
    if state.post.iter().any(|r| !matches!(r, crate::post::PostRec::Fix { .. } | crate::post::PostRec::Subst { .. })) {
        return solve(model, p, None);
    }
    // An empty root has no LP workspace in the current B&B driver.
    if reduced.obj.is_empty() { return solve(model, p, None); }
    let mut result = solve(&reduced, p, None)?;
    if let Some(ref reduced_x) = result.incumbent {
        let mut x = vec![0.0; state.n];
        let mut k = 0;
        for j in 0..state.n {
            if state.col_cnt[j] >= 0 { x[j] = reduced_x[k]; k += 1; }
        }
        for record in state.post.iter().rev() {
            match record {
                crate::post::PostRec::Fix { j, v } => x[*j as usize] = *v,
                crate::post::PostRec::Subst { j, c0, ind, val } => {
                    x[*j as usize] = *c0 + ind.iter().zip(val).map(|(&i, a)| a * x[i as usize]).sum::<f64>();
                }
                _ => unreachable!(),
            }
        }
        x.truncate(model.num_vars());
        let Ok(z) = root::incumbent_objective(model, p, &x) else { return solve(model, p, None); };
        let objective = z * model.sense as f64;
        if result.incumbent_objective.is_some_and(|v| (v - objective).abs() > p.feastol * (1.0 + objective.abs())) {
            return solve(model, p, None);
        }
        result.incumbent = Some(x);
        result.incumbent_objective = Some(objective);
    }
    Ok(result)
}

/// Solve a linear MIP without presolving it. Solutions use the supplied model's coordinates.
pub fn solve(model: &Model, p: &Params, incumbent: Option<&[f64]>) -> Result<SearchResult, &'static str> {
    let started = std::time::Instant::now();
    let root = root::solve_root_progress(model, p, incumbent)?;
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
        let lp = solve_model_opts(&node.model, 1, remaining, false);
        state.nodes += 1.0;
        state.iterations += lp.iter as f64;
        match lp.status {
            LpStatus::Infeasible => {},
            LpStatus::IterLimit => { nodes.push(node); status = Status::IterationLimit; },
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
        incumbent: best, nodes: state.nodes as u64, iterations: state.iterations })
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
