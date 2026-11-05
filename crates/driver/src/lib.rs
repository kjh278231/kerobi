//! optimize() driver for continuous models (docs/02_driver.md, docs/m1_driver.md): `9828a1` → `c23877` → LP
//! dispatch (`0c3d0d` simplex, `2bb770` barrier, `34f033` PDHG), each with presolve → engine → uncrush.
pub mod header;

use rsolver_lp::presolve::{self, postsolve};
use rsolver_lp::simplex::{Engine, Status, TraceRow};
use rsolver_lp::spx::Spx;
use rsolver_lp::{barrier, pdhg};
use rsolver_model::mps_write::c_exp;
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
    match method {
        2 if crossover == 0 && target != 1 => interior(md, &p, Alg::Barrier, h, t0),
        6 if crossover == 0 && target != 1 => interior(md, &p, Alg::Pdhg, h, t0),
        2 => interior(md, &p, Alg::Barrier, h, t0),
        6 => interior(md, &p, Alg::Pdhg, h, t0),
        m => simplex_driver(md, &p, if m == 0 { 0 } else { 1 }, h, t0),
    }
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
    if q.num_constrs() == 0 && q.num_vars() == 0 {
        log("Presolve: All rows and columns removed\n");
    } else {
        log(&format!("Presolved: {} rows, {} columns, {} nonzeros\n", q.num_constrs(), q.num_vars(), q.num_nz()));
    }
    Some((q, post))
}

/// Statuses a presolve with dual reductions cannot prove (`36578f` → 03 §3.6): INFEASIBLE becomes INF_OR_UNBD
/// unless the objective is bounded by the variable bounds (`a7fc52`). shortcut: UNBOUNDED always becomes
/// INF_OR_UNBD (`a80155`, which can keep it, is not read); no Netlib LP reaches this.
fn presolve_status(md: &Model, p: &Params, st: i32) -> i32 {
    if !matches!(st, 3 | 5) || p.int("DualReductions") == 0 {
        return st;
    }
    let thr = p.get("GURO_PAR_INFBOUND").min(p.get("GURO_PAR_PREINFBND"));
    let bounded = (0..md.num_vars()).all(|j| {
        let c = md.obj[j] * md.sense as f64;
        md.vtype[j] == b'B' || c == 0.0 || if c > 0.0 { md.lb[j] > -thr } else { md.ub[j] < thr }
    });
    if st == 3 && bounded {
        3
    } else {
        4
    }
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
    let (n, sc) = (spx.n, &spx.scaling);
    let xs = e.x();
    let dt = |j: usize| if e.vstat[j] == -2 { -e.d[j] } else { e.d[j] };
    let sigma = |i: usize| if spx.sense[i] == b'>' { -1.0 } else { 1.0 };
    let mut x: Vec<f64> = (0..n).map(|j| xs[j] * sc.col[j]).collect();
    x.extend((0..spx.m).map(|i| xs[n + i] / sc.row[i]));
    let mut dj: Vec<f64> = (0..n).map(|j| dt(j) / (sc.col[j] * sc.obj)).collect();
    dj.extend((0..spx.m).map(|i| dt(n + i) * sigma(i) * sc.row[i] / sc.obj));
    let scale = sc.obj * md.sense as f64;
    let trace = e.trace.iter().map(|&(k, z, pi, di, pt)| (k, z / scale + md.objcon, pi, di, pt)).collect();
    (st, e.iter, trace, x, dj, e.vstat.clone())
}

/// `0c3d0d` without warm starts, PreDual, network, sifting or concurrent: presolve → simplex → `363daa` → status.
/// shortcut: the simplex on the original model after uncrush ("Extra simplex iterations after uncrush") needs a
/// warm-start entry the engine lacks; the uncrushed solution is reported as is.
fn simplex_driver(md: &Model, p: &Params, method: i32, h: &mut Hooks, t0: Instant) -> Outcome {
    let pre = presolve_logged(md, p, &mut *h.log);
    let q = pre.as_ref().map_or(md, |(q, _)| q);
    let lim = p.get("IterationLimit");
    let lim = if lim >= 1e18 { u64::MAX } else { lim as u64 };
    let (st, iter, trace, x, dj, vs) = simplex(q, method, lim);
    let code = match st {
        Status::Optimal => 2,
        Status::Infeasible => 3,
        Status::Unbounded => 5,
        Status::IterLimit => 7,
        Status::Numeric => 12,
    };
    let (status, x) = match &pre {
        Some((_, post)) => {
            let u = postsolve::uncrush(post, md, &postsolve::PreSol { x: &x, dj: &dj, st: &vs }, None);
            (presolve_status(md, p, code), u.x[..md.num_vars()].to_vec())
        }
        None => (code, x[..md.num_vars()].to_vec()),
    };
    let obj = objective(md, &x);
    let work = pre.as_ref().map_or(0.0, |(_, post)| post.work) * 1e-9;
    (h.log)(&format!("\nSolved in {iter} iterations and {:.2} seconds ({work:.2} work units)\n", t0.elapsed().as_secs_f64()));
    (h.log)(&status_line(status, obj));
    // shortcut: simplex work is not counted yet (P2 of G1); Work holds the presolve part only
    Outcome { status, obj, x, iter_count: iter, trace, work, ..Default::default() }
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

/// `2bb770` / `34f033` without crossover: presolve → barrier or PDHG → uncrush → status.
fn interior(md: &Model, p: &Params, alg: Alg, h: &mut Hooks, t0: Instant) -> Outcome {
    let pre = presolve_logged(md, p, &mut *h.log);
    let q = pre.as_ref().map_or(md, |(q, _)| q);
    let flipped = flip_upper_only(q);
    let qe = flipped.as_ref().map_or(q, |(f, _)| f);
    let mut out = Outcome::default();
    let (status, xq) = match alg {
        Alg::Barrier => {
            let bp = barrier::Params {
                bar_conv_tol: p.get("BarConvTol"),
                bar_iter_limit: p.get("BarIterLimit") as i64,
                bar_correctors: p.int("BarCorrectors"),
            };
            let r = barrier::solve(qe, &bp, &mut *h.barrier);
            out.bar_iter_count = r.bar_iters;
            out.work = pre.as_ref().map_or(0.0, |(_, post)| post.work) * 1e-9 + r.work;
            (r.status, r.x)
        }
        Alg::Pdhg => {
            (h.log)("\nStart PDHG using 1 threads\n\n");
            let pp = pdhg::Params {
                iter_limit: p.get("PDHGIterLimit"),
                rel_tol: p.get("PDHGRelTol"),
                abs_tol: p.get("PDHGAbsTol"),
                conv_tol: p.get("PDHGConvTol"),
                work_limit: p.get("WorkLimit"),
                work0: pre.as_ref().map_or(0.0, |(_, post)| post.work),
            };
            let r = pdhg::solve(qe, &pp, &mut *h.pdhg);
            out.pdhg_iter_count = r.iters;
            out.work = r.work / 1e9;
            (r.status, r.x)
        }
    };
    let mut xq = xq;
    for &j in flipped.as_ref().map_or(&[][..], |(_, c)| c) {
        xq[j] = -xq[j];
    }
    // shortcut: `36578f` (primal 363164, dual 365159, big-uncrush-violation check) is not ported; x goes through
    // `363daa` with slacks and a dummy all-basic status, which recovers the primal values; duals are not reported.
    let x = match &pre {
        Some((_, post)) => {
            let mut act = vec![0.0; q.num_constrs()];
            for (j, &xj) in xq.iter().enumerate() {
                let b = q.col_beg[j] as usize;
                for k in b..b + q.col_len[j] as usize {
                    act[q.col_ind[k] as usize] += q.col_val[k] * xj;
                }
            }
            let mut xs = xq.clone();
            xs.extend((0..act.len()).map(|i| if q.row_sense[i] == b'>' { act[i] - q.rhs[i] } else { q.rhs[i] - act[i] }));
            let (dj, st) = (vec![0.0; xs.len()], vec![0; xs.len()]);
            postsolve::uncrush(post, md, &postsolve::PreSol { x: &xs, dj: &dj, st: &st }, None).x[..md.num_vars()].to_vec()
        }
        None => xq,
    };
    let status = if pre.is_some() { presolve_status(md, p, status) } else { status };
    let obj = objective(md, &x);
    let name = if alg == Alg::Barrier { "Barrier" } else { "PDHG" };
    let it = if alg == Alg::Barrier { out.bar_iter_count as f64 } else { out.pdhg_iter_count as f64 };
    let secs = t0.elapsed().as_secs_f64();
    if status == 2 {
        (h.log)(&format!("\n{name} solved model in {it:.0} iterations and {secs:.2} seconds ({:.2} work units)\n", out.work));
        (h.log)(&format!("Optimal objective {}\n", c_exp(obj, 8)));
    } else {
        (h.log)(&format!("\n{name} performed {it:.0} iterations in {secs:.2} seconds ({:.2} work units)\n", out.work));
        (h.log)(&status_line(status, obj));
    }
    Outcome { status, obj, x, ..out }
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
