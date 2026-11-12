//! M3-04c-22 replay: runs `push_loop` from the first pass preparation of each M3-04c-21e push capture with a tape
//! backend (supplier FTRAN raw, support BTRAN raw, `05b257` results and renumbering, RHS FTRAN raw) and writes the
//! canonical event trace. The tape backend records the inputs the loop hands it. No original state is injected after
//! the start; the comparison is `tools/push_loop_rust.py --compare`.
use rsolver_lp::crossover::{PushLp, PushParams, PushState, PushStep};
use rsolver_lp::crossover_loop::*;
use rsolver_lp::crossover_round::{RoundExit, RoundState};
use rsolver_lp::crossover_supplier::{PushColumn, SupplierBuffers};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::str::SplitWhitespace;

fn f(token: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(token, 16).unwrap().swap_bytes()) // captures store little-endian bytes
}
fn hex(v: f64) -> String {
    format!("{:016x}", v.to_bits().swap_bytes())
}
fn hexs(v: &[f64]) -> String {
    v.iter().map(|&x| hex(x)).collect::<Vec<_>>().join(" ")
}
fn ints(v: &[i32]) -> String {
    v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" ")
}
fn marks(v: &[u8]) -> String {
    v.iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join(" ")
}
fn opt(v: &Option<Vec<i32>>) -> String {
    v.as_ref().map_or("none".into(), |x| ints(x))
}

struct Tok<'a>(SplitWhitespace<'a>);
impl Tok<'_> {
    fn s(&mut self) -> &str {
        self.0.next().unwrap()
    }
    fn i<T: std::str::FromStr>(&mut self) -> T {
        self.s().parse().ok().unwrap()
    }
    fn f(&mut self) -> f64 {
        f(self.s())
    }
    fn is(&mut self, n: usize) -> Vec<i32> {
        (0..n).map(|_| self.i()).collect()
    }
    fn fs(&mut self, n: usize) -> Vec<f64> {
        (0..n).map(|_| self.f()).collect()
    }
    fn col(&mut self, m: usize) -> PushColumn {
        let count: i32 = self.i();
        if count < 0 {
            PushColumn { count, indices: vec![], values: self.fs(m) }
        } else {
            let n = count as usize;
            PushColumn { count, indices: self.is(n), values: self.fs(n) }
        }
    }
}

struct FactorTape {
    ret: i32,
    numeric: i32,
    failure: i32,
    status: i32,
    stats: FactorStats,
    renumber: Option<(Vec<i32>, Vec<i32>)>,
}

type Out = Rc<RefCell<Vec<String>>>;

struct Tape {
    m: usize,
    ftran: VecDeque<PushColumn>,
    btran: VecDeque<PushColumn>,
    factor: VecDeque<FactorTape>,
    recompute: VecDeque<Vec<f64>>,
    stats: FactorStats,
    out: Out,
}

fn col_fields(prefix: &str, c: &PushColumn) -> String {
    if c.count < 0 {
        format!(";{prefix}count {};{prefix}val {}", c.count, hexs(&c.values))
    } else {
        format!(";{prefix}count {};{prefix}idx {};{prefix}val {}", c.count, ints(&c.indices), hexs(&c.values))
    }
}

impl PushLoopBackend for Tape {
    fn ftran(&mut self, input: &PushColumn, output: &mut PushColumn, spike: Option<&mut PushColumn>) {
        let mut line = "ftran".to_string() + &col_fields("", input);
        line += &format!(";spike {}", spike.is_some() as i32);
        self.out.borrow_mut().push(line);
        *output = self.ftran.pop_front().expect("FTRAN tape exhausted");
    }
    fn btran(&mut self, _input: &PushColumn) -> PushColumn {
        self.btran.pop_front().expect("BTRAN tape exhausted")
    }
    fn stats(&self) -> FactorStats {
        self.stats
    }
    fn factor(&mut self, _site: FactorSite, ctl: &mut FactorCtl, _header: &[i32], _leave: i32)
              -> (i32, Option<Renumber>) {
        let t = self.factor.pop_front().expect("factor tape exhausted");
        ctl.numeric = t.numeric;
        ctl.failure = t.failure;
        ctl.status = t.status;
        self.stats = t.stats;
        (t.ret, t.renumber.map(|(o, n)| Renumber { old_of_new: o, new_of_old: n }))
    }
    fn quality(&mut self, _: &mut FactorCtl, _: i32, _: i32) -> i32 {
        panic!("quality helper is not on the natural tape")
    }
    fn recompute_ftran(&mut self, rhs: &[f64], raw: &mut [f64]) {
        self.out.borrow_mut().push(format!("rin;rhs {}", hexs(rhs)));
        let r = self.recompute.pop_front().expect("recompute tape exhausted");
        assert_eq!(r.len(), self.m);
        raw.copy_from_slice(&r);
    }
}

fn state_fields(st: &PushState, prefix: &str) -> String {
    format!(";{p}vstat {};{p}header {};{p}basic {};{p}width {};{p}marks {};{p}iter {}", ints(&st.vstat),
            ints(&st.header), hexs(&st.basic), hexs(&st.width), marks(&st.marks), hex(st.iterations), p = prefix)
}

fn round_fields(r: &RoundState, with_marks: bool, with_request: bool) -> String {
    let mut s = format!(";cand {};pos {};dfr {};sup {}", ints(&r.candidates), r.position, r.deferred, r.support);
    if with_marks {
        s += &format!(";marks {}", opt(&r.marks));
    }
    if with_request {
        s += &format!(";req {}", r.request);
    }
    s
}

fn site(s: FactorSite) -> &'static str {
    match s {
        FactorSite::Pivot => "pivot",
        FactorSite::Round => "round",
    }
}

#[test]
fn loop_trace() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let text = std::fs::read_to_string(format!("{root}/bench/oracle/m3_push_loop_rust/input.txt")).unwrap();
    let mut lines = text.lines();
    let out: Out = Rc::new(RefCell::new(vec![lines.next().unwrap().to_string()]));
    // Written on drop, also when a tape runs out, so a divergence still leaves a trace to compare.
    struct Dump(Out, String);
    impl Drop for Dump {
        fn drop(&mut self) {
            std::fs::create_dir_all(&self.1).unwrap();
            std::fs::write(format!("{}/trace.txt", self.1), self.0.borrow().join("\n") + "\n").unwrap();
        }
    }
    let _dump = Dump(out.clone(), format!("{root}/target/m3_push_loop_rust"));
    let rest: Vec<&str> = lines.collect();
    for case in rest.chunks(10) {
        let rec = |k: usize, tag: &str| {
            let mut t = Tok(case[k].split_whitespace());
            assert_eq!(t.s(), tag);
            t
        };
        let name = rec(0, "case").s().to_string();
        out.borrow_mut().push(format!("case;{name}"));
        let mut t = rec(1, "model");
        let (n, m, nnz): (usize, usize, usize) = (t.i(), t.i(), t.i());
        let starts: Vec<i64> = (0..n).map(|_| t.i()).collect();
        let lengths = t.is(n);
        let rows = t.is(nnz);
        let values = t.fs(nnz);
        let (cost, lb, ub) = (t.fs(n + m), t.fs(n + m), t.fs(n + m));
        let mut t = rec(2, "cfg");
        let (h, piv_tol, inf, fixed, drop) = (t.f(), t.f(), t.f(), t.f(), t.f());
        let cfg = LoopConfig { model_status: t.i(), attempt: t.i(), numeric_limit: t.i(), drop };
        let period: i32 = t.i();
        let mut t = rec(3, "state");
        let vstat = t.is(n + m);
        let header = t.is(m);
        let xs = t.fs(n + m);
        let (basic, width) = (t.fs(m), t.fs(m));
        let mk: Vec<u8> = (0..m).map(|_| u8::from_str_radix(t.s(), 16).unwrap()).collect();
        let rhs = t.fs(m);
        let iterations = t.f();
        let request: i32 = t.i();
        let pinf = t.f();
        let (enter, leave, direction): (i32, i32, i32) = (t.i(), t.i(), t.i());
        let violations = mk.iter().filter(|&&c| c == b'B' || c == b'A').count() as i32;
        // theta/pivot start as NaN: a value leaking from before the loop would show in the trace.
        let st = PushState { vstat, header, xs, basic, width, marks: mk, rhs, violations, iterations, enter, leave,
                             direction, leave_col: -1, request, theta: f64::NAN, pivot: f64::NAN };
        let mut t = rec(4, "round");
        let total: usize = t.i();
        let candidates = t.is(total);
        let (position, deferred): (usize, usize) = (t.i(), t.i());
        let failures = t.is(n + m);
        let support: i32 = t.i();
        let marks_v = if t.i::<u8>() != 0 { Some(t.is(m)) } else { None };
        let count_sum = t.f();
        let successes: i32 = t.i();
        let (status, numeric, failure, retry): (i32, i32, i32, i32) = (t.i(), t.i(), t.i(), t.i());
        let round = RoundState { m, candidates, position, deferred, failures, marks: marks_v, support, approved: None,
                                 count_sum, successes, request, status, numeric, failure, retry };
        let stat = |t: &mut Tok<'_>| FactorStats { updates: t.i(), limit: t.i(), interval: t.i(),
                                                   column_work: t.i(), eta_count: t.i(), l_nnz: t.i(), u_nnz: t.i() };
        let stats = stat(&mut rec(5, "stats"));
        let mut t = rec(6, "ftran");
        let ftran = (0..t.i::<usize>()).map(|_| t.col(m)).collect();
        let mut t = rec(7, "btran");
        let btran = (0..t.i::<usize>()).map(|_| t.col(m)).collect();
        let mut t = rec(8, "factor");
        let factor = (0..t.i::<usize>())
            .map(|_| {
                let (ret, numeric, failure, status) = (t.i(), t.i(), t.i(), t.i());
                let stats = stat(&mut t);
                let renumber = if t.i::<u8>() != 0 { Some((t.is(m), t.is(m))) } else { None };
                FactorTape { ret, numeric, failure, status, stats, renumber }
            })
            .collect();
        let mut t = rec(9, "recompute");
        let recompute = (0..t.i::<usize>()).map(|_| t.fs(m)).collect();
        let mut tape = Tape { m, ftran, btran, factor, recompute, stats, out: out.clone() };

        let lp = PushLp { n, m, starts: &starts, lengths: &lengths, rows: &rows, values: &values, cost: &cost,
                          lb: &lb, ub: &ub };
        let par = PushParams { h, piv_tol, inf, fixed };
        let mut s = PushLoop { st, round, pinf, period, buffers: SupplierBuffers::default() };
        let sink = out.clone();
        let mut step = String::new();
        let mut trace = |ev: LoopEvent<'_>| {
            let mut o = sink.borrow_mut();
            match ev {
                LoopEvent::Leave(st, pinf) => o.push(format!("leave{};xs {};rhs {};pinf {}", state_fields(st, ""),
                                                             hexs(&st.xs), hexs(&st.rhs), hex(pinf))),
                LoopEvent::Pass(r) => o.push(format!("pass{}", round_fields(r, true, true))),
                LoopEvent::Supplied { ret, round: r, alpha } => {
                    let smarks = if r.support > 0 { opt(&r.marks) } else { "none".into() };
                    let mut l = format!("supply;ret {ret};pos {};dfr {};cand {};fail {};sup {};smarks {smarks};\
                                         csum {};succ {}", r.position, r.deferred, ints(&r.candidates),
                                        ints(&r.failures), r.support, hex(r.count_sum), r.successes);
                    if ret >= 0 {
                        l += &format!(";acount {};aidx {};aval {}", alpha.count, ints(&alpha.indices),
                                      hexs(&alpha.values));
                    }
                    o.push(l);
                }
                LoopEvent::Picked { pick, st, alpha } => {
                    let first: Vec<f64> =
                        if pick.second.is_some() { alpha.iter().map(|v| -v).collect() } else { alpha.to_vec() };
                    let seen = |k: usize| format!("{} {} {}", pick.seen[k].0, pick.seen[k].1, pick.seen[k].2);
                    let ratio = |r: (i32, f64, f64)| format!("{} {} {}", r.0, hex(r.1), hex(r.2));
                    step = format!("step;path {};seen0 {};alpha1 {};ratio1 {}",
                                   if pick.violated { "V" } else { "F" }, seen(0), hexs(&first), ratio(pick.first));
                    if let Some(r2) = pick.second {
                        step += &format!(";seen1 {};alpha2 {};ratio2 {}", seen(1), hexs(alpha), ratio(r2));
                    }
                    let kind = match pick.step {
                        PushStep::Flip | PushStep::Reject => "flip",
                        PushStep::Pivot => "pivot",
                        PushStep::Defer => "defer",
                    };
                    step += &format!(";frame {} {} {};step {kind} {}", hex(pick.entry.r), hex(pick.entry.dl),
                                     hex(pick.entry.du), hex(st.theta));
                }
                LoopEvent::Applied(st) => step += &state_fields(st, "a."),
                LoopEvent::Stepped { step: kind, st, round: r } => {
                    if kind == PushStep::Defer {
                        step += &state_fields(st, "a.");
                        step += &format!(";dfr {};cand {};failj {}", r.deferred, ints(&r.candidates),
                                         r.failures[st.enter as usize]);
                    } else {
                        step += &format!(";approved {}", r.approved.unwrap());
                    }
                    step += &format!(";sel {} {} {}", st.enter, st.leave, st.direction);
                    o.push(std::mem::take(&mut step));
                }
                LoopEvent::Btran { leave, raw, exit, round: r } => {
                    let mut l = format!("btran;leave {leave}") + &col_fields("r", raw);
                    if *exit == RoundExit::Continue {
                        l += &round_fields(r, true, false);
                    }
                    o.push(l);
                }
                LoopEvent::Select(d) => o.push(format!("select;req {};call {}", d.request, d.call as i32)),
                LoopEvent::Factor { site: s_, request, leave, header, ret, st } => o.push(format!(
                    "fcall;site {};req {request};leave {leave};hdr {};ret {ret};hdr2 {};vstat {};leave2 {}",
                    site(s_), ints(header), ints(&st.header), ints(&st.vstat), st.leave)),
                LoopEvent::Recompute { site: s_, exit, st, pinf } => o.push(format!(
                    "recompute;site {};hdr {};basic {};width {};marks {};pinf {};viol {};retry {};exit {}",
                    site(s_), ints(&st.header), hexs(&st.basic), hexs(&st.width), marks(&st.marks), hex(pinf),
                    st.violations, if matches!(exit, rsolver_lp::crossover_recompute::RecomputeExit::Retry) { 1 }
                    else { 0 }, if matches!(exit, rsolver_lp::crossover_recompute::RecomputeExit::Retry) {
                        "retry" } else { "normal" })),
                LoopEvent::Clear { period } => o.push(format!("clear;period {period}")),
                LoopEvent::Exhausted { position, deferred, total, support, approved, end } => o.push(format!(
                    "exhausted;pos {position};dfr {deferred};total {total};sup {support};appr {};end {}",
                    approved.map_or("none".into(), |a| a.to_string()), end as i32)),
                LoopEvent::Round(r) => o.push(format!("round{}", round_fields(r, true, true))),
                LoopEvent::End(r) => o.push(format!("end{}", round_fields(r, true, true))),
            }
        };
        let exit = push_loop(&lp, &par, &cfg, &mut s, &mut tape, &mut trace);
        assert_eq!(exit, LoopExit::End, "{name}");
        assert!(tape.ftran.is_empty() && tape.btran.is_empty() && tape.factor.is_empty() && tape.recompute.is_empty(),
                "{name}: tape left over");
    }
}
