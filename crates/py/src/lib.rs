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

/// A model read from a file (01 §1.1). Arrays are exposed read-only for the Python layer and parity tools.
#[pyclass(name = "CoreModel")]
struct PyCoreModel {
    inner: rsolver_model::Model,
}

#[pymethods]
impl PyCoreModel {
    #[getter]
    fn name(&self) -> String { self.inner.name.clone() }
    #[getter]
    fn sense(&self) -> i32 { self.inner.sense }
    #[getter]
    fn objcon(&self) -> f64 { self.inner.objcon }
    #[getter]
    fn obj(&self) -> Vec<f64> { self.inner.obj.clone() }
    #[getter]
    fn lb(&self) -> Vec<f64> { self.inner.lb.clone() }
    #[getter]
    fn ub(&self) -> Vec<f64> { self.inner.ub.clone() }
    #[getter]
    fn vtype(&self) -> String { String::from_utf8_lossy(&self.inner.vtype).into_owned() }
    #[getter]
    fn row_sense(&self) -> String { String::from_utf8_lossy(&self.inner.row_sense).into_owned() }
    #[getter]
    fn rhs(&self) -> Vec<f64> { self.inner.rhs.clone() }
    #[getter]
    fn var_names(&self) -> Vec<String> { self.inner.var_names.clone() }
    #[getter]
    fn constr_names(&self) -> Vec<String> { self.inner.constr_names.clone() }
    /// Columns as lists of (row, value) in stored order.
    #[getter]
    fn cols(&self) -> Vec<Vec<(i32, f64)>> {
        let m = &self.inner;
        (0..m.num_vars())
            .map(|j| {
                let b = m.col_beg[j] as usize;
                (b..b + m.col_len[j] as usize).map(|k| (m.col_ind[k], m.col_val[k])).collect()
            })
            .collect()
    }
    #[getter]
    fn num_nz(&self) -> usize { self.inner.num_nz() }
    fn fingerprint(&self) -> u32 { rsolver_model::fingerprint::fingerprint(&self.inner) }
    /// (column scales, row scales, objective scale) of the LP simplex setup (04 §3).
    fn scaling(&self) -> (Vec<f64>, Vec<f64>, f64) {
        let s = rsolver_lp::scaling::scale_flag3(&self.inner);
        (s.col, s.row, s.obj)
    }
}

#[pyfunction]
fn read_model(py: Python<'_>, path: String) -> PyResult<PyCoreModel> {
    let m = py.allow_threads(|| rsolver_model::mps::read_file(&path));
    m.map(|inner| PyCoreModel { inner }).map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(solve_problem, m)?)?;
    m.add_function(wrap_pyfunction!(read_model, m)?)?;
    m.add_class::<PyCoreModel>()?;
    m.add("INF", f64::INFINITY)
}
