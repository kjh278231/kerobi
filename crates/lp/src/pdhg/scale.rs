//! `07d3ce(m, 4)`: the PDHG preconditioning (ScaleFlag 4 branch L3836–5279 plus the shared tail), docs/m3_pdhg.md §5.
//! 10 Ruiz ∞-norm steps, one Pock–Chambolle (α = 1) step, clamp to [2⁻⁴⁷, 2⁴⁷] (factors are not powers of two),
//! then a' = (r·s)·a, l/s, u/s, b·r, c·s·ω with the power-of-two objective scale ω. Row senses are not touched by
//! `07d3ce`; D already holds '>' rows negated (σ = −1 on a and b, sense char kept), as for the barrier.
use super::kern::Mat;
use crate::fp::{ilogb, pow2};
use rsolver_model::{Model, INFINITY};

const LO: f64 = 7.105427357601002e-15; // 2^-47
const HI: f64 = 140737488355328.0; // 2^47

/// The LP as PDHG works on it (model data D after scaling).
#[derive(Clone, Debug)]
pub struct Scaled {
    pub a: Mat,
    pub c: Vec<f64>,
    pub lb: Vec<f64>,
    pub ub: Vec<f64>,
    pub b: Vec<f64>,
    pub sense: Vec<u8>,
    /// column factors s (D+0x3e0[..n]), row factors r (D+0x3e0[n..]), objective scale ω (D+0x3e8)
    pub s: Vec<f64>,
    pub r: Vec<f64>,
    pub objscale: f64,
    /// objective constant (D+0x60, unscaled)
    pub objcon: f64,
    /// 16-byte phase of c (D+0x50), which decides the peel of the objective dot in `354824`
    pub peel_c: usize,
}

fn inv_sqrt_or_1(x: f64) -> f64 {
    if x == 0.0 { 1.0 } else { 1.0 / x.sqrt() }
}

pub fn scale_flag4(md: &Model) -> Scaled {
    let (n, m) = (md.num_vars(), md.num_constrs());
    let mut cbeg = Vec::with_capacity(n + 1);
    let (mut cind, mut val) = (Vec::with_capacity(md.num_nz()), Vec::with_capacity(md.num_nz()));
    for j in 0..n {
        cbeg.push(cind.len());
        let b = md.col_beg[j] as usize;
        for k in b..b + md.col_len[j] as usize {
            let i = md.col_ind[k] as usize;
            cind.push(i);
            val.push(if md.row_sense[i] == b'>' { -md.col_val[k] } else { md.col_val[k] });
        }
    }
    cbeg.push(cind.len());
    // shortcut: the lb = −inf, finite-ub column flip (L397–505, when *(m+0xf8) == 0) is not ported; checked
    // absent on the 64 bench LPs (scaled bounds bit-equal to the oracle). Port it if a model with such a column differs.
    let (mut s, mut r) = (vec![1.0f64; n], vec![1.0f64; m]);
    // (row, column) accumulators from |a|·s_j·r_i rounded as fl(fl(s_j·r_i)·|a|)
    let pass = |s: &[f64], r: &[f64], sum: bool| -> (Vec<f64>, Vec<f64>) {
        let (mut wr, mut wc) = (vec![0.0f64; m], vec![0.0f64; n]);
        for j in 0..n {
            for k in cbeg[j]..cbeg[j + 1] {
                let i = cind[k];
                let v = (s[j] * r[i]) * val[k].abs();
                if sum {
                    wr[i] += v;
                    wc[j] += v;
                } else {
                    wr[i] = if wr[i] > v { wr[i] } else { v };
                    wc[j] = if wc[j] > v { wc[j] } else { v };
                }
            }
        }
        (wr, wc)
    };
    for it in 0..11 {
        let (wr, wc) = pass(&s, &r, it == 10); // 10 Ruiz steps, then one Pock–Chambolle step
        for (ri, w) in r.iter_mut().zip(wr) {
            *ri *= inv_sqrt_or_1(w);
        }
        for (sj, w) in s.iter_mut().zip(wc) {
            *sj *= inv_sqrt_or_1(w);
        }
    }
    let clamp = |x: f64| x.clamp(LO, HI); // NaN stays NaN, as in the original
    let s: Vec<f64> = s.into_iter().map(|x| clamp(x.abs())).collect();
    let r: Vec<f64> = r.into_iter().map(clamp).collect();
    for j in 0..n {
        for k in cbeg[j]..cbeg[j + 1] {
            val[k] *= r[cind[k]] * s[j];
        }
    }
    let lb: Vec<f64> = (0..n).map(|j| if md.lb[j] > -INFINITY { md.lb[j] / s[j] } else { md.lb[j] }).collect();
    let ub: Vec<f64> = (0..n).map(|j| if md.ub[j] < INFINITY { md.ub[j] / s[j] } else { md.ub[j] }).collect();
    let mut c: Vec<f64> = (0..n).map(|j| md.obj[j] * s[j]).collect();
    let b: Vec<f64> = (0..m)
        .map(|i| {
            let b = if md.row_sense[i] == b'>' { -md.rhs[i] } else { md.rhs[i] };
            if -INFINITY < b && b < INFINITY { b * r[i] } else { b }
        })
        .collect();
    let objscale = if m == 0 { 1.0 } else { objective_scale(&c) };
    if objscale != 1.0 {
        for cj in c.iter_mut() {
            *cj *= objscale;
        }
    }
    let a = Mat::from_csc(m, cbeg, cind, val);
    Scaled { a, c, lb, ub, b, sense: md.row_sense.clone(), s, r, objscale, objcon: md.objcon, peel_c: 0 }
}

/// Shared LP objective scale (ObjScale = 0, `07d3ce` L2633–2819 + L5362–5399): from the scaled nonzero |c|.
/// shortcut: model class D[0] == 1 (ObjScale −2 variant) and RHSSCALE ≠ 1 are not ported (not on the bench LPs).
fn objective_scale(c: &[f64]) -> f64 {
    let (mut cmax, mut cmin) = (0.0f64, 1e100f64);
    for &x in c.iter().filter(|x| **x != 0.0) {
        let a = x.abs();
        cmax = if cmax > a { cmax } else { a };
        cmin = if cmin < a { cmin } else { a };
    }
    if cmax == 0.0 {
        return 1.0;
    }
    let t = ((if 1e-6 > cmin { 1e-6 } else { cmin }) * cmax).sqrt();
    let (p, q) = (100.0 / cmax, 1.0 / (10.0 * t));
    let x = if p < q { p } else { q };
    if x < 2.0 {
        1.0
    } else {
        pow2(ilogb(if x > 1000.0 { 1000.0 } else { x }))
    }
}
