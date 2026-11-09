//! Whole-MIP checks (`5eda00`, `5edf6b`) in minimization coordinates, independent of LP status.
use crate::Params;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    Running = 1, Optimal = 2, Infeasible = 3, Unbounded = 5, Cutoff = 6,
    IterationLimit = 7, NodeLimit = 8, TimeLimit = 9, SolutionLimit = 10,
    Interrupted = 11, Numeric = 12, UserObjLimit = 15, WorkLimit = 16, MemLimit = 17,
}

/// Tree/pool statistics supplied by the caller; work is measured in public work units.
pub struct State {
    pub status: Status,
    pub incumbent: f64,
    pub bound: f64,
    pub cutoff: f64,
    pub exhausted: bool,
    pub nodes: f64,
    pub root_processed: bool,
    pub iterations: f64,
    pub solutions: i32,
    pub seconds: f64,
    pub work: f64,
    pub memory_gb: f64,
    pub interrupted: bool,
}
impl Default for State {
    fn default() -> Self {
        Self { status: Status::Running, incumbent: 1e100, bound: -1e100, cutoff: 1e100,
            exhausted: false, nodes: 0.0, root_processed: false, iterations: 0.0,
            solutions: 0, seconds: 0.0, work: 0.0, memory_gb: 0.0, interrupted: false }
    }
}

/// `5eda00`: round a valid dual bound up to its objective lattice, without lowering it.
pub fn lattice_bound(bound: f64, step: f64, offset: f64, p: &Params) -> f64 {
    if step <= 0.0 || bound.abs() >= 1e100 { return bound; }
    let tolerance = ((bound - offset).abs() + 1.0) * 1e-6;
    let tolerance = tolerance.max(p.intfeastol * step).max(p.opttol);
    bound.max(offset + step * (((bound - offset) - tolerance) / step).ceil())
}

pub fn check(s: &State, p: &Params) -> Status {
    if s.status != Status::Running { return s.status; }
    if s.memory_gb > p.soft_mem_limit { return Status::MemLimit; }
    if s.interrupted { return Status::Interrupted; }
    if s.seconds >= p.time_limit { return Status::TimeLimit; }
    if s.work > p.work_limit { return Status::WorkLimit; }
    if s.exhausted { return if s.incumbent < 1e100 { Status::Optimal } else { Status::Infeasible }; }
    if s.bound >= 1e100 { return Status::Infeasible; }
    if s.cutoff < s.bound || (s.cutoff == s.bound && s.incumbent < 1e100) { return Status::Optimal; }
    if s.incumbent < 1e100 {
        let gap = s.incumbent - s.bound;
        if (p.mip_gap > 0.0 && gap / s.incumbent.abs().max(1e-6) < p.mip_gap)
            || (p.mip_gap_abs > 0.0 && gap < p.mip_gap_abs) { return Status::Optimal; }
    }
    if s.incumbent < p.best_obj_stop + 1e-10 || p.best_bd_stop - 1e-10 < s.bound {
        return Status::UserObjLimit;
    }
    if (p.node_limit == 0.0 && s.root_processed) || s.nodes >= p.node_limit.max(1.0) { return Status::NodeLimit; }
    if s.iterations >= p.iteration_limit { return Status::IterationLimit; }
    if s.solutions >= p.solution_limit { return Status::SolutionLimit; }
    Status::Running
}

/// ImproveStart is a phase change, not a terminal solver status.
pub fn improve_start(s: &State, p: &Params) -> bool {
    check(s, p) == Status::Running && s.incumbent.abs() < 5e99
        && ((p.improve_start_gap > 0.0 && (s.incumbent - s.bound).abs() / s.incumbent.abs().max(1e-6) < p.improve_start_gap)
            || p.improve_start_nodes < s.nodes || p.improve_start_time < s.seconds || p.improve_start_work < s.work)
}

/// `5bbf7f`: translate an exhausted search without an accepted solution to CUTOFF.
pub fn finish(status: Status, has_solution: bool, p: &Params) -> Status {
    if matches!(status, Status::Optimal | Status::Infeasible) && !has_solution && p.cutoff < 1e100 {
        Status::Cutoff
    } else { status }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn priorities_and_limits() {
        let p = Params { node_limit: 0.0, iteration_limit: 0.0, solution_limit: 0, ..Params::default() };
        let mut s = State { root_processed: true, ..State::default() };
        assert_eq!(check(&s, &p), Status::NodeLimit);
        s.root_processed = false;
        assert_eq!(check(&s, &p), Status::IterationLimit);
        s.iterations = -1.0;
        assert_eq!(check(&s, &p), Status::SolutionLimit);
        s.exhausted = true;
        assert_eq!(check(&s, &p), Status::Infeasible);
        s.incumbent = 0.0;
        assert_eq!(check(&s, &p), Status::Optimal);
        s.interrupted = true;
        assert_eq!(check(&s, &p), Status::Interrupted);
        s.status = Status::Numeric;
        assert_eq!(check(&s, &p), Status::Numeric);
    }
    #[test]
    fn gaps_are_strict_and_zero_disables_them() {
        let s = State { incumbent: 8.0, bound: 7.0, ..State::default() };
        let mut p = Params { mip_gap: 0.125, mip_gap_abs: 1.0, ..Params::default() };
        assert_eq!(check(&s, &p), Status::Running);
        p.mip_gap = 0.126;
        assert_eq!(check(&s, &p), Status::Optimal);
        p.mip_gap = 0.0; p.mip_gap_abs = 1.01;
        assert_eq!(check(&s, &p), Status::Optimal);
        p.mip_gap_abs = 0.0;
        assert_eq!(check(&s, &p), Status::Running);
        let zero = State { incumbent: 0.0, bound: -1e-8, ..State::default() };
        p.mip_gap = 0.02;
        assert_eq!(check(&zero, &p), Status::Optimal);
    }
    #[test]
    fn resource_and_objective_stops() {
        let s = State { seconds: 2.0, work: 3.0, memory_gb: 4.0, incumbent: 5.0, bound: 1.0, ..State::default() };
        for (p, expected) in [
            (Params { time_limit: 2.0, ..Params::default() }, Status::TimeLimit),
            (Params { work_limit: 2.0, ..Params::default() }, Status::WorkLimit),
            (Params { soft_mem_limit: 3.0, ..Params::default() }, Status::MemLimit),
            (Params { best_obj_stop: 5.0, ..Params::default() }, Status::UserObjLimit),
            (Params { best_bd_stop: 1.0, ..Params::default() }, Status::UserObjLimit),
        ] { assert_eq!(check(&s, &p), expected); }
        let p = Params { improve_start_nodes: 0.0, ..Params::default() };
        assert!(!improve_start(&s, &p));
        assert!(improve_start(&State { nodes: 1.0, ..s }, &p));
    }
    #[test]
    fn lattice_and_final_cutoff() {
        let p = Params::default();
        assert_eq!(lattice_bound(7.2, 2.0, 1.0, &p), 9.0);
        assert_eq!(lattice_bound(7.0000001, 2.0, 1.0, &p), 7.0000001);
        assert_eq!(lattice_bound(-4.2, 2.0, 1.0, &p), -3.0);
        let p = Params { cutoff: 3.0, ..p };
        assert_eq!(finish(Status::Infeasible, false, &p), Status::Cutoff);
        assert_eq!(finish(Status::Optimal, true, &p), Status::Optimal);
        assert_eq!(finish(Status::TimeLimit, false, &p), Status::TimeLimit);
    }
}
