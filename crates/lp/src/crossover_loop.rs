//! Initialized serial primal-push loop (double, batch 0): passes, supplier, step pick/apply, support BTRAN, factor
//! control, recomputation and rounds connected in the original order. Every numeric solve, the factor object and its
//! statistics stay behind [`PushLoopBackend`]; no solver path calls this yet. Initialization before the first pass and
//! the final solution install after the loop are the caller's.
use crate::crossover::{push_apply, push_pick, PushLp, PushParams, PushPick, PushState, PushStep};
use crate::crossover_factor::{factor_resume, factor_run, factor_select, FactorDecision, FactorExit, FactorFrame,
                              FactorSelection, FactorState, FactorVectors, PushFactorBackend};
use crate::crossover_recompute::{push_recompute, RecomputeExit, RecomputeInput, RecomputeState, RecomputeValues};
use crate::crossover_round::{round_accept, round_begin, round_btran, round_clear, round_defer, round_exhausted,
                             round_factor, round_pass, round_restart, RoundBtran, RoundExit, RoundFactor, RoundFrame,
                             RoundState};
use crate::crossover_supplier::{supplier_next, PushColumn, SupplierBuffers, SupplierFtran, SupplierModel,
                                SupplierState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactorSite {
    /// Wrapper after an approved pivot (call site 0x1c3538).
    Pivot,
    /// Round factorization after exhaustion or dense BTRAN (call site 0x1c3d31).
    Round,
}

/// Factor values the selector reads: F+0x18c, F+0x188, S+0x46c, F+0x190, F+0x184, F+0x1e0, F+0x1e8.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactorStats {
    pub updates: i32,
    pub limit: i32,
    pub interval: i32,
    pub column_work: i64,
    pub eta_count: i32,
    pub l_nnz: i64,
    pub u_nnz: i64,
}

/// Workspace fields a `05b257` call may change: S+0x294, S+0x178, S+0x184, S+0xf0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactorCtl {
    pub request: i32,
    pub numeric: i32,
    pub failure: i32,
    pub status: i32,
}

/// Permutations of a successful refactorization (F28 = new→old position, F38 = old→new position).
pub struct Renumber {
    pub old_of_new: Vec<i32>,
    pub new_of_old: Vec<i32>,
}

/// Renumber the basis after a refactorization (`05b257` C:2036–2090, pricing objects null):
/// `header'[p] = header[o[p]]`, `vstat[header'[p]] = p`, and a nonnegative S+0x2f8 becomes `n[leave]`.
pub fn renumber(st: &mut PushState, r: &Renumber) {
    let old = st.header.clone();
    for (p, h) in st.header.iter_mut().enumerate() {
        *h = old[r.old_of_new[p] as usize];
    }
    for (p, &j) in st.header.iter().enumerate() {
        st.vstat[j as usize] = p as i32;
    }
    if st.leave >= 0 {
        st.leave = r.new_of_old[st.leave as usize];
    }
}

/// Numeric boundary of the loop. Results are those of the original routines; the loop never inspects factor data.
pub trait PushLoopBackend {
    /// Supplier FTRAN (`09d008`): raw result into `output`; `spike` is supplied only with support 0.
    fn ftran(&mut self, input: &PushColumn, output: &mut PushColumn, spike: Option<&mut PushColumn>);
    /// Support BTRAN (`09e376`) of the unit vector `input`: sparse in constraint rows, or m dense values.
    fn btran(&mut self, input: &PushColumn) -> PushColumn;
    /// Current factor statistics; they change only inside [`PushLoopBackend::factor`].
    fn stats(&self) -> FactorStats;
    /// One `05b257` call on the current push state (header, S+0x2f8, vstat) with the work column S+0x210 (the
    /// wrapper's current alpha; None at the round site, which only refactors). Returns its value and, after a
    /// successful refactorization, the renumbering the loop applies to the basis.
    fn factor(&mut self, site: FactorSite, ctl: &mut FactorCtl, st: &PushState, workspace: Option<&PushColumn>)
              -> (i32, Option<Renumber>);
    /// Quality helper `089f61`; returns the period increment.
    fn quality(&mut self, ctl: &mut FactorCtl, enter: i32, period: i32) -> i32;
    /// RHS FTRAN `09cf9b` (count -1) of `rhs` with the factor's new header; m raw values.
    fn recompute_ftran(&mut self, rhs: &[f64], raw: &mut [f64]);
}

/// Fixed push-call values: attempt (`param_4`), model status (`param_3`), numeric limit (local_f8) and S+0x3a8.
#[derive(Clone, Copy, Debug)]
pub struct LoopConfig {
    pub attempt: i32,
    pub model_status: i32,
    pub numeric_limit: i32,
    pub drop: f64,
}

/// Initialized push state at the first pass preparation (0x1c1ff8). `round.request` is S+0x294 throughout;
/// `st.request` is synchronized around step selection only.
pub struct PushLoop {
    pub st: PushState,
    pub round: RoundState,
    /// S+0x120.
    pub pinf: f64,
    /// Quality period local_100.
    pub period: i32,
    pub buffers: SupplierBuffers,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LoopExit {
    /// Loop finished (`LAB_002c4050`); the caller installs the solution.
    End,
    /// Retry flag set (`*param_9 = 1`).
    Retry,
    /// Factor error code passed to the cleanup.
    Error(i32),
}

/// Observation points, in the original order. `Leave` and `Pass` are reported at the next supplier entry.
pub enum LoopEvent<'a> {
    Leave(&'a PushState, f64),
    Pass(&'a RoundState),
    Supplied { ret: i32, round: &'a RoundState, alpha: &'a PushColumn },
    Picked { pick: &'a PushPick, st: &'a PushState, alpha: &'a [f64] },
    Applied(&'a PushState),
    Stepped { step: PushStep, st: &'a PushState, round: &'a RoundState },
    Btran { leave: i32, raw: &'a PushColumn, exit: &'a RoundExit, round: &'a RoundState },
    Select(&'a FactorDecision),
    /// One `05b257` call: its entry request, S+0x2f8 and header, return, the renumbered state and the statistics after.
    Factor { site: FactorSite, request: i32, leave: i32, header: &'a [i32], ret: i32, st: &'a PushState, stats: FactorStats },
    Recompute { site: FactorSite, exit: &'a RecomputeExit, st: &'a PushState, pinf: f64 },
    Clear { period: i32 },
    /// State at the exhaustion gate before it may clear support; `end` is the gate's outcome.
    Exhausted { position: usize, deferred: usize, total: usize, support: i32, approved: Option<i32>, end: bool },
    Round(&'a RoundState),
    End(&'a RoundState),
}

type Trace<'t> = dyn FnMut(LoopEvent<'_>) + 't;

struct Ftran<'b, B>(&'b mut B);

impl<B: PushLoopBackend> SupplierFtran for Ftran<'_, B> {
    fn ftran(&mut self, input: &PushColumn, output: &mut PushColumn, spike: Option<&mut PushColumn>, _: &mut f64) {
        self.0.ftran(input, output, spike);
    }
}

struct Btran<'b, B> {
    backend: &'b mut B,
    raw: PushColumn,
}

impl<B: PushLoopBackend> RoundBtran for Btran<'_, B> {
    fn btran(&mut self, input: &PushColumn, _: &mut f64) -> PushColumn {
        self.raw = self.backend.btran(input);
        self.raw.clone()
    }
}

/// One `05b257` call on behalf of either caller; applies the renumbering and reports the call.
fn call_factor<B: PushLoopBackend>(backend: &mut B, st: &mut PushState, site: FactorSite, ctl: &mut FactorCtl,
                                   workspace: Option<&PushColumn>, trace: &mut Trace<'_>) -> i32 {
    let (request, leave, header) = (ctl.request, st.leave, st.header.clone());
    let (ret, renumbering) = backend.factor(site, ctl, st, workspace);
    if let Some(r) = renumbering {
        renumber(st, &r);
    }
    let stats = backend.stats();
    trace(LoopEvent::Factor { site, request, leave, header: &header, ret, st, stats });
    ret
}

struct PivotFactor<'b, 't, B> {
    backend: &'b mut B,
    st: &'b mut PushState,
    trace: &'b mut Trace<'t>,
}

fn ctl_of(s: &FactorState) -> FactorCtl {
    FactorCtl { request: s.request, numeric: s.numeric, failure: s.failure, status: s.status }
}

fn ctl_into(c: &FactorCtl, s: &mut FactorState) {
    s.request = c.request;
    s.numeric = c.numeric;
    s.failure = c.failure;
    s.status = c.status;
}

impl<B: PushLoopBackend> PushFactorBackend for PivotFactor<'_, '_, B> {
    fn factor(&mut self, state: &mut FactorState, vectors: &FactorVectors<'_>) -> i32 {
        let mut ctl = ctl_of(state);
        let ret = call_factor(self.backend, self.st, FactorSite::Pivot, &mut ctl, Some(vectors.workspace), self.trace);
        ctl_into(&ctl, state);
        state.updates = self.backend.stats().updates;
        ret
    }

    fn quality(&mut self, state: &mut FactorState, enter: i32, period: i32, _: &FactorVectors<'_>) -> i32 {
        let mut ctl = ctl_of(state);
        let increment = self.backend.quality(&mut ctl, enter, period);
        ctl_into(&ctl, state);
        state.updates = self.backend.stats().updates;
        increment
    }
}

struct RoundCall<'b, 't, B> {
    backend: &'b mut B,
    st: &'b mut PushState,
    trace: &'b mut Trace<'t>,
}

impl<B: PushLoopBackend> RoundFactor for RoundCall<'_, '_, B> {
    fn factor(&mut self, state: &mut RoundState) -> i32 {
        let mut ctl =
            FactorCtl { request: state.request, numeric: state.numeric, failure: state.failure, status: state.status };
        let ret = call_factor(self.backend, self.st, FactorSite::Round, &mut ctl, None, self.trace);
        state.request = ctl.request;
        state.numeric = ctl.numeric;
        state.failure = ctl.failure;
        state.status = ctl.status;
        ret
    }
}

enum Next {
    Supply,
    /// Dense BTRAN went through the support gate and compaction; request 10 is set.
    Round,
    Exit(LoopExit),
}

struct Run<'a, 't, B> {
    lp: &'a PushLp<'a>,
    par: &'a PushParams,
    cfg: &'a LoopConfig,
    s: &'a mut PushLoop,
    backend: &'a mut B,
    trace: &'a mut Trace<'t>,
    pending_leave: bool,
}

impl<B: PushLoopBackend> Run<'_, '_, B> {
    /// Supplier entry; reports the previous step's state and a fresh pass first.
    fn supply(&mut self, pass: bool) -> i32 {
        let s = &mut *self.s;
        if self.pending_leave {
            (self.trace)(LoopEvent::Leave(&s.st, s.pinf));
            self.pending_leave = false;
        }
        if pass {
            (self.trace)(LoopEvent::Pass(&s.round));
        }
        let r = &mut s.round;
        let mut ss = SupplierState {
            candidates: std::mem::take(&mut r.candidates),
            position: r.position,
            deferred: r.deferred,
            failures: std::mem::take(&mut r.failures),
            support: r.support,
            support_marks: r.marks.take(),
            count_sum: r.count_sum,
            successes: r.successes,
        };
        let model = SupplierModel { m: self.lp.m, n: self.lp.n, starts: self.lp.starts, lengths: self.lp.lengths,
                                    rows: self.lp.rows, values: self.lp.values, drop: self.cfg.drop };
        let mut work = 0.0;
        let ret = supplier_next(&model, &mut ss, &mut s.buffers, &mut work, &mut Ftran(&mut *self.backend));
        r.candidates = ss.candidates;
        r.position = ss.position;
        r.deferred = ss.deferred;
        r.failures = ss.failures;
        r.support = ss.support;
        r.marks = ss.support_marks;
        r.count_sum = ss.count_sum;
        r.successes = ss.successes;
        (self.trace)(LoopEvent::Supplied { ret, round: &s.round, alpha: &s.buffers.alpha });
        ret
    }

    /// RHS FTRAN and reclassification at either recomputation site; PInf and violations replace the old ones.
    fn recompute(&mut self, site: FactorSite) -> RecomputeExit {
        let s = &mut *self.s;
        let st = &mut s.st;
        let mut raw = vec![0.0; self.lp.m];
        self.backend.recompute_ftran(&st.rhs, &mut raw);
        let input = RecomputeInput { header: &st.header, lower: self.lp.lb, upper: self.lp.ub,
                                     infinity: self.par.inf, tolerance: self.par.h,
                                     model_status: self.cfg.model_status, attempt: self.cfg.attempt };
        let mut rs = RecomputeState { pinf: s.pinf, violations: st.violations, retry: s.round.retry,
                                      request: s.round.request };
        let exit = push_recompute(&input, RecomputeValues::Separate { raw: &raw, basic: &mut st.basic },
                                  &mut st.width, &mut st.marks, &mut rs);
        s.pinf = rs.pinf;
        st.violations = rs.violations;
        s.round.retry = rs.retry;
        (self.trace)(LoopEvent::Recompute { site, exit: &exit, st, pinf: s.pinf });
        exit
    }

    /// One supplied column up to the next supplier call (C 1177–2232).
    fn step(&mut self, j: i32) -> Next {
        let (lp, par) = (self.lp, self.par);
        {
            let s = &mut *self.s;
            s.st.request = s.round.request;
            let updates = self.backend.stats().updates;
            let alpha = &mut s.buffers.alpha;
            let pick = push_pick(lp, &mut s.st, par, j as usize, &alpha.indices, &mut alpha.values, &mut || updates);
            s.round.request = s.st.request;
            (self.trace)(LoopEvent::Picked { pick: &pick, st: &s.st, alpha: &alpha.values });
            if pick.step == PushStep::Defer {
                round_defer(&mut s.round, j);
                (self.trace)(LoopEvent::Stepped { step: pick.step, st: &s.st, round: &s.round });
                return Next::Supply;
            }
            round_accept(&mut s.round);
            let trace = &mut *self.trace;
            push_apply(lp, &mut s.st, par, &pick, &mut alpha.indices, &mut alpha.values,
                       &mut |x: &PushState| trace(LoopEvent::Applied(x)));
            alpha.count = alpha.indices.len() as i32;
            (self.trace)(LoopEvent::Stepped { step: pick.step, st: &s.st, round: &s.round });
            self.pending_leave = true;
            if s.st.leave < 0 {
                return Next::Supply;
            }
            if s.round.support != 0 {
                let leave = s.st.leave;
                let mut bt = Btran { backend: &mut *self.backend, raw: PushColumn::default() };
                let mut work = 0.0;
                let exit = round_btran(&mut s.round, leave, &mut work, &mut bt);
                (self.trace)(LoopEvent::Btran { leave, raw: &bt.raw, exit: &exit, round: &s.round });
                match exit {
                    RoundExit::Continue => {}
                    RoundExit::Factor => return Next::Round,
                    RoundExit::End => {
                        (self.trace)(LoopEvent::End(&s.round));
                        return Next::Exit(LoopExit::End);
                    }
                    other => unreachable!("BTRAN caller returned {other:?}"),
                }
            }
        }
        self.factor_step()
    }

    /// Factor decision and wrapper after an approved pivot (C 1723–1997).
    fn factor_step(&mut self) -> Next {
        let stats = self.backend.stats();
        let s = &mut *self.s;
        let sel = FactorSelection { m: self.lp.m as i32, request: s.round.request, numeric: s.round.numeric,
                                    updates: stats.updates, limit: stats.limit, interval: stats.interval,
                                    column_work: stats.column_work, eta_count: stats.eta_count,
                                    l_nnz: stats.l_nnz, u_nnz: stats.u_nnz, support: s.round.support };
        let decision = factor_select(&sel);
        (self.trace)(LoopEvent::Select(&decision));
        if !decision.call {
            s.round.request = 0;
            (self.trace)(LoopEvent::Clear { period: s.period });
            return Next::Supply;
        }
        s.round.request = decision.request;
        let mut fs = FactorState { request: s.round.request, numeric: s.round.numeric, failure: s.round.failure,
                                   status: s.round.status, updates: stats.updates };
        let mut frame = FactorFrame { period: s.period, attempt: self.cfg.attempt,
                                      numeric_limit: self.cfg.numeric_limit, enter: s.st.enter };
        let mut resume = false;
        loop {
            let exit = {
                let s = &mut *self.s;
                let mut vectors = FactorVectors { workspace: &s.buffers.alpha, current: &s.buffers.alpha,
                                                  spike: &s.buffers.spike };
                let mut fb = PivotFactor { backend: &mut *self.backend, st: &mut s.st, trace: &mut *self.trace };
                if resume {
                    factor_resume(&mut fs, &mut frame, &mut vectors, &mut fb)
                } else {
                    factor_run(&mut fs, &mut frame, &mut vectors, &mut fb)
                }
            };
            let s = &mut *self.s;
            s.round.request = fs.request;
            s.round.numeric = fs.numeric;
            s.round.failure = fs.failure;
            s.round.status = fs.status;
            s.period = frame.period;
            match exit {
                FactorExit::Continue => {
                    (self.trace)(LoopEvent::Clear { period: s.period });
                    return Next::Supply;
                }
                FactorExit::Recompute => {
                    if self.recompute(FactorSite::Pivot) == RecomputeExit::Retry {
                        return Next::Exit(LoopExit::Retry);
                    }
                    let s = &mut *self.s;
                    fs.request = s.round.request;
                    resume = true;
                }
                FactorExit::Retry => {
                    s.round.retry = 1;
                    return Next::Exit(LoopExit::Retry);
                }
                FactorExit::Error(code) => return Next::Exit(LoopExit::Error(code)),
            }
        }
    }

    /// Round factorization, recomputation and request clear (C 2271–2414). Request 10 is already set.
    fn round(&mut self) -> Option<LoopExit> {
        let s = &mut *self.s;
        (self.trace)(LoopEvent::Round(&s.round));
        let frame = RoundFrame { attempt: self.cfg.attempt, numeric_limit: self.cfg.numeric_limit };
        let mut call = RoundCall { backend: &mut *self.backend, st: &mut s.st, trace: &mut *self.trace };
        match round_factor(&mut s.round, &frame, &mut call) {
            RoundExit::Recompute => {}
            RoundExit::Retry => return Some(LoopExit::Retry),
            RoundExit::Error(code) => return Some(LoopExit::Error(code)),
            other => unreachable!("round factor returned {other:?}"),
        }
        if self.recompute(FactorSite::Round) == RecomputeExit::Retry {
            return Some(LoopExit::Retry);
        }
        let s = &mut *self.s;
        round_clear(&mut s.round);
        if s.round.candidates.is_empty() {
            (self.trace)(LoopEvent::End(&s.round));
            return Some(LoopExit::End);
        }
        None
    }
}

/// Run the initialized loop until it ends or a retry/error leaves it. Termination checks and callbacks before each
/// step (`9a91f3`, `a83b2f`, `8b129`) are assumed to continue.
pub fn push_loop<B: PushLoopBackend>(lp: &PushLp<'_>, par: &PushParams, cfg: &LoopConfig, s: &mut PushLoop,
                                     backend: &mut B, trace: &mut Trace<'_>) -> LoopExit {
    let mut run = Run { lp, par, cfg, s, backend, trace, pending_leave: false };
    loop {
        round_pass(&mut run.s.round);
        let mut j = run.supply(true);
        let mut dense = false;
        if j >= 0 {
            round_begin(&mut run.s.round);
            loop {
                match run.step(j) {
                    Next::Supply => {}
                    Next::Round => {
                        dense = true;
                        break;
                    }
                    Next::Exit(exit) => return exit,
                }
                j = run.supply(false);
                if j < 0 {
                    break;
                }
            }
        }
        if !dense {
            let r = &run.s.round;
            let (position, deferred, total, support, approved) =
                (r.position, r.deferred, r.candidates.len(), r.support, r.approved);
            let exit = round_exhausted(&mut run.s.round);
            let end = exit == RoundExit::End;
            (run.trace)(LoopEvent::Exhausted { position, deferred, total, support, approved, end });
            if end {
                (run.trace)(LoopEvent::End(&run.s.round));
                return LoopExit::End;
            }
            round_restart(&mut run.s.round);
        }
        if let Some(exit) = run.round() {
            return exit;
        }
    }
}
