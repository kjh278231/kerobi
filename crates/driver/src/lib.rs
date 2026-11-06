//! optimize() driver for continuous models (docs/02_driver.md, docs/m1_driver.md): `9828a1` → `c23877` → LP
//! dispatch (`0c3d0d` simplex, `2bb770` barrier, `34f033` PDHG), each with presolve → engine → uncrush.
pub mod header;

use rsolver_lp::presolve::{self, postsolve};
use rsolver_lp::simplex::{Engine, Status, TraceRow};
use rsolver_lp::spx::Spx;
use rsolver_lp::{barrier, pdhg};
use rsolver_model::mps_write::{c_exp, c_g};
use rsolver_model::params::Params;
use rsolver_model::Model;
use std::time::Instant;

/// What one optimize leaves on the model (attributes) besides the log.
#[derive(Debug, Default)]
pub struct Outcome {
    pub status: i32,
    pub obj: f64,
    /// columns of the original model
    pub x: Vec<f64>,
    /// row slacks, row duals (Pi) and reduced costs (RC) of the original model; empty without a solution
    pub slack: Vec<f64>,
    pub pi: Vec<f64>,
    pub rc: Vec<f64>,
    pub iter_count: u64,
    pub bar_iter_count: usize,
    pub pdhg_iter_count: i64,
    pub work: f64,
    /// SIMPLEX callback rows of the presolved solve (objective in the original space)
    pub trace: Vec<TraceRow>,
}

/// Callbacks the engines call while they run, and the log sink.
pub struct Hooks<'a> {
    pub log: &'a mut dyn FnMut(&str),
    pub barrier: &'a mut dyn FnMut(&barrier::IterInfo),
    pub pdhg: &'a mut dyn FnMut(&pdhg::IterInfo),
}

/// `9828a1` + `c23877` for a continuous model read into `md` (minimization or maximization).
pub fn optimize(md: &Model, user: &Params, h: &mut Hooks) -> Outcome {
    let t0 = Instant::now();
    let mut p = user.clone(); // step A: adjustments live for this optimize only (11 §3.1)
    (h.log)("\n");
    (h.log)(&header::header(md, p.int("NumericFocus")));
    (h.log)("\n");
    let method = resolve_method(md, &mut p, &mut *h.log);
    adjust(&mut p);
    let crossover = p.int("Crossover");
    let target = p.int("SolutionTarget");
    // step I (11 §3.10). shortcut: crossover is not ported, so Method 2/6 with Crossover != 0 (0c3d0d → 2bb770/34f033
    // → crossover) run without it; concurrent (3, 4) runs the dual simplex, which is what 0c3d0d does with one thread.
    let out = match method {
        2 if crossover == 0 && target != 1 => interior(md, &p, Alg::Barrier, h, t0),
        6 if crossover == 0 && target != 1 => interior(md, &p, Alg::Pdhg, h, t0),
        2 => interior(md, &p, Alg::Barrier, h, t0),
        6 => interior(md, &p, Alg::Pdhg, h, t0),
        m => simplex_driver(md, &p, if m == 0 { 0 } else { 1 }, h, t0),
    };
    (h.log)("\n");
    out
}

/// `c23877` step F for a pure LP (11 §3.6; no warm starts, PWL objective or concurrent envs exist yet). Writes the
/// env values the original changes (Method, ConcurrentMethod, LPWarmStart) and returns the method to run.
pub fn resolve_method(md: &Model, p: &mut Params, log: &mut dyn FnMut(&str)) -> i32 {
    let mut m = p.int("Method");
    if m == 5 {
        log("Warning: Method GRB_METHOD_DETERMINISTIC_CONCURRENT_SIMPLEX (5) has been deprecated.\n");
        m = 3;
        p.set("Method", 3.0);
        if p.int("ConcurrentMethod") != -1 {
            log(&format!("Warning: Ignoring non-default concurrent method {}.\n", p.int("ConcurrentMethod")));
        }
        p.set("ConcurrentMethod", 3.0);
    }
    if p.int("NetworkAlg") == 1 {
        m = 0;
    } else if m == -1 {
        m = if md.num_nz() < 30000 { 1 } else { 3 };
    } else if p.int("LPWarmStart") == -1 {
        p.set("LPWarmStart", if m == 6 { 2.0 } else { 1.0 });
    }
    let user_method = p.int("Method");
    let target = p.int("SolutionTarget");
    if target == 1 && user_method == -1 {
        m = 3;
        p.set("Method", 3.0);
        let s = header::stats(md);
        // ac7d4e < 9: the engines here run one thread
        if p.int("ConcurrentMethod") == -1 && (s.obj.0 - s.obj.1).abs() > 1e-10 {
            p.set("ConcurrentMethod", 1.0);
        }
    } else if target == 0 && user_method == -1 && p.int("Crossover") != 0 {
        let s = header::stats(md);
        let big = |v: f64| (v >= 9.5e7) as i32;
        let v = (s.matrix.0 >= 9.5e7 || s.matrix.0 / (s.matrix.1 + 1e-10) >= 9.5e9) as i32 + big(s.obj.0) + big(s.bounds.0) + big(s.rhs.0);
        if v >= 2 {
            m = 3;
            p.set("Method", 3.0);
            if p.int("ConcurrentMethod") == -1 {
                p.set("ConcurrentMethod", 1.0);
            }
        } else {
            m = 2;
            p.set("Method", 2.0);
        }
    }
    if p.int("LPWarmStart") == -1 {
        p.set("LPWarmStart", 1.0);
    }
    m
}

/// `c23877` step H for a continuous model (11 §3.9): the rows of the adjustment table an LP reaches.
pub fn adjust(p: &mut Params) {
    p.set("UpdateMode", 0.0);
    if p.int("Presolve") == 0 || p.int("PoolSearchMode") > 1 {
        p.set("DualReductions", 0.0);
    }
    if p.int("LazyConstraints") != 0 {
        p.set("DualReductions", 0.0);
        p.set("PreCrush", p.get("PreCrush").max(1.0));
    }
    if p.int("GURO_PAR_PRESYMMPASSES") < 0 && p.int("Symmetry") == 0 {
        p.set("GURO_PAR_PRESYMMPASSES", 0.0);
    }
    let nf = p.int("NumericFocus");
    if nf >= 1 && p.int("GURO_PAR_SIMPLEXENDFAC") == 1 {
        p.set("GURO_PAR_SIMPLEXENDFAC", 2.0);
    }
    if nf == 1 {
        p.set("GURO_PAR_AGGMULTLIMIT", 1e10);
        p.set("MarkowitzTol", p.get("MarkowitzTol").max(0.5));
    }
    if nf >= 2 {
        p.set("GURO_PAR_AGGMULTLIMIT", 1e6);
        p.set("MarkowitzTol", p.get("MarkowitzTol").max(0.9));
        if p.int("GomoryPasses") == -1 {
            p.set("GomoryPasses", 0.0);
        }
    }
    if nf == 2 {
        for (k, v) in [("CutPasses", 3.0), ("CutAggPasses", 1.0)] {
            if p.int(k) == -1 {
                p.set(k, v);
            }
        }
    }
    if nf == 3 {
        if p.int("Quad") < 0 {
            p.set("Quad", 1.0);
        }
        for k in ["CutPasses", "CutAggPasses"] {
            if p.int(k) == -1 {
                p.set(k, 0.0);
            }
        }
    }
}

/// LP presolve with its result log (`390989` end, `0c3d0d`/`2bb770`/`34f033`), or None when Presolve is 0.
fn presolve_logged(md: &Model, p: &Params, log: &mut dyn FnMut(&str)) -> Option<(Model, postsolve::Post)> {
    if p.int("Presolve") == 0 {
        return None;
    }
    let t = Instant::now();
    let (q, post) = presolve::run_post(md, &presolve::Params::from_pairs(&p.pairs()));
    let [_, _, _, rows, cols] = post.counts;
    if rows != 0 || cols != 0 {
        log(&format!("Presolve removed {rows} rows and {cols} columns\n"));
    }
    log(&format!("Presolve time: {:.2}s\n", t.elapsed().as_secs_f64()));
    if post.status != 0 {
        // a proved status frees the presolved model before its size is logged
    } else if q.num_constrs() == 0 && q.num_vars() == 0 {
        log("Presolve: All rows and columns removed\n");
    } else {
        log(&format!("Presolved: {} rows, {} columns, {} nonzeros\n", q.num_constrs(), q.num_vars(), q.num_nz()));
    }
    Some((q, post))
}

/// `0c3d0d` L2696–2703 and L2944–2952: a presolved simplex result 3 or 5 is INF_OR_UNBD under DualReductions; at the
/// end INF_OR_UNBD is INFEASIBLE when the objective is bounded by the variable bounds (`a7fc52`, min(INFBOUND,
/// PREINFBND)). shortcut: the InfUnbdInfo and dual-model (model+0xf0, PreDual) branches are not ported.
fn simplex_status(md: &Model, p: &Params, st: i32, presolved: bool) -> i32 {
    let st = if presolved && p.int("DualReductions") != 0 && matches!(st, 3 | 5) { 4 } else { st };
    let thr = p.get("GURO_PAR_INFBOUND").min(p.get("GURO_PAR_PREINFBND"));
    if st == 4 && presolve::objective_bounded(md, thr) {
        3
    } else {
        st
    }
}

/// `36578f` L25–55: with DualReductions and Presolve on, INFEASIBLE becomes INF_OR_UNBD unless the objective is
/// bounded by the variable bounds (`a7fc52`, INFBOUND), and UNBOUNDED unless the model has no rows (`a80155`).
fn interior_status(md: &Model, p: &Params, st: i32) -> i32 {
    if !matches!(st, 3 | 5) || p.int("DualReductions") == 0 || p.int("Presolve") == 0 {
        return st;
    }
    match st {
        3 if !presolve::objective_bounded(md, p.get("GURO_PAR_INFBOUND")) => 4,
        5 if !presolve::no_rows_consistent(md) => 4,
        s => s,
    }
}

/// `a91184`, LP part: the largest row violation (r = rhs − Ax; `=` |r|, `<` −r, `>` r when positive) or finite
/// bound violation of x. shortcut: Ax is summed column by column as `a8ff5e` does; a91184's own order is not read
/// (the value only enters the big-uncrush-violation threshold).
fn max_violation(md: &Model, x: &[f64]) -> f64 {
    let mut act = vec![0.0; md.num_constrs()];
    for (j, &xj) in x.iter().enumerate().take(md.num_vars()) {
        let b = md.col_beg[j] as usize;
        for k in b..b + md.col_len[j] as usize {
            act[md.col_ind[k] as usize] += xj * md.col_val[k];
        }
    }
    let mut v = 0f64;
    for (i, &a) in act.iter().enumerate() {
        let r = md.rhs[i] - a;
        v = v.max(match md.row_sense[i] {
            b'=' => r.abs(),
            b'<' => -r,
            _ => r,
        });
    }
    for (j, &xj) in x.iter().enumerate().take(md.num_vars()) {
        if -1e30 < md.lb[j] {
            v = v.max(md.lb[j] - xj);
        }
        if md.ub[j] < 1e30 {
            v = v.max(xj - md.ub[j]);
        }
    }
    v
}

/// `36578f`: the uncrushed solution's violation is "big" past max(1e4, 1/min scale) · max(FeasibilityTol, violation in
/// the presolved model).
fn big_uncrush_violation(viol: f64, pre_viol: f64, min_scale: f64, feastol: f64) -> bool {
    1e4f64.max(1.0 / min_scale) * feastol.max(pre_viol) < viol
}

/// The log and outcome when presolve itself decided the status (`390989` *param_3, see `presolve::proved_status`):
/// no presolved model, no solution, "… solved model in 0 iterations" (`0c3d0d`, `2bb770`, `34f033`).
fn proved(st: i32, name: &str, h: &mut Hooks, t0: Instant, work: f64) -> Outcome {
    let secs = t0.elapsed().as_secs_f64();
    (h.log)(&format!("\n{name} in 0 iterations and {secs:.2} seconds ({:.2} work units)\n", work * 1e-9));
    if name == "Solved" {
        (h.log)(&status_line(st, 0.0));
    } else {
        (h.log)(match st {
            3 => "Model is infeasible\n",
            5 => "Model is unbounded\n",
            4 => "Model is infeasible or unbounded\n",
            _ => "",
        });
    }
    Outcome { status: st, work: work * 1e-9, ..Default::default() }
}

fn objective(md: &Model, x: &[f64]) -> f64 {
    md.obj.iter().zip(x).map(|(c, x)| c * x).sum::<f64>() + md.objcon
}

/// `c9457` status line after a simplex solve.
fn status_line(st: i32, obj: f64) -> String {
    match st {
        2 => format!("Optimal objective {:>16}\n", c_exp(obj, 9)),
        3 => "Infeasible model\n".into(),
        4 => "Infeasible or unbounded model\n".into(),
        5 => "Unbounded model\n".into(),
        6 => "Model objective exceeds cutoff\n".into(),
        7 => "Iteration limit reached\n".into(),
        9 => "Time limit reached\n".into(),
        12 => "Numeric error\n".into(),
        16 => "Work limit reached\n".into(),
        _ => String::new(),
    }
}

/// Attributes from arrays in `363daa`/`365159` output form (x then slack, d then y with `>` rows already signed,
/// duals of the minimization form): (x, Slack, Pi = −y, RC = d), duals in the model's sense.
fn attrs(md: &Model, x: &[f64], dual: &[f64]) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let (n, sg) = (md.num_vars(), md.sense as f64);
    (x[..n].to_vec(), x[n..].to_vec(), dual[n..].iter().map(|v| -(sg * v)).collect(), dual[..n].iter().map(|v| sg * v).collect())
}

/// Spx arrays of the original model (no presolve record) into the uncrush output form: slack rhs − Ax as `a8ff5e`
/// sums it, `>` row duals negated as `363daa`/`365159` end.
fn identity_out(md: &Model, x: &[f64], dj: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let n = md.num_vars();
    let mut sl = vec![0.0; md.num_constrs()];
    for (j, &xj) in x.iter().enumerate().take(n) {
        let b = md.col_beg[j] as usize;
        for k in b..b + md.col_len[j] as usize {
            sl[md.col_ind[k] as usize] += xj * md.col_val[k];
        }
    }
    let mut xo = x[..n].to_vec();
    xo.extend((0..sl.len()).map(|i| md.rhs[i] - sl[i]));
    let mut d = dj.to_vec();
    for (i, &c) in md.row_sense.iter().enumerate() {
        if c == b'>' {
            d[n + i] = -d[n + i];
        }
    }
    (xo, d)
}

/// The engine's solution as the original's spx arrays: x (slack for rows) and dj (dual y for rows), unscaled.
fn spx_arrays(e: &Engine, spx: &Spx) -> (Vec<f64>, Vec<f64>) {
    let (n, sc) = (spx.n, &spx.scaling);
    let xs = e.x();
    let dt = |j: usize| if e.vstat[j] == -2 { -e.d[j] } else { e.d[j] };
    let sigma = |i: usize| if spx.sense[i] == b'>' { -1.0 } else { 1.0 };
    let mut x: Vec<f64> = (0..n).map(|j| xs[j] * sc.col[j]).collect();
    x.extend((0..spx.m).map(|i| xs[n + i] / sc.row[i]));
    let mut dj: Vec<f64> = (0..n).map(|j| dt(j) / (sc.col[j] * sc.obj)).collect();
    dj.extend((0..spx.m).map(|i| dt(n + i) * sigma(i) * sc.row[i] / sc.obj));
    (x, dj)
}

/// Simplex on a (presolved) model: status, iterations, trace, and its solution as `363daa` reads it — x (slack
/// for rows), dj (dual y for rows) and status, columns then rows, unscaled.
#[allow(clippy::type_complexity)]
pub fn simplex(md: &Model, method: i32, iter_limit: u64) -> (Status, u64, Vec<TraceRow>, Vec<f64>, Vec<f64>, Vec<i32>) {
    let mut spx = Spx::new(md);
    if md.sense < 0 {
        for c in &mut spx.c {
            *c = -*c;
        }
    }
    let mut e = Engine::new(&spx);
    e.iter_limit = iter_limit;
    let st = e.solve(method);
    let (r, trace, basis) = finish_simplex(md, &spx, &e, st);
    (r.0, r.1, trace, r.2, r.3, basis)
}

fn code(st: Status) -> i32 {
    match st {
        Status::Optimal => 2,
        Status::Infeasible => 3,
        Status::Unbounded => 5,
        Status::IterLimit => 7,
        Status::Numeric => 12,
    }
}

/// (status, iterations, x and dj as spx arrays, the engine's objective)
type Reopt = (Status, u64, Vec<f64>, Vec<f64>, f64);

/// `2e9ae`: check a scaled optimum in the original units and continue from its basis when necessary. Both the
/// presolved solve and the post-uncrush solve need this, just like `simplex::solve_model_run`.
#[allow(clippy::type_complexity)]
fn finish_simplex(md: &Model, spx: &Spx, e: &Engine, status: Status) -> (Reopt, Vec<TraceRow>, Vec<i32>) {
    let collect = |sp: &Spx, en: &Engine, st| {
        let (x, dj) = spx_arrays(en, sp);
        let scale = sp.scaling.obj * md.sense as f64;
        let obj = en.objective() / scale + md.objcon;
        let trace: Vec<TraceRow> = en.trace.iter().map(|&(k, z, pi, di, pt)| (k, z / scale + md.objcon, pi, di, pt)).collect();
        ((st, en.iter, x, dj, obj), trace, en.vstat.clone())
    };
    let (r, mut trace, basis) = collect(spx, e, status);
    if status == Status::Optimal {
        let mut unscaled = Spx::unscaled(md);
        if md.sense < 0 {
            for c in &mut unscaled.c {
                *c = -*c;
            }
        }
        let mut check = Engine::new(&unscaled);
        check.iter = e.iter;
        check.iter_limit = e.iter_limit;
        if let Some(st) = check.resolve_unscaled(basis.clone()) {
            let (r, more, basis) = collect(&unscaled, &check, st);
            trace.extend(more);
            return (r, trace, basis);
        }
    }
    (r, trace, basis)
}

/// `cda5b` on the original model started from the uncrushed basis (`0c3d0d` L444–476, "Extra simplex iterations
/// after uncrush"). Start with the selected algorithm; the dual's warm entry prepares pricing, weights and row
/// copies without discarding the uncrushed basis. None when the basis does not give a complete factorization.
fn reoptimize(md: &Model, basis: &[i32], method: i32, lim: u64) -> Option<Reopt> {
    use rsolver_lp::lu::Lu;
    use rsolver_model::INFINITY;
    let mut spx = Spx::new(md);
    if md.sense < 0 {
        for c in &mut spx.c {
            *c = -*c;
        }
    }
    let (n, m) = (spx.n, spx.m);
    if basis.len() != n + m {
        return None;
    }
    let (mut vstat, mut head) = (vec![0i32; n + m], Vec::with_capacity(m));
    for j in 0..n + m {
        vstat[j] = if basis[j] >= 0 {
            head.push(j);
            head.len() as i32 - 1
        } else if spx.u[j] - spx.l[j] <= 0.0 {
            -4
        } else if basis[j] == -2 && spx.u[j] < INFINITY || basis[j] != -2 && spx.l[j] <= -INFINITY && spx.u[j] < INFINITY {
            -2
        } else if spx.l[j] <= -INFINITY && INFINITY <= spx.u[j] {
            -3
        } else {
            -1
        };
    }
    if head.len() != m {
        return None;
    }
    let lu = Lu::factor(&spx, &head);
    if !lu.complete() {
        return None;
    }
    let mut e = Engine::new(&spx);
    e.iter_limit = lim;
    let st = if method == 0 {
        (e.vstat, e.head, e.lu) = (vstat, head, lu);
        e.primal()
    } else {
        // `363daa` uses basic positions and −4 for fixed nonbasics; the warm API takes VBasis/CBasis codes.
        let codes: Vec<i32> = vstat.iter().map(|&s| if s >= 0 { 0 } else if s == -4 { -1 } else { s }).collect();
        e.solve_warm(&codes[..n], &codes[n..])
    };
    e.compute_duals();
    Some(finish_simplex(md, &spx, &e, st).0)
}

/// `0c3d0d` without warm starts, PreDual, network, sifting or concurrent: presolve → simplex → `363daa` → simplex
/// on the original model from the uncrushed basis → status.
fn simplex_driver(md: &Model, p: &Params, method: i32, h: &mut Hooks, t0: Instant) -> Outcome {
    let pre = presolve_logged(md, p, &mut *h.log);
    let work = pre.as_ref().map_or(0.0, |(_, post)| post.work);
    if let Some((_, post)) = pre.as_ref().filter(|(_, post)| post.status != 0) {
        return proved(post.status, "Solved", h, t0, post.work);
    }
    let q = pre.as_ref().map_or(md, |(q, _)| q);
    let lim = p.get("IterationLimit");
    let lim = if lim >= 1e18 { u64::MAX } else { lim as u64 };
    let (st, mut iter, trace, x, dj, vs) = simplex(q, method, lim);
    // `0d07d1` table: the first and the last row. shortcut: the engine keeps no time per iteration, so the rows
    // DisplayInterval would add in between are not printed and the first row's time is 0.
    if let (Some(a), Some(b)) = (trace.first(), trace.last()) {
        (h.log)("\nIteration    Objective       Primal Inf.    Dual Inf.      Time\n");
        let row = |r: &TraceRow, t: f64| {
            format!(" {:7}   {:>14}  {:>13}  {:>13} {:6.0}s\n", r.0, c_exp(r.1, 7), c_exp(r.2, 6), c_exp(r.3, 6), t)
        };
        // the dual start's free-column loop logs its first row in mode 1 (`0d07d1` L121): free nonbasic columns
        // after the crash (`1e7531`) mean that loop runs
        let free = method == 1 && rsolver_lp::start::start_basis(&Spx::new(q)).contains(&-3);
        if free {
            (h.log)(&format!(" {:7.0}      handle free variables                     {:6.0}s\n", 0.0, 0.0));
        } else {
            (h.log)(&row(a, 0.0));
        }
        if trace.len() > 1 {
            (h.log)(&row(b, t0.elapsed().as_secs_f64().floor()));
        }
    }
    let none = (vec![], vec![], vec![], vec![]);
    let mut engine_obj = None;
    let (status, (x, slack, pi, rc)) = match &pre {
        Some((_, post)) if code(st) == 2 && !post.is_identity() => {
            let u = postsolve::uncrush(post, md, &postsolve::PreSol { x: &x, dj: &dj, st: &vs }, None);
            match reoptimize(md, &u.basis, method, lim.saturating_sub(iter)) {
                Some((st2, extra, x2, d2, o2)) => {
                    engine_obj = Some(o2);
                    if extra > 0 {
                        (h.log)(&format!("\nExtra simplex iterations after uncrush: {extra}\n"));
                    }
                    iter += extra;
                    let (xo, d) = identity_out(md, &x2, &d2);
                    (simplex_status(md, p, code(st2), false), attrs(md, &xo, &d))
                }
                None => (2, attrs(md, &u.x, &u.dual)),
            }
        }
        // a non-optimal presolved result is not uncrushed: no solution
        Some((_, post)) if !post.is_identity() => (simplex_status(md, p, code(st), true), none),
        _ => {
            let (xo, d) = identity_out(md, &x, &dj);
            (simplex_status(md, p, code(st), false), attrs(md, &xo, &d))
        }
    };
    let obj = engine_obj.unwrap_or(if x.is_empty() { 0.0 } else { objective(md, &x) });
    let work = work * 1e-9;
    (h.log)(&format!("\nSolved in {iter} iterations and {:.2} seconds ({work:.2} work units)\n", t0.elapsed().as_secs_f64()));
    (h.log)(&status_line(status, obj));
    // shortcut: simplex work is not counted yet (P2 of G1); Work holds the presolve part only
    Outcome { status, obj, x, slack, pi, rc, iter_count: iter, trace, work, ..Default::default() }
}

#[derive(Clone, Copy, PartialEq)]
enum Alg {
    Barrier,
    Pdhg,
}

/// `07d3ce` on the model data: columns with only an upper bound (the D+0x408 marks) become x' = −x — cost and
/// column negated, bounds (−ub, +inf). Returns the flipped model and the flipped columns (None if there are none).
/// The engines' own setup never sees such a column then; the solution is flipped back by the caller.
fn flip_upper_only(q: &Model) -> Option<(Model, Vec<usize>)> {
    use rsolver_model::INFINITY;
    let cols: Vec<usize> = (0..q.num_vars()).filter(|&j| q.lb[j] <= -INFINITY && q.ub[j] < INFINITY).collect();
    if cols.is_empty() {
        return None;
    }
    let mut f = q.clone();
    for &j in &cols {
        f.obj[j] = -f.obj[j];
        (f.lb[j], f.ub[j]) = (-q.ub[j], INFINITY);
        let b = f.col_beg[j] as usize;
        for v in &mut f.col_val[b..b + f.col_len[j] as usize] {
            *v = -*v;
        }
    }
    Some((f, cols))
}

/// `2bb770` / `34f033` without crossover: presolve → barrier or PDHG → `36578f` (status rules, `365159` uncrush,
/// big-uncrush-violation check).
fn interior(md: &Model, p: &Params, alg: Alg, h: &mut Hooks, t0: Instant) -> Outcome {
    let name = if alg == Alg::Barrier { "Barrier" } else { "PDHG" };
    let pre = presolve_logged(md, p, &mut *h.log);
    if let Some((_, post)) = pre.as_ref().filter(|(_, post)| post.status != 0) {
        return proved(post.status, &format!("{name} solved model"), h, t0, post.work);
    }
    let q = pre.as_ref().map_or(md, |(q, _)| q);
    let flipped = flip_upper_only(q);
    let qe = flipped.as_ref().map_or(q, |(f, _)| f);
    let mut out = Outcome::default();
    let (engine_status, engine_obj, (mut xq, slq, mut dq, yq)) = match alg {
        Alg::Barrier => {
            let bp = barrier::Params {
                bar_conv_tol: p.get("BarConvTol"),
                bar_iter_limit: p.get("BarIterLimit") as i64,
                bar_correctors: p.int("BarCorrectors"),
            };
            let r = barrier::solve_logged(qe, &bp, &mut *h.barrier, &mut *h.log, t0);
            out.bar_iter_count = r.bar_iters;
            out.work = pre.as_ref().map_or(0.0, |(_, post)| post.work) * 1e-9 + r.work;
            (r.status, r.obj, (r.x, r.slack, r.d, r.y))
        }
        Alg::Pdhg => {
            (h.log)("\nStart PDHG using 1 threads\n");
            let pp = pdhg::Params {
                iter_limit: p.get("PDHGIterLimit"),
                rel_tol: p.get("PDHGRelTol"),
                abs_tol: p.get("PDHGAbsTol"),
                conv_tol: p.get("PDHGConvTol"),
                work_limit: p.get("WorkLimit"),
                work0: pre.as_ref().map_or(0.0, |(_, post)| post.work),
            };
            let r = pdhg::solve_logged(qe, &pp, &mut *h.pdhg, &mut *h.log, t0, p.get("DisplayInterval"));
            out.pdhg_iter_count = r.iters;
            out.work = r.work / 1e9;
            (r.status, r.obj, (r.x, r.slack, r.d, r.y))
        }
    };
    for &j in flipped.as_ref().map_or(&[][..], |(_, c)| c) {
        xq[j] = -xq[j];
        dq[j] = -dq[j];
    }
    let mut status = if pre.is_some() { interior_status(md, p, engine_status) } else { engine_status };
    // `36578f`: no solution for 3, 4, 5, 6, 12; otherwise `365159` on the presolved solution as `7a069` hands it
    // over (x then slack, d then y, the presolved model's space; 64/64 bitwise on the oracle, docs/m1_driver.md §8)
    let (x, slack, pi, rc) = if matches!(status, 3..=6 | 12) {
        (vec![], vec![], vec![], vec![])
    } else {
        let (xs, dj) = ([&xq[..], &slq[..]].concat(), [&dq[..], &yq[..]].concat());
        let u = match &pre {
            Some((_, post)) => postsolve::uncrush_nonbasic(post, md, &postsolve::PreSol { x: &xs, dj: &dj, st: &[] }, None),
            None => {
                let (x, dual) = identity_out(md, &xq, &dj);
                postsolve::Uncrushed { basis: vec![], x, dual }
            }
        };
        attrs(md, &u.x, &u.dual)
    };
    let obj = if x.is_empty() { 0.0 } else { objective(md, &x) };
    let it = if alg == Alg::Barrier { out.bar_iter_count as f64 } else { out.pdhg_iter_count as f64 };
    let secs = t0.elapsed().as_secs_f64();
    if engine_status == 2 {
        (h.log)(&format!("\n{name} solved model in {it:.0} iterations and {secs:.2} seconds ({:.2} work units)\n", out.work));
        // `2bf1ea` / `34c004` print the engine's objective (presolved model, before uncrush)
        (h.log)(&format!("Optimal objective {}\n", c_exp(engine_obj, 8)));
    } else {
        (h.log)(&format!("\n{name} performed {it:.0} iterations in {secs:.2} seconds ({:.2} work units)\n", out.work));
        (h.log)(&status_line(status, obj));
    }
    (h.log)("\n"); // `2bf1ea` / `34c004` close their block with a blank line
    // `36578f` big uncrush violation: an OPTIMAL solution whose violation in the original model exceeds
    // max(1e4, 1/min scale) · max(FeasibilityTol, violation in the presolved model) is SUBOPTIMAL.
    // shortcut: the presolved violation (result+0x58) is recomputed here from the presolved model and x.
    if let (2, Some((_, post))) = (status, &pre) {
        let (smin, pre_viol) = if post.is_identity() {
            (1.0, 0.0)
        } else {
            let smin = match alg {
                Alg::Barrier => {
                    let sc = rsolver_lp::scaling::scale_flag3(qe);
                    sc.col.iter().chain(&sc.row).fold(f64::INFINITY, |a, &v| a.min(v))
                }
                Alg::Pdhg => {
                    let d = pdhg::scale::scale_flag4(qe);
                    d.s.iter().chain(&d.r).fold(f64::INFINITY, |a, &v| a.min(v))
                }
            };
            (smin, max_violation(q, &xq))
        };
        let viol = max_violation(md, &x);
        if big_uncrush_violation(viol, pre_viol, smin, p.get("FeasibilityTol")) {
            (h.log)(&format!(
                "Warning: Changing status from GRB_OPTIMAL to GRB_SUBOPTIMALdue to big uncrush violation {} (presolved model = {}).\n",
                c_g(viol, 6),
                c_g(pre_viol, 6)
            ));
            status = 13;
        }
    }
    Outcome { status, obj, x, slack, pi, rc, ..out }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An LP with `nnz` nonzeros in one row: min x_0, 0 <= x <= 1, sum x <= nnz.
    fn lp(nnz: usize) -> Model {
        let mut m = Model { sense: 1, ..Default::default() };
        for j in 0..nnz {
            m.obj.push(if j == 0 { 1.0 } else { 0.0 });
            m.lb.push(0.0);
            m.ub.push(1.0);
            m.vtype.push(b'C');
            m.col_beg.push(j as i64);
            m.col_len.push(1);
            m.col_ind.push(0);
            m.col_val.push(1.0);
        }
        m.col_beg.push(nnz as i64);
        m.row_sense.push(b'<');
        m.rhs.push(nnz as f64);
        m
    }

    fn resolve(md: &Model, pairs: &[(&str, f64)]) -> (i32, Params, String) {
        let mut p = Params::from_pairs(&pairs.iter().map(|&(k, v)| (k.to_string(), v)).collect::<Vec<_>>());
        let mut log = String::new();
        let m = resolve_method(md, &mut p, &mut |t| log.push_str(t));
        (m, p, log)
    }

    #[test]
    fn method_table() {
        let (small, big) = (lp(3), lp(30000));
        assert_eq!(resolve(&small, &[]).0, 1); // nnz < 30,000: dual
        assert_eq!(resolve(&big, &[]).0, 3); // concurrent
        assert_eq!(resolve(&small, &[("NetworkAlg", 1.0), ("Method", 2.0)]).0, 0);
        let (m, p, log) = resolve(&small, &[("Method", 5.0), ("ConcurrentMethod", 1.0)]);
        assert_eq!((m, p.int("Method"), p.int("ConcurrentMethod")), (3, 3, 3));
        assert!(log.contains("(5) has been deprecated") && log.contains("Ignoring non-default concurrent method 1."));
        let (m, p, _) = resolve(&small, &[("SolutionTarget", 1.0)]);
        assert_eq!((m, p.int("Method")), (3, 3));
        assert_eq!(resolve(&small, &[("SolutionTarget", 0.0)]).0, 2); // no large statistics: barrier
        assert_eq!(resolve(&small, &[("SolutionTarget", 0.0), ("Crossover", 0.0)]).0, 1);
        assert_eq!(resolve(&small, &[("Method", 6.0)]).1.int("LPWarmStart"), 2);
        assert_eq!(resolve(&small, &[("Method", 1.0)]).1.int("LPWarmStart"), 1);
        assert_eq!(resolve(&small, &[]).1.int("LPWarmStart"), 1);
    }

    #[test]
    fn big_uncrush_status_matches_gurobi_fault_injection() {
        // `tools/uncrush_oracle.py --uncrush-delta` on afiro, Method 2/6, Crossover 0. These controlled
        // postsolve errors exercise both sides of Gurobi's rare status-13 branch, including its PDHG residual term.
        let fixture = include_str!("../../../bench/gurobi_status/uncrush_violation.tsv");
        for row in fixture.lines().skip(1) {
            let f: Vec<&str> = row.split('\t').collect();
            let status: i32 = f[2].parse().unwrap();
            let pre: f64 = f[3].parse().unwrap();
            let scale: f64 = f[4].parse().unwrap();
            let viol: f64 = f[5].parse().unwrap();
            assert_eq!(big_uncrush_violation(viol, pre, scale, 1e-6), status == 13, "{row}");
        }
    }

    #[test]
    fn status_rules() {
        // big uncrush violation: 1e4 · max(1e-6, pre) unless 1/min scale is larger
        assert!(!big_uncrush_violation(0.01, 0.0, 1.0, 1e-6));
        assert!(big_uncrush_violation(0.0101, 0.0, 1.0, 1e-6));
        assert!(!big_uncrush_violation(0.5, 1e-4, 1.0, 1e-6)); // threshold 1e4 · 1e-4 = 1
        assert!(big_uncrush_violation(0.5, 1e-5, 1.0, 1e-6)); // threshold 0.1
        assert!(!big_uncrush_violation(0.05, 0.0, 1.0 / 65536.0, 1e-6));
        assert!(big_uncrush_violation(0.07, 0.0, 1.0 / 65536.0, 1e-6));
        // max_violation: rows by sense, then finite bounds
        let mut md = lp(2); // x0 + x1 <= 2, 0 <= x <= 1
        assert_eq!(max_violation(&md, &[1.0, 1.0]), 0.0);
        assert_eq!(max_violation(&md, &[1.5, 1.0]), 0.5);
        md.row_sense[0] = b'>';
        assert_eq!(max_violation(&md, &[0.25, 0.5]), 1.25);
        md.row_sense[0] = b'=';
        assert_eq!(max_violation(&md, &[1.0, 0.5]), 0.5);
        // a7fc52 / a80155 through the path rules
        let p = Params::default();
        assert_eq!(interior_status(&md, &p, 3), 3); // bounded objective: infeasible stays
        assert_eq!(interior_status(&md, &p, 5), 4); // rows present: unbounded becomes INF_OR_UNBD
        let mut free = md.clone();
        free.lb[0] = -1e30;
        assert_eq!(interior_status(&free, &p, 3), 4);
        assert_eq!(simplex_status(&md, &p, 5, true), 3); // 5 -> 4 -> bounded -> 3
        assert_eq!(simplex_status(&free, &p, 3, true), 4);
        let off = Params::from_pairs(&[("DualReductions".into(), 0.0)]);
        assert_eq!(interior_status(&free, &off, 3), 3);
    }

    #[test]
    fn adjust_table() {
        let mut p = Params::from_pairs(&[("Presolve".into(), 0.0), ("NumericFocus".into(), 2.0)]);
        adjust(&mut p);
        assert_eq!(p.int("DualReductions"), 0);
        assert_eq!(p.get("GURO_PAR_AGGMULTLIMIT"), 1e6);
        assert_eq!(p.get("MarkowitzTol"), 0.9);
        assert_eq!((p.int("GURO_PAR_SIMPLEXENDFAC"), p.int("GomoryPasses"), p.int("CutPasses")), (2, 0, 3));
        let mut q = Params::default();
        adjust(&mut q);
        assert_eq!((q.int("DualReductions"), q.get("MarkowitzTol"), q.int("UpdateMode")), (1, Params::default().get("MarkowitzTol"), 0));
    }
}
