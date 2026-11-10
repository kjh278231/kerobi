//! 03 MIP presolve (`3cd406`), entered from `Model.presolve()` through `999702` -> `3db19d` with probe mode
//! GURO_PAR_PREPROBE, no MIP context and forSolve 0 (docs/m2/m2_presolve.md §1, docs/m2/m2_presolve_state.md).
//! Ported so far: the state (`3a484d` MIP branch), the prelude up to the first reduction, finishing and model
//! generation. Every reduction call counts toward GURO_PAR_PRELIMIT; reaching it jumps to finishing, which makes each
//! reduction checkable on its own against `tools/presolve_oracle.py --param GURO_PAR_PRELIMIT=k`.
// Ported comparisons keep the original's form (`!(a < b)` differs from `a >= b` on NaN), and loops index several
// parallel arrays at once; `x = t + x` keeps the original operand order.
#![allow(clippy::neg_cmp_op_on_partial_ord, clippy::needless_range_loop, clippy::assign_op_pattern)]
pub mod bnb;
mod aggpass;
pub mod clique;
mod cliquerows;
pub mod conflict;
pub mod driver;
pub mod dump;
mod grow;
mod helpers;
mod heur;
mod iint;
mod knapsack;
mod inner;
mod cancel;
mod aggregate;
mod colpass;
mod components;
mod covers;
mod cutoff;
pub mod termination;
pub mod propagate;
mod compress;
mod deprow;
mod derived;
mod disagg;
mod dominate;
mod doubleton;
mod dualbin;
mod dualbnd;
mod dualagg;
mod dualeq;
mod dualred;
mod finalinl;
mod lists;
mod mergevub;
mod modk;
mod parallel;
pub mod post;
pub mod probe;
pub mod retry;
mod probe0;
mod probe_apply;
pub mod probe_engine;
pub mod root;
mod roundend;
mod roundend_b;
mod setpack;
mod sparsify;
pub mod state;
mod subst;
mod ss2mip;
mod symmetry;
mod vublift;
pub mod tracker;
mod transred;
mod tworows;
mod xorrows;

use rsolver_model::Model;
use state::State;

/// Declares `Params`: field, type, default, the gurobipy name (lower case), and `from_pairs`.
macro_rules! params {
    ($($(#[$m:meta])* $f:ident: $t:ty = $d:expr, $key:literal;)*) => {
        /// The Gurobi parameters MIP presolve reads. Values as gurobipy takes them; names match case-insensitively.
        /// Defaults are hard-coded here until the shared parameter table (M1 LP driver stream) replaces this struct.
        #[derive(Clone, Debug)]
        pub struct Params { $($(#[$m])* pub $f: $t,)* }
        impl Default for Params {
            fn default() -> Self { Params { $($f: $d,)* } }
        }
        impl Params {
            pub fn from_pairs(pairs: &[(String, f64)]) -> Params {
                let mut p = Params::default();
                for (k, v) in pairs {
                    match k.to_ascii_lowercase().as_str() {
                        $($key => p.$f = *v as $t,)*
                        _ => {}
                    }
                }
                p
            }
        }
    };
}

params! {
    mip_gap: f64 = 1e-4, "mipgap";
    mip_gap_abs: f64 = 1e-10, "mipgapabs";
    best_obj_stop: f64 = -1e100, "bestobjstop";
    best_bd_stop: f64 = 1e100, "bestbdstop";
    node_limit: f64 = 1e100, "nodelimit";
    iteration_limit: f64 = 1e100, "iterationlimit";
    solution_limit: i32 = 2_000_000_000, "solutionlimit";
    time_limit: f64 = 1e100, "timelimit";
    work_limit: f64 = 1e100, "worklimit";
    soft_mem_limit: f64 = 1e100, "softmemlimit";
    improve_start_gap: f64 = 0.0, "improvestartgap";
    improve_start_nodes: f64 = 1e100, "improvestartnodes";
    improve_start_time: f64 = 1e100, "improvestarttime";
    improve_start_work: f64 = 1e100, "improvestartwork";
    /// env+0x24a8 Cutoff, in the input model's objective direction (1e100 disables it).
    cutoff: f64 = 1e100, "cutoff";
    /// env+0x2810 GURO_PAR_OBJGCD
    objgcd: i32 = 1, "guro_par_objgcd";
    /// env+0x2424 Presolve (-1 auto, 0 off, 1 conservative, 2 aggressive)
    presolve: i32 = -1, "presolve";
    /// env+0x26ec GURO_PAR_PRELIMIT: number of reduction calls before presolve stops
    prelimit: i32 = 2_000_000_000, "guro_par_prelimit";
    /// env+0x2430 DualReductions
    dual_reductions: i32 = 1, "dualreductions";
    /// env+0x2544 GURO_PAR_PREBOUND
    prebound: i32 = 1, "guro_par_prebound";
    /// env+0x2540 PrePasses (-1 auto)
    prepasses: i32 = -1, "prepasses";
    /// env+0x2828 GURO_PAR_PREPROBE (also 3cd406's probe mode, from `999702`)
    preprobe: i32 = 1, "guro_par_preprobe";
    /// env+0x2680 PreSparsify
    presparsify: i32 = -1, "presparsify";
    /// env+0x26f4 PreCrush
    precrush: i32 = 0, "precrush";
    /// env+0x2460 FeasibilityTol
    feastol: f64 = 1e-6, "feasibilitytol";
    /// env+0x2468 IntFeasTol
    intfeastol: f64 = 1e-5, "intfeastol";
    /// env+0x2470 OptimalityTol
    opttol: f64 = 1e-6, "optimalitytol";
    /// env+0x2718 NumericFocus
    numeric_focus: i32 = 0, "numericfocus";
    /// env+0x257c GURO_PAR_PRESINGLECOL (-1 auto)
    presinglecol: i32 = -1, "guro_par_presinglecol";
    /// env+0x26c0 GURO_PAR_AGGMULTLIMIT
    agg_mult_limit: f64 = 1e14, "guro_par_aggmultlimit";
    /// env+0x2584 GURO_PAR_PREQSUBST (-1 auto)
    preqsubst: i32 = -1, "guro_par_preqsubst";
    /// env+0x242c Aggregate
    aggregate: i32 = 1, "aggregate";
    /// env+0x2428 AggFill (-1: `3db19d` makes it 0 for MIP presolve)
    aggfill: i32 = -1, "aggfill";
    /// env+0x2594 GURO_PAR_PREPAIRAGG
    prepairagg: i32 = 3, "guro_par_prepairagg";
    /// env+0x2534 PreDepRow (read as is, no auto value in MIP presolve)
    pre_dep_row: i32 = -1, "predeprow";
    /// env+0x2864 Symmetry
    symmetry: i32 = -1, "symmetry";
    /// env+0x2548 / 0x2688 / 0x2560 / 0x2598 / 0x255c GURO_PAR_PRECOEFFICIENT / PREXORRED / PREPARALLEL /
    /// PREPARTOBJ / PREDUALSLACK
    precoefficient: i32 = 1, "guro_par_precoefficient";
    prexorred: i32 = 1, "guro_par_prexorred";
    preparallel: i32 = 1, "guro_par_preparallel";
    prepartobj: i32 = -1, "guro_par_prepartobj";
    predualslack: i32 = 0, "guro_par_predualslack";
    /// env+0x2574 / 0x2684 / 0x259c / 0x25a0 / 0x2558 / 0x2550 PRETRANSRED / PREDOMINATE / PRECANCEL /
    /// PRECANCELCOL / PRESUBSUME / PRECLIQSUB
    pretransred: i32 = -1, "guro_par_pretransred";
    predominate: i32 = 1, "guro_par_predominate";
    precancel: i32 = -1, "guro_par_precancel";
    precancelcol: i32 = -1, "guro_par_precancelcol";
    /// env+0x25a4 GURO_PAR_PRECANCELSELECT: how `40d2bc` picks between a candidate's two multipliers
    precancelselect: i32 = -1, "guro_par_precancelselect";
    presubsume: i32 = 0, "guro_par_presubsume";
    precliqsub: i32 = 0, "guro_par_precliqsub";
    /// env+0x26c8 / 0x254c / 0x253c / 0x2538 AGEQ / PRECLIQUE / PREMERGEVUBS / PRETWOROWS
    ageq: i32 = -1, "guro_par_ageq";
    preclique: i32 = 1, "guro_par_preclique";
    premergevubs: i32 = -1, "guro_par_premergevubs";
    pretworows: i32 = -1, "guro_par_pretworows";
    /// env+0x2c30 / 0x2530 / 0x282c / 0x2554 / 0x25ac / 0x25b0 / 0x25b4 PREVUBLIFT / PRESS2MIP / PREPROBE0 /
    /// PRECOVERS / PREDUALREDUNDANT / PREDERIVED / PRESMARTDISAGG
    prevublift: i32 = 1, "guro_par_prevublift";
    press2mip: i32 = 1, "guro_par_press2mip";
    preprobe0: i32 = 1, "guro_par_preprobe0";
    precovers: i32 = 1, "guro_par_precovers";
    predualredundant: i32 = 1, "guro_par_predualredundant";
    prederived: i32 = 1, "guro_par_prederived";
    presmartdisagg: i32 = 5, "guro_par_presmartdisagg";
    /// env+0x28a4 / 0x2590 / 0x289c / 0x28a0 / 0x2820 / 0x27b8 / 0x25a8 PRELIFTGROW / SOLREDUCTION / PRECONNECTED /
    /// PREBICONNECTED / IMPLIEDINT / SYMMTRANS / PREDUALCONVTOBIN
    preliftgrow: i32 = 1, "guro_par_preliftgrow";
    solreduction: i32 = 1, "guro_par_solreduction";
    preconnected: i32 = -1, "guro_par_preconnected";
    prebiconnected: i32 = 1, "guro_par_prebiconnected";
    impliedint: i32 = 1, "guro_par_impliedint";
    symmtrans: i32 = -1, "guro_par_symmtrans";
    predualconvtobin: i32 = -1, "guro_par_predualconvtobin";
    /// env+0x258c GURO_PAR_PRESPLITROWS (-1: split only rows of 100+ continuous entries, MIP)
    presplitrows: i32 = -1, "guro_par_presplitrows";
    /// env+0x256c / 0x2570 / 0x2564 GURO_PAR_PREPWLRED / PREOBJPWLRED / PREPOWER (3ff744 second phases, 6aec61)
    prepwlred: i32 = 1, "guro_par_prepwlred";
    preobjpwlred: i32 = 1, "guro_par_preobjpwlred";
    prepower: i32 = 0, "guro_par_prepower";
    /// env+0x2568 GURO_PAR_PREKNAPLIFTLEN (-1: 512 in `6aec61` mode 2)
    preknapliftlen: i32 = -1, "guro_par_preknapliftlen";
    /// env+0x2438 / 0x2708 / 0x233c Method / Threads / ConcurrentMethod: the root LP method (`69f0bd`, root.rs)
    method: i32 = -1, "method";
    threads: i32 = 0, "threads";
    concurrent_method: i32 = -1, "concurrentmethod";
    /// Initial primal heuristic; zero disables automatic incumbent search.
    heuristics: f64 = 0.05, "heuristics";
    prepreheur: i32 = 1, "guro_par_prepreheur";
    seed: i32 = 0, "seed";
}

/// 3cd406's locals that the reduction loop reads.
#[derive(Clone, Debug)]
pub struct Run {
    /// `local_68`: reduction calls so far, compared with state+0x1dc (GURO_PAR_PRELIMIT) after each
    pub calls: i32,
    /// `local_928`: passes (PrePasses; auto 2e9 with Presolve 2, 20 with Presolve 1, else 100)
    pub passes: i32,
    /// `local_58`: probe mode (GURO_PAR_PREPROBE; 3 for large models whose binaries carry most nonzeros)
    pub probe: i32,
    /// `local_c90`: work limit the reduction timers are set from (L1522-1602)
    pub limit: f64,
    /// what each count was (`3cd406` call or inline block), when tracing (`trace`)
    pub trace: Option<Vec<&'static str>>,
}

impl Run {
    /// One reduction call has finished: count it. True = PRELIMIT reached, go to finishing.
    pub fn counted(&mut self, st: &State, what: &'static str) -> bool {
        self.calls += 1;
        if let Some(t) = &mut self.trace {
            t.push(what);
        }
        st.prelimit <= self.calls
    }
}

/// MIP presolve: the presolved model, or Err(status) when presolve proves the model infeasible (3); 3cd406 then
/// skips model generation and `Model.presolve()` fails with "Unable to create presolved model" (checked on a model
/// with an integer column fixed at 2.5).
pub fn presolve(md: &Model, p: &Params) -> Result<Model, i32> {
    presolve_with_state(md, p).map(|(model, _)| model)
}

/// Presolve while retaining the reduction records and index state needed to restore a solution.
pub fn presolve_with_state(md: &Model, p: &Params) -> Result<(Model, State), i32> {
    let mut st = State::new(md, p);
    reduce(&mut st, p)?;
    match finish(&mut st, p) {
        0 => match split_rows(&mut st, p) {
            0 => Ok((st.to_model(md), st)),
            rc => Err(rc),
        },
        rc => Err(rc),
    }
}

/// `3b0917` before it builds the model: opposite parallel `<` rows that `3ff744` left (+0x42c) are split
/// (`3ff744(st, env, 0, 2, 0)`) when PRESPLITROWS > 0, or < 0 and some column is continuous. Returns a status.
/// shortcut: env+0x2728 (unnamed, 0 at defaults) is assumed 0.
pub fn split_rows(st: &mut State, p: &Params) -> i32 {
    if st.antipar == 0 || p.presplitrows == 0 || (p.presplitrows < 0 && st.nint >= st.n as i32) {
        return 0;
    }
    st.parallel(p, 2, 0)
}

/// 3cd406 up to model generation: the prelude, then the rounds (`driver`). Err = status (3 infeasible, ...).
pub fn reduce(st: &mut State, p: &Params) -> Result<(), i32> {
    reduce_traced(st, p, false).0
}

/// `reduce`, also returning what each PRELIMIT count was when `trace` (count k is element k-1).
pub fn reduce_traced(st: &mut State, p: &Params, trace: bool) -> (Result<(), i32>, Vec<&'static str>) {
    match prelude(st, p, trace) {
        Ok(Some(mut run)) => {
            let r = driver::run(st, p, &mut run);
            (r, run.trace.unwrap_or_default())
        }
        Ok(None) => (Ok(()), Vec::new()),
        Err(e) => (Err(e), Vec::new()),
    }
}

/// 3cd406 from the constructor to the pass loop (decompiled L756-1696). None = go to finishing (PRELIMIT reached
/// or Presolve 0); Err = status (3 infeasible).
pub fn prelude(st: &mut State, p: &Params, trace: bool) -> Result<Option<Run>, i32> {
    // `5b84b1` (state+0x3f0 scratch), `3ba5ac` (callback, log) and `3ba9de` (time/work limits) change nothing here.
    // GURO_PAR_DISTRIBUTED (env+0x2ca0) > 1 would go straight to finishing; it is 0 by default.
    let passes = if p.prepasses >= 0 { p.prepasses } else if p.presolve == 2 { 2_000_000_000 } else if p.presolve == 1 { 20 } else { 100 };
    let mut run = Run { calls: 0, passes, probe: p.preprobe, limit: 0.0, trace: None };
    if trace {
        run.trace = Some(Vec::new());
    }
    if let Err(j) = st.round_integers() {
        st.bad_col = j as i32;
        return Err(3);
    }
    if run.counted(st, "round_integers") {
        return Ok(None);
    }
    if p.presolve != 0 {
        // env+0x2728 (inside a sub-MIP) is 0 and there is no MIP context, so the restart block (L1022) is skipped
        let (n, m) = (st.n as f64, st.m as f64);
        if n * 0.99 <= st.nbin as f64 && n * 1.2 < m {
            let eq = st.sense.iter().filter(|&&c| c == b'=').count();
            st.work += m * st.work_mult;
            st.eq_rows = m < eq as f64 * 1000.0;
        }
        run.probe = large_probe(st, p, run.probe);
    }
    // L1522: the work limit, from nnz (+0xd8; Q, SOS, QC and general constraint sizes are 0 here)
    let size = st.nnz as f64;
    let base = if p.presolve == 2 || 1 < p.prepasses || 1 < p.preprobe || 0 < p.presparsify {
        1e100
    } else if p.presolve == 1 {
        size * 500.0
    } else {
        size * 2500.0
    };
    run.limit = base + 1e7;
    if st.empty_and_free_rows(p.feastol) != 0 {
        return Err(3);
    }
    if run.counted(st, "empty_and_free_rows") {
        return Ok(None);
    }
    // `9f566a` (PWL functions, state+0x240) is not ported: linear models only
    st.activity(None);
    if p.presolve == 0 {
        return Ok(None);
    }
    // L1638 `3ff744` mode 0 runs when 0.2 n <= local_91c, which stays 0 here: only for n = 0.
    // L1668: problem type 11 goes back to 0 without a MIP solution (`5b58f4` not reached); never set here.
    Ok(Some(run))
}

/// L1214/L3582 large-model check (n > 10000, 2-20% binaries, probe mode 1, PREPROBE 1): when the binaries hold
/// most nonzeros and few carry objective, probing runs in mode 3. A continuous/integer objective column stops it.
fn large_probe(st: &mut State, p: &Params, probe: i32) -> i32 {
    let n = st.n as i32;
    if !(10000 < n && st.nbin < n / 5 && n / 50 + 1 < st.nbin && probe == 1 && p.preprobe == 1) {
        return probe;
    }
    let (mut len, mut costly, mut long) = (0i64, 0, 0);
    let mut j = 0;
    while j < st.n {
        let has_cost = 1e-10 < st.obj[j].abs();
        if st.vtype[j] == b'B' {
            len += st.col_cnt[j] as i64;
            if has_cost {
                if 20 < st.col_cnt[j] {
                    long += 1;
                }
                costly += 1;
            }
        } else if has_cost {
            break;
        }
        j += 1;
    }
    st.work += (st.work_mult * 3.0) * j as f64;
    if j < st.n || 20 <= long || 50 <= costly {
        return probe;
    }
    let share = (st.nbin as f64 * 5.0 / n as f64).max(0.5);
    if share * (st.nnz as f64) < len as f64 { 3 } else { probe }
}

/// Finishing (`LAB_004b2415`, decompiled L9601-9766) up to model generation. Returns a status.
/// shortcut: the PWL, NL, QC and general-constraint blocks before it and the Q/QC/PSD/PreQLinearize/MIQCP work after
/// `3b0917` are not ported (linear models only); `4ca023` (OptimalityTarget 1 with QC) likewise.
pub fn finish(st: &mut State, p: &Params) -> i32 {
    let near = |x: &mut f64| {
        let r = (*x + 0.5).floor();
        if (r - *x).abs() <= 1e-13 {
            *x = r;
        }
    };
    // `538353` / `5383d0`: the raw row and column lists at the tracker's level (removed ones included)
    let lv = st.tracker.level;
    let rows = st.tracker.rows.list[lv][..st.tracker.rows.a[lv]].to_vec();
    for &i in &rows {
        near(&mut st.rhs[i as usize]);
    }
    st.work += (rows.len() as f64 + rows.len() as f64) * st.work_mult;
    let cols = st.tracker.cols.list[lv][..st.tracker.cols.a[lv]].to_vec();
    for &j in &cols {
        let j = j as usize;
        near(&mut st.lb[j]);
        near(&mut st.ub[j]);
        near(&mut st.obj[j]);
    }
    st.work += (st.work_mult * 4.0) * cols.len() as f64;
    st.empty_and_free_rows(p.feastol)
}

impl State {
    /// `5369b8` on the state's tracker.
    pub fn reset_tracker(&mut self) {
        let State { tracker, row_cnt, col_cnt, work, work_mult, .. } = self;
        tracker.reset(row_cnt, col_cnt, work, *work_mult);
    }

    /// 3cd406 L800-868: integer bounds rounded inward (IntFeasTol), binaries clipped to [0, 1], [0, 1] integers
    /// become binaries, a fixed integer at a fractional value within tolerance becomes continuous. Counts +0xf4
    /// (non-continuous) and +0xf0 (binary). Err(j) = column j is a binary fixed outside [0, 1] or an integer fixed
    /// at a fractional value (infeasible).
    pub fn round_integers(&mut self) -> Result<(), usize> {
        let tol = self.intfeastol;
        let mut done = 0;
        let mut bad = None;
        for j in 0..self.n {
            if self.vtype[j] == b'C' {
                done += 1;
                continue;
            }
            self.nint += 1;
            if self.vtype[j] == b'B' {
                self.nbin += 1;
            }
            let l = self.lb[j];
            if l == self.ub[j] {
                if (self.vtype[j] == b'B' && (l < -tol || 1.0 + tol < l)) || tol < l - (tol + l).floor() {
                    bad = Some(j);
                    break;
                }
                if l.floor() != l {
                    self.vtype[j] = b'C';
                }
            } else {
                self.lb[j] = (l - tol).ceil();
                self.ub[j] = (self.ub[j] + tol).floor();
                if self.vtype[j] == b'B' {
                    self.lb[j] = if self.lb[j] > 0.0 { self.lb[j] } else { 0.0 };
                    self.ub[j] = if self.ub[j] < 1.0 { self.ub[j] } else { 1.0 };
                } else if self.lb[j] == 0.0 && self.ub[j] == 1.0 {
                    self.nbin += 1;
                    self.vtype[j] = b'B';
                }
            }
            done += 1;
        }
        self.work += (done as f64 * 3.0) * self.work_mult;
        bad.map_or(Ok(()), Err)
    }
}

#[cfg(test)]
mod presolve_state_tests {
    use super::*;

    #[test]
    fn retains_records_for_removed_fixed_variable() {
        let model = Model {
            sense: 1, obj: vec![2.0], lb: vec![3.0], ub: vec![3.0], vtype: b"I".to_vec(),
            col_beg: vec![0], col_len: vec![0], ..Model::default()
        };
        let (reduced, state) = presolve_with_state(&model, &Params::default()).unwrap();
        assert!(reduced.obj.is_empty());
        assert!(state.col_cnt[0] < 0);
        assert!(state.post.iter().any(|record| matches!(record, post::PostRec::Fix { j: 0, v } if *v == 3.0)));
        assert_eq!(reduced.objcon, 6.0);
    }
}
