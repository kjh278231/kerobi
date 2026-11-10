//! MIP root LP relaxation (docs/m2/m2_root.md): `5bbf7f` → `5daa82` (Mip::new) → `64da9f` (root) → `69f0bd` (LP holder,
//! method) → `5ce67b` → `5ccea5` → `cda5b(lp, method, no basis)`. On all 37 miplib_small models with a root the root
//! LP is a fresh dual simplex (method 1, no workspace, no warm basis) on the presolved model with the domain store's
//! bounds and the column types kept (spx+0xb8). Equality-based domain initialization and optional validated
//! incumbent bounds and initial heuristics are applied before the diagnostic LP. The progress APIs add generic
//! linear propagation and whole-MIP termination checks; cuts and branching remain.
use rsolver_lp::simplex::{solve_model_deadline, LpResult, Status};
use rsolver_model::Model;

use crate::Params;

/// `69f0bd` for a MILP (no Q, `a7f0ad` == 0): the root LP method from Method, Threads and the model size (rows m,
/// columns n, nonzeros nnz of model+0xe8 +0x8/+0xc/+0x10). Returns (method, concurrent method).
pub fn root_method(p: &Params, m: i32, n: i32, nnz: i64) -> (i32, i32) {
    let cm = p.concurrent_method;
    match p.method {
        // explicit concurrent / deterministic concurrent: one thread makes it dual
        3..=5 => {
            if p.threads == 1 {
                (1, cm)
            } else {
                (p.method, cm)
            }
        }
        // automatic: large models go to concurrent (4), even with Threads=1
        -1 => {
            if nnz >= 500_000 && m > 999 && n > 19_999 {
                (4, cm)
            } else if nnz >= 100_000 && m > 999 && n > 9_999 {
                (4, 3)
            } else {
                (1, cm)
            }
        }
        _ => (p.method, cm),
    }
}

/// The root LP: the method `69f0bd` chose and the simplex result.
pub struct RootLp {
    pub method: i32,
    pub lp: LpResult,
    /// Feasible solution in presolved coordinates, retained separately from the relaxation.
    pub incumbent: Option<Vec<f64>>,
}

/// Initialize binary bounds and implied integer domains from equality rows (`66bdf6`). An equality with one continuous
/// column implies integrality when its RHS and every coefficient are integer multiples of that column's
/// coefficient. Newly integral columns can expose further such equalities.
fn initialize_domain(model: &Model, p: &Params) -> Model {
    let mut domain = model.clone();
    for j in 0..domain.obj.len() {
        if domain.vtype[j] == b'B' {
            domain.lb[j] = domain.lb[j].max(0.0);
            domain.ub[j] = domain.ub[j].min(1.0);
            if domain.lb[j] > domain.ub[j] {
                for endpoint in [0.0, 1.0] {
                    if endpoint >= model.lb[j] - p.feastol && endpoint <= model.ub[j] + p.feastol {
                        domain.lb[j] = endpoint;
                        domain.ub[j] = endpoint;
                        break;
                    }
                }
            }
        }
    }
    let mut rows = vec![Vec::new(); model.rhs.len()];
    for j in 0..model.obj.len() {
        let start = model.col_beg[j] as usize;
        for k in start..start + model.col_len[j] as usize {
            rows[model.col_ind[k] as usize].push((j, model.col_val[k]));
        }
    }
    let integral = |x: f64| x.is_finite() && x - (x + 1e-10).floor() <= 1e-10;
    loop {
        let mut changed = false;
        for (i, row) in rows.iter().enumerate() {
            if model.row_sense[i] != b'=' { continue; }
            let mut continuous = row.iter().filter(|&&(j, _)| domain.vtype[j] == b'C');
            let Some(&(j, a)) = continuous.next() else { continue };
            if continuous.next().is_some() || a == 0.0 || !integral(model.rhs[i] / a)
                || row.iter().any(|&(_, v)| !integral(v / a)) { continue; }
            if domain.lb[j] > -1e30 { domain.lb[j] = (domain.lb[j] - p.intfeastol).ceil(); }
            if domain.ub[j] < 1e30 { domain.ub[j] = (domain.ub[j] + p.intfeastol).floor(); }
            domain.vtype[j] = if domain.lb[j] == 0.0 && domain.ub[j] == 1.0 { b'B' } else { b'I' };
            changed = true;
        }
        if !changed { break; }
    }
    domain
}

/// Validate an incumbent in the input model coordinates and return its minimization objective.
pub(crate) fn incumbent_objective(model: &Model, p: &Params, x: &[f64]) -> Result<f64, &'static str> {
    if x.len() != model.obj.len() { return Err("incumbent length differs from model"); }
    let mut activity = vec![0.0; model.rhs.len()];
    for (j, &v) in x.iter().enumerate() {
        let (lo, hi) = if model.vtype[j] == b'B' {
            (model.lb[j].max(0.0), model.ub[j].min(1.0))
        } else { (model.lb[j], model.ub[j]) };
        if !v.is_finite() || v < lo - p.feastol || v > hi + p.feastol
            || (matches!(model.vtype[j], b'B' | b'I') && (v - v.round()).abs() > p.intfeastol) {
            return Err("incumbent violates a column domain");
        }
        let start = model.col_beg[j] as usize;
        for k in start..start + model.col_len[j] as usize {
            activity[model.col_ind[k] as usize] += model.col_val[k] * v;
        }
    }
    for (i, &a) in activity.iter().enumerate() {
        let violation = match model.row_sense[i] {
            b'<' => a - model.rhs[i], b'>' => model.rhs[i] - a, _ => (a - model.rhs[i]).abs(),
        };
        if !a.is_finite() || violation > p.feastol { return Err("incumbent violates a row"); }
    }
    let sense = model.sense as f64;
    let objective = sense * (model.objcon + model.obj.iter().zip(x).map(|(c, v)| c * v).sum::<f64>());
    if !objective.is_finite() { return Err("incumbent objective is not finite"); }
    Ok(objective)
}

struct ObjectiveCutoff { incumbent: f64, user: f64, step: f64, offset: f64 }

impl ObjectiveCutoff {
    fn new(domain: &Model, p: &Params, incumbent: Option<f64>) -> Self {
        Self {
            incumbent: incumbent.unwrap_or(1e100),
            user: if p.cutoff < 1e100 { domain.sense as f64 * p.cutoff } else { 1e100 },
            step: crate::cutoff::objective_step(domain, p),
            offset: domain.sense as f64 * domain.objcon,
        }
    }
}

fn apply_incumbent(model: &mut Model, p: &Params, x: &[f64]) -> Result<(), &'static str> {
    let upper = ObjectiveCutoff::new(model, p, Some(incumbent_objective(model, p, x)?));
    tighten_for_objective(model, p, &upper);
    Ok(())
}

fn tighten_for_objective(model: &mut Model, p: &Params, upper: &ObjectiveCutoff) {
    let sense = model.sense as f64;
    let offset = sense * model.objcon;
    let step = crate::cutoff::inherit_step(crate::cutoff::objective_step(model, p), offset, upper.step, upper.offset);
    let cutoff = crate::cutoff::improvement_cutoff(upper.incumbent, upper.user, step, offset, p);
    if cutoff >= 1e100 { return; }
    let n = model.obj.len();
    let mut prefix = vec![(0.0, 0.0, 0usize); n + 1];
    for j in 0..n {
        let c = sense * model.obj[j];
        let b = if c > 0.0 { model.lb[j] } else { model.ub[j] };
        let (sum, magnitude, infinite) = prefix[j];
        prefix[j + 1] = if c == 0.0 { prefix[j] } else if b.abs() >= 1e30 {
            (sum, magnitude, infinite + 1)
        } else { (sum + c * b, magnitude + (c * b).abs(), infinite) };
    }
    let mut suffix = (0.0, 0.0, 0usize);
    for j in (0..n).rev() {
        let c = sense * model.obj[j];
        let b = if c > 0.0 { model.lb[j] } else { model.ub[j] };
        let (sum, magnitude, infinite) = prefix[j];
        if c != 0.0 && infinite + suffix.2 == 0 {
            let min_other = sense * model.objcon + (sum + suffix.0);
            let error = (magnitude + suffix.1 + model.objcon.abs()) * f64::EPSILON * (n as f64 + 2.0);
            let bound = (cutoff - min_other + error) / c;
            if bound.is_finite() {
                if c > 0.0 {
                    let bound = if matches!(model.vtype[j], b'B' | b'I') { (bound + p.intfeastol).floor() } else { bound };
                    if bound >= model.lb[j] - p.feastol { model.ub[j] = model.ub[j].min(bound.max(model.lb[j])); }
                } else {
                    let bound = if matches!(model.vtype[j], b'B' | b'I') { (bound - p.intfeastol).ceil() } else { bound };
                    if bound <= model.ub[j] + p.feastol { model.lb[j] = model.lb[j].max(bound.min(model.ub[j])); }
                }
            }
        }
        if c != 0.0 {
            if b.abs() >= 1e30 { suffix.2 += 1; }
            else { suffix.0 += c * b; suffix.1 += (c * b).abs(); }
        }
    }
}

/// Solve the root LP of a presolved MIP (`5ce67b` at the root, `local_110` = root method, no warm start). None when
/// presolve removed every column ("All rows and columns removed": no root relaxation line).
// shortcut: 66bdf6 inequality inference, row propagation and the full heuristic scheduler remain unported; extend with MIP state.
// shortcut: methods other than 0/1 (concurrent 4 for nnz ≥ 1e5 with m > 999, n > 9999; barrier 2) run the dual;
// none of miplib_small takes them; port the 98b671 branch of 5ccea5 when a model in the set does.
pub fn solve_root(presolved: &Model, p: &Params) -> Option<RootLp> {
    solve_root_with_incumbent(presolved, p, None).expect("no incumbent to validate")
}

/// Find an initial feasible solution in presolved coordinates, without changing the input model.
pub fn initial_incumbent(presolved: &Model, p: &Params) -> Option<Vec<f64>> {
    validated_initial_incumbent(presolved, &initialize_domain(presolved, p), p)
}

fn validated_initial_incumbent(presolved: &Model, domain: &Model, p: &Params) -> Option<Vec<f64>> {
    validated_candidate(presolved, domain, p).map(|(x, _)| x)
}

fn validated_candidate(model: &Model, domain: &Model, p: &Params) -> Option<(Vec<f64>, f64)> {
    let x = crate::heur::initial_incumbent(domain, p)?;
    let objective = incumbent_objective(model, p, &x).ok()?;
    Some((x, objective))
}

fn empty_domain(model: &Model) -> bool {
    model.lb.iter().zip(&model.ub).any(|(lo, hi)| lo > hi)
}

fn prepare_domain(presolved: &Model, p: &Params, incumbent: Option<&[f64]>)
    -> Result<(Model, Option<Vec<f64>>), &'static str>
{
    if let Some(x) = incumbent { incumbent_objective(presolved, p, x)?; }
    let mut domain = initialize_domain(presolved, p);
    if empty_domain(&domain) {
        if incumbent.is_some() { return Err("incumbent cannot satisfy an empty column domain"); }
        return Ok((domain, None));
    }
    let incumbent = match incumbent {
        Some(x) => Some(x.to_vec()),
        None => validated_initial_incumbent(presolved, &domain, p),
    };
    if let Some(x) = &incumbent { apply_incumbent(&mut domain, p, x)?; }
    else {
        let upper = ObjectiveCutoff::new(&domain, p, None);
        tighten_for_objective(&mut domain, p, &upper);
    }
    // 5ca999 uses the separate LP type array (+0x660), not 66bdf6's domain types (+0x650).
    domain.vtype.clone_from(&presolved.vtype);
    Ok((domain, incumbent))
}

/// Construct the LP domain; absent an explicit incumbent, run the initial heuristic automatically.
pub fn root_domain(presolved: &Model, p: &Params, incumbent: Option<&[f64]>) -> Result<Model, &'static str> {
    let (domain, _) = prepare_domain(presolved, p, incumbent)?;
    if empty_domain(&domain) { return Err("root has an empty column domain"); }
    Ok(domain)
}

/// Solve using an optional feasible incumbent in presolved column coordinates.
pub fn solve_root_with_incumbent(presolved: &Model, p: &Params, incumbent: Option<&[f64]>) -> Result<Option<RootLp>, &'static str> {
    let (domain, incumbent) = prepare_domain(presolved, p, incumbent)?;
    Ok(solve_domain(presolved, &domain, p, incumbent))
}

fn solve_domain(presolved: &Model, domain: &Model, p: &Params, incumbent: Option<Vec<f64>>) -> Option<RootLp> {
    solve_domain_limit(presolved, domain, p, incumbent, u64::MAX, None)
}

fn solve_domain_limit(presolved: &Model, domain: &Model, p: &Params, incumbent: Option<Vec<f64>>, iterations: u64, deadline: Option<std::time::Instant>) -> Option<RootLp> {
    let n = presolved.obj.len();
    if n == 0 { return None; }
    let (method, _) = root_method(p, presolved.rhs.len() as i32, n as i32, presolved.num_nz() as i64);
    let lp = if empty_domain(domain) {
        LpResult { status: Status::Infeasible, obj: f64::NAN, x: Vec::new(), iter: 0,
            trace: Vec::new(), path: Vec::new(), basis: Vec::new() }
    } else { solve_model_deadline(domain, if method == 0 { 0 } else { 1 }, iterations, deadline) };
    Some(RootLp { method, lp, incumbent })
}

/// A feasible incumbent in the original model's coordinates and objective direction.
pub struct OriginalIncumbent { pub objective: f64, pub x: Vec<f64> }
/// The unchanged presolve result and the domain actually passed to the root LP.
pub struct PresolvedRoot {
    pub presolved: Model,
    pub root: Option<RootLp>,
    pub lower_bounds: Vec<f64>,
    pub upper_bounds: Vec<f64>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum InitialRootError { Presolve(i32), UnsupportedModel }
pub struct PresolveRootRun {
    pub original_incumbent: Option<OriginalIncumbent>,
    pub presolve: Result<PresolvedRoot, InitialRootError>,
}

/// Run the initial heuristic once on the original MIP, then presolve and solve its root relaxation.
pub fn solve_root_from_original(raw: &Model, p: &Params) -> PresolveRootRun {
    if raw.vtype.iter().any(|t| !matches!(t, b'C' | b'B' | b'I'))
        || (!raw.obj.is_empty() && !raw.vtype.iter().any(|t| matches!(t, b'B' | b'I'))) {
        return PresolveRootRun { original_incumbent: None, presolve: Err(InitialRootError::UnsupportedModel) };
    }
    let original_domain = initialize_domain(raw, p);
    let candidate = validated_candidate(raw, &original_domain, p);
    let upper = ObjectiveCutoff::new(&original_domain, p, candidate.as_ref().map(|(_, z)| *z));
    let original_incumbent = candidate.map(|(x, z)| OriginalIncumbent { objective: z * raw.sense as f64, x });
    let presolve = crate::presolve(raw, p).map_err(InitialRootError::Presolve).map(|presolved| {
        let mut domain = initialize_domain(&presolved, p);
        if !empty_domain(&domain) {
            tighten_for_objective(&mut domain, p, &upper);
        }
        domain.vtype.clone_from(&presolved.vtype);
        let root = solve_domain(&presolved, &domain, p, None);
        PresolvedRoot { presolved, root, lower_bounds: domain.lb, upper_bounds: domain.ub }
    });
    PresolveRootRun { original_incumbent, presolve }
}

/// Result of follow-up propagation and the whole-MIP check. Running means branching is still required.
pub struct RootProgress {
    pub status: crate::termination::Status,
    /// Objective values in the input model's direction, unlike the internal minimization state.
    pub bound: f64,
    pub incumbent_objective: Option<f64>,
    pub incumbent: Option<Vec<f64>>,
    pub root: Option<RootLp>,
    pub domain: Model,
    pub propagation_complete: bool,
}

/// Continue past the diagnostic root LP: propagate rows/cutoff, submit integral LP solutions, check termination.
/// Input and returned solutions use the same (usually presolved) coordinates.
pub fn solve_root_progress(model: &Model, p: &Params, incumbent: Option<&[f64]>) -> Result<RootProgress, &'static str> {
    solve_root_progress_started(model, p, incumbent, std::time::Instant::now())
}

pub(crate) fn solve_root_progress_started(model: &Model, p: &Params, incumbent: Option<&[f64]>, started: std::time::Instant) -> Result<RootProgress, &'static str> {
    if model.vtype.iter().any(|t| !matches!(t, b'C' | b'B' | b'I')) {
        return Err("root progress requires C, B, or I columns");
    }
    if let Some(x) = incumbent { incumbent_objective(model, p, x)?; }
    let domain = initialize_domain(model, p);
    let candidate = incumbent.map(<[f64]>::to_vec)
        .or_else(|| if empty_domain(&domain) { None } else { validated_initial_incumbent(model, &domain, p) });
    let objective = candidate.as_ref().map(|x| incumbent_objective(model, p, x)).transpose()?;
    let upper = ObjectiveCutoff::new(&domain, p, objective);
    Ok(progress_domain(model, domain, p, candidate, upper, started))
}

fn normalized_limits(p: &Params, sense: i32) -> Params {
    let mut limits = p.clone();
    if sense == -1 {
        if p.best_obj_stop != -1e100 { limits.best_obj_stop = -p.best_obj_stop; }
        if p.best_bd_stop != 1e100 { limits.best_bd_stop = -p.best_bd_stop; }
    }
    limits
}

fn progress_domain(model: &Model, mut domain: Model, p: &Params, mut candidate: Option<Vec<f64>>,
    mut upper: ObjectiveCutoff, started: std::time::Instant) -> RootProgress
{
    use crate::termination::{self, State, Status as MipStatus};
    let mut objective = (upper.incumbent < 1e100).then_some(upper.incumbent);
    // The pool accepts only solutions inside the user cutoff, even if a rejected heuristic is feasible.
    if objective.is_some_and(|z| z > upper.user) { candidate = None; objective = None; upper.incumbent = 1e100; }
    let sense = model.sense as f64;
    let mut state = State { incumbent: objective.unwrap_or(1e100), cutoff: crate::cutoff::improvement_cutoff(
        upper.incumbent, upper.user, upper.step, upper.offset, p), solutions: i32::from(objective.is_some()),
        ..State::default() };
    let limits = normalized_limits(p, model.sense);
    state.seconds = started.elapsed().as_secs_f64();
    let mut status = termination::check(&state, &limits);
    let mut complete = false;
    let mut root = None;
    if status == MipStatus::Running {
        let propagation = crate::propagate::run(&mut domain, p, Some(state.cutoff));
        complete = propagation.complete;
        if propagation.conflict {
            state.bound = state.cutoff;
            state.exhausted = true;
        } else {
            domain.vtype.clone_from(&model.vtype);
            root = solve_domain_limit(model, &domain, p, candidate.clone(), p.iteration_limit.max(0.0).ceil() as u64, crate::bnb::deadline(started, p));
            if let Some(r) = &root {
                state.iterations = r.lp.iter as f64;
                state.root_processed = true;
                match r.lp.status {
                    Status::Optimal => {
                        state.bound = termination::lattice_bound(sense * r.lp.obj, upper.step, upper.offset, p);
                        if let Ok(z) = incumbent_objective(model, p, &r.lp.x) {
                            if z <= upper.user && z < state.incumbent {
                                state.incumbent = z;
                                state.solutions += 1;
                                objective = Some(z);
                                candidate = Some(r.lp.x.clone());
                                state.cutoff = crate::cutoff::improvement_cutoff(z, upper.user, upper.step, upper.offset, p);
                                // Submission can tighten another row even after an integral LP.
                                let follow = crate::propagate::run(&mut domain, p, Some(state.cutoff));
                                complete = follow.complete;
                                state.exhausted = follow.conflict;
                            }
                        }
                    }
                    Status::Infeasible => state.exhausted = true,
                    // An unbounded relaxation alone does not certify an unbounded MIP.
                    Status::Unbounded => state.bound = -1e100,
                    Status::IterLimit => state.status = if started.elapsed().as_secs_f64() >= p.time_limit { MipStatus::TimeLimit } else { MipStatus::IterationLimit },
                    Status::Numeric => state.status = MipStatus::Numeric,
                }
            } else {
                // Empty models still have a constant objective and may have contradictory empty rows.
                let z = sense * model.objcon;
                state.bound = z;
                if z <= upper.user && z < state.incumbent {
                    objective = Some(z); candidate = Some(Vec::new());
                    state.incumbent = z; state.solutions += 1;
                }
                state.exhausted = true;
            }
        }
        state.seconds = started.elapsed().as_secs_f64();
        status = termination::check(&state, &limits);
    }
    status = termination::finish(status, objective.is_some(), p);
    if state.exhausted && objective.is_some() { state.bound = state.incumbent; }
    if status == MipStatus::Optimal { state.bound = state.bound.min(state.incumbent); }
    domain.vtype.clone_from(&model.vtype);
    if let Some(r) = &mut root { r.incumbent.clone_from(&candidate); }
    RootProgress { status, bound: sense * state.bound, incumbent_objective: objective.map(|z| sense * z),
        incumbent: candidate, root, domain, propagation_complete: complete }
}

/// Original-coordinate incumbent stays outside the presolved progress result; no guessed crush mapping.
pub struct PresolveProgressRun {
    pub original_incumbent: Option<OriginalIncumbent>,
    pub presolve: Result<(Model, RootProgress), InitialRootError>,
}

/// Original heuristic → unchanged presolve → cutoff/row propagation → root LP → MIP termination.
pub fn solve_root_progress_from_original(raw: &Model, p: &Params) -> PresolveProgressRun {
    let started = std::time::Instant::now();
    if raw.vtype.iter().any(|t| !matches!(t, b'C' | b'B' | b'I'))
        || (!raw.obj.is_empty() && !raw.vtype.iter().any(|t| matches!(t, b'B' | b'I'))) {
        return PresolveProgressRun { original_incumbent: None, presolve: Err(InitialRootError::UnsupportedModel) };
    }
    let original_domain = initialize_domain(raw, p);
    let candidate = validated_candidate(raw, &original_domain, p);
    let upper = ObjectiveCutoff::new(&original_domain, p, candidate.as_ref().map(|(_, z)| *z));
    let original_incumbent = candidate.map(|(x, z)| OriginalIncumbent { objective: z * raw.sense as f64, x });
    let presolve = crate::presolve(raw, p).map_err(InitialRootError::Presolve).map(|presolved| {
        let domain = initialize_domain(&presolved, p);
        let offset = domain.sense as f64 * domain.objcon;
        let step = crate::cutoff::inherit_step(crate::cutoff::objective_step(&domain, p), offset, upper.step, upper.offset);
        let upper = ObjectiveCutoff { step, offset, ..upper };
        let progress = progress_domain(&presolved, domain, p, None, upper, started);
        (presolved, progress)
    });
    PresolveProgressRun { original_incumbent, presolve }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_maximization_normalizes_cutoff_bounds_and_objective_stops() {
        use crate::termination::Status as MipStatus;
        let m = Model { sense: -1, obj: vec![1.0, 1.0], lb: vec![0.0; 2], ub: vec![1.0; 2],
            vtype: b"BB".to_vec(), col_beg: vec![0,1], col_len: vec![1,1], col_ind: vec![0,0],
            col_val: vec![2.0,2.0], rhs: vec![3.0], row_sense: vec![b'<'], ..Model::default() };
        let p = Params { heuristics: 0.0, ..Params::default() };
        let r = solve_root_progress(&m, &p, None).unwrap();
        assert_eq!(r.status, MipStatus::Running);
        assert_eq!(r.bound, 1.0);
        assert_eq!(r.root.unwrap().lp.obj, 1.5);
        let r = solve_root_progress(&m, &Params { best_bd_stop: 1.1, ..p.clone() }, None).unwrap();
        assert_eq!(r.status, MipStatus::UserObjLimit);
        let run = solve_root_progress_from_original(&m, &Params { best_bd_stop: 1.1, prelimit: 0, ..p.clone() });
        assert_eq!(run.presolve.unwrap().1.status, MipStatus::UserObjLimit);
        let r = solve_root_progress(&m, &Params { cutoff: 2.0, ..p }, Some(&[1.0,0.0])).unwrap();
        assert_eq!(r.status, MipStatus::Cutoff);
        assert!(r.incumbent.is_none());
    }

    #[test]
    fn objective_lattice_tightens_integer_bounds_and_can_be_disabled() {
        let m = Model { sense: 1, objcon: 7.0, obj: vec![2.0, 3.0],
            lb: vec![0.0; 2], ub: vec![20.0; 2], vtype: b"II".to_vec(),
            col_beg: vec![0; 2], col_len: vec![0; 2], ..Model::default() };
        let mut domain = m.clone();
        apply_incumbent(&mut domain, &Params::default(), &[2.0, 2.0]).unwrap();
        assert_eq!(domain.ub, vec![4.0, 3.0]);
        let p = Params { objgcd: 0, ..Params::default() };
        let mut domain = m.clone();
        apply_incumbent(&mut domain, &p, &[2.0, 2.0]).unwrap();
        assert_eq!(domain.ub, vec![5.0, 3.0]);
        assert_eq!(m.ub, vec![20.0; 2]);
    }

    #[test]
    fn user_cutoff_applies_without_an_incumbent_in_both_directions() {
        for sense in [1, -1] {
            let m = Model { sense, objcon: 7.0, obj: vec![2.0, 0.0],
                lb: vec![0.0; 2], ub: vec![20.0; 2], vtype: b"CI".to_vec(),
                col_beg: vec![0; 2], col_len: vec![0; 2], ..Model::default() };
            let p = Params { cutoff: 15.0, heuristics: 0.0, ..Params::default() };
            let d = root_domain(&m, &p, None).unwrap();
            if sense == 1 { assert!((d.ub[0] - 4.0).abs() < 1e-12); }
            else { assert!((d.lb[0] - 4.0).abs() < 1e-12); }
            let run = solve_root_from_original(&m, &Params { prelimit: 0, ..p });
            assert!(run.original_incumbent.is_none());
            let result = run.presolve.unwrap();
            if sense == 1 { assert!((result.upper_bounds[0] - 4.0).abs() < 1e-12); }
            else { assert!((result.lower_bounds[0] - 4.0).abs() < 1e-12); }
        }
    }

    #[test]
    fn original_pipeline_handles_presolve_failure_and_preserves_incumbent() {
        let p = Params::default();
        let impossible = Model { sense: 1, obj: vec![0.0], lb: vec![2.5], ub: vec![2.5],
            vtype: b"I".to_vec(), col_beg: vec![0], col_len: vec![0], ..Model::default() };
        let run = solve_root_from_original(&impossible, &p);
        assert!(run.original_incumbent.is_none());
        assert!(matches!(run.presolve, Err(InitialRootError::Presolve(3))));
        let unbounded = Model { sense: 1, obj: vec![-1.0], lb: vec![0.0], ub: vec![1e30],
            vtype: b"I".to_vec(), col_beg: vec![0], col_len: vec![0], ..Model::default() };
        let run = solve_root_from_original(&unbounded, &p);
        assert!(run.original_incumbent.is_some());
        assert!(matches!(run.presolve, Err(InitialRootError::Presolve(5))));
    }

    #[test]
    fn original_pipeline_keeps_maximization_incumbent_after_elimination() {
        let m = Model { sense: -1, objcon: 7.0, obj: vec![3.0, 2.0],
            lb: vec![1.0, 0.0], ub: vec![1.0, 1.0], vtype: b"BB".to_vec(),
            col_beg: vec![0, 0], col_len: vec![0, 0], ..Model::default() };
        let run = solve_root_from_original(&m, &Params::default());
        let inc = run.original_incumbent.unwrap();
        assert_eq!(inc.x, vec![1.0, 1.0]);
        assert_eq!(inc.objective, 12.0);
        let result = run.presolve.unwrap();
        assert!(result.presolved.obj.len() < inc.x.len());
        assert!(result.root.is_none());
        assert_eq!(result.presolved.sense, 1);
        assert_eq!(result.presolved.objcon, -12.0);
        assert_eq!(m.objcon, 7.0);
    }

    #[test]
    fn original_cutoff_uses_the_reduced_maximization_objective() {
        let raw = Model { sense: -1, objcon: 7.0, obj: vec![3.0, 2.0, 1.0],
            lb: vec![1.0, 0.0, 0.0], ub: vec![1.0, 1.0, 3.0], vtype: b"BBC".to_vec(),
            col_beg: vec![0, 0, 1], col_len: vec![0, 1, 1], col_ind: vec![0, 0],
            col_val: vec![1.0, 1.0], row_sense: b"<".to_vec(), rhs: vec![1.5], ..Model::default() };
        let p = Params::from_pairs(&[("GURO_PAR_PRELIMIT".into(), 4.0)]);
        let run = solve_root_from_original(&raw, &p);
        let inc = run.original_incumbent.unwrap();
        let result = run.presolve.unwrap();
        assert_eq!(inc.x.len(), 3);
        assert_eq!(inc.objective, 12.5);
        assert_eq!(result.presolved.obj, vec![2.0, 1.0]);
        assert_eq!(result.presolved.objcon, 10.0);
        assert!(result.lower_bounds[1] > 0.5 && result.lower_bounds[1] < 0.50001);
        assert_eq!(result.presolved.lb[1], 0.0);
        let root = result.root.unwrap();
        assert_eq!(root.lp.status, Status::Optimal);
        assert!(root.incumbent.is_none());
    }

    #[test]
    fn root_without_columns_returns_none_even_with_residual_rows() {
        let p0 = Model { sense: 1, rhs: vec![0.0], row_sense: b"<".to_vec(), ..Model::default() };
        assert!(solve_domain(&p0, &p0, &Params::default(), None).is_none());
    }

    #[test]
    fn domain_infers_integer_equalities_without_changing_input() {
        let m = Model {
            obj: vec![0.0; 3], lb: vec![0.2, 0.1, 0.0], ub: vec![3.8, 3.9, 1.0],
            vtype: vec![b'C', b'C', b'B'],
            col_beg: vec![0, 1, 3], col_len: vec![1, 2, 1],
            col_ind: vec![0, 0, 1, 1], col_val: vec![2.0, 2.0, 1.0, 1.0],
            row_sense: vec![b'=', b'='], rhs: vec![4.0, 2.0], ..Model::default()
        };
        let d = initialize_domain(&m, &Params::default());
        assert_eq!(d.vtype, b"IIB");
        assert_eq!(d.lb, vec![1.0, 1.0, 0.0]);
        assert_eq!(d.ub, vec![3.0, 3.0, 1.0]);
        assert_eq!(m.vtype, b"CCB");
        let mut nonintegral = m.clone();
        nonintegral.rhs[1] = 2.5;
        assert_eq!(initialize_domain(&nonintegral, &Params::default()).vtype, b"CCB");
    }

    #[test]
    fn incumbent_bounds_respect_sense_and_validation() {
        let p = Params::default();
        let m = Model { sense: 1, objcon: 3.0, obj: vec![1.0, -2.0],
            lb: vec![0.0, 0.0], ub: vec![30.0, 10.0], vtype: vec![b'C', b'C'],
            col_beg: vec![0, 0], col_len: vec![0, 0], ..Model::default() };
        let mut d = m.clone();
        apply_incumbent(&mut d, &p, &[4.0, 2.0]).unwrap();
        assert!(d.ub[0] < 30.0 && d.ub[0] > 0.0);
        assert!(d.lb[1] > 0.0 && d.lb[1] < 2.0);
        assert!(apply_incumbent(&mut m.clone(), &p, &[4.0]).is_err());
        assert!(apply_incumbent(&mut m.clone(), &p, &[31.0, 2.0]).is_err());
        let mut row_model = Model { sense: 1, obj: vec![1.0], lb: vec![0.0], ub: vec![10.0],
            vtype: vec![b'C'], col_beg: vec![0], col_len: vec![1], col_ind: vec![0], col_val: vec![1.0],
            row_sense: vec![b'<'], rhs: vec![1.0], ..Model::default() };
        assert!(apply_incumbent(&mut row_model, &p, &[2.0]).is_err());
        let mut max = Model { sense: -1, obj: vec![1.0], lb: vec![0.0], ub: vec![10.0],
            vtype: vec![b'C'], col_beg: vec![0], col_len: vec![0], ..Model::default() };
        apply_incumbent(&mut max, &p, &[4.0]).unwrap();
        assert!(max.lb[0] > 4.0 && max.lb[0] < 4.00001);
        let mut optimal = Model { sense: 1, obj: vec![1.0], lb: vec![0.0], ub: vec![1.0],
            vtype: vec![b'B'], col_beg: vec![0], col_len: vec![0], ..Model::default() };
        apply_incumbent(&mut optimal, &p, &[0.0]).unwrap();
        assert_eq!(optimal.lb[0], 0.0);
        // The next lattice point is -1: contradictory objective bounds are still skipped.
        assert_eq!(optimal.ub[0], 1.0);
    }

    fn raw_binary(lb: f64, ub: f64, c: f64) -> Model {
        Model { sense: 1, obj: vec![c], lb: vec![lb], ub: vec![ub], vtype: vec![b'B'],
            col_beg: vec![0], col_len: vec![0], ..Model::default() }
    }

    #[test]
    fn binary_incumbent_rejects_values_outside_its_type_domain() {
        let m = raw_binary(-1.0, 2.0, 1.0);
        for p in [Params::default(), Params { heuristics: 0.0, ..Params::default() }] {
            for v in [-1.0, 2.0] {
                assert!(incumbent_objective(&m, &p, &[v]).is_err());
                assert!(root_domain(&m, &p, Some(&[v])).is_err());
            }
            let mut integer = m.clone();
            integer.vtype[0] = b'I';
            assert_eq!(incumbent_objective(&integer, &p, &[2.0]), Ok(2.0));
        }
    }

    #[test]
    fn binary_type_bounds_constrain_the_root_lp_with_or_without_heuristics() {
        for p in [Params::default(), Params { heuristics: 0.0, ..Params::default() }] {
            for (lo, hi, c, objective) in [(0.0, 2.0, -1.0, -1.0), (-1.0, 1.0, 1.0, 0.0)] {
                let m = raw_binary(lo, hi, c);
                let r = solve_root(&m, &p).unwrap();
                assert_eq!(r.lp.status, rsolver_lp::simplex::Status::Optimal);
                assert_eq!(r.lp.obj, objective);
                let d = root_domain(&m, &p, None).unwrap();
                assert!(d.lb[0] >= 0.0 && d.ub[0] <= 1.0);
                assert_eq!(m.lb, vec![lo]);
                assert_eq!(m.ub, vec![hi]);
            }
        }
    }

    #[test]
    fn binary_fixed_and_empty_domains_have_consistent_root_contracts() {
        for p in [Params::default(), Params { heuristics: 0.0, ..Params::default() }] {
            for fixed in [0.0, 1.0] {
                let m = raw_binary(fixed, fixed, 1.0);
                let d = root_domain(&m, &p, Some(&[fixed])).unwrap();
                assert_eq!(d.lb, vec![fixed]);
                assert_eq!(d.ub, vec![fixed]);
                assert_eq!(solve_root(&m, &p).unwrap().lp.obj, fixed);
            }
            for (lo, hi) in [(2.0, 3.0), (-3.0, -2.0), (2.0, 2.0)] {
                let m = raw_binary(lo, hi, 1.0);
                assert!(root_domain(&m, &p, None).is_err());
                assert!(root_domain(&m, &p, Some(&[lo])).is_err());
                assert!(solve_root_with_incumbent(&m, &p, Some(&[lo])).is_err());
                assert!(initial_incumbent(&m, &p).is_none());
                let r = solve_root(&m, &p).unwrap();
                assert_eq!(r.lp.status, Status::Infeasible);
                assert_eq!(r.lp.iter, 0);
                assert!(r.lp.obj.is_nan() && r.lp.x.is_empty() && r.incumbent.is_none());
                assert_eq!(m.lb, vec![lo]);
                assert_eq!(m.ub, vec![hi]);
            }
        }
    }

    #[test]
    fn binary_validation_uses_bound_and_integrality_tolerances_separately() {
        let m = raw_binary(-1.0, 2.0, 1.0);
        let p = Params::default();
        assert!(incumbent_objective(&m, &p, &[1.0 + p.feastol * 0.5]).is_ok());
        assert!(incumbent_objective(&m, &p, &[-p.feastol * 0.5]).is_ok());
        assert!(incumbent_objective(&m, &p, &[1.0 + p.feastol * 2.0]).is_err());
        let strict_int = Params { intfeastol: p.feastol * 0.1, ..p.clone() };
        assert!(incumbent_objective(&m, &strict_int, &[1.0 + p.feastol * 0.5]).is_err());
        let tightened = raw_binary(0.2, 0.8, 0.0);
        assert!(incumbent_objective(&tightened, &p, &[0.0]).is_err());
        assert!(incumbent_objective(&tightened, &p, &[1.0]).is_err());
    }

    #[test]
    fn binary_empty_intersections_within_bound_tolerance_fix_to_the_endpoint() {
        let p = Params { heuristics: 0.0, ..Params::default() };
        let tol = p.feastol;
        for (lo, hi, endpoint, invalid) in [
            (1.0 + tol * 0.5, 2.0, 1.0, 1.0 - tol * 0.75),
            (-1.0, -tol * 0.5, 0.0, tol * 0.75),
        ] {
            let m = raw_binary(lo, hi, 1.0);
            assert!(incumbent_objective(&m, &p, &[endpoint]).is_ok());
            for incumbent in [None, Some([endpoint].as_slice())] {
                let d = root_domain(&m, &p, incumbent).unwrap();
                assert_eq!(d.lb, vec![endpoint]);
                assert_eq!(d.ub, vec![endpoint]);
            }
            assert!(root_domain(&m, &p, Some(&[invalid])).is_err());
            let r = solve_root(&m, &p).unwrap();
            assert_eq!(r.lp.status, Status::Optimal);
            assert_eq!(r.lp.obj, endpoint);
            assert_eq!(m.lb, vec![lo]);
            assert_eq!(m.ub, vec![hi]);
        }
        for (lo, hi, endpoint) in [(1.0 + tol * 2.0, 2.0, 1.0), (-1.0, -tol * 2.0, 0.0)] {
            let m = raw_binary(lo, hi, 1.0);
            assert!(incumbent_objective(&m, &p, &[endpoint]).is_err());
            assert!(root_domain(&m, &p, None).is_err());
            assert_eq!(solve_root(&m, &p).unwrap().lp.status, Status::Infeasible);
        }
    }

    #[test]
    fn automatic_incumbents_must_satisfy_the_original_presolved_bounds() {
        let p = Params::default();
        let m = Model {
            sense: 1, obj: vec![1.0, 0.0], lb: vec![0.0, 1.0], ub: vec![0.999995, 1.0],
            vtype: b"CB".to_vec(), col_beg: vec![0, 1], col_len: vec![1, 1],
            col_ind: vec![0, 0], col_val: vec![1.0, -1.0], row_sense: vec![b'='], rhs: vec![0.0],
            ..Model::default()
        };
        assert!(initial_incumbent(&m, &p).is_none());
        assert!(solve_root(&m, &p).unwrap().incumbent.is_none());
        assert!(root_domain(&m, &p, Some(&[1.0, 1.0])).is_err());
        assert_eq!(m.ub, vec![0.999995, 1.0]);
    }

    #[test]
    fn root_method_thresholds() {
        let mut p = Params::default();
        assert_eq!(root_method(&p, 1000, 10_000, 100_000), (4, 3));
        assert_eq!(root_method(&p, 1000, 10_000, 99_999), (1, -1));
        assert_eq!(root_method(&p, 999, 30_000, 600_000), (1, -1));
        assert_eq!(root_method(&p, 1000, 20_000, 500_000), (4, -1));
        assert_eq!(root_method(&p, 1000, 15_000, 500_000), (4, 3));
        p.method = 3;
        p.threads = 1;
        assert_eq!(root_method(&p, 10, 10, 10).0, 1);
        p.threads = 0;
        assert_eq!(root_method(&p, 10, 10, 10).0, 3);
        p.method = 0;
        assert_eq!(root_method(&p, 10, 10, 10).0, 0);
    }
}
