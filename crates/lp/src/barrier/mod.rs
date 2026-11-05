//! 05 §2 Barrier (Method=2) body, ported against the barrier-state oracle `tools/bar_oracle.py`
//! (docs/m3_barrier.md, docs/m3_ipm.md, docs/m3_rust.md).
//! Flow: ordering (`analyze`) → internal LP (`setup`) → work estimates (`2e7f38`) → IPM loop (`ipm`).
//! shortcut: Presolve=0 and Crossover=0 are assumed (no presolve, no crossover; the last iterate is the answer).

pub mod chol;
pub mod ipm;
pub mod ldl;
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
    /// B+0x198 = ‖r_p‖₂/τ (BARRIER_PRIMINF)
    pub priminf: f64,
    /// B+0x1a8 = ‖r_d‖₂/τ (BARRIER_DUALINF)
    pub dualinf: f64,
    /// B+0x1f0 (BARRIER_COMPL)
    pub compl: f64,
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
}

/// Solve the LP `m` with the barrier; `cb` runs once per iteration (BARRIER callback, before the termination test).
pub fn solve(m: &Model, p: &Params, cb: &mut dyn FnMut(&IterInfo)) -> BarResult {
    let sym = ordering::analyze(m);
    let lp = setup::lp_from_model(m, &sym.perm, p.bar_conv_tol);
    let (f160, f168) = work_estimates(&sym, m.num_constrs(), cpu_divisor());
    ipm::run(&lp, f160, f168, p, cb)
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
