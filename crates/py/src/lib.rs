//! PyO3 layer: one call in, one tuple out. Solve runs without the GIL (cf. re/notes/02).
use pyo3::prelude::*;

/// build_model(obj, rows, lb, ub, vtype) -> CoreModel for a model built through the API (minimization form).
/// rows: list of ([(col, coef), ...], lo, hi). A ranged row becomes an equality plus a slack column in
/// [0, hi − lo] appended after the variables, as the MPS reader does for RANGES.
/// Rows of a built model: (sparse coefficients, lower, upper).
type Rows = Vec<(Vec<(usize, f64)>, f64, f64)>;

#[pyfunction]
fn build_model(
    obj: Vec<f64>,
    rows: Rows,
    lb: Vec<f64>,
    ub: Vec<f64>,
    vtype: String,
) -> PyResult<PyCoreModel> {
    use rsolver_model::INFINITY;
    let n = obj.len();
    if lb.len() != n || ub.len() != n || vtype.len() != n {
        return Err(pyo3::exceptions::PyValueError::new_err("column vectors differ in length"));
    }
    if let Some((j, _)) = rows.iter().flat_map(|r| &r.0).find(|&&(j, _)| j >= n) {
        return Err(pyo3::exceptions::PyIndexError::new_err(format!("column {j} out of range")));
    }
    let clip = |v: f64| v.clamp(-INFINITY, INFINITY);
    let mut md = rsolver_model::Model { sense: 1, obj, vtype: vtype.into_bytes(), ..Default::default() };
    md.lb = lb.into_iter().map(clip).collect();
    md.ub = ub.into_iter().map(clip).collect();
    let mut cols: Vec<Vec<(i32, f64)>> = vec![Vec::new(); n];
    for (i, (coefs, lo, hi)) in rows.iter().enumerate() {
        for &(j, a) in coefs {
            cols[j].push((i as i32, a));
        }
        let (lo, hi) = (clip(*lo), clip(*hi));
        let (sense, rhs) = if lo == hi {
            (b'=', lo)
        } else if lo <= -INFINITY {
            (b'<', hi)
        } else if hi >= INFINITY {
            (b'>', lo)
        } else {
            md.obj.push(0.0);
            md.lb.push(0.0);
            md.ub.push(hi - lo);
            md.vtype.push(b'C');
            cols.push(vec![(i as i32, 1.0)]);
            (b'=', hi)
        };
        md.row_sense.push(sense);
        md.rhs.push(rhs);
        md.constr_names.push(format!("R{i}"));
    }
    md.var_names = (0..cols.len()).map(|j| format!("C{j}")).collect();
    md.col_beg.push(0);
    for c in &cols {
        md.col_len.push(c.len() as i32);
        for &(i, a) in c {
            md.col_ind.push(i);
            md.col_val.push(a);
        }
        md.col_beg.push(md.col_ind.len() as i64);
    }
    Ok(PyCoreModel { inner: md })
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
    /// (`9e60b` row-cost estimate by factor step, basis position of each step, estimate usable) for a status array.
    fn row_costs(&self, vstat: Vec<i32>) -> (Vec<f64>, Vec<usize>, bool) {
        let spx = rsolver_lp::spx::Spx::new(&self.inner);
        let lu = rsolver_lp::lu::Lu::factor(&spx, &rsolver_lp::lu::basis_head(spx.m, &vstat));
        let (c, ok) = lu.row_cost_estimate();
        (c, lu.steps.iter().map(|s| s.1).collect(), ok)
    }
    /// Status array of the dual start basis (crash + first factorization); M1 parity tests.
    fn start_basis(&self) -> Vec<i32> {
        rsolver_lp::start::start_basis(&rsolver_lp::spx::Spx::new(&self.inner))
    }
    /// Barrier ordering and symbolic analysis (05 §2.3): (perm = final position of each row, AA' NZ both triangles,
    /// Factor NZ, Factor Ops, free columns left after `30f5b9`) — the "Barrier statistics" log block.
    fn barrier_symbolic(&self) -> (Vec<i32>, i64, i64, f64, usize) {
        let s = rsolver_lp::barrier::ordering::analyze(&self.inner);
        let lp = rsolver_lp::barrier::setup::lp_from_model(&self.inner, &s.perm, 1e-8);
        let free = lp.t[..lp.n].iter().filter(|&&t| t == 0).count();
        (s.perm, s.aat_nnz, s.nnz_l, s.ops, free)
    }
    /// Barrier (05 §2; Method=2, Presolve=0, Crossover=0): `callback(iter, pobj, dobj, priminf, dualinf, compl)` runs
    /// every iteration. Returns (status, objective, BarIterCount, x, work).
    #[pyo3(signature = (callback, bar_conv_tol=1e-8, bar_iter_limit=1000, bar_correctors=-1))]
    fn barrier(
        &self,
        py: Python<'_>,
        callback: PyObject,
        bar_conv_tol: f64,
        bar_iter_limit: i64,
        bar_correctors: i32,
    ) -> PyResult<(i32, f64, usize, Vec<f64>, f64)> {
        let p = rsolver_lp::barrier::Params { bar_conv_tol, bar_iter_limit, bar_correctors };
        let mut err = None;
        let r = rsolver_lp::barrier::solve(&self.inner, &p, &mut |i| {
            if err.is_none() {
                err = callback.call1(py, (i.iter, i.pobj, i.dobj, i.priminf, i.dualinf, i.compl)).err();
            }
        });
        match err {
            Some(e) => Err(e),
            None => Ok((r.status, r.obj, r.bar_iters, r.x, r.work)),
        }
    }
    /// LP solve without presolve: (status, objective, x, iterations, trace rows, pivot path).
    #[allow(clippy::type_complexity)]
    fn solve(
        &self,
        py: Python<'_>,
        method: i32,
        iter_limit: f64,
    ) -> (i32, f64, Vec<f64>, f64, Vec<(f64, f64, f64, f64, bool)>, Vec<(u64, i64, i64)>) {
        use rsolver_lp::simplex::{solve_model, Status};
        let lim = if iter_limit >= 1e18 { u64::MAX } else { iter_limit as u64 };
        let r = py.allow_threads(|| solve_model(&self.inner, method, lim));
        let code = match r.status {
            Status::Optimal => 2,
            Status::Infeasible => 3,
            Status::Unbounded => 5,
            Status::IterLimit => 7,
            Status::Numeric => 12,
        };
        let trace = r.trace.iter().map(|&(k, z, p, d, pt)| (k as f64, z, p, d, pt)).collect();
        (code, r.obj, r.x, r.iter as f64, trace, r.path)
    }
    fn scaling(&self) -> (Vec<f64>, Vec<f64>, f64) {
        let s = rsolver_lp::scaling::scale_flag3(&self.inner);
        (s.col, s.row, s.obj)
    }
}

/// `aabbe3` descending sort (index order of the result), for parity checks.
#[pyfunction]
fn sort_desc(mut key: Vec<f64>, mut idx: Vec<i32>) -> (Vec<f64>, Vec<i32>) {
    rsolver_lp::common::sort_desc(&mut key, &mut idx);
    (key, idx)
}

#[pyfunction]
fn read_model(py: Python<'_>, path: String) -> PyResult<PyCoreModel> {
    let m = py.allow_threads(|| rsolver_model::mps::read_file(&path));
    m.map(|inner| PyCoreModel { inner }).map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(build_model, m)?)?;
    m.add_function(wrap_pyfunction!(read_model, m)?)?;
    m.add_function(wrap_pyfunction!(sort_desc, m)?)?;
    m.add_class::<PyCoreModel>()?;
    m.add("INF", f64::INFINITY)
}
