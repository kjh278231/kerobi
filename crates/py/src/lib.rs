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
    /// The optimize header log ("Optimize a model with …" through the coefficient statistics and warnings).
    fn header(&self, numeric_focus: i32) -> String { rsolver_driver::header::header(&self.inner, numeric_focus) }
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
    /// Presolve, then simplex on the presolved model: (x, dj, status) as `363daa` reads them (debug of the driver).
    fn presolved_spx(&self, py: Python<'_>, params: Vec<(String, f64)>) -> (Vec<f64>, Vec<f64>, Vec<i32>) {
        let p = rsolver_lp::presolve::Params::from_pairs(&params);
        py.allow_threads(|| {
            let pre = rsolver_lp::presolve::run(&self.inner, &p);
            let (_, _, _, x, dj, st) = rsolver_driver::simplex(&pre, 1, u64::MAX);
            (x, dj, st)
        })
    }
    /// Presolve, then `365159` (uncrush without a basis) of a presolved solution given as x and slack, d and y:
    /// (x, dual) of this model, columns then rows.
    fn uncrush_nonbasic(&self, py: Python<'_>, params: Vec<(String, f64)>, x: Vec<f64>, dj: Vec<f64>) -> (Vec<f64>, Vec<f64>) {
        let p = rsolver_lp::presolve::Params::from_pairs(&params);
        py.allow_threads(|| {
            let (_, post) = rsolver_lp::presolve::run_post(&self.inner, &p);
            let st = vec![0; x.len()];
            let r = rsolver_lp::presolve::postsolve::uncrush_nonbasic(
                &post, &self.inner, &rsolver_lp::presolve::postsolve::PreSol { x: &x, dj: &dj, st: &st }, None);
            (r.x, r.dual)
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
    /// (record count, [bounds tightened, coefficients changed, inequalities made equalities, rows removed, columns removed]).
    fn post_counts(&self, params: Vec<(String, f64)>) -> (usize, Vec<i32>) {
        let p = rsolver_lp::presolve::Params::from_pairs(&params);
        let (_, post) = rsolver_lp::presolve::run_post(&self.inner, &p);
        (post.recs.len(), post.counts.to_vec())
    }
    /// Presolve work when model generation starts (compare with tools/work_oracle.py).
    fn presolve_work(&self, params: Vec<(String, f64)>) -> f64 {
        let p = rsolver_lp::presolve::Params::from_pairs(&params);
        rsolver_lp::presolve::run_post(&self.inner, &p).1.work
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
    /// PDHG (05 §4; Method=6, Presolve=0, Crossover=0): `callback(iter, pobj, dobj, priminf, dualinf, compl)` runs at
    /// every PDHG callback. Returns (status, objective, PDHGIterCount, x, work units).
    #[pyo3(signature = (callback, iter_limit=1e100, rel_tol=1e-6, abs_tol=1e-6, conv_tol=1e-6, work_limit=1e100))]
    #[allow(clippy::too_many_arguments)]
    fn pdhg(
        &self,
        py: Python<'_>,
        callback: PyObject,
        iter_limit: f64,
        rel_tol: f64,
        abs_tol: f64,
        conv_tol: f64,
        work_limit: f64,
    ) -> PyResult<(i32, f64, i64, Vec<f64>, f64)> {
        let p = rsolver_lp::pdhg::Params { iter_limit, rel_tol, abs_tol, conv_tol, work_limit, work0: 0.0 };
        let mut err = None;
        let r = rsolver_lp::pdhg::solve(&self.inner, &p, &mut |i| {
            if err.is_none() {
                err = callback.call1(py, (i.iter as f64, i.pobj, i.dobj, i.priminf, i.dualinf, i.compl)).err();
            }
        });
        match err {
            Some(e) => Err(e),
            None => Ok((r.status, r.obj, r.iters, r.x, r.work / 1e9)),
        }
    }
    /// optimize() of a continuous model through the driver (docs/m1_driver.md) with Gurobi parameter (name, value)
    /// pairs; `bar_cb`/`pdhg_cb(iter, pobj, dobj, priminf, dualinf, compl)` run per BARRIER/PDHG callback. Returns
    /// (status, objective, x, IterCount, BarIterCount, PDHGIterCount, work, SIMPLEX trace rows, log text, Slack, Pi,
    /// RC).
    #[allow(clippy::type_complexity)]
    fn optimize(
        &self,
        py: Python<'_>,
        params: Vec<(String, f64)>,
        bar_cb: PyObject,
        pdhg_cb: PyObject,
    ) -> PyResult<(i32, f64, Vec<f64>, f64, usize, f64, f64, Vec<(f64, f64, f64, f64, bool)>, String, Vec<f64>, Vec<f64>, Vec<f64>)> {
        let p = rsolver_model::params::Params::from_pairs(&params);
        let (mut log, mut err_b, mut err_p) = (String::new(), Ok(()), Ok(()));
        let out = {
            let mut bar = |i: &rsolver_lp::barrier::IterInfo| {
                if err_b.is_ok() {
                    err_b = bar_cb.call1(py, (i.iter as f64, i.pobj, i.dobj, i.priminf, i.dualinf, i.compl)).map(|_| ());
                }
            };
            let mut pd = |i: &rsolver_lp::pdhg::IterInfo| {
                if err_p.is_ok() {
                    err_p = pdhg_cb.call1(py, (i.iter as f64, i.pobj, i.dobj, i.priminf, i.dualinf, i.compl)).map(|_| ());
                }
            };
            let mut lg = |t: &str| log.push_str(t);
            let mut h = rsolver_driver::Hooks { log: &mut lg, barrier: &mut bar, pdhg: &mut pd };
            rsolver_driver::optimize(&self.inner, &p, &mut h)
        };
        err_b?;
        err_p?;
        let trace = out.trace.iter().map(|&(k, z, pi, di, pt)| (k as f64, z, pi, di, pt)).collect();
        Ok((out.status, out.obj, out.x, out.iter_count as f64, out.bar_iter_count, out.pdhg_iter_count as f64, out.work, trace, log,
            out.slack, out.pi, out.rc))
    }
    /// Barrier duals for checks: (slack, d, y) in the model's space as `7a069` hands them to `365159`.
    fn barrier_duals(&self, py: Python<'_>) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let r = py.allow_threads(|| rsolver_lp::barrier::solve(&self.inner, &rsolver_lp::barrier::Params::default(), &mut |_| {}));
        (r.slack, r.d, r.y)
    }
    /// PDHG duals for checks: (slack, d, y) in the model's space as `7a069` hands them to `365159`.
    fn pdhg_duals(&self, py: Python<'_>) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let r = py.allow_threads(|| rsolver_lp::pdhg::solve(&self.inner, &rsolver_lp::pdhg::Params::default(), &mut |_| {}));
        (r.slack, r.d, r.y)
    }
    /// LP solve without presolve: (status, objective, x, iterations, trace rows, pivot path). `trace` false skips
    /// the per-iteration trace (no SIMPLEX callback to replay).
    #[allow(clippy::type_complexity)]
    #[pyo3(signature = (method, iter_limit, trace=true))]
    fn solve(
        &self,
        py: Python<'_>,
        method: i32,
        iter_limit: f64,
        trace: bool,
    ) -> (i32, f64, Vec<f64>, f64, Vec<(f64, f64, f64, f64, bool)>, Vec<(u64, i64, i64)>) {
        use rsolver_lp::simplex::{solve_model_opts, Status};
        let lim = if iter_limit >= 1e18 { u64::MAX } else { iter_limit as u64 };
        let r = py.allow_threads(|| solve_model_opts(&self.inner, method, lim, trace));
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

/// (name, type 1 int / 2 double / 3 string, min, max, default, string default) of a parameter, ignoring case.
#[pyfunction]
fn param_def(name: &str) -> Option<(&'static str, i32, f64, f64, f64, &'static str)> {
    use rsolver_model::params::{default_str, find, Kind};
    find(name).map(|d| {
        let k = match d.kind { Kind::Int => 1, Kind::Dbl => 2, Kind::Str => 3 };
        (d.name, k, d.min, d.max, d.default, default_str(d))
    })
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
    m.add_function(wrap_pyfunction!(param_def, m)?)?;
    m.add_class::<PyCoreModel>()?;
    m.add("INF", f64::INFINITY)
}
