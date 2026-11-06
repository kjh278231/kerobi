//! 3cd406 after the prelude (decompiled L1696-9600): phases (at most 2) of rounds (at most 5) of passes, each pass an
//! inner reduction loop then the outer reductions (docs/m2_spine_b.md), the round-end stage (m2_spine_c.md) and the
//! final stage (m2_spine_d.md). Every PRELIMIT count sits where Gurobi has it, including the ones made without a
//! reduction call. Blocks for Q, QC, SOS, PWL, NL and a MIP context (restart presolve) are left out: linear models
//! from `Model.presolve()` only.
use crate::state::{Snapshot, State, CTR_TYPES};
use crate::{Params, Run};

/// One of 3cd406's 28 reduction timers (12 §4.4): `ade8ca` start (+1 per start), `ade770` stop, `adf236` usage.
#[derive(Clone, Copy, Debug, Default)]
pub struct Timer {
    acc: f64,
    running: bool,
    pub limit: f64,
}

impl Timer {
    pub fn start(&mut self, w: f64) {
        self.running = true;
        self.acc = (self.acc - w) + 1.0;
    }
    pub fn stop(&mut self, w: f64) {
        self.running = false;
        self.acc += w;
    }
    /// `adf236`
    pub fn usage(&self, w: f64) -> f64 {
        if self.limit < 0.0 {
            1.0
        } else if self.limit == 0.0 {
            0.0
        } else {
            (self.acc + if self.running { w } else { 0.0 }) / self.limit
        }
    }
    /// accumulated value and running flag (probing reads its timer, `4d3a1d` param_5)
    pub fn acc(&self) -> f64 {
        self.acc
    }
    pub fn running(&self) -> bool {
        self.running
    }
    /// `adf00e`: work used so far
    pub fn used(&self, w: f64) -> f64 {
        self.acc + if self.running { w } else { 0.0 }
    }
    /// The callers' `0.8 > adf236` test (false on NaN).
    pub fn ok(&self, w: f64) -> bool {
        0.8 > self.usage(w)
    }
}

/// Why the loops stop early: PRELIMIT reached (finishing) or a status from a reduction (3 infeasible, 5 unbounded).
enum Stop {
    Finish,
    Exit(i32),
}
type Flow = Result<(), Stop>;

fn rc(r: i32) -> Flow {
    if r != 0 { Err(Stop::Exit(r)) } else { Ok(()) }
}

struct Driver<'a> {
    st: &'a mut State,
    p: &'a Params,
    run: &'a mut Run,
    t: [Timer; 28],
    /// `local_1e8`: taken at the start of every inner iteration; `local_198`: at each pass head; `local_148`: at
    /// each round end (zero-filled at the prologue)
    snap_it: Snapshot,
    snap_pass: Snapshot,
    snap_round: Snapshot,
    /// `local_40`: passes so far, over all rounds and phases
    pass: i32,
}

/// The rounds and the final stage. Ok = go to finishing; Err = status.
pub fn run(st: &mut State, p: &Params, run: &mut Run) -> Result<(), i32> {
    // prelude 0x3b1445-0x3b15c7: every timer gets the limit B, some a multiple
    let b = run.limit;
    let mut t = [Timer { acc: 0.0, running: false, limit: b }; 28];
    for k in [17, 15, 16, 18, 14, 12, 7, 20, 23] {
        t[k].limit = b * 0.5;
    }
    t[24].limit = b * 2.0;
    t[21].limit = b * 4.0;
    if st.eq_rows || run.probe >= 3 {
        for k in [8, 9, 10] {
            t[k].limit = b * 3.0;
        }
    }
    let mut d = Driver { st, p, run, t, snap_it: Snapshot::default(), snap_pass: Snapshot::default(),
                         snap_round: Snapshot::default(), pass: 0 };
    match d.main() {
        Ok(()) | Err(Stop::Finish) => Ok(()),
        Err(Stop::Exit(r)) => Err(r),
    }
}

impl Driver<'_> {
    fn count(&mut self, what: &'static str) -> Flow {
        if self.run.counted(self.st, what) { Err(Stop::Finish) } else { Ok(()) }
    }

    /// `3ba9de` (time and work limits are the caller's business here; only the budget stop is modelled)
    fn budget(&mut self) {
        self.st.budget_check();
    }

    fn ok(&self, k: usize) -> bool {
        self.t[k].ok(self.st.work)
    }

    /// `ade8ca(T_k)`, f, `ade770(T_k)`; returns f's value.
    fn timed<R>(&mut self, k: usize, f: impl FnOnce(&mut State, &Params) -> R) -> R {
        self.t[k].start(self.st.work);
        let r = f(self.st, self.p);
        self.t[k].stop(self.st.work);
        r
    }

    fn main(&mut self) -> Flow {
        let p = self.p;
        // before the loop (LAB_004b196d); the Q (`42872b`, `429e1b`), QC (`451195`) and NL (`5b1161`) blocks are
        // for nonlinear models only
        if p.prebound != 0 && p.dual_reductions != 0 {
            rc(self.st.r5a9282(p))?;
            self.count("r5a9282")?;
        }
        self.budget();
        let mut phase = 0;
        loop {
            let mut round = 0;
            loop {
                self.round(round)?;
                if !self.round_end(round, phase)? {
                    break;
                }
                round += 1;
            }
            // LAB_004b9027: second probing, then a second phase when it changed enough
            self.st.tracker.level = 0;
            self.budget();
            if phase == 0 {
                let snap = self.st.snapshot();
                self.second_probe()?;
                if self.st.progress(&snap) > 1e-4 {
                    phase = 1;
                    continue;
                }
            }
            return self.final_chain();
        }
    }

    /// One round (LAB_004b1cb4) up to the round-end stage.
    fn round(&mut self, round: i32) -> Flow {
        let p = self.p;
        let dual = p.dual_reductions != 0;
        if self.pass >= self.run.passes {
            return Ok(());
        }
        let mut k = 0;
        loop {
            // inner loop
            let mut i = -5;
            loop {
                self.snap_it = self.st.snapshot();
                self.st.tracker.level = 2;
                self.st.reset_tracker();
                self.budget();
                rc(self.timed(0, |s, p| s.col_pass(p)))?;
                self.count("col_pass")?;
                rc(self.timed(0, |s, p| s.row_clean(p)))?;
                self.count("row_clean")?;
                if p.prebound != 0 {
                    rc(self.timed(2, |s, p| s.bound_tighten(p)))?;
                    self.count("bound_tighten")?;
                    self.budget();
                }
                let mut gc = 0;
                rc(self.timed(0, |s, p| s.singleton_rows(p, &mut gc)))?;
                self.count("singleton_rows")?;
                if gc > 1_000_000 {
                    self.timed(27, |s, _| s.gc(1));
                    rc(self.timed(0, |s, p| s.row_clean(p)))?;
                    self.count("row_clean")?;
                    self.budget();
                }
                self.st.i_to_b();
                self.count("i_to_b")?;
                if p.precoefficient != 0 {
                    self.timed(3, |s, p| s.coef_strengthen(p));
                    self.count("coef_strengthen")?;
                }
                self.budget();
                if p.aggregate != 0 {
                    let mut status = 0;
                    rc(self.timed(5, |s, p| s.doubleton_agg(p, &mut status)))?;
                    rc(status)?;
                    self.count("doubleton_agg")?;
                }
                self.timed(27, |s, _| s.gc(0));
                let rho = self.st.progress(&self.snap_it);
                if rho <= 0.001 * i.max(0) as f64 {
                    break;
                }
                i += 1;
            }
            self.st.tracker.level = 1;
            // PWL general constraints (st+0x240) and NL (st+0x300) branches: nonlinear models only
            // pass head
            if self.st.n == 0 || self.st.m == 0 {
                return Ok(());
            }
            self.budget();
            if self.pass > 0 {
                let rho = self.st.progress(&self.snap_pass);
                if rho <= 0.001 * (self.pass - 2).max(0) as f64 {
                    return Ok(());
                }
            }
            self.snap_pass = self.st.snapshot();
            self.st.reset_tracker();
            // SOS (586049), `585a6e` (st+0x228), PWL columns (58b41b), Q, QC, NL and PREQSUBST blocks: not for
            // linear models
            if p.prexorred != 0 && self.st.precrush == 0 {
                rc(self.timed(26, |s, p| s.xor_rows(p)))?;
                self.count("xor_rows")?;
            }
            if p.preparallel != 0 && self.st.precrush == 0 {
                if dual && self.ok(6) {
                    rc(self.timed(6, |s, p| s.parallel(p, 3, 0)))?;
                    self.count("parallel")?;
                }
                self.budget();
            }
            if p.precoefficient != 0 {
                rc(self.timed(3, |s, p| s.implied_int(p)))?;
                self.count("implied_int")?;
                rc(self.timed(3, |s, p| s.knapsack(p)))?;
                self.count("knapsack")?;
                self.budget();
            }
            if p.prepartobj != 0 {
                self.timed(3, |s, p| s.prepartobj(p));
                self.count("prepartobj")?;
                self.budget();
            }
            if dual && self.pass < 3 && self.ok(4) {
                rc(self.timed(4, |s, p| s.dual_bounds(p)))?;
                self.count("dual_bounds")?;
                self.budget();
            }
            // shortcut: PREDUALSLACK (`4178ae`, default 0) is not ported
            if p.preparallel != 0 {
                if dual && self.ok(6) {
                    rc(self.timed(6, |s, p| s.parallel(p, 1, 1)))?;
                    self.count("parallel")?;
                    self.budget();
                }
                if self.ok(6) {
                    rc(self.timed(6, |s, p| s.parallel(p, 0, 1)))?;
                    self.count("parallel")?;
                    self.budget();
                }
                // modes 4 and 5 run only with QC (st+0x27c)
            }
            self.budget();
            if round == 0 && k == 0 && p.pretransred != 0 {
                rc(self.timed(25, |s, p| s.trans_red(p)))?;
                self.count("trans_red")?;
                self.budget();
            }
            if dual && p.predominate != 0 && self.ok(17) {
                let t = self.t[17];
                rc(self.timed(17, |s, p| s.dominated(p, t)))?;
                self.count("dominated")?;
                self.budget();
            }
            if p.precancel != 0 && self.ok(15) {
                let m = if p.precancel == -1 { 2 } else { p.precancel - 1 };
                let t = self.t[15];
                rc(self.timed(15, |s, p| s.cancel_rows(p, m, t)))?;
                self.count("cancel_rows")?;
                self.budget();
            }
            // shortcut: PRESUBSUME (`416036`, k == 0) and PRECLIQSUB (`414ca3`) are 0 by default and not ported
            self.timed(27, |s, _| s.gc(1));
            rc(self.timed(27, |s, p| s.compress(p)))?;
            self.budget();
            if dual {
                self.timed(4, |s, p| s.dual_ineq_to_eq(p));
                self.count("dual_ineq_to_eq")?;
                self.budget();
            }
            if p.aggregate != 0 && self.ok(1) {
                if p.prebound != 0 {
                    rc(self.timed(2, |s, p| s.bound_tighten(p)))?;
                    self.count("bound_tighten")?;
                    self.budget();
                }
                rc(self.timed(1, |s, p| s.aggregate(p)))?;
                self.count("aggregate")?;
            }
            self.timed(27, |s, _| s.activity(None));
            self.budget();
            if p.precancelcol != 0 && self.ok(16) {
                let m = if p.precancelcol == -1 { 2 } else { p.precancelcol - 1 };
                let t = self.t[16];
                rc(self.timed(16, |s, p| s.cancel_cols(p, m, t)))?;
                self.count("cancel_cols")?;
                self.budget();
            }
            // shortcut: AGEQ >= 1 (`37c34c`, default -1) is not ported
            if p.prebound != 0 && dual {
                self.timed(4, |s, p| s.implied_bounds(p));
                self.count("implied_bounds")?;
                self.budget();
            }
            if p.aggregate != 0 && dual {
                rc(self.timed(4, |s, p| s.dual_agg(p)))?;
                self.count("dual_agg")?;
                self.budget();
                if self.st.opt_target_off {
                    rc(self.timed(4, |s, p| s.dual_conv_bin(p)))?;
                    self.count("dual_conv_bin")?;
                    self.budget();
                }
            }
            if p.preclique != 0 {
                if self.ok(14) {
                    rc(self.timed(14, |s, p| s.clique(p)))?;
                    self.count("clique")?;
                    self.budget();
                }
                if self.ok(11) {
                    rc(self.timed(11, |s, p| s.set_packing(p)))?;
                    self.count("set_packing")?;
                    self.budget();
                }
                if self.ok(12) {
                    let m = if p.presolve > 1 || p.preclique > 1 { 2 } else { 1 };
                    rc(self.timed(12, |s, p| s.clique_rows(p, m)))?;
                    self.count("clique_rows")?;
                    self.budget();
                }
            }
            self.budget();
            if p.premergevubs != 0 && self.ok(13) {
                let t = self.t[13];
                rc(self.timed(13, |s, p| s.merge_vubs(p, t)))?;
                self.count("merge_vubs")?;
                self.budget();
            }
            if p.pretworows != 0 && p.presolve != 1 && self.ok(7) {
                let m = p.pretworows.abs();
                let t = self.t[7];
                rc(self.timed(7, |s, p| s.two_rows(p, m, t)))?;
                self.count("two_rows")?;
                self.budget();
            }
            if k == 0 && p.pre_dep_row > 0 && self.ok(19) {
                rc(self.timed(19, |s, p| s.dep_rows(p, 1)))?;
                self.count("dep_rows")?;
                self.budget();
            }
            if k == 0 && p.pre_dep_row != 0 && dual && self.ok(19) {
                rc(self.timed(19, |s, p| s.dep_rows(p, 2)))?;
                self.count("dep_rows")?;
                self.budget();
            }
            // pass end: counted without a reduction
            self.count("pass_end")?;
            self.budget();
            let rho = self.st.progress(&self.snap_it);
            if rho <= 1e-4 {
                return Ok(());
            }
            self.pass += 1;
            k += 1;
            if self.pass >= self.run.passes {
                return Ok(());
            }
        }
    }

    /// Round-end stage (LAB_004b7b7c). Ok(true) = next round, Ok(false) = final stage.
    fn round_end(&mut self, round: i32, phase: i32) -> Result<bool, Stop> {
        let p = self.p;
        let dual = p.dual_reductions != 0;
        self.st.tracker.level = 0;
        self.budget();
        if phase > 0 || round > 0 {
            let thr = if phase == 0 && (round < 2 || self.run.probe > 1 || p.presolve > 1) { 0.01 } else { 0.05 };
            if self.st.progress(&self.snap_round) <= thr {
                return Ok(false);
            }
        }
        if self.run.probe < 3 && !self.st.eq_rows && p.presolve != 0 {
            self.col_type_stats();
        }
        self.snap_round = self.st.snapshot();
        self.st.reset_tracker();
        if p.aggregate != 0 && dual && self.st.opt_target_off {
            rc(self.timed(4, |s, p| s.q_diag(p)))?;
            self.count("q_diag")?;
            self.budget();
        }
        // SOS (`586049`, `57b2bf`): no SOS in linear models
        if round == 0 && p.symmetry != 0 && dual {
            let (mut limit, mut nodes) = (1e100, 20000);
            if self.t[24].limit != 0.0 {
                limit = self.t[24].limit - self.t[24].used(self.st.work);
            }
            if p.symmetry >= 2 {
                nodes = 50000;
                if limit < 1e100 {
                    limit *= 10.0;
                }
            }
            // f4 (local_1a4) goes to the presolved model's +0x218, not to the MPS
            let mut f4 = 0;
            rc(self.timed(24, |s, p| s.symmetry(p, nodes, limit, &mut f4)))?;
            self.count("symmetry")?;
            self.budget();
        }
        if p.prebound != 0 && dual {
            rc(self.st.r5a9282(p))?;
            self.count("r5a9282")?;
            self.budget();
            rc(self.st.r5ab429(p))?;
            self.count("r5ab429")?;
            self.budget();
        }
        if p.prevublift != 0 {
            rc(self.timed(13, |s, p| s.vub_lift(p)))?;
            self.count("vub_lift")?;
            self.budget();
        }
        if round == 0 && p.presparsify != 0 {
            rc(self.timed(20, |s, p| s.agg_pass(p)))?;
            self.count("agg_pass")?;
            self.budget();
        }
        if p.press2mip != 0 && round == 0 {
            rc(self.timed(20, |s, p| s.ss2mip(p)))?;
            self.count("ss2mip")?;
            self.budget();
        }
        if p.preprobe0 != 0 && self.ok(10) {
            let probe = self.run.probe;
            rc(self.timed(10, |s, p| s.probe0(p, probe)))?;
            self.count("probe0")?;
            self.budget();
        }
        if p.precovers != 0 && self.ok(23) {
            rc(self.timed(23, |s, p| s.covers(p)))?;
            self.count("covers")?;
            self.budget();
        }
        if dual {
            if p.predualredundant != 0 && self.st.precrush == 0 {
                rc(self.timed(4, |s, p| s.dual_redundant(p)))?;
                self.count("dual_redundant")?;
                self.budget();
            }
            if p.prederived != 0 && self.st.precrush == 0 {
                rc(self.st.derived_a(p))?;
                self.count("derived_a")?;
                rc(self.st.derived_b(p))?;
                self.count("derived_b")?;
                self.budget();
            }
            if p.predualredundant != 0 {
                rc(self.st.r5aaaf7(p))?;
                self.count("r5aaaf7")?;
                self.budget();
            }
        }
        for k in [2, 3, 5] {
            rc(self.st.mod_k(p, k))?;
            self.count("mod_k")?;
            self.budget();
        }
        rc(self.st.r579e67(p))?;
        self.count("r579e67")?;
        self.budget();
        if self.run.probe != 0 && self.ok(9) {
            let mode = self.run.probe.max(2);
            let t = self.t[9];
            rc(self.timed(9, |s, p| s.probe(p, mode, 0, 0, 0, 0, t)))?;
            self.count("probe")?;
            self.budget();
        }
        // PRECLIQSUB (`414ca3`) is 0 by default
        if p.presmartdisagg >= 2 {
            rc(self.st.smart_disagg(p))?;
            self.count("smart_disagg")?;
            self.budget();
        }
        // `5a7c0f` and `45feaa(2)` run only with a MIP context or PRETWOROWS 2
        if p.pretworows == 2 {
            if self.ok(7) {
                let t = self.t[7];
                rc(self.timed(7, |s, p| s.two_rows(p, 2, t)))?;
                self.count("two_rows")?;
            }
            self.budget();
        }
        let rho = self.st.progress(&self.snap_it);
        if rho <= 1e-4 {
            return Ok(false);
        }
        Ok(round + 1 < 5)
    }

    /// Round end 0x3b7c39 / 0x3bd3cc: with mostly binaries and many `=` rows, set +0xf8 and double the probing
    /// timers' limits (once: +0xf8 stays set).
    fn col_type_stats(&mut self) {
        let st = &mut *self.st;
        let cols = (st.n as i32 - st.cols_removed) as f64;
        let rows = (st.m as i32 - st.rows_removed) as f64;
        if !(rows > 1.2 * cols) {
            return;
        }
        let nbin = (0..st.n).filter(|&j| st.col_cnt[j] > 0 && st.vtype[j] == b'B').count();
        st.work += (st.n as f64 + st.n as f64) * st.work_mult;
        if nbin as f64 >= 0.99 * cols {
            let neq = (0..st.m).filter(|&i| st.row_cnt[i] > 0 && st.sense[i] == b'=').count();
            st.work += (st.m as f64 + st.m as f64) * st.work_mult;
            if neq as f64 * 1000.0 > rows {
                st.eq_rows = true;
                for k in [8, 9, 10] {
                    self.t[k].limit *= 2.0;
                }
            }
        }
    }

    /// Final stage part (a): the second probing (`4d3a1d` with timer 8), m2_spine_d.md §2 and §5.1.
    fn second_probe(&mut self) -> Flow {
        let p = self.p;
        let probe = self.run.probe;
        let st = &*self.st;
        let mut pr = probe;
        let mut c_flag = false;
        let alive_c = (0..st.n).filter(|&j| st.col_cnt[j] >= 0 && st.vtype[j] == b'C').count();
        if probe == 1 && st.n as i32 > st.nint && alive_c as f64 > (st.n as i32 - st.cols_removed) as f64 * 0.05 {
            c_flag = true;
            pr = 2;
        } else if probe <= 1 {
            return Ok(());
        }
        if !self.ok(8) {
            return Ok(());
        }
        let flag = if p.preliftgrow == 0 {
            false
        } else if p.preprobe != 1 || p.presolve != -1 {
            true
        } else {
            let mut nz = st.obj.iter().filter(|&&c| c != 0.0);
            nz.next().is_some() && nz.next().is_some() && (pr >= 4 || c_flag)
        };
        if !flag && probe <= 1 {
            return Ok(());
        }
        let mode = pr.max(2);
        let t = self.t[8];
        rc(self.timed(8, |s, p| s.probe(p, mode, 1, 1, flag as i32, 1, t)))?;
        self.count("probe")?;
        self.budget();
        Ok(())
    }

    /// Final stage part (b) (0x3b908e on): components, sparsify, `57999f`, row cleanup with final dual fixing,
    /// implied-integer scaling; then finishing.
    fn final_chain(&mut self) -> Flow {
        let p = self.p;
        let dual = p.dual_reductions != 0;
        let st_rows_left = self.st.m as i32 - self.st.rows_removed;
        if p.solreduction >= 1 && dual && st_rows_left == 1 {
            // shortcut: `490ab6` (solve a one-row model directly) is not ported; no model of the 38 gets there
            self.count("one_row")?;
        }
        let mut ncomp = 0;
        if self.st.opt_target_off && p.preconnected != 0 && dual {
            rc(self.timed(21, |s, p| s.components(p, &mut ncomp)))?;
            self.count("components")?;
        }
        if p.prebiconnected != 0 && dual {
            rc(self.timed(22, |s, p| s.biconnected(p)))?;
            self.count("biconnected")?;
        }
        // the component block (ncomp > 0) only treats SOS, QC and NL parts: nothing for linear models
        if p.presolve != 0 && p.presparsify != 0 && self.ok(20) {
            rc(self.timed(20, |s, p| s.sparsify(p)))?;
            self.count("sparsify")?;
        }
        self.budget();
        // SOS, `585a6e`, PWL, Q, QC, NL blocks and `5852f1` (env+0x2650 < 1e100): not for linear models
        if p.presolve != 0 {
            self.st.r57999f(p);
            self.count("r57999f")?;
        }
        loop {
            rc(self.st.row_clean(p))?;
            self.count("row_clean")?;
            let c0 = self.st.cols_removed;
            if dual {
                rc(self.st.dual_fix_empty_cols(p))?;
            }
            self.count("dual_fix_empty_cols")?;
            if !(c0 < self.st.cols_removed) {
                break;
            }
        }
        if p.impliedint != 0 && p.presolve != 0 {
            rc(self.st.implied_int_scale(p))?;
            self.count("implied_int_scale")?;
        }
        self.budget();
        // SYMMTRANS (`54807f`) needs SYMMTRANS >= 2 or a MIP context
        Ok(())
    }
}

impl State {
    /// Inner loop I->B (3cd406 0x3b2080): [0, 1] integer columns marked at level 2 become binaries.
    pub fn i_to_b(&mut self) {
        let lv = self.tracker.level;
        let list = self.tracker.cols.list[lv][..self.tracker.cols.a[lv]].to_vec();
        for &j in &list {
            let ju = j as usize;
            if self.col_cnt[ju] >= 0 && self.vtype[ju] == b'I' && self.lb[ju] == 0.0 && self.ub[ju] == 1.0 {
                self.vtype[ju] = b'B';
                self.tracker.mark_col(j);
                self.ctr[CTR_TYPES] += 1;
            }
        }
        self.work += (list.len() as f64 * 4.0) * self.work_mult;
    }

    /// PREPARTOBJ (3cd406 0x3bbaf0, m2_spine_b.md §5.2): an `=` row whose coefficients are all proportional to the
    /// costs of its columns moves those costs into the objective constant.
    pub fn prepartobj(&mut self, p: &Params) {
        let n = self.n;
        let rows = self.rows_list();
        let nnzc = (0..n).filter(|&j| self.obj[j].abs() > 1e-10).count() as i32;
        self.work += n as f64 * self.work_mult;
        let ft = p.feastol;
        for &i in &rows {
            let i = i as usize;
            if self.sense[i] != b'=' {
                continue;
            }
            let rc = self.row_cnt[i];
            if !(0 < rc && rc < nnzc) {
                continue;
            }
            // `36dce1`: the row cannot be violated
            if self.inf_max[i] == 0 && self.act_max[i] < ft && self.inf_min[i] == 0 && -ft < self.act_min[i] {
                continue;
            }
            let (mut ratio, mut seen, mut ok) = (0.0, 0, true);
            let mut q = self.row_head[i];
            while q != crate::state::NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 {
                    let cj = self.obj[nd.col as usize];
                    if cj.abs() < 1e-10 {
                        ok = false;
                        break;
                    }
                    if ratio == 0.0 {
                        ratio = nd.val / cj;
                    } else if (nd.val - cj * ratio).abs() > 1e-10 {
                        ok = false;
                        break;
                    }
                }
                seen += 1;
                q = nd.nr;
            }
            self.work += (seen as f64 * 4.0) * self.work_mult;
            if !ok {
                continue;
            }
            self.objcon += self.rhs[i] / ratio;
            let (mut t, mut q) = (0, self.row_head[i]);
            while q != crate::state::NIL {
                let nd = self.nd[q as usize];
                if nd.col >= 0 {
                    self.obj[nd.col as usize] = 0.0;
                    self.tracker.mark_col(nd.col);
                }
                t += 1;
                q = nd.nr;
            }
            self.work += (t as f64 + t as f64) * self.work_mult;
        }
        self.work += (rows.len() as f64 * 3.0) * self.work_mult;
    }
}
