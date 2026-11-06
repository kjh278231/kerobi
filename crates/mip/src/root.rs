//! MIP root LP relaxation (docs/m2_root.md): `5bbf7f` → `5daa82` (Mip::new) → `64da9f` (root) → `69f0bd` (LP holder,
//! method) → `5ce67b` → `5ccea5` → `cda5b(lp, method, no basis)`. On all 37 miplib_small models with a root the root
//! LP is a fresh dual simplex (method 1, no workspace, no warm basis) on the presolved model with the domain store's
//! bounds and the column types kept (spx+0xb8). Stops here: bound changes before the root LP (`66bdf6` domain
//! init, `5ebfbc` incumbent, `8ac5fe` root heuristics), cut loop, heuristics, branching.
use rsolver_lp::simplex::{solve_model_opts, LpResult};
use rsolver_model::Model;

use crate::Params;

/// `69f0bd` for a MILP (no Q, `a7f0ad` == 0): the root LP method from Method, Threads and the model size (rows m,
/// columns n, nonzeros nnz of model+0xe8 +0x8/+0xc/+0x10). Returns (method, concurrent method).
pub fn root_method(p: &Params, m: i32, n: i32, nnz: i64) -> (i32, i32) {
    let cm = p.concurrent_method;
    match p.method {
        // explicit concurrent / deterministic concurrent: one thread makes it dual
        3..=5 => {
            if p.threads == 1 {
                (1, cm)
            } else {
                (p.method, cm)
            }
        }
        // automatic: large models go to concurrent (4), even with Threads=1
        -1 => {
            if nnz >= 500_000 && m > 999 && n > 19_999 {
                (4, cm)
            } else if nnz >= 100_000 && m > 999 && n > 9_999 {
                (4, 3)
            } else {
                (1, cm)
            }
        }
        _ => (p.method, cm),
    }
}

/// The root LP: the method `69f0bd` chose and the simplex result.
pub struct RootLp {
    pub method: i32,
    pub lp: LpResult,
}

/// Solve the root LP of a presolved MIP (`5ce67b` at the root, `local_110` = root method, no warm start). None when
/// presolve removed every column ("All rows and columns removed": no root relaxation line).
// shortcut: the LP bounds are the presolved model's, not the domain store's after 66bdf6/5ebfbc/8ac5fe (docs/m2_root.md
// §3); port with domain propagation (M3 propagate).
// shortcut: methods other than 0/1 (concurrent 4 for nnz ≥ 1e5 with m > 999, n > 9999; barrier 2) run the dual;
// none of miplib_small takes them; port the 98b671 branch of 5ccea5 when a model in the set does.
pub fn solve_root(presolved: &Model, p: &Params) -> Option<RootLp> {
    let n = presolved.obj.len();
    if n == 0 {
        return None;
    }
    let (method, _) = root_method(p, presolved.rhs.len() as i32, n as i32, presolved.num_nz() as i64);
    let lp = solve_model_opts(presolved, if method == 0 { 0 } else { 1 }, u64::MAX, false);
    Some(RootLp { method, lp })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_method_thresholds() {
        let mut p = Params::default();
        assert_eq!(root_method(&p, 1000, 10_000, 100_000), (4, 3));
        assert_eq!(root_method(&p, 1000, 10_000, 99_999), (1, -1));
        assert_eq!(root_method(&p, 999, 30_000, 600_000), (1, -1));
        assert_eq!(root_method(&p, 1000, 20_000, 500_000), (4, -1));
        assert_eq!(root_method(&p, 1000, 15_000, 500_000), (4, 3));
        p.method = 3;
        p.threads = 1;
        assert_eq!(root_method(&p, 10, 10, 10).0, 1);
        p.threads = 0;
        assert_eq!(root_method(&p, 10, 10, 10).0, 3);
        p.method = 0;
        assert_eq!(root_method(&p, 10, 10, 10).0, 0);
    }
}
