//! MIP root LP relaxation (docs/m2_root.md): `5bbf7f` → `5daa82` (Mip::new) → `64da9f` (root) → `69f0bd` (LP holder,
//! method) → `5ce67b` → `5ccea5` → `cda5b(lp, method, no basis)`. On all 37 miplib_small models with a root the root
//! LP is a fresh dual simplex (method 1, no workspace, no warm basis) on the presolved model with the domain store's
//! bounds and the column types kept (spx+0xb8). Equality-based domain initialization and optional validated
//! incumbent bounds are applied before the LP; root heuristics, further propagation, cuts and branching remain.
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

/// Initialize implied integer domains from equality rows (`66bdf6`). An equality with one continuous
/// column implies integrality when its RHS and every coefficient are integer multiples of that column's
/// coefficient. Newly integral columns can expose further such equalities.
fn initialize_domain(model: &Model, p: &Params) -> Model {
    let mut domain = model.clone();
    let mut rows = vec![Vec::new(); model.rhs.len()];
    for j in 0..model.obj.len() {
        let start = model.col_beg[j] as usize;
        for k in start..start + model.col_len[j] as usize {
            rows[model.col_ind[k] as usize].push((j, model.col_val[k]));
        }
    }
    let integral = |x: f64| x.is_finite() && x - (x + 1e-10).floor() <= 1e-10;
    loop {
        let mut changed = false;
        for (i, row) in rows.iter().enumerate() {
            if model.row_sense[i] != b'=' { continue; }
            let mut continuous = row.iter().filter(|&&(j, _)| domain.vtype[j] == b'C');
            let Some(&(j, a)) = continuous.next() else { continue };
            if continuous.next().is_some() || a == 0.0 || !integral(model.rhs[i] / a)
                || row.iter().any(|&(_, v)| !integral(v / a)) { continue; }
            if domain.lb[j] > -1e30 { domain.lb[j] = (domain.lb[j] - p.intfeastol).ceil(); }
            if domain.ub[j] < 1e30 { domain.ub[j] = (domain.ub[j] + p.intfeastol).floor(); }
            domain.vtype[j] = if domain.lb[j] == 0.0 && domain.ub[j] == 1.0 { b'B' } else { b'I' };
            changed = true;
        }
        if !changed { break; }
    }
    domain
}

/// Apply an incumbent in presolved coordinates. Feasibility is checked before its objective can restrict the domain.
fn apply_incumbent(model: &mut Model, p: &Params, x: &[f64]) -> Result<(), &'static str> {
    if x.len() != model.obj.len() { return Err("incumbent length differs from model"); }
    let mut activity = vec![0.0; model.rhs.len()];
    for (j, &v) in x.iter().enumerate() {
        if !v.is_finite() || v < model.lb[j] - p.feastol || v > model.ub[j] + p.feastol
            || (matches!(model.vtype[j], b'B' | b'I') && (v - v.round()).abs() > p.intfeastol) {
            return Err("incumbent violates a column domain");
        }
        let start = model.col_beg[j] as usize;
        for k in start..start + model.col_len[j] as usize {
            activity[model.col_ind[k] as usize] += model.col_val[k] * v;
        }
    }
    for (i, &a) in activity.iter().enumerate() {
        let violation = match model.row_sense[i] {
            b'<' => a - model.rhs[i], b'>' => model.rhs[i] - a, _ => (a - model.rhs[i]).abs(),
        };
        if !a.is_finite() || violation > p.feastol { return Err("incumbent violates a row"); }
    }
    let sense = model.sense as f64;
    let objective = sense * (model.objcon + model.obj.iter().zip(x).map(|(c, v)| c * v).sum::<f64>());
    if !objective.is_finite() { return Err("incumbent objective is not finite"); }
    // 6f45ed: the cutoff seeks an improvement over the incumbent.
    let cutoff = objective - (objective.abs() + 1.0) * 1e-10;
    let n = x.len();
    let mut prefix = vec![(0.0, 0.0, 0usize); n + 1];
    for j in 0..n {
        let c = sense * model.obj[j];
        let b = if c > 0.0 { model.lb[j] } else { model.ub[j] };
        let (sum, magnitude, infinite) = prefix[j];
        prefix[j + 1] = if c == 0.0 { prefix[j] } else if b.abs() >= 1e30 {
            (sum, magnitude, infinite + 1)
        } else { (sum + c * b, magnitude + (c * b).abs(), infinite) };
    }
    let mut suffix = (0.0, 0.0, 0usize);
    for j in (0..n).rev() {
        let c = sense * model.obj[j];
        let b = if c > 0.0 { model.lb[j] } else { model.ub[j] };
        let (sum, magnitude, infinite) = prefix[j];
        if c != 0.0 && infinite + suffix.2 == 0 {
            let min_other = sense * model.objcon + (sum + suffix.0);
            let error = (magnitude + suffix.1 + model.objcon.abs()) * f64::EPSILON * (n as f64 + 2.0);
            let bound = (cutoff - min_other + error) / c;
            if bound.is_finite() {
                if c > 0.0 {
                    let bound = if matches!(model.vtype[j], b'B' | b'I') { (bound + p.intfeastol).floor() } else { bound };
                    if bound >= model.lb[j] - p.feastol { model.ub[j] = model.ub[j].min(bound.max(model.lb[j])); }
                } else {
                    let bound = if matches!(model.vtype[j], b'B' | b'I') { (bound - p.intfeastol).ceil() } else { bound };
                    if bound <= model.ub[j] + p.feastol { model.lb[j] = model.lb[j].max(bound.min(model.ub[j])); }
                }
            }
        }
        if c != 0.0 {
            if b.abs() >= 1e30 { suffix.2 += 1; }
            else { suffix.0 += c * b; suffix.1 += (c * b).abs(); }
        }
    }
    Ok(())
}

/// Solve the root LP of a presolved MIP (`5ce67b` at the root, `local_110` = root method, no warm start). None when
/// presolve removed every column ("All rows and columns removed": no root relaxation line).
// shortcut: 66bdf6 inequality inference, row propagation and automatic heuristics remain unported; extend with MIP state.
// shortcut: methods other than 0/1 (concurrent 4 for nnz ≥ 1e5 with m > 999, n > 9999; barrier 2) run the dual;
// none of miplib_small takes them; port the 98b671 branch of 5ccea5 when a model in the set does.
pub fn solve_root(presolved: &Model, p: &Params) -> Option<RootLp> {
    solve_root_with_incumbent(presolved, p, None).expect("no incumbent to validate")
}

/// Construct the LP domain using an optional feasible incumbent in presolved coordinates.
pub fn root_domain(presolved: &Model, p: &Params, incumbent: Option<&[f64]>) -> Result<Model, &'static str> {
    let mut domain = initialize_domain(presolved, p);
    if let Some(x) = incumbent { apply_incumbent(&mut domain, p, x)?; }
    // 5ca999 uses the separate LP type array (+0x660), not 66bdf6's domain types (+0x650).
    domain.vtype.clone_from(&presolved.vtype);
    Ok(domain)
}

/// Solve using an optional feasible incumbent in presolved column coordinates.
pub fn solve_root_with_incumbent(presolved: &Model, p: &Params, incumbent: Option<&[f64]>) -> Result<Option<RootLp>, &'static str> {
    let n = presolved.obj.len();
    if n == 0 {
        return Ok(None);
    }
    let (method, _) = root_method(p, presolved.rhs.len() as i32, n as i32, presolved.num_nz() as i64);
    let domain = root_domain(presolved, p, incumbent)?;
    let lp = solve_model_opts(&domain, if method == 0 { 0 } else { 1 }, u64::MAX, false);
    Ok(Some(RootLp { method, lp }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_infers_integer_equalities_without_changing_input() {
        let m = Model {
            obj: vec![0.0; 3], lb: vec![0.2, 0.1, 0.0], ub: vec![3.8, 3.9, 1.0],
            vtype: vec![b'C', b'C', b'B'],
            col_beg: vec![0, 1, 3], col_len: vec![1, 2, 1],
            col_ind: vec![0, 0, 1, 1], col_val: vec![2.0, 2.0, 1.0, 1.0],
            row_sense: vec![b'=', b'='], rhs: vec![4.0, 2.0], ..Model::default()
        };
        let d = initialize_domain(&m, &Params::default());
        assert_eq!(d.vtype, b"IIB");
        assert_eq!(d.lb, vec![1.0, 1.0, 0.0]);
        assert_eq!(d.ub, vec![3.0, 3.0, 1.0]);
        assert_eq!(m.vtype, b"CCB");
        let mut nonintegral = m.clone();
        nonintegral.rhs[1] = 2.5;
        assert_eq!(initialize_domain(&nonintegral, &Params::default()).vtype, b"CCB");
    }

    #[test]
    fn incumbent_bounds_respect_sense_and_validation() {
        let p = Params::default();
        let m = Model { sense: 1, objcon: 3.0, obj: vec![1.0, -2.0],
            lb: vec![0.0, 0.0], ub: vec![30.0, 10.0], vtype: vec![b'C', b'C'],
            col_beg: vec![0, 0], col_len: vec![0, 0], ..Model::default() };
        let mut d = m.clone();
        apply_incumbent(&mut d, &p, &[4.0, 2.0]).unwrap();
        assert!(d.ub[0] < 30.0 && d.ub[0] > 0.0);
        assert!(d.lb[1] > 0.0 && d.lb[1] < 2.0);
        assert!(apply_incumbent(&mut m.clone(), &p, &[4.0]).is_err());
        assert!(apply_incumbent(&mut m.clone(), &p, &[31.0, 2.0]).is_err());
        let mut row_model = Model { sense: 1, obj: vec![1.0], lb: vec![0.0], ub: vec![10.0],
            vtype: vec![b'C'], col_beg: vec![0], col_len: vec![1], col_ind: vec![0], col_val: vec![1.0],
            row_sense: vec![b'<'], rhs: vec![1.0], ..Model::default() };
        assert!(apply_incumbent(&mut row_model, &p, &[2.0]).is_err());
        let mut max = Model { sense: -1, obj: vec![1.0], lb: vec![0.0], ub: vec![10.0],
            vtype: vec![b'C'], col_beg: vec![0], col_len: vec![0], ..Model::default() };
        apply_incumbent(&mut max, &p, &[4.0]).unwrap();
        assert!(max.lb[0] > 4.0 && max.lb[0] < 4.00001);
        let mut optimal = Model { sense: 1, obj: vec![1.0], lb: vec![0.0], ub: vec![1.0],
            vtype: vec![b'B'], col_beg: vec![0], col_len: vec![0], ..Model::default() };
        apply_incumbent(&mut optimal, &p, &[0.0]).unwrap();
        assert_eq!(optimal.lb[0], 0.0);
        assert_eq!(optimal.ub[0], 0.0);
    }

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
