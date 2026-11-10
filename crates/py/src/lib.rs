//! PyO3 layer: one call in, one tuple out. Solve runs without the GIL (cf. re/notes/02).
use pyo3::prelude::*;

/// build_model(obj, rows, lb, ub, vtype) -> CoreModel for a model built through the API (minimization form).
/// rows: list of ([(col, coef), ...], lo, hi). A ranged row becomes an equality plus a slack column in
/// [0, hi − lo] appended after the variables, as the MPS reader does for RANGES.
/// Rows of a built model: (sparse coefficients, lower, upper).
type Rows = Vec<(Vec<(usize, f64)>, f64, f64)>;
/// MIP postsolve records: (type, data bytes, padding ranges, pointer offsets).
type Recs = Vec<(i32, Vec<u8>, Vec<(usize, usize)>, Vec<usize>)>;
type PresolveProgressOutput = (Option<(f64, Vec<f64>)>, (i32, Option<PyCoreModel>, Option<RootProgressOutput>));
type MipSolveOutput = (i32, f64, Option<f64>, Option<Vec<f64>>, u64, f64);
type RootProgressOutput = (i32, f64, Option<f64>, Option<Vec<f64>>, Option<RootSummary>, (Vec<f64>, Vec<f64>), bool);
type RootSummary = (i32, i32, f64, f64);
type InitialRootOutput = (Option<(f64, Vec<f64>)>, (i32, Option<PyCoreModel>, Option<RootSummary>, Option<(Vec<f64>, Vec<f64>)>));

fn root_summary(r: rsolver_mip::root::RootLp) -> RootSummary {
    use rsolver_lp::simplex::Status;
    let status = match r.lp.status {
        Status::Optimal => 2, Status::Infeasible => 3, Status::Unbounded => 5,
        Status::IterLimit => 7, Status::Numeric => 12,
    };
    (r.method, status, r.lp.obj, r.lp.iter as f64)
}

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
    /// Run the whole `4d3a1d` call (write-back included) on a state dumped by tools/mip_state_dump.js at its entry,
    /// with the conflict graph and timer of tools/probe_trace.js (tools/probe_apply_check.py builds `d`). Returns the
    /// state at the end in dump form, the new postsolve records (type, bytes, pad ranges, pointer offsets) and the
    /// status.
    fn mip_probe_full<'py>(&self, py: Python<'py>, d: &Bound<'py, pyo3::types::PyDict>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        use pyo3::types::PyDict;
        use rsolver_mip::{conflict::ConflictGraph, dump::{Parts, QueueParts}, probe};
        fn get<'py, T: pyo3::FromPyObject<'py>>(d: &Bound<'py, PyDict>, k: &str) -> PyResult<T> {
            d.get_item(k)?.ok_or_else(|| pyo3::exceptions::PyKeyError::new_err(k.to_string()))?.extract()
        }
        let q = |k: &str| -> PyResult<QueueParts> {
            let t: Bound<'py, PyDict> = get(d, k)?;
            Ok(QueueParts { flag: get(&t, "flag")?, a: get(&t, "a")?, b: get(&t, "b")?, list: get(&t, "list")?,
                            cache: get(&t, "cache")?, cache2: get(&t, "cache2")? })
        };
        let parts = Parts {
            n: get(d, "n")?, m: get(d, "m")?, lb: get(d, "lb")?, ub: get(d, "ub")?, obj: get(d, "obj")?, rhs: get(d, "rhs")?,
            sense: get(d, "sense")?, vtype: get(d, "vtype")?, row_cnt: get(d, "row_cnt")?, col_cnt: get(d, "col_cnt")?,
            inf_max: get(d, "inf_max")?, inf_min: get(d, "inf_min")?, act_max: get(d, "act_max")?, act_min: get(d, "act_min")?,
            mult_lo: get(d, "mult_lo")?, mult_hi: get(d, "mult_hi")?, col_flag: get(d, "col_flag")?, objcon: get(d, "objcon")?,
            nnz: get(d, "nnz")?, work: get(d, "work")?, work_mult: get(d, "work_mult")?, counters: get(d, "counters")?,
            nfree: get(d, "nfree")?, npend: get(d, "npend")?, rows: get(d, "rows")?, cols: get(d, "cols")?,
            level: get(d, "level")?, active: get(d, "active")?, r: q("tr_r")?, c: q("tr_c")?,
            nbin: d.get_item("nbin")?.map(|x| x.extract()).transpose()?.unwrap_or(-1),
            nint: d.get_item("nint")?.map(|x| x.extract()).transpose()?.unwrap_or(-1),
            node_cap: 0, chunk: 0,
        };
        let p = rsolver_mip::Params::from_pairs(&get::<Vec<(String, f64)>>(d, "params")?);
        let mut st = rsolver_mip::state::State::from_parts(&parts, &p);
        st.prelimit = get(d, "prelimit")?;
        st.cg = ConflictGraph { n0: get(d, "g_n0")?, rc0: get(d, "g_rc0")?, ptr: get(d, "g_ptr")?, adj: get(d, "g_adj")?,
                                pa: get(d, "g_pa")?, pb: get(d, "g_pb")?, flag: get(d, "g_flag")? };
        let a: Vec<i32> = get(d, "args")?;
        let args = probe::Args { mode: a[0], a7: a[1], a8: a[2], a9: a[3], a10: a[4] };
        let t = probe::TimerView { acc: get(d, "t_acc")?, running: get::<i32>(d, "t_running")? != 0, limit: get(d, "t_limit")? };
        let rc = st.probe_run(&p, &args, &t);
        let e = st.to_parts();
        let o = PyDict::new(py);
        o.set_item("rc", rc)?;
        o.set_item("bad_col", st.bad_col)?;
        for (k, v) in [("lb", &e.lb), ("ub", &e.ub), ("obj", &e.obj), ("rhs", &e.rhs), ("act_max", &e.act_max), ("act_min", &e.act_min),
                       ("mult_lo", &e.mult_lo), ("mult_hi", &e.mult_hi)] {
            o.set_item(k, v.clone())?;
        }
        for (k, v) in [("row_cnt", &e.row_cnt), ("col_cnt", &e.col_cnt), ("inf_max", &e.inf_max), ("inf_min", &e.inf_min), ("counters", &e.counters)] {
            o.set_item(k, v.clone())?;
        }
        o.set_item("sense", e.sense.clone())?;
        o.set_item("vtype", e.vtype.clone())?;
        o.set_item("objcon", e.objcon)?;
        o.set_item("nnz", e.nnz)?;
        o.set_item("work", e.work)?;
        o.set_item("free", (e.nfree, e.npend))?;
        o.set_item("rows", e.rows.clone())?;
        o.set_item("cols", e.cols.clone())?;
        type Rec = (i32, Vec<u8>, Vec<(usize, usize)>, Vec<usize>);
        let recs: Vec<Rec> =
            st.post.iter().map(|r| { let l = r.layout(); (r.kind(), l.bytes, l.pads, l.ptrs) }).collect();
        o.set_item("recs", recs)?;
        o.set_item("g_edges", st.cg.edges())?;
        for (k, t) in [("tr_r", &e.r), ("tr_c", &e.c)] {
            o.set_item(k, (t.flag.clone(), t.a, t.b, t.list.clone()))?;
        }
        Ok(o)
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
    /// MIP presolve postsolve records in push order as (type, data bytes in Gurobi's layout, padding byte ranges,
    /// pointer field offsets); pointers hold their target's offset from the data start (tools/mip_rec_check.py).
    fn mip_post_records(&self, params: Vec<(String, f64)>) -> Recs {
        let p = rsolver_mip::Params::from_pairs(&params);
        let mut st = rsolver_mip::state::State::new(&self.inner, &p);
        if rsolver_mip::reduce(&mut st, &p).is_ok() && rsolver_mip::finish(&mut st, &p) == 0 {
            rsolver_mip::split_rows(&mut st, &p);
        }
        st.post.iter().map(|r| {
            let l = r.layout();
            (r.kind(), l.bytes, l.pads, l.ptrs)
        }).collect()
    }
    /// Restore and validate an incumbent supplied in the presolved model's coordinates.
    fn mip_restore_candidate(&self, params: Vec<(String, f64)>, reduced_x: Vec<f64>) -> PyResult<(Vec<f64>, f64)> {
        let p = rsolver_mip::Params::from_pairs(&params);
        let (_, state) = rsolver_mip::presolve_with_state(&self.inner, &p)
            .map_err(|s| pyo3::exceptions::PyValueError::new_err(format!("presolve status {s}")))?;
        rsolver_mip::bnb::restore_candidate(&self.inner, &p, &state, &reduced_x)
            .map_err(pyo3::exceptions::PyValueError::new_err)
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
    /// optimize() of a continuous model through the driver (docs/m1/m1_driver.md) with Gurobi parameter (name, value)
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
    /// MIP root LP of this presolved model (crates/mip root.rs): (method, status, objective, iterations), status coded
    /// as in `solve`; None when the model has no columns.
    #[pyo3(signature = (params, incumbent=None))]
    fn mip_root(&self, py: Python<'_>, params: Vec<(String, f64)>, incumbent: Option<Vec<f64>>) -> PyResult<Option<(i32, i32, f64, f64)>> {
        let p = rsolver_mip::Params::from_pairs(&params);
        let r = py.allow_threads(|| rsolver_mip::root::solve_root_with_incumbent(&self.inner, &p, incumbent.as_deref()))
            .map_err(pyo3::exceptions::PyValueError::new_err)?;
        Ok(r.map(root_summary))
    }
    /// Continue a presolved MIP root: (MIP status, bound, incumbent objective, incumbent,
    /// root LP summary, bounds after propagation, propagation complete). Status 1 requires further search.
    #[pyo3(signature = (params, incumbent=None))]
    fn mip_root_progress(&self, py: Python<'_>, params: Vec<(String, f64)>, incumbent: Option<Vec<f64>>) -> PyResult<RootProgressOutput> {
        let p = rsolver_mip::Params::from_pairs(&params);
        let r = py.allow_threads(|| rsolver_mip::root::solve_root_progress(&self.inner, &p, incumbent.as_deref()))
            .map_err(pyo3::exceptions::PyValueError::new_err)?;
        Ok((r.status as i32, r.bound, r.incumbent_objective, r.incumbent, r.root.map(root_summary),
            (r.domain.lb, r.domain.ub), r.propagation_complete))
    }
    /// Solve a linear MIP in this model's coordinates: status, bound, objective, solution, nodes, iterations.
    #[pyo3(signature = (params, incumbent=None))]
    fn mip_solve(&self, py: Python<'_>, params: Vec<(String, f64)>, incumbent: Option<Vec<f64>>)
        -> PyResult<MipSolveOutput>
    {
        let p = rsolver_mip::Params::from_pairs(&params);
        let r = py.allow_threads(|| rsolver_mip::bnb::solve(&self.inner, &p, incumbent.as_deref()))
            .map_err(pyo3::exceptions::PyValueError::new_err)?;
        Ok((r.status as i32, r.bound, r.incumbent_objective, r.incumbent, r.nodes, r.iterations))
    }
    /// Solve a MIP through presolve and restore the incumbent to the input model.
    fn mip_solve_original(&self, py: Python<'_>, params: Vec<(String, f64)>)
        -> PyResult<MipSolveOutput>
    {
        let p = rsolver_mip::Params::from_pairs(&params);
        let r = py.allow_threads(|| rsolver_mip::bnb::solve_original(&self.inner, &p))
            .map_err(pyo3::exceptions::PyValueError::new_err)?;
        Ok((r.status as i32, r.bound, r.incumbent_objective, r.incumbent, r.nodes, r.iterations))
    }
    /// Solve a MIP and report whether presolve was used or why the original model was solved.
    fn mip_solve_diagnostics(&self, py: Python<'_>, params: Vec<(String, f64)>)
        -> PyResult<(MipSolveOutput, &'static str)>
    {
        let p = rsolver_mip::Params::from_pairs(&params);
        let r = py.allow_threads(|| rsolver_mip::bnb::solve_original(&self.inner, &p))
            .map_err(pyo3::exceptions::PyValueError::new_err)?;
        Ok(((r.status as i32, r.bound, r.incumbent_objective, r.incumbent, r.nodes, r.iterations), r.path))
    }
    /// Original incumbent and (presolve status, unchanged P0, whole-MIP root progress).
    fn mip_presolve_progress(&self, py: Python<'_>, params: Vec<(String, f64)>)
        -> PyResult<PresolveProgressOutput>
    {
        use rsolver_mip::root::InitialRootError;
        let p = rsolver_mip::Params::from_pairs(&params);
        let run = py.allow_threads(|| rsolver_mip::root::solve_root_progress_from_original(&self.inner, &p));
        let incumbent = run.original_incumbent.map(|v| (v.objective, v.x));
        match run.presolve {
            Ok((model, r)) => Ok((incumbent, (0, Some(PyCoreModel { inner: model }), Some((
                r.status as i32, r.bound, r.incumbent_objective, r.incumbent, r.root.map(root_summary),
                (r.domain.lb, r.domain.ub), r.propagation_complete))))),
            Err(InitialRootError::Presolve(status)) => Ok((incumbent, (status, None, None))),
            Err(InitialRootError::UnsupportedModel) => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "mip_presolve_progress requires a linear MIP with C, B, or I columns")),
        }
    }
    /// Run the initial heuristic on this original MIP, then presolve and solve its root.
    fn mip_presolve_root(&self, py: Python<'_>, params: Vec<(String, f64)>) -> PyResult<InitialRootOutput> {
        use rsolver_mip::root::InitialRootError;
        let p = rsolver_mip::Params::from_pairs(&params);
        let run = py.allow_threads(|| rsolver_mip::root::solve_root_from_original(&self.inner, &p));
        let incumbent = run.original_incumbent.map(|v| (v.objective, v.x));
        match run.presolve {
            Ok(q) => Ok((incumbent, (0, Some(PyCoreModel { inner: q.presolved }),
                q.root.map(root_summary), Some((q.lower_bounds, q.upper_bounds))))),
            Err(InitialRootError::Presolve(status)) => Ok((incumbent, (status, None, None, None))),
            Err(InitialRootError::UnsupportedModel) => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "mip_presolve_root requires a linear MIP with C, B, or I columns")),
        }
    }
    /// Initial heuristic solution in presolved coordinates: (objective, values), or None if none found.
    fn mip_initial_incumbent(&self, py: Python<'_>, params: Vec<(String, f64)>) -> Option<(f64, Vec<f64>)> {
        let p = rsolver_mip::Params::from_pairs(&params);
        let x = py.allow_threads(|| rsolver_mip::root::initial_incumbent(&self.inner, &p))?;
        let obj = self.inner.objcon + self.inner.obj.iter().zip(&x).map(|(c, v)| c * v).sum::<f64>();
        Some((obj, x))
    }
    /// Bounds used by the MIP root LP after domain initialization and automatic or explicit incumbent propagation.
    #[pyo3(signature = (params, incumbent=None))]
    fn mip_root_bounds(&self, params: Vec<(String, f64)>, incumbent: Option<Vec<f64>>) -> PyResult<(Vec<f64>, Vec<f64>)> {
        let p = rsolver_mip::Params::from_pairs(&params);
        let d = rsolver_mip::root::root_domain(&self.inner, &p, incumbent.as_deref())
            .map_err(pyo3::exceptions::PyValueError::new_err)?;
        Ok((d.lb, d.ub))
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

/// Run one MIP presolve reduction (`State::run_named`) on a state given as a dict in `tools/mip_state_dump.js`'s
/// field names (tracker flattened by tools/mip_dump_check.py); returns the resulting state as such a dict plus
/// `ret`, `extra`, `bad_row`, `bad_col` and `recs` (new records as in `mip_post_records`).
#[pyfunction]
fn mip_dump_run<'py>(py: Python<'py>, name: &str, d: &Bound<'py, pyo3::types::PyDict>, params: Vec<(String, f64)>)
                     -> PyResult<Bound<'py, pyo3::types::PyDict>> {
    use rsolver_mip::dump::{Parts, QueueParts};
    fn g<'py, T: FromPyObject<'py>>(d: &Bound<'py, pyo3::types::PyDict>, k: &str) -> PyResult<T> {
        d.get_item(k)?.ok_or_else(|| pyo3::exceptions::PyKeyError::new_err(k.to_string()))?.extract()
    }
    let q = |k: &str| -> PyResult<QueueParts> {
        let lists: Vec<Vec<i32>> = g(d, &format!("{k}_list"))?;
        let cache: Vec<Option<Vec<i32>>> = g(d, &format!("{k}_cache"))?;
        let cache2: Vec<Option<Vec<i32>>> = g(d, &format!("{k}_cache2"))?;
        let a: Vec<usize> = g(d, &format!("{k}_a"))?;
        let b: Vec<usize> = g(d, &format!("{k}_b"))?;
        Ok(QueueParts { flag: g(d, &format!("{k}_flag"))?, a: [a[0], a[1], a[2]], b: [b[0], b[1], b[2]],
                        list: [lists[0].clone(), lists[1].clone(), lists[2].clone()],
                        cache: [cache[0].clone(), cache[1].clone(), cache[2].clone()],
                        cache2: [cache2[0].clone(), cache2[1].clone(), cache2[2].clone()] })
    };
    let active: Vec<bool> = g(d, "active")?;
    let parts = Parts {
        n: g(d, "n")?, m: g(d, "m")?, lb: g(d, "lb")?, ub: g(d, "ub")?, obj: g(d, "obj")?, rhs: g(d, "rhs")?,
        sense: g(d, "sense")?, vtype: g(d, "vtype")?, row_cnt: g(d, "row_cnt")?, col_cnt: g(d, "col_cnt")?,
        inf_max: g(d, "inf_max")?, inf_min: g(d, "inf_min")?, act_max: g(d, "act_max")?, act_min: g(d, "act_min")?,
        mult_lo: g(d, "mult_lo")?, mult_hi: g(d, "mult_hi")?, col_flag: g(d, "col_flag")?, objcon: g(d, "objcon")?,
        nnz: g(d, "nnz")?, work: g(d, "work")?, work_mult: g(d, "work_mult")?, counters: g(d, "counters")?,
        nfree: g(d, "nfree")?, npend: g(d, "npend")?, rows: g(d, "rows")?, cols: g(d, "cols")?,
        level: g(d, "level")?, active: [active[0], active[1], active[2]], r: q("r")?, c: q("c")?,
        nbin: d.get_item("nbin")?.map(|x| x.extract()).transpose()?.unwrap_or(-1),
        nint: d.get_item("nint")?.map(|x| x.extract()).transpose()?.unwrap_or(-1),
        node_cap: g(d, "node_cap").unwrap_or(0), chunk: g(d, "chunk").unwrap_or(0),
    };
    let p = rsolver_mip::Params::from_pairs(&params);
    let mut st = rsolver_mip::state::State::from_parts(&parts, &p);
    let (ret, extra) = st.run_named(name, &p)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err(format!("unknown reduction {name}")))?;
    let o = st.to_parts();
    let r = pyo3::types::PyDict::new(py);
    macro_rules! put { ($($k:ident),*) => { $( r.set_item(stringify!($k), o.$k.clone())?; )* } }
    put!(n, m, lb, ub, obj, rhs, sense, vtype, row_cnt, col_cnt, inf_max, inf_min, act_max, act_min, mult_lo, mult_hi,
         col_flag, objcon, nnz, work, work_mult, counters, nfree, npend, rows, cols, level, node_cap, chunk);
    r.set_item("active", o.active.to_vec())?;
    for (k, qp) in [("r", &o.r), ("c", &o.c)] {
        r.set_item(format!("{k}_flag"), qp.flag.clone())?;
        r.set_item(format!("{k}_a"), qp.a.to_vec())?;
        r.set_item(format!("{k}_b"), qp.b.to_vec())?;
        r.set_item(format!("{k}_list"), qp.list.to_vec())?;
    }
    r.set_item("ret", ret)?;
    r.set_item("extra", extra)?;
    r.set_item("bad_row", st.bad_row)?;
    r.set_item("bad_col", st.bad_col)?;
    let recs: Recs = st.post.iter().map(|x| {
        let l = x.layout();
        (x.kind(), l.bytes, l.pads, l.ptrs)
    }).collect();
    r.set_item("recs", recs)?;
    Ok(r)
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(build_model, m)?)?;
    m.add_function(wrap_pyfunction!(read_model, m)?)?;
    m.add_function(wrap_pyfunction!(sort_desc, m)?)?;
    m.add_function(wrap_pyfunction!(mip_dump_run, m)?)?;
    m.add_function(wrap_pyfunction!(param_def, m)?)?;
    m.add_class::<PyCoreModel>()?;
    m.add("INF", f64::INFINITY)
}
