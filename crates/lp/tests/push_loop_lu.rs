//! M3-04c-24 replay: `push_loop` with the Rust serial backend (`crossover_backend::SerialBackend`) instead of a tape.
//! The backend starts from the Rust refactor of the push's initial factor call; after that no original factor or
//! numeric output enters. The supplier FTRAN and RHS FTRAN inputs are recorded as in the M3-04c-22 trace; the
//! comparison is `tools/push_loop_rust.py --lu --compare`.
mod loop_io;

use loop_io::*;
use rsolver_lp::crossover::PushState;
use rsolver_lp::crossover_backend::SerialBackend;
use rsolver_lp::crossover_loop::*;
use rsolver_lp::crossover_refactor::{Alloc, Matrix};
use rsolver_lp::crossover_supplier::PushColumn;
use rsolver_lp::lu::BlockState;
use std::cell::RefCell;
use std::rc::Rc;

/// Records the inputs the loop hands the numeric backend (the `ftran` and `rin` lines), then delegates.
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

#[test]
fn loop_lu_trace() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let text = std::fs::read_to_string(format!("{root}/bench/oracle/m3_push_loop_lu/input.txt"))
        .expect("missing bench/oracle/m3_push_loop_lu/input.txt (tools/push_loop_rust.py --lu --write)");
    let mut lines = text.lines();
    let out: Out = Rc::new(RefCell::new(vec![lines.next().unwrap().to_string()]));
    let _dump = Dump(out.clone(), format!("{root}/target/m3_push_loop_lu"));
    let rest: Vec<&str> = lines.collect();
    let mut failed = Vec::new();
    for case in rest.chunks(6) {
        let name = rec(case, 0, "case").s().to_string();
        out.borrow_mut().push(format!("case;{name}"));
        let (c, mut s) = parse_case(case);
        let (n, m) = (c.n, c.m);
        let mut t = rec(case, 5, "lu");
        let (zero, tol) = (t.f(), t.f());
        let alloc = Alloc { x158: t.i(), pool_mod16: [t.i(), t.i()], x188: t.i(), x1f8: t.f() };
        let block = BlockState { x2c0: t.i(), x4e0: t.i(), x4e4: t.i() };
        let interval: i32 = t.i();
        let header = t.is(m);
        let leave: i32 = t.i();
        let vstat = t.is(n + m);
        t.done();
        let matrix = Matrix { m, n, beg: c.starts.clone(), len: c.lengths.clone(), ind: c.rows.clone(),
                              val: c.values.clone() };
        let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let init = SerialBackend::initial(matrix, zero, tol, alloc, block, interval, c.cfg.drop, &header, leave,
                                              &vstat);
            out.borrow_mut().push(format!("init;hdr {};leave {};vstat {};stats {}", ints(&init.header), init.leave,
                                          ints(&init.vstat), stats_text(&init.backend.stats())));
            // the loop starts on the captured push state; its basis must be the one the Rust factor renumbered
            assert_eq!(init.header, s.st.header, "{name}: initial factor header differs from the loop start");
            let mut backend = Recording { inner: init.backend, out: out.clone() };
            let lp = c.lp();
            let mut trace = tracer(out.clone(), true);
            push_loop(&lp, &c.par, &c.cfg, &mut s, &mut backend, &mut trace)
        }));
        match run {
            Ok(LoopExit::End) => {}
            Ok(other) => failed.push(format!("{name}: {other:?}")),
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
