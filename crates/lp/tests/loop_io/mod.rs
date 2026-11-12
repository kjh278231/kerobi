//! Input parsing and canonical trace lines shared by the push loop replays (M3-04c-22 tape, M3-04c-24 Rust backend).
#![allow(dead_code)]
use rsolver_lp::crossover::{PushLp, PushParams, PushState, PushStep};
use rsolver_lp::crossover_loop::*;
use rsolver_lp::crossover_recompute::RecomputeExit;
use rsolver_lp::crossover_round::{RoundExit, RoundState};
use rsolver_lp::crossover_supplier::PushColumn;
use std::cell::RefCell;
use std::rc::Rc;
use std::str::SplitWhitespace;

pub fn f(token: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(token, 16).unwrap().swap_bytes()) // captures store little-endian bytes
}
pub fn hex(v: f64) -> String {
    format!("{:016x}", v.to_bits().swap_bytes())
}
pub fn hexs(v: &[f64]) -> String {
    v.iter().map(|&x| hex(x)).collect::<Vec<_>>().join(" ")
}
pub fn ints(v: &[i32]) -> String {
    v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" ")
}
pub fn marks(v: &[u8]) -> String {
    v.iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join(" ")
}
pub fn opt(v: &Option<Vec<i32>>) -> String {
    v.as_ref().map_or("none".into(), |x| ints(x))
}

pub struct Tok<'a>(pub SplitWhitespace<'a>);
impl Tok<'_> {
    pub fn s(&mut self) -> &str {
        self.0.next().unwrap()
    }
    pub fn i<T: std::str::FromStr>(&mut self) -> T {
        self.s().parse().ok().unwrap()
    }
    pub fn f(&mut self) -> f64 {
        f(self.s())
    }
    pub fn is(&mut self, n: usize) -> Vec<i32> {
        (0..n).map(|_| self.i()).collect()
    }
    pub fn fs(&mut self, n: usize) -> Vec<f64> {
        (0..n).map(|_| self.f()).collect()
    }
    pub fn col(&mut self, m: usize) -> PushColumn {
        let count: i32 = self.i();
        if count < 0 {
            PushColumn { count, indices: vec![], values: self.fs(m) }
        } else {
            let n = count as usize;
            PushColumn { count, indices: self.is(n), values: self.fs(n) }
        }
    }
    pub fn done(&mut self) {
        assert!(self.0.next().is_none(), "trailing input");
    }
}

pub fn rec<'a>(case: &[&'a str], k: usize, tag: &str) -> Tok<'a> {
    let mut t = Tok(case[k].split_whitespace());
    assert_eq!(t.s(), tag);
    t
}

pub fn stat(t: &mut Tok<'_>) -> FactorStats {
    FactorStats { updates: t.i(), limit: t.i(), interval: t.i(), column_work: t.i(), eta_count: t.i(), l_nnz: t.i(),
                  u_nnz: t.i() }
}

pub fn stats_text(s: &FactorStats) -> String {
    format!("{} {} {} {} {} {} {}", s.updates, s.limit, s.interval, s.column_work, s.eta_count, s.l_nnz, s.u_nnz)
}

/// The model and push parameters of lines 1–2; `parse_case` also returns the initialized loop state (lines 3–4).
pub struct Case {
    pub n: usize,
    pub m: usize,
    pub starts: Vec<i64>,
    pub lengths: Vec<i32>,
    pub rows: Vec<i32>,
    pub values: Vec<f64>,
    pub cost: Vec<f64>,
    pub lb: Vec<f64>,
    pub ub: Vec<f64>,
    pub par: PushParams,
    pub cfg: LoopConfig,
}

impl Case {
    pub fn lp(&self) -> PushLp<'_> {
        PushLp { n: self.n, m: self.m, starts: &self.starts, lengths: &self.lengths, rows: &self.rows,
                 values: &self.values, cost: &self.cost, lb: &self.lb, ub: &self.ub }
    }
}

pub fn parse_case(case: &[&str]) -> (Case, PushLoop) {
    let mut t = rec(case, 1, "model");
    let (n, m, nnz): (usize, usize, usize) = (t.i(), t.i(), t.i());
    let starts: Vec<i64> = (0..n).map(|_| t.i()).collect();
    let lengths = t.is(n);
    let rows = t.is(nnz);
    let values = t.fs(nnz);
    let (cost, lb, ub) = (t.fs(n + m), t.fs(n + m), t.fs(n + m));
    t.done();
    let mut t = rec(case, 2, "cfg");
    let (h, piv_tol, inf, fixed, drop) = (t.f(), t.f(), t.f(), t.f(), t.f());
    let cfg = LoopConfig { model_status: t.i(), attempt: t.i(), numeric_limit: t.i(), drop };
    let period: i32 = t.i();
    t.done();
    let mut t = rec(case, 3, "state");
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
    t.done();
    let violations = mk.iter().filter(|&&c| c == b'B' || c == b'A').count() as i32;
    // theta/pivot start as NaN: a value leaking from before the loop would show in the trace.
    let st = PushState { vstat, header, xs, basic, width, marks: mk, rhs, violations, iterations, enter, leave,
                         direction, leave_col: -1, request, theta: f64::NAN, pivot: f64::NAN };
    let mut t = rec(case, 4, "round");
    let total: usize = t.i();
    let candidates = t.is(total);
    let (position, deferred): (usize, usize) = (t.i(), t.i());
    let failures = t.is(n + m);
    let support: i32 = t.i();
    let marks_v = if t.i::<u8>() != 0 { Some(t.is(m)) } else { None };
    let count_sum = t.f();
    let successes: i32 = t.i();
    let (status, numeric, failure, retry): (i32, i32, i32, i32) = (t.i(), t.i(), t.i(), t.i());
    t.done();
    let round = RoundState { m, candidates, position, deferred, failures, marks: marks_v, support, approved: None,
                             count_sum, successes, request, status, numeric, failure, retry };
    let start = PushLoop { st, round, pinf, period, buffers: Default::default() };
    (Case { n, m, starts, lengths, rows, values, cost, lb, ub, par: PushParams { h, piv_tol, inf, fixed }, cfg }, start)
}

pub type Out = Rc<RefCell<Vec<String>>>;

pub fn col_fields(prefix: &str, c: &PushColumn) -> String {
    if c.count < 0 {
        format!(";{prefix}count {};{prefix}val {}", c.count, hexs(&c.values))
    } else {
        format!(";{prefix}count {};{prefix}idx {};{prefix}val {}", c.count, ints(&c.indices), hexs(&c.values))
    }
}

pub fn state_fields(st: &PushState, prefix: &str) -> String {
    format!(";{p}vstat {};{p}header {};{p}basic {};{p}width {};{p}marks {};{p}iter {}", ints(&st.vstat),
            ints(&st.header), hexs(&st.basic), hexs(&st.width), marks(&st.marks), hex(st.iterations), p = prefix)
}

pub fn round_fields(r: &RoundState, with_marks: bool, with_request: bool) -> String {
    let mut s = format!(";cand {};pos {};dfr {};sup {}", ints(&r.candidates), r.position, r.deferred, r.support);
    if with_marks {
        s += &format!(";marks {}", opt(&r.marks));
    }
    if with_request {
        s += &format!(";req {}", r.request);
    }
    s
}

pub fn site(s: FactorSite) -> &'static str {
    match s {
        FactorSite::Pivot => "pivot",
        FactorSite::Round => "round",
    }
}

/// The canonical line writer for `LoopEvent`s; `with_stats` adds the factor statistics after each `05b257` call.
pub fn tracer(sink: Out, with_stats: bool) -> impl FnMut(LoopEvent<'_>) {
    let mut step = String::new();
    move |ev: LoopEvent<'_>| {
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
            LoopEvent::Factor { site: s_, request, leave, header, ret, st, stats } => {
                let mut l = format!("fcall;site {};req {request};leave {leave};hdr {};ret {ret};hdr2 {};vstat {};\
                                     leave2 {}", site(s_), ints(header), ints(&st.header), ints(&st.vstat), st.leave);
                if with_stats {
                    l += &format!(";stats {}", stats_text(&stats));
                }
                o.push(l);
            }
            LoopEvent::Recompute { site: s_, exit, st, pinf } => {
                let retry = matches!(exit, RecomputeExit::Retry);
                o.push(format!(
                    "recompute;site {};hdr {};basic {};width {};marks {};pinf {};viol {};retry {};exit {}",
                    site(s_), ints(&st.header), hexs(&st.basic), hexs(&st.width), marks(&st.marks), hex(pinf),
                    st.violations, retry as i32, if retry { "retry" } else { "normal" }))
            }
            LoopEvent::Clear { period } => o.push(format!("clear;period {period}")),
            LoopEvent::Exhausted { position, deferred, total, support, approved, end } => o.push(format!(
                "exhausted;pos {position};dfr {deferred};total {total};sup {support};appr {};end {}",
                approved.map_or("none".into(), |a| a.to_string()), end as i32)),
            LoopEvent::Round(r) => o.push(format!("round{}", round_fields(r, true, true))),
            LoopEvent::End(r) => o.push(format!("end{}", round_fields(r, true, true))),
        }
    }
}

/// Writes the collected lines to `<dir>/trace.txt` on drop, also when a replay panics.
pub struct Dump(pub Out, pub String);
impl Drop for Dump {
    fn drop(&mut self) {
        std::fs::create_dir_all(&self.1).unwrap();
        std::fs::write(format!("{}/trace.txt", self.1), self.0.borrow().join("\n") + "\n").unwrap();
    }
}
