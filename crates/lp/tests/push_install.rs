//! M3-04f-1 install boundary replay: `push_install` from the original's `install:enter` state (with the install-time
//! edits applied) and the condition the original's `12f282` returned there. The comparison with `push:leave` is
//! `tools/push_install_rust.py --compare-unit`. Contract: docs/m3/primal_push_install.md.
mod loop_io;

use loop_io::*;
use rsolver_lp::crossover::{PushLp, PushParams, PushState};
use rsolver_lp::crossover_install::{push_install, InstallEntry};
use std::cell::RefCell;
use std::rc::Rc;

fn run(t: &mut Tok<'_>) -> String {
    let (n, m): (usize, usize) = (t.i(), t.i());
    let (lb, ub) = (t.fs(n + m), t.fs(n + m));
    let (tau, inf) = (t.f(), t.f());
    let (first, attempt, model_status, interval, env_2ba0): (i32, i32, i32, i32, i32) =
        (t.i(), t.i(), t.i(), t.i(), t.i());
    let k: usize = t.i();
    let candidates = t.is(k);
    let header = t.is(m);
    let (basic, width) = (t.fs(m), t.fs(m));
    let vstat = t.is(n + m);
    let xs = t.fs(n + m);
    let condition = t.f();
    t.done();
    // only lb/ub, h (τ) and inf are read by the install
    let empty_i: [i32; 0] = [];
    let lp = PushLp { n, m, starts: &[], lengths: &empty_i, rows: &empty_i, values: &[], cost: &[], lb: &lb, ub: &ub };
    let par = PushParams { h: tau, piv_tol: f64::NAN, inf, fixed: f64::NAN };
    let mut st = PushState { vstat, header, xs, basic, width, marks: vec![], rhs: vec![], violations: 0,
                             iterations: 0.0, enter: -1, leave: -1, direction: 0, leave_col: -1, request: 0,
                             theta: f64::NAN, pivot: f64::NAN };
    let e = InstallEntry { first: first != 0, attempt, model_status, candidates: &candidates, interval, env_2ba0 };
    let r = push_install(&lp, &par, &mut st, &e, &mut || condition);
    format!("leave;retry {};rem {};pinf {};viol {};cond {};vstat {};xs {}", r.retry as i32, r.remaining, hex(r.pinf),
            r.violations, hex(r.condition), ints(&st.vstat), hexs(&st.xs))
}

#[test]
fn install_unit_trace() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let text = std::fs::read_to_string(format!("{root}/bench/oracle/m3_push_install_rust/unit.txt"))
        .expect("missing bench/oracle/m3_push_install_rust/unit.txt (tools/push_install_rust.py --write)");
    let mut lines = text.lines();
    let out: Out = Rc::new(RefCell::new(vec![lines.next().unwrap().to_string()]));
    let _dump = Dump(out.clone(), format!("{root}/target/m3_push_install_unit"));
    for l in lines {
        let mut t = Tok(l.split_whitespace());
        assert_eq!(t.s(), "case");
        let (name, call) = (t.s().to_string(), t.s().to_string());
        out.borrow_mut().push(format!("case;{name} {call}"));
        let line = run(&mut t);
        out.borrow_mut().push(line);
    }
}
