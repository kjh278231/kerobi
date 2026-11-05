//! 05 §2 Barrier (Method=2) body, ported against the barrier-state oracle `tools/bar_oracle.py`
//! (docs/m3_barrier.md, docs/m3_ipm.md, docs/m3_rust.md).
//! Flow: ordering (`analyze`) → internal LP (`setup`) → work estimates (`2e7f38`) → IPM loop (`ipm`).
//! shortcut: Presolve=0 and Crossover=0 are assumed (no presolve, no crossover; the last iterate is the answer).

pub mod chol;
pub mod f80;
pub mod ipm;
pub mod ldl;
pub mod mkl;
pub mod normal;
pub mod ordering;
pub mod setup;

use rsolver_model::Model;

/// Parameters the barrier reads (Gurobi names: BarConvTol, BarIterLimit, BarCorrectors).
#[derive(Clone, Debug)]
pub struct Params {
    pub bar_conv_tol: f64,
    pub bar_iter_limit: i64,
    /// −1 = automatic (work-timer rule)
    pub bar_correctors: i32,
}

impl Default for Params {
    fn default() -> Self {
        Params { bar_conv_tol: 1e-8, bar_iter_limit: 1000, bar_correctors: -1 }
    }
}

/// BARRIER callback values of one iteration (objectives in the model's space and sense).
#[derive(Clone, Debug)]
pub struct IterInfo {
    pub iter: usize,
    pub pobj: f64,
    pub dobj: f64,
    /// √(B+0x198² + B+0x1b8²) = √(‖r_p‖₂² + ‖r_u‖₂²)/τ (BARRIER_PRIMINF; 64/64 bitwise on the iteration-0 oracle)
    pub priminf: f64,
    /// B+0x1a8 = ‖r_d‖₂/τ (BARRIER_DUALINF)
    pub dualinf: f64,
    /// B+0x1f0 (BARRIER_COMPL)
    pub compl: f64,
    /// log columns: B+0x1d8 = max(‖r_p‖∞, ‖r_u‖∞)/τ, B+0x1e0 (dual residual), B+0x1f8 = μ
    pub log_pinf: f64,
    pub log_dinf: f64,
    pub mu: f64,
}

#[derive(Clone, Debug)]
pub struct BarResult {
    /// Gurobi status code: 2 optimal, 7 iteration limit, 13 suboptimal
    pub status: i32,
    pub obj: f64,
    pub bar_iters: usize,
    /// column values in the model's space
    pub x: Vec<f64>,
    pub work: f64,
    /// slack, reduced costs and row duals in the model's space (`7a069`), rows as stored (`>` negated)
    pub slack: Vec<f64>,
    pub d: Vec<f64>,
    pub y: Vec<f64>,
}

/// Solve the LP `m` with the barrier; `cb` runs once per iteration (BARRIER callback, before the termination test).
pub fn solve(m: &Model, p: &Params, cb: &mut dyn FnMut(&IterInfo)) -> BarResult {
    solve_logged(m, p, cb, &mut |_| {}, std::time::Instant::now())
}

/// `solve` with the barrier's own log (`2bd9ab`/`2e8590`/`2bf1ea`): "Ordering time", "Barrier statistics", the
/// iteration table (one row per iteration). `start` is the optimize's start for the time column.
pub fn solve_logged(m: &Model, p: &Params, cb: &mut dyn FnMut(&IterInfo), log: &mut dyn FnMut(&str),
                    start: std::time::Instant) -> BarResult {
    let sym = ordering::analyze(m);
    log(&format!("Ordering time: {:.2}s\n", start.elapsed().as_secs_f64()));
    let lp = setup::lp_from_model(m, &sym.perm, p.bar_conv_tol);
    log(&statistics(&sym, &lp, m));
    let (f160, f168) = work_estimates(&sym, m.num_constrs(), cpu_divisor());
    log("\n                  Objective                Residual\n");
    log("Iter       Primal          Dual         Primal    Dual     Compl     Time\n");
    let mut row = |i: &IterInfo| {
        cb(i);
        // `2bf1ea` L437–442; the '*' rows (B+0x690 small, homogeneous) do not occur without the homogeneous mode
        log(&format!(" {:3}  {:>15} {:>15}  {} {}  {}  {:4.0}s\n", i.iter, e(i.pobj, 8), e(i.dobj, 8), e(i.log_pinf, 2),
                     e(i.log_dinf, 2), e(i.mu, 2), start.elapsed().as_secs_f64().floor()));
    };
    ipm::run(&lp, &sym, f160, f168, p, &mut row)
}

fn e(x: f64, prec: usize) -> String {
    rsolver_model::mps_write::c_exp(x, prec)
}

/// `2e8590`: the "Barrier statistics" block. The memory note comes from `2e44ba`'s estimate for factorizations of at
/// most 1e8 operations (no timing run): 4·(row indices of L) + 8·Factor NZ + 400·(m + n) bytes, rounded to one
/// significant digit in MB. shortcut: larger factorizations time a test kernel there; not ported.
fn statistics(s: &ordering::Symbolic, lp: &setup::Lp, m: &Model) -> String {
    let free = lp.t[..lp.n].iter().filter(|&&t| t == 0).count();
    let mut o = String::from("\nBarrier statistics:\n");
    if free > 0 {
        o += &format!(" Free vars  : {free}\n");
    }
    o += &format!(" AA' NZ     : {}\n", e(s.aat_nnz as f64 * 0.5, 3));
    let bytes = s.row_idx.len() as f64 * 4.0 + s.nnz_l as f64 * 8.0 + (m.num_constrs() + m.num_vars()) as f64 * 400.0;
    let mb = round1(bytes / 1e6);
    o += &if mb >= 1000.0 {
        format!(" Factor NZ  : {} (roughly {:.1} GB of memory)\n", e(s.nnz_l as f64, 3), mb / 1000.0)
    } else if mb < 1.0 {
        format!(" Factor NZ  : {}\n", e(s.nnz_l as f64, 3))
    } else {
        format!(" Factor NZ  : {} (roughly {mb:.0} MB of memory)\n", e(s.nnz_l as f64, 3))
    };
    o += &format!(" Factor Ops : {} (less than 1 second per iteration)\n", e(s.ops, 3));
    o + " Threads    : 1\n"
}

/// `2e8590`'s rounding: v to one significant digit (the largest power of ten p with |round(v/10^p)·10^p − v| < 0.1 v,
/// tried from the top), then to the nearest integer.
fn round1(v: f64) -> f64 {
    let v = if v == 0.0 { 1e-10 } else { v };
    let mut k = (v.ln() / 10f64.ln()) as i32 + 1;
    let mut r = v;
    while k > 0 {
        let t = (v / 10f64.powi(k) + 0.5).floor() * 10f64.powi(k);
        if (t - v).abs() < v * 0.1 {
            r = t;
            break;
        }
        k -= 1;
        r = v;
    }
    (r + 0.5).floor()
}

/// `2e7f38`: (F+0x160 = 2 nnz(L), F+0x168 = factor work estimate) from the symbolic statistics.
/// shortcut: F+0x50/F+0x58 (dense-column terms) are 0 and Threads = 1 (pow(threads, −0.9) = 1); port `2fd14c`
/// with dense columns.
fn work_estimates(s: &ordering::Symbolic, rows: usize, cpu: f64) -> (f64, f64) {
    let (f48, f50, f58, f60, f68) = (s.ops_aat, 0.0, 0.0, s.ops, s.ops_small);
    let nnz_l = s.nnz_l as f64;
    let f170 = (nnz_l * 8.0 + f68 / 5.0) + (2.0 * f48) * (1.0 - f50);
    let f170 = ((f60 - f68) + f48 * f50) / cpu + f170;
    (2.0 * nnz_l, (f170 + f58) + rows as f64 * 75.0)
}

/// `a80670` → `2e7f38` divisor from raw CPUID bits (no OS check): no AVX 10, AVX 18, AVX2 30, AVX2 + AVX-512DQ 50.
fn cpu_divisor() -> f64 {
    use std::arch::x86_64::{__cpuid, __cpuid_count};
    #[allow(unused_unsafe)] // __cpuid is a safe fn on newer toolchains
    let (ecx1, ebx7) = unsafe { (__cpuid(1).ecx, __cpuid_count(7, 0).ebx) };
    if ecx1 & 0x1000_0000 == 0 {
        10.0
    } else if ebx7 & 0x20 == 0 {
        18.0
    } else if ebx7 & 0x2_0000 != 0 {
        50.0
    } else {
        30.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn netlib(name: &str) -> Model {
        let p = format!("{}/../../../re/bench/data/netlib/{name}.mps.gz", env!("CARGO_MANIFEST_DIR"));
        rsolver_model::mps::read_file(&p).unwrap()
    }

    /// F+0x168 of the iteration-0 oracle (bench/oracle/bar/it0, machine with AVX2 and no AVX-512: divisor 30).
    #[test]
    fn work_estimate_matches_oracle() {
        for (nm, f160, f168) in [
            ("afiro", 436.0, 4330.533333333333),
            ("25fv47", 73582.0, 630201.7),
            ("perold", 57596.0, 443172.30000000005),
            ("fit1p", 393756.0, 11699291.333333334),
        ] {
            let m = netlib(nm);
            assert_eq!(work_estimates(&ordering::analyze(&m), m.num_constrs(), 30.0), (f160, f168), "{nm}");
        }
    }

    /// Gurobi: 7 iterations, ObjVal −464.7531428098293 (bench/gurobi_bar_nopre/afiro).
    #[test]
    fn afiro() {
        let mut n = 0;
        let r = solve(&netlib("afiro"), &Params::default(), &mut |_| n += 1);
        assert_eq!((r.status, r.bar_iters, n), (2, 7, 8));
        assert!((r.obj + 464.7531428098293).abs() < 1e-6 * 464.75, "{}", r.obj);
        assert_eq!(r.x.len(), 32);
    }
}
