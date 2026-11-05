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
    /// x_B of the LP simplex workspace for a given status array (04 §4); used by M1 parity tests.
    fn basic_solution(&self, vstat: Vec<i32>) -> Vec<f64> {
        let spx = rsolver_lp::spx::Spx::new(&self.inner);
        rsolver_lp::lu::basic_solution(&spx, &vstat)
    }
    /// (reduced costs d, DSE weights w) of the LP simplex workspace for a given status array; M1 parity tests.
    fn dual_start(&self, vstat: Vec<i32>) -> (Vec<f64>, Vec<f64>) {
        let spx = rsolver_lp::spx::Spx::new(&self.inner);
        rsolver_lp::lu::dual_start(&spx, &vstat)
    }
    /// The model as `Model.write("*.mps")` text (01 §1.5); P0 compares it byte for byte.
    fn to_mps(&self) -> String { rsolver_model::mps_write::to_string(&self.inner) }
    /// LP presolve (03 §1) with Gurobi parameter (name, value) pairs; returns the presolved model.
    fn presolve(&self, py: Python<'_>, params: Vec<(String, f64)>) -> PyCoreModel {
        let p = rsolver_lp::presolve::Params::from_pairs(&params);
        PyCoreModel { inner: py.allow_threads(|| rsolver_lp::presolve::run(&self.inner, &p)) }
    }
    /// Presolve, then uncrush the presolved solution (x, dj, status as `363daa` reads them) back to this model:
    /// (basis, x, dual). `aligns` (two parities per record, oldest first) overrides the arena simulation.
    #[pyo3(signature = (params, x, dj, st, aligns=None))]
    fn uncrush(&self, py: Python<'_>, params: Vec<(String, f64)>, x: Vec<f64>, dj: Vec<f64>, st: Vec<i32>,
               aligns: Option<Vec<u8>>) -> (Vec<i32>, Vec<f64>, Vec<f64>) {
        let p = rsolver_lp::presolve::Params::from_pairs(&params);
        py.allow_threads(|| {
            let (_, post) = rsolver_lp::presolve::run_post(&self.inner, &p);
            let al: Option<Vec<[u8; 2]>> = aligns.map(|a| a.chunks(2).map(|c| [c[0], c[1]]).collect());
            let r = rsolver_lp::presolve::postsolve::uncrush(
                &post, &self.inner, &rsolver_lp::presolve::postsolve::PreSol { x: &x, dj: &dj, st: &st }, al.as_deref());
            (r.basis, r.x, r.dual)
        })
    }
    /// Record kinds (oldest first) and simulated array parities of the presolve of this model.
    fn post_records(&self, params: Vec<(String, f64)>) -> Vec<(i32, u8, u8, i64)> {
        let p = rsolver_lp::presolve::Params::from_pairs(&params);
        let (_, post) = rsolver_lp::presolve::run_post(&self.inner, &p);
        post.recs.iter().zip(&post.aligns).map(|(r, a)| (r.kind(), a[0], a[1], rsolver_lp::presolve::postsolve::layout(r).0)).collect()
    }
    /// Debug text of the presolve records, oldest first (for oracle comparisons).
    fn post_debug(&self, params: Vec<(String, f64)>) -> Vec<String> {
        let p = rsolver_lp::presolve::Params::from_pairs(&params);
        let (_, post) = rsolver_lp::presolve::run_post(&self.inner, &p);
        post.recs.iter().map(|r| format!("{r:?}")).collect()
    }
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
