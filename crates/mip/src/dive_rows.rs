//! DynamicRows dive row selection of `5fb8ec` (C:1625–1842, after the gate at `0x513078` up to `0x513951`): the rows the
//! dive node deactivates through `10e5fc` (docs/m2/mip_dive_rows.md, docs/m2/mip_dive_rows_rust.md).
//! shortcut: selection only. The gate (DynamicRows, dive depth, node state, `a7f0ad`/`a7f923`) and the deactivated list
//! (`10e5fc`/`edd71`/`10e662`) are not ported and nothing calls this from the B&B yet; upgrade with M2-03d-4.
use crate::knapsack::act_sum;

/// What the selection reads. Model rows are the cut-free MIP LP `(mip+8)->D` in row-major form; cut j is row nrows+j.
pub struct Sel<'a> {
    /// Row i entries are `ind/val[beg[i]..end[i]]` (`D+0x118/0x120/0x130/0x138`).
    pub beg: &'a [usize],
    pub end: &'a [usize],
    pub ind: &'a [i32],
    pub val: &'a [f64],
    /// `D+0x380`, `D+0x388` (`<`, `>`, `=` as bytes), `mip+0x650`.
    pub rhs: &'a [f64],
    pub sense: &'a [u8],
    pub vtype: &'a [u8],
    /// Node bounds `I+0x80/0x88`.
    pub lb: &'a [f64],
    pub ub: &'a [f64],
    /// Node row flags `I+0xa0`, nrows + ncuts.
    pub flags: &'a [i32],
    /// Node LP slack `(I+8)->+0x20` (read at nrows+j) and reduced costs `(I+8)->+0x28`.
    pub slack: &'a [f64],
    pub rc: &'a [f64],
    /// `7889a5(mip+0x32e8, i)` array: row i is excluded when `i < len` and the entry is ≥ 0. Empty without the object.
    pub b7: &'a [i32],
    /// Per cut: type `+0x2c` and whether `+0x40` is set.
    pub cuts: &'a [(i32, bool)],
    /// FeasTol env+0x2460, DualReductions env+0x2430.
    pub feastol: f64,
    pub dual_reductions: i32,
    /// `5e7204(mip)` (1e100 without an incumbent) and `I+0x10`.
    pub cutoff: f64,
    pub obj_off: f64,
}

/// Selected rows in order (model rows ascending, then cuts). `calls` receives every `66661c` call as (row, result).
/// `w` is the work counter `param_3` (C only, not captured).
pub fn select(s: &Sel, calls: &mut Vec<(i32, bool)>, w: &mut f64) -> Vec<i32> {
    let nrows = s.rhs.len();
    let mut co = s.cutoff;
    if co < 1e100 {
        co = co - s.obj_off;
    }
    let mut out = Vec::new();
    for i in 0..nrows {
        if s.flags[i] != 0 || s.b7.get(i).is_some_and(|&v| v >= 0) {
            continue;
        }
        let (ind, val) = (&s.ind[s.beg[i]..s.end[i]], &s.val[s.beg[i]..s.end[i]]);
        let rhs = s.rhs[i];
        let maxact = -act_sum(ind, val, s.lb, s.ub, -1.0, w);
        let cutoff_path = if !(s.feastol * 0.1 + rhs >= maxact) {
            true
        } else {
            let minact = if s.sense[i] == b'=' { act_sum(ind, val, s.lb, s.ub, 1.0, w) } else { 1e100 };
            !(minact >= rhs - s.feastol * 0.1)
        };
        if cutoff_path {
            if s.dual_reductions == 0 || !(1e100 > co) || s.sense[i] == b'=' {
                continue;
            }
            let r = f66661c(rhs, co, ind, val, s.vtype, s.lb, s.ub, s.rc, w);
            calls.push((i as i32, r));
            if !r {
                continue;
            }
        }
        out.push(i as i32);
    }
    *w = nrows as f64 + nrows as f64 + *w;
    for (j, &(kind, attr40)) in s.cuts.iter().enumerate() {
        let k = nrows + j;
        if s.flags[k] == 0 && kind != 0x14 && !attr40 && 1e-05 < s.slack[k] {
            out.push(k as i32);
        }
    }
    *w = *w + s.cuts.len() as f64 * 3.0;
    out
}

/// `66661c`: true when lifting the row activity to rhs (integral rows: the next integer above rhs) costs more than
/// `gap` in reduced cost, taking the five cheapest |rc/a| columns first.
#[allow(clippy::too_many_arguments)]
fn f66661c(rhs: f64, gap: f64, ind: &[i32], val: &[f64], vtype: &[u8], lb: &[f64], ub: &[f64], rc: &[f64], w: &mut f64) -> bool {
    let mut loc = 0.0;
    let mut integral = true;
    // slots 1..=5 by |rc/a| ascending: |rc|, |a|, ub - lb
    let (mut e, mut c, mut wd) = ([0.0f64; 6], [0.0f64; 6], [0.0f64; 6]);
    let mut k = 0usize;
    for (&j, &a) in ind.iter().zip(val) {
        let j = j as usize;
        let (l, u) = (lb[j], ub[j]);
        let d = u - l;
        if d <= 1e-10 {
            loc = loc + a * 0.5 * (l + u);
            continue;
        }
        integral = integral && vtype[j] != b'C' && a - (a + 1e-10).floor() <= 1e-10;
        if a.abs() <= 1e-6 {
            return false;
        }
        let r = rc[j];
        if !(a <= 0.0 || 1e-10 < r) {
            loc = loc + u * a;
            continue;
        }
        if 0.0 < a && l <= -1e30 {
            return false;
        }
        if !(0.0 <= a || r < -1e-10) {
            loc = loc + l * a;
            continue;
        }
        if a < 0.0 && 1e30 <= u {
            return false;
        }
        let b = if 0.0 < a { l } else { u };
        loc = loc + b * a;
        let q = (r / a).abs();
        let mut p = k;
        while p > 0 && !(e[p] / c[p] <= q) {
            if p < 5 {
                (e[p + 1], c[p + 1], wd[p + 1]) = (e[p], c[p], wd[p]);
            }
            p -= 1;
        }
        if p < 5 {
            (e[p + 1], c[p + 1], wd[p + 1]) = (r.abs(), a.abs(), d);
            if k < 5 {
                k += 1;
            }
        }
    }
    *w = ind.len() as f64 * 14.0 + *w;
    let rhs = if integral { (rhs + 9.999999999999999e-05).floor() + 1.0 } else { rhs };
    if !(loc < rhs - 1e-06) {
        return false;
    }
    let (mut cost, mut s) = (0.0, 0);
    while s < k {
        let mut step = (rhs - loc) / c[s + 1];
        if s + 1 < k && wd[s + 1] <= step {
            step = wd[s + 1];
        }
        loc = loc + c[s + 1] * step;
        cost = cost + e[s + 1] * step;
        if rhs - 1e-06 <= loc {
            break;
        }
        s += 1;
    }
    *w = s as f64 * 4.0 + *w;
    gap < cost
}

#[cfg(test)]
mod tests {
    use super::*;

    // self-check of the port on a hand-made node; the original comparison is tests/dive_rows_l1.rs
    #[test]
    fn clauses() {
        // rows: x0 + x1 <= 2 (redundant), x0 + x1 <= 1 (flagged), x0 + x1 <= 1 (cutoff path); cuts: slack 1e-5, 2e-5
        let (beg, end, ind, val) = ([0, 2, 4], [2, 4, 6], [0, 1, 0, 1, 0, 1], [1.0; 6]);
        let (lb, ub, rc) = ([0.0; 2], [1.0; 2], [3.0, 4.0]);
        let mut s = Sel {
            beg: &beg, end: &end, ind: &ind, val: &val, rhs: &[2.0, 1.0, 1.0], sense: b"<<<", vtype: b"BB",
            lb: &lb, ub: &ub, flags: &[0, 1, 0, 0, 0], slack: &[0.0, 0.0, 0.0, 1e-5, 2e-5], rc: &rc, b7: &[],
            cuts: &[(0, false), (0, false)], feastol: 1e-6, dual_reductions: 1, cutoff: 10.0, obj_off: 4.0,
        };
        // row 2: integral, rhs -> 2, cost 3 (x0 to 1) + 4 (x1 to 1) = 7 > gap 6
        let (mut calls, mut w) = (vec![], 0.0);
        assert_eq!(select(&s, &mut calls, &mut w), vec![0, 2, 4]);
        assert_eq!(calls, vec![(2, true)]);
        s.obj_off = 3.0; // gap 7: not more than the cost
        calls.clear();
        assert_eq!(select(&s, &mut calls, &mut w), vec![0, 4]);
        s.dual_reductions = 0;
        calls.clear();
        assert_eq!(select(&s, &mut calls, &mut w), vec![0, 4]);
        assert!(calls.is_empty());
    }
}
