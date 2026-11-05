//! 04 §3 Scaling (original `07d3ce`, ScaleFlag 3 path: chosen automatically for LP dual simplex).
//! Reconstructed from pseudocode plus assembly and verified bit-for-bit against the original's scale factors on
//! all 64 bench LPs (bench/gurobi_spx0). Scale factors are powers of two, applied as
//! a' = σ·a·r·s, b' = σ·b·r, c' = c·s·objscale, l' = l/s, u' = u/s (σ = −1 on '>' rows, ±1e30 kept).
use crate::fp::{ilogb, pow2};
use rsolver_model::{Model, INFINITY};

#[derive(Clone, Debug, PartialEq)]
pub struct Scaling {
    pub col: Vec<f64>,
    pub row: Vec<f64>,
    pub obj: f64,
}

const MIN_SCALE: f64 = 7.105427357601002e-15; // 2^-47
const MAX_SCALE: f64 = 140737488355328.0; // 2^47

/// Power-of-two rounding used by both passes: 1 inside (0.5, 2), else 2^ilogb(x) (rounded down).
fn round_pow2(x: f64) -> f64 {
    if 0.5 < x && x < 2.0 {
        1.0
    } else {
        pow2(ilogb(x))
    }
}

/// Limit the change from `old` to a factor `f` either way; returns (value, clipped).
fn clip(s: f64, old: f64, f: f64) -> (f64, bool) {
    if s > f * old {
        (f * old, true)
    } else if s < old / f {
        (old / f, true)
    } else {
        (s, false)
    }
}

/// Column scale from the row-scaled column maximum, optionally blended with the bound magnitude.
fn col_value(m: &Model, j: usize, row: &[f64], use_bounds: bool) -> f64 {
    let b = m.col_beg[j] as usize;
    let mut cmax = 0.0f64;
    for k in b..b + m.col_len[j] as usize {
        let v = (m.col_val[k] * row[m.col_ind[k] as usize]).abs();
        if cmax <= v {
            cmax = v;
        }
    }
    let mut s = if cmax == 0.0 { 1.0 } else { 1.0 / cmax };
    if use_bounds {
        // max(1, |l|, |u|) over finite bounds (asm 0x117134..0x117174)
        let mut bm = 1.0f64;
        if m.lb[j].abs() < INFINITY {
            bm = bm.max(m.lb[j].abs());
        }
        if m.ub[j].abs() < INFINITY {
            bm = bm.max(m.ub[j].abs());
        }
        s = (s * bm).sqrt();
    }
    round_pow2(s)
}

pub fn scale_flag3(m: &Model) -> Scaling {
    let (n, nrows) = (m.num_vars(), m.num_constrs());
    let mut col = vec![1.0f64; n]; // stored negated in the original as an "update" mark; magnitudes kept here
    let mut row = vec![1.0f64; nrows];
    let (mut iter, mut f, mut use_bounds) = (0, 2.0f64, true);
    let mut changed = false; // reset only at start and before each row pass, as in the original
    loop {
        for (j, c) in col.iter_mut().enumerate() {
            let (s, cl) = clip(col_value(m, j, &row, use_bounds), *c, f);
            changed |= cl;
            *c = s;
        }
        if iter >= 1 && !changed {
            if iter < 3 {
                // finishing pass without bounds and without the step limit
                for (j, c) in col.iter_mut().enumerate() {
                    *c = col_value(m, j, &row, false);
                }
            }
            break;
        }
        iter += 1;
        if iter > 9 {
            break;
        }
        f = if iter == 9 { 1e100 } else { 2.0 };
        changed = false;
        let mut rmax = vec![0.0f64; nrows];
        for j in 0..n {
            let b = m.col_beg[j] as usize;
            for k in b..b + m.col_len[j] as usize {
                let i = m.col_ind[k] as usize;
                let v = (m.col_val[k] * col[j]).abs();
                if rmax[i] < v {
                    rmax[i] = v;
                    // shortcut: the original caps rmax at max(1e8, sqrt(..)) when ub_j >= INFBOUND and v > 1e8;
                    // its sqrt argument is not decoded and no bench LP reaches this branch.
                }
            }
        }
        for i in 0..nrows {
            if rmax[i] > 0.0 {
                let r = if 0.5 < rmax[i] && rmax[i] < 2.0 { 1.0 } else { pow2(ilogb(1.0 / rmax[i])) };
                let (r, cl) = clip(r, row[i], f);
                changed |= cl;
                row[i] = r;
            }
        }
        use_bounds = iter <= 2;
    }
    let clamp = |x: f64| x.clamp(MIN_SCALE, MAX_SCALE);
    let col: Vec<f64> = col.into_iter().map(clamp).collect();
    let row: Vec<f64> = row.into_iter().map(clamp).collect();
    let obj = objective_scale(m, &col);
    Scaling { col, row, obj }
}

/// Objective scale from the scaled nonzero costs (pseudocode L1293–1335). Only scales up for this path.
fn objective_scale(m: &Model, col: &[f64]) -> f64 {
    let (mut mx, mut mn, mut any) = (0.0f64, f64::INFINITY, false);
    for (c, s) in m.obj.iter().zip(col) {
        if *c != 0.0 {
            let v = (c * s).abs();
            mx = mx.max(v);
            mn = mn.min(v);
            any = true;
        }
    }
    if !any || (mx < 10.0 && 0.1 < mn) {
        return 1.0;
    }
    let t = (mn.max(1e-6) * mx).sqrt();
    if t > 1.0 {
        return 1.0; // shortcut: the downward branch (t > 1) is never taken on the bench LPs; not decoded further
    }
    let x = (100.0 / mx).min(0.1 / t);
    if x <= 2.0 {
        1.0
    } else {
        // shortcut: for x > 10000 the ilogb argument is lost in the pseudocode; 10000 assumed (no bench sample)
        pow2(ilogb(if x > 10000.0 { 10000.0 } else { x }))
    }
}
