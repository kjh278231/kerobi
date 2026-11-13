//! Crossover dual values: `251b4f` (the reduced costs `d = c − Aᵀy` over the current basis, with the dual push's
//! correction list) and the dual infeasibility the crossover stores in S+0x2d8 after the pushes. The BTRAN in the
//! middle is the caller's (`btran` closure): the original wraps `221173` in `09e288` with no cache, a dense input and
//! `work += F200·1.0`. Contract and C references: docs/m3/crossover_dual_values.md.

use crate::crossover_btran::{btran, BtranFactor, BtranState};
use crate::crossover_ftran::Buffer;

/// The row-wise matrix S+0x1b0 (row start), S+0x1b8 (row length), S+0x1c0 (column), S+0x1c8 (value).
pub struct RowMatrix<'a> {
    pub start: &'a [usize],
    pub len: &'a [usize],
    pub cols: &'a [usize],
    pub vals: &'a [f64],
}

/// What `251b4f` reads.
pub struct DualInput<'a> {
    pub n: usize,
    pub m: usize,
    /// S+0x98 over the n columns and m slacks.
    pub cost: &'a [f64],
    /// S+0x168: basis position (≥ 0) or nonbasic status.
    pub vstat: &'a [i32],
    /// F+0x18: the variable at each basis position.
    pub header: &'a [i32],
    /// The correction list `p6[0..p3) ∪ p6[p4..p5)` in that order, each with its value
    /// (`p7[j]` for a column, `p8[j − n]` for a slack).
    pub corrections: &'a [(usize, f64)],
    /// S+0x3a8: rows whose |y| does not exceed it are skipped in `Aᵀy`.
    pub drop: f64,
    pub a: RowMatrix<'a>,
}

/// The BTRAN right-hand side: the basic costs in basis order, minus the corrections at their basis positions
/// (C 47–96).
pub fn btran_rhs(e: &DualInput) -> Vec<f64> {
    let mut cb: Vec<f64> = (0..e.m).map(|i| e.cost[e.header[i] as usize]).collect();
    for &(j, v) in e.corrections {
        let p = e.vstat[j] as usize;
        cb[p] -= v;
    }
    cb
}

/// `d` from the BTRAN output y (C 97–253, asm 0x1c9a39–0x1c9c5f): the column costs, `−y` on the slacks, `d −= y·a`
/// along the rows with `drop < |y|`, then the sign of the columns at their upper bound (−2) flipped.
pub fn reduced_costs(e: &DualInput, y: &[f64]) -> Vec<f64> {
    let (n, m) = (e.n, e.m);
    let mut d = vec![0.0; n + m];
    d[..n].copy_from_slice(&e.cost[..n]);
    for i in 0..m {
        d[n + i] = -y[i];
        if y[i].abs() > e.drop {
            let s = e.a.start[i];
            for k in s..s + e.a.len[i] {
                let c = e.a.cols[k];
                d[c] = -(y[i] * e.a.vals[k]) + d[c];
            }
        }
    }
    for j in 0..n {
        if e.vstat[j] == -2 {
            d[j] = -d[j];
        }
    }
    d
}

/// `09e288`'s work step with F+0x1f8 = 1.0 set by `251b4f` around the call.
pub fn wrapper_work(f200: f64, work: f64) -> f64 {
    f200 * 1.0 + work
}

/// `13f716` C 1225–1316 (asm 0x1bf0a2–0x1bf17a, 0x1bf951–0x1bfa08): S+0x2d8 = the nonbasic violations at least
/// `tol` (S+0x380) plus the basic |d|.
pub fn dual_infeasibility(d: &[f64], vstat: &[i32], tol: f64) -> f64 {
    let (mut nb, mut b) = (0.0f64, 0.0f64);
    for (&dj, &s) in d.iter().zip(vstat) {
        if s >= 0 {
            b += dj.abs();
            continue;
        }
        if s == -3 {
            continue;
        }
        let v = if s < -3 {
            0.0
        } else {
            let t = -dj;
            if 0.0 > t {
                0.0
            } else {
                t
            }
        };
        if tol > v {
            continue;
        }
        nb += v;
    }
    nb + b
}

/// The whole `251b4f` call with the original BTRAN on a given factor: the BTRAN entry/exit state, y, d and the
/// caller's work after the wrapper.
pub struct DualCall {
    pub rhs: Vec<f64>,
    pub btran: BtranState,
    pub y: Vec<f64>,
    pub d: Vec<f64>,
    pub work: Option<f64>,
}

/// `251b4f` with `09e288 → 221173(F, S+0x220, S+0x228, 0)`. `out_count`, `x1d8` and `bucket` are the BTRAN entry
/// state the original leaves in the factor and the output header; the wrapper zeroes F+0x200 before the call.
pub fn dual_values(
    e: &DualInput,
    f: &BtranFactor,
    out_count: i32,
    x1d8: i32,
    bucket: Option<Vec<i32>>,
    work: Option<f64>,
) -> DualCall {
    let rhs = btran_rhs(e);
    let mut s = BtranState::new(e.m, -1, &[], &rhs, out_count, None, x1d8, 0.0, bucket);
    btran(f, &mut s);
    let work = work.map(|w| wrapper_work(s.work, w));
    let vb = match s.output.val {
        Buffer::In => 0,
        Buffer::Out => 1,
    };
    let y: Vec<f64> = (0..e.m).map(|i| s.val[vb].get(i)).collect();
    let d = reduced_costs(e, &y);
    DualCall { rhs, btran: s, y, d, work }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infeasibility_follows_the_asm() {
        // basic |d| always counts; −3 skipped; −4 contributes 0; −1/−2 count max(0, −d) only when not below tol.
        let d = [-2.0, 3.0, -0.5, 7.0, -1e-9, 0.25];
        let vstat = [0, -1, -2, -3, -1, -4];
        let tol = 1e-6;
        assert_eq!(dual_infeasibility(&d, &vstat, tol).to_bits(), (0.5f64 + 2.0).to_bits());
        // −0 from d = +0 is kept by maxsd (0 > −0 is false) and added when tol ≤ −0.
        assert_eq!(dual_infeasibility(&[0.0], &[-1], 0.0).to_bits(), (0.0f64 + -0.0f64 + 0.0).to_bits());
        assert!(dual_infeasibility(&[f64::NAN], &[-1], 1.0).is_nan());
    }

    #[test]
    fn reduced_costs_rows_and_flip() {
        // n = 2, m = 2; row 0 = [1, 2], row 1 = [3, 0] (stored [col 0: 3]); y = [1, 1e-12] with drop 1e-11.
        let a = RowMatrix { start: &[0, 2], len: &[2, 1], cols: &[0, 1, 0], vals: &[1.0, 2.0, 3.0] };
        let e = DualInput { n: 2, m: 2, cost: &[5.0, 6.0, 0.0, 0.0], vstat: &[0, -2, -1, 1], header: &[0, 3],
                            corrections: &[(3, 0.5)], drop: 1e-11, a };
        assert_eq!(btran_rhs(&e), vec![5.0, -0.5]);
        let d = reduced_costs(&e, &[1.0, 1e-12]);
        assert_eq!(d, vec![4.0, -4.0, -1.0, -1e-12]);
    }
}
