//! M3-04c-1d replay: `push_init` on the M3-04c-1b oracle calls (natural, first=0 and synthetic edits) from the
//! push:enter state plus the values outside it (param_1, S+0x120, S+0x3c8, the environment flag, the `12f282`
//! outputs; for first=0 the dense FTRAN output). It stops before the first supply; the comparison is
//! `tools/push_init_rust.py --compare`. Contract: docs/m3/primal_push_init_ext_rust.md.
//! M3-04c-1c: with a `cond` record the `12f282` outputs are not injected; `116fd7` runs on the Rust factor
//! (`crossover_condition`) and its internal boundaries are written as `cnorm`/`cft`/`cest`/`cbt`/`cres` lines
//! (`tools/push_cond_rust.py --compare`, contract docs/m3/primal_push_condition.md).
//! M3-04f-1: with an `inst` record the call continues into `push_loop` (the serial backend) and `push_install`
//! (condition from `116fd7` on the factor after the loop); the install entry, its `116fd7` lines (prefixed `i`) and
//! the push result are written (`tools/push_install_rust.py --compare`, contract docs/m3/primal_push_install.md).
mod loop_io;

use loop_io::*;
use rsolver_lp::crossover::{PushLp, PushParams, PushState};
use rsolver_lp::crossover_backend::{SerialBackend, SerialStart};
use rsolver_lp::crossover_condition::{CondEntry, CondEvent, CondVec, Phys};
use rsolver_lp::crossover_init::*;
use rsolver_lp::crossover_install::{push_install, InstallEntry};
use rsolver_lp::crossover_loop::*;
use rsolver_lp::crossover_refactor::{Alloc, Matrix};
use rsolver_lp::lu::BlockState;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

fn opt_hex(v: Option<f64>) -> String {
    v.map_or("none".into(), hex)
}

/// The Rust backend (or, for first=0, the captured FTRAN output) with the captured condition outputs; writes the
/// canonical lines at the original's capture points.
struct Ext {
    serial: Option<SerialStart>,
    tape: VecDeque<Vec<f64>>,
    conditions: VecDeque<f64>,
    /// M3-04c-1c: `116fd7` entries of the Rust estimate (None: the output tape).
    entries: Option<VecDeque<CondEntry>>,
    /// S+0x3c8 and, without a factor, F+0x188.
    s3c8: i32,
    f188: i32,
    out: Out,
}

impl Ext {
    fn stats(&self) -> Option<FactorStats> {
        self.serial.as_ref().and_then(|s| s.backend.as_ref()).map(|b| b.stats())
    }
    fn limit(&self) -> i32 {
        self.stats().map_or(self.f188, |s| s.limit)
    }
    fn line(&self, s: String) {
        self.out.borrow_mut().push(s);
    }
}

impl PushInitBackend for Ext {
    fn initial_factor(&mut self, ctl: &mut FactorCtl, st: &PushState) -> (i32, Option<Renumber>) {
        let (ret, r) = self.serial.as_mut().expect("first push without a factor input").initial_factor(ctl, st);
        let mut after = st.clone();
        if let Some(r) = &r {
            renumber(&mut after, r);
        }
        self.line(format!("lu;ret {ret};hdr {};vstat {}", ints(&after.header), ints(&after.vstat)));
        (ret, r)
    }
    fn rhs_ftran(&mut self, rhs: &[f64], raw: &mut [f64]) {
        self.line(format!("ftin;v {}", hexs(rhs)));
        match self.serial.as_mut() {
            Some(s) => s.rhs_ftran(rhs, raw),
            None => raw.copy_from_slice(&self.tape.pop_front().expect("FTRAN tape exhausted")),
        }
        self.line(format!("ftout;v {}", hexs(raw)));
    }
    fn tighten(&mut self, markowitz: f64, limit: Option<i32>) {
        self.serial.as_mut().expect("tightening without a factor").tighten(markowitz, limit);
        if let Some(l) = limit {
            self.s3c8 = l;
        }
    }
    fn condition(&mut self, _before: Option<f64>) -> Option<f64> {
        let Some(entries) = self.entries.as_mut() else {
            return Some(self.conditions.pop_front().expect("12f282 output tape exhausted"));
        };
        let e = entries.pop_front().expect("116fd7 entry tape exhausted");
        let out = self.out.clone();
        let b = self.serial.as_mut().and_then(|s| s.backend.as_mut()).expect("116fd7 without a factor");
        let mut lines = Vec::new();
        let c = b.condition(e, &mut |ev| lines.push(cond_line(ev)));
        out.borrow_mut().extend(lines);
        Some(c)
    }
    fn observe(&mut self, ev: InitEvent<'_>) {
        let s = match ev {
            InitEvent::Residual(v) => format!("res;v {}", hexs(v)),
            InitEvent::Strengthen { markowitz, residual } => format!("mk;new {};resid {}", hex(markowitz), hex(residual)),
            InitEvent::Condition { pinf, violations, before, markowitz, numeric } => format!(
                "cond;pinf {};viol {violations};before {};s390 {};s3c8 {};f188 {};numeric {numeric}",
                hex(pinf), opt_hex(before), hex(markowitz), self.s3c8, self.limit()),
            InitEvent::AdjustGate { adjusted, candidates, counts, tau, pinf } => {
                let c = counts.expect("adjustment gate without param_1");
                format!("gate;adj {};cand {candidates};c01 {};c001 {};c0001 {};tau {};pinf {}", adjusted as i32, c[2],
                        c[1], c[0], opt_hex(tau), hex(pinf))
            }
        };
        self.line(s);
    }
}

fn ph(p: Phys) -> &'static str {
    match p {
        Phys::X => "X",
        Phys::Y => "Y",
    }
}

fn cvec(v: &CondVec) -> String {
    format!("{} | {} | {}", v.count, ints(&v.idx), hexs(&v.val))
}

fn cond_line(ev: CondEvent<'_>) -> String {
    match ev {
        CondEvent::Norm { norm, header } => format!("cnorm;norm {};hdr {}", hex(norm), ints(header)),
        CondEvent::Ftran { k, input, output, x, y } => {
            format!("cft;k {k};in {};out {};x {};y {}", cvec(input), cvec(output), ph(x), ph(y))
        }
        CondEvent::Estimate { k, est } => format!("cest;k {k};est {}", hex(est)),
        CondEvent::Btran { k, input, output, x, y } => {
            format!("cbt;k {k};in {};out {};x {};y {}", hexs(input), cvec(output), ph(x), ph(y))
        }
        CondEvent::Done { stop, iterations, condition } => {
            format!("cres;stop {stop:?};iter {iterations};cond {}", hex(condition))
        }
    }
}

fn run_call(case: &[&str], prev: Option<SerialBackend>, out: &Out) -> Option<SerialBackend> {
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
    let mut t = rec(case, 4, "ext");
    let (tau_in, prev_pinf) = (t.f(), t.f());
    let (s3c8, env_2c54, f188): (i32, i32, i32) = (t.i(), t.i(), t.i());
    let condition = t.f();
    let k: usize = t.i();
    let conditions: VecDeque<f64> = t.fs(k).into();
    t.done();
    let mut t = rec(case, 5, "lu");
    let mut tape = VecDeque::new();
    let serial = match t.s() {
        "new" => {
            let (zero, tol) = (t.f(), t.f());
            let alloc = Alloc { x158: t.i(), pool_mod16: [t.i(), t.i()], x188: t.i(), x1f8: t.f() };
            let block = BlockState { x2c0: t.i(), x4e0: t.i(), x4e4: t.i() };
            let interval: i32 = t.i();
            let matrix = Matrix { m, n, beg: starts.clone(), len: lengths.clone(), ind: rows.clone(),
                                  val: values.clone() };
            Some(SerialStart::new(matrix, zero, tol, alloc, block, interval, drop))
        }
        "carry" => {
            // the caller wrote S+0x390 before calling the push again; the factor object stays on S
            let mut b = prev.expect("retry call without the previous call's factor");
            b.tol = t.f();
            Some(SerialStart::existing(b))
        }
        "tape" => {
            let k: usize = t.i();
            for _ in 0..k {
                tape.push_back(t.fs(m));
            }
            None
        }
        other => panic!("lu kind {other}"),
    };
    t.done();
    let entries = (case.len() > 6).then(|| {
        let mut t = rec(case, 6, "cond");
        let k: usize = t.i();
        let v: VecDeque<CondEntry> = (0..k)
            .map(|_| CondEntry { f0c: t.i(), a_val_mod16: t.i(), x_mod16: t.i(), y_mod16: t.i() })
            .collect();
        t.done();
        v
    });
    let install = (case.len() > 7).then(|| {
        let mut t = rec(case, 7, "inst");
        let (env_2ba0, interval): (i32, i32) = (t.i(), t.i());
        let e = CondEntry { f0c: t.i(), a_val_mod16: t.i(), x_mod16: t.i(), y_mod16: t.i() };
        t.done();
        (env_2ba0, interval, e)
    });

    let lp = PushLp { n, m, starts: &starts, lengths: &lengths, rows: &rows, values: &values, cost: &cost, lb: &lb,
                      ub: &ub };
    let mut backend = Ext { serial, tape, conditions, entries, s3c8, f188, out: out.clone() };
    let entry = InitEntry { rhs: &rhs, vstat, header, xs, iterations, request, numeric, failure, status, enter, leave,
                            direction: 0, markowitz, first: first != 0, attempt, model_status,
                            tau_in: Some(tau_in), prev_pinf: Some(prev_pinf), condition: Some(condition),
                            tighten: Some(TightenInput { update_limit: s3c8, env_2c54 }) };
    let res = push_init(&lp, &par, entry, &mut backend);
    assert!(backend.conditions.is_empty() && backend.tape.is_empty(), "unused condition or FTRAN tape");
    assert!(backend.entries.as_ref().is_none_or(|e| e.is_empty()), "unused 116fd7 entries");
    let s = &res.state;
    let r = &s.round;
    match res.exit {
        InitExit::Retry => backend.line(format!(
            "retry;numeric {};s390 {};s3c8 {};f188 {};s120 {};cond {}", r.numeric, hex(res.markowitz), backend.s3c8,
            backend.limit(), hex(prev_pinf), opt_hex(res.condition))),
        InitExit::Loop => {
            let fs = backend.stats().map_or("none".to_string(), |f| {
                format!("{} {} {} {}", f.updates, f.limit, f.l_nnz, f.u_nnz)
            });
            backend.line(format!(
                "start;cand {};sup {};smarks {};pos {};dfr {};fail {};csum {};succ {};vstat {};header {};xs {};rhs {};\
                 basic {};width {};marks {};pinf {};iter {};numeric {};failure {};request {};status {};s390 {};\
                 factor {fs};retry {};nlimit {};adj {}",
                ints(&r.candidates), r.support, opt(&r.marks), r.position, r.deferred, ints(&r.failures),
                hex(r.count_sum), r.successes, ints(&s.st.vstat), ints(&s.st.header), hexs(&s.st.xs), hexs(&s.st.rhs),
                hexs(&s.st.basic), hexs(&s.st.width), marks(&s.st.marks), hex(s.pinf), hex(s.st.iterations), r.numeric,
                r.failure, r.request, r.status, hex(res.markowitz), r.retry, res.numeric_limit, res.adjusted as i32));
        }
        other => backend.line(format!("exit;{other:?}")),
    }
    let mut serial = backend.serial.take().and_then(|s| s.backend);
    if let Some((env_2ba0, interval, centry)) = install {
        if matches!(res.exit, InitExit::Loop | InitExit::End) {
            let sb = serial.as_mut().expect("install chain without a Rust factor");
            let mut s = res.state;
            let exit = if res.exit == InitExit::Loop {
                let cfg = LoopConfig { attempt, model_status, numeric_limit: res.numeric_limit, drop };
                push_loop(&lp, &par, &cfg, &mut s, sb, &mut |_: LoopEvent<'_>| {})
            } else {
                LoopExit::End
            };
            if exit == LoopExit::End {
                // every list entry is valid at the end (exhaustion: all deferred; otherwise empty)
                let cands = s.round.candidates.clone();
                backend.line(format!("inst;cnt {};cand {};hdr {};basic {};width {};vstat {};xs {};pinf {};iter {}",
                                     cands.len(), ints(&cands), ints(&s.st.header), hexs(&s.st.basic),
                                     hexs(&s.st.width), ints(&s.st.vstat), hexs(&s.st.xs), hex(s.pinf),
                                     hex(s.st.iterations)));
                let mut lines = Vec::new();
                let e = InstallEntry { first: first != 0, attempt, model_status, candidates: &cands, interval, env_2ba0 };
                let r = push_install(&lp, &par, &mut s.st, &e, &mut || {
                    sb.condition(centry, &mut |ev| lines.push(format!("i{}", cond_line(ev))))
                });
                out.borrow_mut().extend(lines);
                backend.line(format!("leave;retry {};rem {};pinf {};viol {};cond {};vstat {};xs {}", r.retry as i32,
                                     r.remaining, hex(r.pinf), r.violations, hex(r.condition), ints(&s.st.vstat),
                                     hexs(&s.st.xs)));
            } else {
                backend.line(format!("exit;{exit:?}"));
            }
        }
    }
    serial
}

#[test]
fn init_ext_trace() {
    replay("m3_push_init_ext_rust", "tools/push_init_rust.py --write");
}

#[test]
fn init_cond_trace() {
    replay("m3_push_cond_rust", "tools/push_cond_rust.py --write");
}

#[test]
fn init_install_trace() {
    replay("m3_push_install_rust", "tools/push_install_rust.py --write");
}

/// Records of one call: from a `case` line up to the next.
fn records<'a>(rest: &[&'a str]) -> Vec<Vec<&'a str>> {
    let mut out: Vec<Vec<&str>> = Vec::new();
    for &l in rest {
        if l.starts_with("case ") {
            out.push(Vec::new());
        }
        out.last_mut().expect("input does not start with a case record").push(l);
    }
    out
}

fn replay(dir: &str, writer: &str) {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let Ok(text) = std::fs::read_to_string(format!("{root}/bench/oracle/{dir}/input.txt")) else {
        panic!("missing bench/oracle/{dir}/input.txt ({writer})");
    };
    let mut lines = text.lines();
    let out: Out = Rc::new(RefCell::new(vec![lines.next().unwrap().to_string()]));
    let _dump = Dump(out.clone(), format!("{root}/target/{dir}"));
    let rest: Vec<&str> = lines.collect();
    let mut failed = Vec::new();
    let mut prev: Option<(String, Option<SerialBackend>)> = None;
    for case in records(&rest) {
        let case = &case[..];
        let mut t = rec(case, 0, "case");
        let name = t.s().to_string();
        let call: i32 = t.i();
        out.borrow_mut().push(format!("case;{name} {call}"));
        let carried = match prev.take() {
            Some((p, b)) if p == name => b,
            _ => None,
        };
        let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_call(case, carried, &out)));
        match run {
            Ok(b) => prev = Some((name, b)),
            Err(e) => {
                let msg = e.downcast_ref::<String>().cloned()
                    .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                out.borrow_mut().push(format!("error;msg {}", msg.replace([';', '\n'], " ")));
                failed.push(format!("{name} {call}: {msg}"));
            }
        }
    }
    assert!(failed.is_empty(), "{failed:?}");
}
