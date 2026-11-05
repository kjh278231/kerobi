//! Depth-first branch & bound on LP relaxations (most-fractional branching).
//! shortcut: no presolve, cuts, heuristics or pseudocosts (notes 12, 15, 16);
//! add them when a benchmark instance from re/bench stalls.
use crate::{lp, Problem, Solution, Status, INF};

const INT_TOL: f64 = 1e-6;

pub fn branch_and_bound(p: &Problem) -> Solution {
    let n = p.obj.len();
    let mut best = Solution { status: Status::Infeasible, obj_val: INF, x: vec![0.0; n] };
    let mut stack = vec![(p.lb.clone(), p.ub.clone())];
    let mut root_unbounded = false;
    while let Some((lb, ub)) = stack.pop() {
        let node = Problem { lb, ub, ..p.clone() };
        let rel = lp::solve(&node);
        match rel.status {
            Status::Infeasible => continue,
            Status::Unbounded => {
                root_unbounded = true; // shortcut: unbounded relaxation reported as unbounded MIP
                break;
            }
            Status::Optimal => {}
        }
        if rel.obj_val >= best.obj_val - 1e-9 {
            continue;
        }
        let frac = (0..n)
            .filter(|&j| p.integer[j])
            .map(|j| (j, (rel.x[j] - rel.x[j].round()).abs()))
            .filter(|&(_, f)| f > INT_TOL)
            .max_by(|a, b| a.1.total_cmp(&b.1));
        match frac {
            None => {
                let mut x = rel.x;
                (0..n).filter(|&j| p.integer[j]).for_each(|j| x[j] = x[j].round());
                best = Solution { status: Status::Optimal, obj_val: rel.obj_val, x };
            }
            Some((j, _)) => {
                let v = rel.x[j];
                let (mut lo, mut hi) = (node.lb.clone(), node.ub.clone());
                lo[j] = v.ceil();
                hi[j] = v.floor();
                stack.push((lo, node.ub.clone()));
                stack.push((node.lb.clone(), hi));
            }
        }
    }
    if root_unbounded {
        best.status = Status::Unbounded;
        best.obj_val = -INF;
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knapsack() {
        // max 5a + 4b + 3c s.t. 2a + 3b + c <= 5, 4a + b + 2c <= 11, 3a + 4b + 2c <= 8, binary
        let p = Problem {
            obj: vec![-5.0, -4.0, -3.0],
            rows: vec![
                (vec![(0, 2.0), (1, 3.0), (2, 1.0)], -INF, 5.0),
                (vec![(0, 4.0), (1, 1.0), (2, 2.0)], -INF, 11.0),
                (vec![(0, 3.0), (1, 4.0), (2, 2.0)], -INF, 8.0),
            ],
            lb: vec![0.0; 3],
            ub: vec![1.0; 3],
            integer: vec![true; 3],
        };
        let s = branch_and_bound(&p);
        assert_eq!(s.status, Status::Optimal);
        assert!((s.obj_val + 9.0).abs() < 1e-7, "{s:?}");
        assert_eq!(s.x, vec![1.0, 1.0, 0.0]);
    }
}
