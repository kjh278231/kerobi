//! M3-04f-3 cleanup entry replay: from the original's `sel:enter` state, Rust runs the retry reset, the cleanup
//! choice (reduced costs of the dual check injected from `sel:dual`) and, on the basis call site, the `0325b4`
//! install with its own status array. The comparison is `tools/xo_cleanup_rust.py --compare`. Contract:
//! docs/m3/crossover_cleanup_rust.md.
mod loop_io;

use loop_io::*;
use rsolver_lp::crossover_cleanup::*;
use std::cell::RefCell;
use std::rc::Rc;

fn run(lines: &mut std::str::Lines<'_>, out: &Out) {
    let mut t = Tok(lines.next().unwrap().split_whitespace());
    assert_eq!(t.s(), "sel");
    let (n, m): (usize, usize) = (t.i(), t.i());
    let (crossover, retry, remaining): (i32, i32, i32) = (t.i(), t.i(), t.i());
    let (s2d8, pinf) = (t.f(), t.f());
    let precision: i32 = t.i();
    let opt_tol = t.f();
    let (restarts, s178, s184): (i32, i32, i32) = (t.i(), t.i(), t.i());
    let mut vstat = t.is(n + m);
    let x = t.fs(n);
    t.done();

    let mut t = Tok(lines.next().unwrap().split_whitespace());
    assert_eq!(t.s(), "reset");
    let (fixtol, feastol, inf, perturb) = (t.f(), t.f(), t.f(), t.f());
    let (dlb, dub) = (t.fs(n), t.fs(n));
    t.done();

    let mut t = Tok(lines.next().unwrap().split_whitespace());
    assert_eq!(t.s(), "dual");
    let k: usize = t.i();
    let d = t.fs(k);
    t.done();

    let mut t = Tok(lines.next().unwrap().split_whitespace());
    assert_eq!(t.s(), "inst");
    let has: i32 = t.i();
    let inst = (has != 0).then(|| {
        let (tau, fixtol, opt_tol) = (t.f(), t.f(), t.f());
        let flag: i32 = t.i();
        let flags: Option<Vec<u8>> = (flag == 1).then(|| (0..n).map(|_| t.i::<u8>()).collect());
        let (lb, ub) = (t.fs(n + m), t.fs(n + m));
        let (cost, lengths) = (t.fs(n), t.is(n));
        let slack = t.fs(m);
        (tau, fixtol, opt_tol, flags, lb, ub, cost, lengths, slack)
    });
    t.done();

    // retry reset (C 1363–1521)
    let (mut remaining, mut s184) = (remaining, s184);
    if retry != 0 {
        let r = retry_reset(&ResetInput { n, m, lb: &dlb, ub: &dub, x: &x, fixtol, feastol, inf });
        vstat = r.vstat;
        remaining = 0;
        s184 = 0;
        out.borrow_mut().push(format!("reset;rem {remaining};s184 {s184};perturb {};vstat {}", hex(perturb), ints(&vstat)));
    } else {
        out.borrow_mut().push(format!("reset;rem {remaining};s184 {s184};vstat {}", ints(&vstat)));
    }

    // choice (C 1523–1690)
    let sel = SelectInput { crossover, retry, remaining, s2d8, pinf, precision, opt_tol, restarts, s178 };
    let s = select(&sel, &vstat, &mut || d.clone());
    let call = match s.call {
        Cleanup::Dual => "dual",
        Cleanup::Primal => "primal_nobasis",
        Cleanup::PrimalBasis => "primal_basis",
    };
    out.borrow_mut().push(format!("select;r14 {};checked {};call {call};s180 {};s178 {}", s.dual as i32,
                                  s.checked as i32, s.s180, s.s178));

    // 0cda5b → 0325b4 (basis call site only)
    if s.call != Cleanup::PrimalBasis {
        return;
    }
    let Some((tau, fixtol, opt_tol, flags, lb, ub, cost, lengths, slack)) = inst else {
        out.borrow_mut().push("install;inputs-missing".into());
        return;
    };
    let (vb, cb) = (vstat[..n].to_vec(), vstat[n..].to_vec());
    let e = InstallInput { n, m, vbasis: Some(&vb), cbasis: Some(&cb), x: Some(&x), slack: Some(&slack), lb: &lb,
                           ub: &ub, cost: &cost, lengths: &lengths, tau, fixtol, opt_tol, flipped: flags.as_deref() };
    let r = install_basis(&e, &mut vstat).unwrap();
    let (idx, vals): (Vec<i32>, Vec<f64>) = r.list.iter().copied().unzip();
    out.borrow_mut().push(format!("install;vstat {};s450 {};list {};vals {};x {};slack {}", ints(&r.vstat),
                                  r.list.len(), ints(&idx), hexs(&vals), hexs(&x), hexs(&slack)));
}

#[test]
fn cleanup_entry_trace() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let text = std::fs::read_to_string(format!("{root}/bench/oracle/m3_xo_cleanup_rust/input.txt"))
        .expect("missing bench/oracle/m3_xo_cleanup_rust/input.txt (tools/xo_cleanup_rust.py --write)");
    let mut lines = text.lines();
    let out: Out = Rc::new(RefCell::new(vec![lines.next().unwrap().to_string()]));
    let _dump = Dump(out.clone(), format!("{root}/target/m3_xo_cleanup_rust"));
    while let Some(l) = lines.next() {
        let mut t = Tok(l.split_whitespace());
        assert_eq!(t.s(), "case");
        let name = t.s().to_string();
        out.borrow_mut().push(format!("case;{name}"));
        run(&mut lines, &out);
    }
}
