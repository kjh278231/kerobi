//! PyO3 layer: one call in, one tuple out. Solve runs without the GIL (cf. re/notes/02).
use pyo3::prelude::*;
use rsolver_core::{solve, Problem, Status};

/// solve(obj, rows, lb, ub, integer) -> (status: int, obj_val, x)
/// rows: list of ([(col, coef), ...], lo, hi). Status: 2 optimal, 3 infeasible, 5 unbounded
/// (same numbers gurobipy uses).
#[pyfunction]
fn solve_problem(
    py: Python<'_>,
    obj: Vec<f64>,
    rows: Vec<(Vec<(usize, f64)>, f64, f64)>,
    lb: Vec<f64>,
    ub: Vec<f64>,
    integer: Vec<bool>,
) -> PyResult<(i32, f64, Vec<f64>)> {
    let n = obj.len();
    if lb.len() != n || ub.len() != n || integer.len() != n {
        return Err(pyo3::exceptions::PyValueError::new_err("column vectors differ in length"));
    }
    if let Some((j, _)) = rows.iter().flat_map(|r| &r.0).find(|&&(j, _)| j >= n) {
        return Err(pyo3::exceptions::PyIndexError::new_err(format!("column {j} out of range")));
    }
    let p = Problem { obj, rows, lb, ub, integer };
    let s = py.allow_threads(move || solve(&p));
    let code = match s.status {
        Status::Optimal => 2,
        Status::Infeasible => 3,
        Status::Unbounded => 5,
    };
    Ok((code, s.obj_val, s.x))
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(solve_problem, m)?)?;
    m.add("INF", f64::INFINITY)
}
