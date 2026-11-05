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
    /// The model as `Model.write("*.mps")` text (01 §1.5); P0 compares it byte for byte.
    fn to_mps(&self) -> String { rsolver_model::mps_write::to_string(&self.inner) }
    /// Presolve (03 §1) with Gurobi parameter (name, value) pairs: MIP presolve when any column is not continuous,
    /// else LP presolve. Returns the presolved model.
    fn presolve(&self, py: Python<'_>, params: Vec<(String, f64)>) -> PyResult<PyCoreModel> {
        let md = &self.inner;
        let inner = if md.vtype.iter().any(|&t| t != b'C') {
            let p = rsolver_mip::Params::from_pairs(&params);
            py.allow_threads(|| rsolver_mip::presolve(md, &p))
                .map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("Unable to create presolved model"))?
        } else {
            let p = rsolver_lp::presolve::Params::from_pairs(&params);
            py.allow_threads(|| rsolver_lp::presolve::run(md, &p))
        };
        Ok(PyCoreModel { inner })
    }
    /// Replay one probing pass from a tools/probe_trace.js dump (tools/probe_check.py builds `d`): conflict-graph
    /// merge, probe matrix, engine, implication export and final merge. Returns what each stage produced.
    fn mip_probe_pass<'py>(&self, py: Python<'py>, d: &Bound<'py, pyo3::types::PyDict>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        use pyo3::types::PyDict;
        use rsolver_mip::{conflict::ConflictGraph, probe, probe_engine::EngineEnd};
        fn get<'py, T: pyo3::FromPyObject<'py>>(d: &Bound<'py, PyDict>, k: &str) -> PyResult<T> {
            d.get_item(k)?.ok_or_else(|| pyo3::exceptions::PyKeyError::new_err(k.to_string()))?.extract()
        }
        let graph = |pre: &str| -> PyResult<ConflictGraph> {
            Ok(ConflictGraph { n0: get(d, &format!("{pre}n0"))?, rc0: get(d, &format!("{pre}rc0"))?, ptr: get(d, &format!("{pre}ptr"))?,
                               adj: get(d, &format!("{pre}adj"))?, pa: get(d, &format!("{pre}pa"))?, pb: get(d, &format!("{pre}pb"))?,
                               flag: get(d, &format!("{pre}flag"))? })
        };
        let p = rsolver_mip::Params::from_pairs(&get::<Vec<(String, f64)>>(d, "params")?);
        let mut st = rsolver_mip::state::State::new(&self.inner, &p);
        st.lb = get(d, "lb")?; st.ub = get(d, "ub")?; st.obj = get(d, "obj")?; st.vtype = get(d, "vtype")?;
        st.rhs = get(d, "rhs")?; st.sense = get(d, "sense")?; st.row_cnt = get(d, "row_cnt")?; st.col_cnt = get(d, "col_cnt")?;
        st.inf_max = get(d, "inf_max")?; st.inf_min = get(d, "inf_min")?; st.act_max = get(d, "act_max")?; st.act_min = get(d, "act_min")?;
        st.mult_lo = get(d, "mult_lo")?; st.mult_hi = get(d, "mult_hi")?; st.objcon = get(d, "objcon")?; st.nnz = get(d, "nnz")?;
        st.cutoff = get(d, "cutoff")?; st.work_mult = get(d, "mu")?; st.work_base = get(d, "work_base")?; st.budget = get(d, "budget")?;
        st.prelimit = get(d, "prelimit")?; st.eq_rows = get::<i32>(d, "eq_rows")? != 0;
        st.cols_removed = get(d, "cols_removed")?; st.rows_removed = get(d, "rows_removed")?;
        // Gurobi's state can have more columns than the file (e.g. a range slack): sizes come from the dump
        st.n = st.lb.len();
        st.m = st.rhs.len();
        st.load_rows(&get::<Vec<i64>>(d, "rbeg")?, &get::<Vec<i32>>(d, "rcol")?, &get::<Vec<f64>>(d, "rval")?);
        st.cg = graph("g_")?;
        st.work = get(d, "work")?;
        let a: Vec<i32> = get(d, "args")?;
        let args = probe::Args { mode: a[0], a7: a[1], a8: a[2], a9: a[3], a10: a[4] };
        let t = probe::TimerView { acc: get(d, "t_acc")?, running: get::<i32>(d, "t_running")? != 0, limit: get(d, "t_limit")? };
        let o = PyDict::new(py);
        st.cg.merge(st.n, st.cols_removed, &st.col_cnt, &mut st.work, st.work_mult);
        let g = &st.cg;
        o.set_item("merged_work", st.work)?;
        o.set_item("merged_g", (g.n0, g.rc0, g.ptr.clone(), g.adj.clone()))?;
        let (mut c, mut cand) = match st.probe_build(&p, &args, get::<i32>(d, "pass")? == 1, &t) {
            Ok(x) => x,
            Err(e) => {
                o.set_item("build_stop", format!("{e:?}"))?;
                o.set_item("build_work", st.work)?;
                return Ok(o);
            }
        };
        let ctx = |c: &probe::Ctx| -> PyResult<Bound<'py, PyDict>> {
            let x = PyDict::new(py);
            x.set_item("rbeg", c.rbeg.clone())?; x.set_item("rnb", c.rnb.clone())?; x.set_item("rcol", c.rcol.clone())?;
            x.set_item("rval", c.rval.clone())?; x.set_item("cbeg", c.cbeg.clone())?; x.set_item("crow", c.crow.clone())?;
            x.set_item("cval", c.cval.clone())?; x.set_item("rhs", c.rhs.clone())?; x.set_item("range", c.range.clone())?;
            x.set_item("sense", c.sense.clone())?; x.set_item("lb", c.lb.clone())?; x.set_item("ub", c.ub.clone())?;
            x.set_item("amin", c.amin.clone())?; x.set_item("amax", c.amax.clone())?; x.set_item("budget", c.budget)?;
            x.set_item("fac", c.fac)?; x.set_item("stop", c.stop as i32)?; x.set_item("nchg", c.nchg)?; x.set_item("nbin", c.nbin)?;
            x.set_item("neq", c.neq)?; x.set_item("nagg", c.nagg)?; x.set_item("aggto", c.aggto.clone())?;
            x.set_item("aggmul", c.aggmul.clone())?; x.set_item("aggoff", c.aggoff.clone())?; x.set_item("cnt0", c.cnt0.clone())?;
            x.set_item("cnt1", c.cnt1.clone())?; x.set_item("gbeg", c.gbeg.clone())?; x.set_item("imps", c.imps.clone())?;
            x.set_item("imp", c.imp as i32)?; x.set_item("cap", c.cap)?; x.set_item("rec", c.rec as i32)?;
            x.set_item("rhead", c.rhead.clone())?; x.set_item("rnext", c.rnext.clone())?; x.set_item("rlit", c.rlit.clone())?;
            x.set_item("rslack", c.rslack.clone())?;
            Ok(x)
        };
        o.set_item("ctx_in", ctx(&c)?)?;
        o.set_item("cand_in", cand.clone())?;
        o.set_item("in_work", st.work)?;
        let mut w = st.work;
        let end = st.probe_engine(&p, &mut c, &mut cand, &mut w);
        o.set_item("engine_end", format!("{end:?}"))?;
        o.set_item("ctx_out", ctx(&c)?)?;
        o.set_item("cand_out", cand.clone())?;
        o.set_item("out_work", w)?;
        if end != EngineEnd::Done || d.get_item("imp_work")?.is_none() {
            return Ok(o);
        }
        // the export reads the state after the write-back: take its bounds and counts from the dump
        st.lb = get(d, "imp_lb")?; st.ub = get(d, "imp_ub")?; st.col_cnt = get(d, "imp_col_cnt")?; st.nnz = get(d, "imp_nnz")?;
        st.cols_removed = get(d, "imp_cols_removed")?; st.rows_removed = get(d, "imp_rows_removed")?;
        st.cg = graph("imp_g_")?;
        st.work = get(d, "imp_work")?;
        match st.probe_export(&p, &c, &cand) {
            None => o.set_item("export", py.None())?,
            Some(nadd) => {
                o.set_item("export", nadd)?;
                let out = st.cg.merge(st.n, st.cols_removed, &st.col_cnt, &mut st.work, st.work_mult);
                let g = &st.cg;
                o.set_item("end_out", out)?;
                o.set_item("end_g", (g.n0, g.rc0, g.ptr.clone(), g.adj.clone()))?;
                o.set_item("end_work", st.work)?;
            }
        }
        Ok(o)
    }
    /// What each PRELIMIT count of MIP presolve was (count k is element k-1), as far as the port runs.
    fn mip_presolve_trace(&self, params: Vec<(String, f64)>) -> Vec<String> {
        let p = rsolver_mip::Params::from_pairs(&params);
        let mut st = rsolver_mip::state::State::new(&self.inner, &p);
        rsolver_mip::reduce_traced(&mut st, &p, true).1.into_iter().map(String::from).collect()
    }
    /// MIP presolve state where it stopped (`finish`: when model generation starts) as `tools/work_oracle.py --mip`
    /// records it: (work, problem type, integer columns, binary columns, +0xf8, rows removed, columns removed,
    /// tracker level).
    #[allow(clippy::type_complexity)]
    fn mip_presolve_state(&self, params: Vec<(String, f64)>, finish: bool) -> (f64, i32, i32, i32, bool, i32, i32, usize) {
        let p = rsolver_mip::Params::from_pairs(&params);
        let mut st = rsolver_mip::state::State::new(&self.inner, &p);
        if rsolver_mip::reduce(&mut st, &p).is_ok() && finish {
            rsolver_mip::finish(&mut st, &p);
        }
        (st.work, st.ptype, st.nint, st.nbin, st.eq_rows, st.rows_removed, st.cols_removed, st.tracker.level)
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
        let p = rsolver_lp::pdhg::Params { iter_limit, rel_tol, abs_tol, conv_tol, work_limit };
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
