//! M3-04c-1a replay: `push_init` from the push:enter state with the Rust serial backend, then `push_loop` to its end.
//! The input holds no loop-start state and no numeric output; the comparison is
//! `tools/push_loop_rust.py --init --compare`.
mod loop_io;

use loop_io::*;
use rsolver_lp::crossover::{PushLp, PushParams, PushState};
use rsolver_lp::crossover_backend::{SerialBackend, SerialStart};
use rsolver_lp::crossover_init::*;
use rsolver_lp::crossover_loop::*;
use rsolver_lp::crossover_refactor::{Alloc, Matrix};
use rsolver_lp::crossover_supplier::PushColumn;
use rsolver_lp::lu::BlockState;
use std::cell::RefCell;
use std::rc::Rc;

/// Writes the `init` line after the first factor call, then delegates.
struct InitRecording {
    inner: SerialStart,
    out: Out,
}

impl PushInitBackend for InitRecording {
    fn initial_factor(&mut self, ctl: &mut FactorCtl, st: &PushState) -> (i32, Option<Renumber>) {
        let (ret, r) = self.inner.initial_factor(ctl, st);
        let mut after = st.clone();
        if let Some(r) = &r {
            renumber(&mut after, r);
        }
        let stats = self.inner.backend.as_ref().unwrap().stats();
        self.out.borrow_mut().push(format!("init;hdr {};leave {};vstat {};stats {}", ints(&after.header), after.leave,
                                           ints(&after.vstat), stats_text(&stats)));
        (ret, r)
    }
    fn rhs_ftran(&mut self, rhs: &[f64], raw: &mut [f64]) {
        self.inner.rhs_ftran(rhs, raw)
    }
    fn tighten(&mut self, markowitz: f64, limit: Option<i32>) {
        self.inner.tighten(markowitz, limit)
    }
}

/// Records the loop's numeric inputs (the `ftran` and `rin` lines), as in the M3-04c-24 replay.
struct Recording {
    inner: SerialBackend,
    out: Out,
}

impl PushLoopBackend for Recording {
    fn ftran(&mut self, input: &PushColumn, output: &mut PushColumn, spike: Option<&mut PushColumn>) {
        let line = "ftran".to_string() + &col_fields("", input) + &format!(";spike {}", spike.is_some() as i32);
        self.out.borrow_mut().push(line);
        self.inner.ftran(input, output, spike);
    }
    fn btran(&mut self, input: &PushColumn) -> PushColumn {
        self.inner.btran(input)
    }
    fn stats(&self) -> FactorStats {
        self.inner.stats()
    }
    fn factor(&mut self, site: FactorSite, ctl: &mut FactorCtl, st: &PushState, work: Option<&PushColumn>)
              -> (i32, Option<Renumber>) {
        self.inner.factor(site, ctl, st, work)
    }
    fn quality(&mut self, ctl: &mut FactorCtl, enter: i32, period: i32) -> i32 {
        self.inner.quality(ctl, enter, period)
    }
    fn recompute_ftran(&mut self, rhs: &[f64], raw: &mut [f64]) {
        self.out.borrow_mut().push(format!("rin;rhs {}", hexs(rhs)));
        self.inner.recompute_ftran(rhs, raw);
    }
}

fn run_case(case: &[&str], out: &Out) -> Result<(), String> {
    let mut t = rec(case, 1, "model");
    let (n, m, nnz): (usize, usize, usize) = (t.i(), t.i(), t.i());
    let starts: Vec<i64> = (0..n).map(|_| t.i()).collect();
    let lengths = t.is(n);
    let rows = t.is(nnz);
    let values = t.fs(nnz);
    let (cost, lb, ub) = (t.fs(n + m), t.fs(n + m), t.fs(n + m));
    t.done();
    let mut t = rec(case, 2, "cfg");
    let par = PushParams { h: t.f(), piv_tol: t.f(), inf: t.f(), fixed: t.f() };
    let drop = t.f();
    let (model_status, attempt, first): (i32, i32, i32) = (t.i(), t.i(), t.i());
    t.done();
    let mut t = rec(case, 3, "enter");
    let vstat = t.is(n + m);
    let header = t.is(m);
    let mut xs = t.fs(n);
    xs.extend(t.fs(m));
    let rhs = t.fs(m);
    let markowitz = t.f();
    let iterations = t.f();
    let (request, numeric, failure, status, enter, leave): (i32, i32, i32, i32, i32, i32) =
        (t.i(), t.i(), t.i(), t.i(), t.i(), t.i());
    t.done();
    let mut t = rec(case, 4, "lu");
    let (zero, tol) = (t.f(), t.f());
    let alloc = Alloc { x158: t.i(), pool_mod16: [t.i(), t.i()], x188: t.i(), x1f8: t.f() };
    let block = BlockState { x2c0: t.i(), x4e0: t.i(), x4e4: t.i() };
    let interval: i32 = t.i();
    t.done();

    let lp = PushLp { n, m, starts: &starts, lengths: &lengths, rows: &rows, values: &values, cost: &cost, lb: &lb,
                      ub: &ub };
    let matrix = Matrix { m, n, beg: starts.clone(), len: lengths.clone(), ind: rows.clone(), val: values.clone() };
    let mut backend = InitRecording { inner: SerialStart::new(matrix, zero, tol, alloc, block, interval, drop),
                                      out: out.clone() };
    // S+0x328 is neither written by the initialization nor read by the loop before push_pick writes it.
    let entry = InitEntry { rhs: &rhs, vstat, header, xs, iterations, request, numeric, failure, status, enter, leave,
                            direction: 0, markowitz, first: first != 0, attempt, model_status, tau_in: None,
                            prev_pinf: None, condition: None, tighten: None };
    let res = push_init(&lp, &par, entry, &mut backend);
    let s = &res.state;
    let r = &s.round;
    let limit = backend.inner.backend.as_ref().map_or(-1, |b| b.stats().limit);
    out.borrow_mut().push(format!(
        "start;cand {};total {};sup {};vstat {};header {};leave {};rhs {};basic {};width {};marks {};pinf {};iter {};\
         req {};numeric {};failure {};status {};retry {};nlimit {};mk {};limit {limit};adj {};cond {};pos {};dfr {};\
         fail {};smarks {};csum {};succ {}",
        ints(&r.candidates), r.candidates.len(), r.support, ints(&s.st.vstat), ints(&s.st.header), s.st.leave,
        hexs(&s.st.rhs), hexs(&s.st.basic), hexs(&s.st.width), marks(&s.st.marks), hex(s.pinf), hex(s.st.iterations),
        r.request, r.numeric, r.failure, r.status, r.retry, res.numeric_limit, hex(res.markowitz),
        res.adjusted as i32, if res.condition_known { "known" } else { "unknown" }, r.position, r.deferred,
        ints(&r.failures), opt(&r.marks), hex(r.count_sum), r.successes));
    if res.exit != InitExit::Loop {
        return Err(format!("initialization exit {:?}", res.exit));
    }
    let cfg = LoopConfig { attempt, model_status, numeric_limit: res.numeric_limit, drop };
    let mut s = res.state;
    let mut lb_ = Recording { inner: backend.inner.backend.take().unwrap(), out: out.clone() };
    let mut trace = tracer(out.clone(), true);
    match push_loop(&lp, &par, &cfg, &mut s, &mut lb_, &mut trace) {
        LoopExit::End => Ok(()),
        other => Err(format!("{other:?}")),
    }
}

#[test]
fn init_loop_trace() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let text = std::fs::read_to_string(format!("{root}/bench/oracle/m3_push_init_rust/input.txt"))
        .expect("missing bench/oracle/m3_push_init_rust/input.txt (tools/push_loop_rust.py --init --write)");
    let mut lines = text.lines();
    let out: Out = Rc::new(RefCell::new(vec![lines.next().unwrap().to_string()]));
    let _dump = Dump(out.clone(), format!("{root}/target/m3_push_init_rust"));
    let rest: Vec<&str> = lines.collect();
    let mut failed = Vec::new();
    for case in rest.chunks(5) {
        let name = rec(case, 0, "case").s().to_string();
        out.borrow_mut().push(format!("case;{name}"));
        let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_case(case, &out)));
        match run {
            Ok(Ok(())) => {}
            Ok(Err(e)) => failed.push(format!("{name}: {e}")),
            Err(e) => {
                let msg = e.downcast_ref::<String>().cloned()
                    .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                out.borrow_mut().push(format!("error;msg {}", msg.replace([';', '\n'], " ")));
                failed.push(format!("{name}: {msg}"));
            }
        }
    }
    assert!(failed.is_empty(), "{failed:?}");
}
