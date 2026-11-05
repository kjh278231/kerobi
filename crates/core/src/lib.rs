//! Pure-Rust LP/MIP core. Layout follows re/notes/00_overview.md
//! (driver -> LP -> branch & bound); algorithms are textbook, not Gurobi's code.
pub mod lp;
pub mod mip;

/// One constraint: (sparse coefficients, row lower, row upper).
pub type Row = (Vec<(usize, f64)>, f64, f64);

/// Minimise c·x subject to lo <= A x <= hi, lb <= x <= ub. Use ±INF for free sides.
#[derive(Clone, Debug, Default)]
pub struct Problem {
    pub obj: Vec<f64>,
    /// rows[i] = (sparse coefficients, row lower, row upper)
    pub rows: Vec<Row>,
    pub lb: Vec<f64>,
    pub ub: Vec<f64>,
    pub integer: Vec<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Optimal,
    Infeasible,
    Unbounded,
}

#[derive(Clone, Debug)]
pub struct Solution {
    pub status: Status,
    pub obj_val: f64,
    pub x: Vec<f64>,
}

pub const INF: f64 = f64::INFINITY;

/// Driver: continuous problems go straight to LP, anything with integers to B&B.
pub fn solve(p: &Problem) -> Solution {
    if p.integer.iter().any(|&b| b) {
        mip::branch_and_bound(p)
    } else {
        lp::solve(p)
    }
}
