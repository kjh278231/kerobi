//! Conservative linear row/objective propagation after incumbent submission.
//! shortcut: generic activity propagation, not native 6c5042's cached inference/conflict engine.
use std::collections::VecDeque;
use rsolver_model::Model;
use crate::Params;

#[derive(Debug, Default)]
pub struct Result {
    pub conflict: bool,
    pub changes: usize,
    /// False when the work budget stopped propagation; the bounds remain valid.
    pub complete: bool,
}

#[derive(Clone, Copy, Default)]
struct Activity { sum: f64, magnitude: f64, infinite: usize }
impl Activity {
    fn add(self, c: f64, b: f64) -> Self {
        if b.abs() >= 1e30 { Self { infinite: self.infinite + 1, ..self } }
        else { Self { sum: self.sum + c * b, magnitude: self.magnitude + (c * b).abs(), ..self } }
    }
    fn plus(self, b: Self) -> Self {
        Self { sum: self.sum + b.sum, magnitude: self.magnitude + b.magnitude, infinite: self.infinite + b.infinite }
    }
}

/// Enforce all rows and an optional minimization objective upper bound. Changes only domain bounds.
pub fn run(model: &mut Model, p: &Params, cutoff: Option<f64>) -> Result {
    let n = model.obj.len();
    let mut result = Result::default();
    for j in 0..n {
        if matches!(model.vtype[j], b'B' | b'I') {
            model.lb[j] = (model.lb[j] - p.intfeastol).ceil();
            model.ub[j] = (model.ub[j] + p.intfeastol).floor();
        }
        if model.lb[j] > model.ub[j] { result.conflict = true; result.complete = true; return result; }
    }
    let mut rows: Vec<Vec<(usize, f64)>> = vec![Vec::new(); model.rhs.len()];
    for j in 0..n {
        let start = model.col_beg[j] as usize;
        for k in start..start + model.col_len[j] as usize {
            let row = &mut rows[model.col_ind[k] as usize];
            if let Some((_, c)) = row.last_mut().filter(|(last, _)| *last == j) {
                *c += model.col_val[k];
            } else { row.push((j, model.col_val[k])); }
        }
    }
    let mut rhs = Vec::new();
    let mut inequalities = Vec::new();
    for (i, row) in rows.into_iter().enumerate() {
        if model.row_sense[i] != b'>' {
            inequalities.push(row.clone()); rhs.push(model.rhs[i]);
        }
        if model.row_sense[i] != b'<' {
            inequalities.push(row.iter().map(|&(j, c)| (j, -c)).collect()); rhs.push(-model.rhs[i]);
        }
    }
    if let Some(cutoff) = cutoff.filter(|v| *v < 1e100) {
        let sense = model.sense as f64;
        inequalities.push(model.obj.iter().enumerate().map(|(j, &c)| (j, sense * c)).collect());
        rhs.push(cutoff - sense * model.objcon);
    }
    for row in &mut inequalities { row.retain(|&(_, c)| c != 0.0); }
    let mut incident = vec![Vec::new(); n];
    for (i, row) in inequalities.iter().enumerate() {
        for &(j, _) in row { incident[j].push(i); }
    }
    let mut queued = vec![true; inequalities.len()];
    let mut queue: VecDeque<_> = (0..inequalities.len()).collect();
    // Cyclic continuous rows can converge asymptotically; a budget never proves infeasibility.
    let budget = 100 * (inequalities.len() + n + 1);
    for _ in 0..budget {
        let Some(i) = queue.pop_front() else { result.complete = true; return result; };
        queued[i] = false;
        let row = &inequalities[i];
        let endpoint = |j: usize, c: f64| if c > 0.0 { model.lb[j] } else { model.ub[j] };
        let mut prefix = vec![Activity::default(); row.len() + 1];
        for (k, &(j, c)) in row.iter().enumerate() { prefix[k + 1] = prefix[k].add(c, endpoint(j, c)); }
        let error = (prefix[row.len()].magnitude + rhs[i].abs()) * f64::EPSILON * (row.len() as f64 + 2.0);
        let slack = p.feastol + error;
        let total = prefix[row.len()];
        if total.infinite == 0 && total.sum > rhs[i] + slack { result.conflict = true; result.complete = true; return result; }
        let mut suffix = Activity::default();
        for (k, &(j, c)) in row.iter().enumerate().rev() {
            // Keep suffix on the same snapshot as prefix.
            let old = if c > 0.0 { model.lb[j] } else { model.ub[j] };
            let other = prefix[k].plus(suffix);
            suffix = suffix.add(c, old);
            if other.infinite != 0 { continue; }
            let mut bound = (rhs[i] - other.sum + slack) / c;
            if !bound.is_finite() || bound.abs() >= 1e30 { continue; }
            let integer = matches!(model.vtype[j], b'B' | b'I');
            if integer { bound = if c > 0.0 { (bound + p.intfeastol).floor() } else { (bound - p.intfeastol).ceil() }; }
            let changed = if c > 0.0 {
                if bound < model.lb[j] - p.feastol { result.conflict = true; result.complete = true; return result; }
                bound = bound.max(model.lb[j]);
                if bound < model.ub[j] - p.feastol { model.ub[j] = bound; true } else { false }
            } else {
                if bound > model.ub[j] + p.feastol { result.conflict = true; result.complete = true; return result; }
                bound = bound.min(model.ub[j]);
                if bound > model.lb[j] + p.feastol { model.lb[j] = bound; true } else { false }
            };
            if changed {
                result.changes += 1;
                for &r in &incident[j] {
                    if !queued[r] { queued[r] = true; queue.push_back(r); }
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn chain() -> Model {
        Model { sense: 1, obj: vec![1.0, 0.0, 0.0], lb: vec![0.0; 3], ub: vec![10.0; 3],
            vtype: b"III".to_vec(), col_beg: vec![0, 1, 3], col_len: vec![1, 2, 1],
            col_ind: vec![0, 0, 1, 1], col_val: vec![1.0, -1.0, 1.0, -1.0],
            row_sense: b"==".to_vec(), rhs: vec![0.0, 0.0], ..Model::default() }
    }
    #[test]
    fn objective_reaches_all_rows_and_detects_exhaustion() {
        let mut m = chain();
        let r = run(&mut m, &Params::default(), Some(3.0));
        assert!(r.complete && !r.conflict && r.changes >= 3);
        assert_eq!(m.ub, vec![3.0; 3]);
        m.lb[2] = 4.0;
        assert!(run(&mut m, &Params::default(), Some(3.0)).conflict);
    }
    #[test]
    fn infinite_endpoints_negative_coefficients_and_empty_rows() {
        let mut m = chain();
        m.lb = vec![-1e30; 3]; m.ub = vec![1e30; 3];
        m.obj[0] = -1.0;
        let r = run(&mut m, &Params::default(), Some(-3.0));
        assert!(r.complete && !r.conflict);
        assert_eq!(m.lb, vec![3.0; 3]);
        assert_eq!(m.ub, vec![1e30; 3]);
        m.rhs.push(-1.0); m.row_sense.push(b'<');
        assert!(run(&mut m, &Params::default(), None).conflict);
    }
    #[test]
    fn duplicate_entries_and_tolerated_continuous_solution() {
        let mut m = Model { sense: 1, obj: vec![0.0], lb: vec![0.0], ub: vec![10.0], vtype: b"C".to_vec(),
            col_beg: vec![0], col_len: vec![2], col_ind: vec![0,0], col_val: vec![1.0,-1.0],
            row_sense: b"=".to_vec(), rhs: vec![0.0], ..Model::default() };
        assert!(run(&mut m, &Params::default(), None).complete);
        assert_eq!(m.ub, vec![10.0]);
        m.col_val = vec![1.0,1.0]; m.lb[0] = 5e-7;
        assert!(!run(&mut m, &Params::default(), None).conflict);
    }
    #[test]
    fn propagation_preserves_every_feasible_point_on_small_integer_models() {
        let mut seed = 17u64;
        let mut next = || { seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1); (seed >> 32) as i32 & 0x7fffffff };
        for _ in 0..256 {
            let coefficients: Vec<f64> = (0..9).map(|_| (next() % 7 - 3) as f64).collect();
            let obj: Vec<f64> = (0..3).map(|_| (next() % 5 - 2) as f64).collect();
            let rhs: Vec<f64> = (0..3).map(|_| (next() % 13 - 4) as f64).collect();
            let senses: Vec<u8> = (0..3).map(|_| b"<>="[(next() % 3) as usize]).collect();
            let cutoff = (next() % 13 - 4) as f64;
            let mut m = Model { sense: 1, obj, lb: vec![0.0; 3], ub: vec![3.0; 3], vtype: b"III".to_vec(),
                col_beg: vec![0,3,6], col_len: vec![3; 3], col_ind: vec![0,1,2,0,1,2,0,1,2],
                col_val: coefficients, rhs, row_sense: senses, ..Model::default() };
            let before = m.clone();
            let result = run(&mut m, &Params::default(), Some(cutoff));
            for a in 0..4 { for b in 0..4 { for c in 0..4 {
                let x = [a as f64, b as f64, c as f64];
                if before.obj.iter().zip(x).map(|(c,v)| c*v).sum::<f64>() > cutoff { continue; }
                if (0..3).any(|i| {
                    let activity: f64 = (0..3).map(|j| before.col_val[j*3+i]*x[j]).sum();
                    match before.row_sense[i] { b'<' => activity > before.rhs[i], b'>' => activity < before.rhs[i], _ => activity != before.rhs[i] }
                }) { continue; }
                assert!(!result.conflict, "feasible point {x:?} removed from {before:?}");
                assert!((0..3).all(|j| m.lb[j] <= x[j] && x[j] <= m.ub[j]), "feasible point {x:?} removed");
            } } }
        }
    }

}
