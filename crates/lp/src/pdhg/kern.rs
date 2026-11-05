//! PDHG per-iteration kernels (`35645c`, `3565e5`, `2f9515`/`354e23`, `35207d`, `3524ec`, `352bcb`, `3528a6`),
//! bit-exact with Gurobi 13.0.3 on one thread (docs/m3_pdhg.md §4). All SSE2, no FMA: every product and sum rounds.

/// Reduction pattern shared by both SpMVs and the dot `2f9515`: below 8 terms (or peel + 8) a plain left-to-right sum
/// from 0.0; otherwise `peel` scalar terms (1 when the aligned operand starts at an 8-mod-16 address), an 8-wide loop
/// into four 2-lane accumulators combined as (A+B)+(C+D) per lane, then lo+hi, then the scalar tail in order.
#[inline]
pub fn red8(n: usize, peel: usize, prod: impl Fn(usize) -> f64) -> f64 {
    if n < peel + 8 {
        return (0..n).fold(0.0, |s, k| s + prod(k));
    }
    let mut acc = [[0.0f64; 2]; 4];
    acc[0][0] = (0..peel).fold(0.0, |s, k| s + prod(k));
    let vend = n - ((n - peel) & 7);
    let mut k = peel;
    while k < vend {
        for (a, lane) in acc.iter_mut().enumerate() {
            lane[0] += prod(k + 2 * a);
            lane[1] += prod(k + 2 * a + 1);
        }
        k += 8;
    }
    let lane = |l: usize| (acc[0][l] + acc[1][l]) + (acc[2][l] + acc[3][l]);
    (vend..n).fold(lane(0) + lane(1), |s, k| s + prod(k))
}

/// `354e23` on one thread: 0.0 + `2f9515`(a, b); `peel_b` is b's 16-byte phase (b decides the peel).
pub fn dot(a: &[f64], b: &[f64], peel_b: usize) -> f64 {
    0.0 + red8(a.len(), peel_b, |k| a[k] * b[k])
}

/// The scaled LP as PDHG sees it (model data D after `07d3ce` flag 4): CSC for Aᵀy, a row-wise copy for Ax.
/// Value arrays are 16-byte aligned at index 0 (checked on the oracle), so a segment's peel is its start's parity.
#[derive(Clone, Debug, Default)]
pub struct Mat {
    pub m: usize,
    pub n: usize,
    pub cbeg: Vec<usize>,
    pub cind: Vec<usize>,
    pub cval: Vec<f64>,
    pub rbeg: Vec<usize>,
    pub rind: Vec<usize>,
    pub rval: Vec<f64>,
}

impl Mat {
    pub fn from_csc(m: usize, cbeg: Vec<usize>, cind: Vec<usize>, cval: Vec<f64>) -> Mat {
        let n = cbeg.len() - 1;
        let mut rbeg = vec![0usize; m + 1];
        for &i in &cind {
            rbeg[i + 1] += 1;
        }
        for i in 0..m {
            rbeg[i + 1] += rbeg[i];
        }
        let (mut at, mut rind, mut rval) = (rbeg.clone(), vec![0; cind.len()], vec![0.0; cind.len()]);
        for j in 0..n {
            for k in cbeg[j]..cbeg[j + 1] {
                let i = cind[k];
                rind[at[i]] = j;
                rval[at[i]] = cval[k];
                at[i] += 1;
            }
        }
        Mat { m, n, cbeg, cind, cval, rbeg, rind, rval }
    }
    pub fn nnz(&self) -> usize {
        self.cind.len()
    }
    /// `35645c`: out = A·v over the row-wise copy.
    pub fn ax(&self, v: &[f64], out: &mut [f64]) {
        for (i, o) in out.iter_mut().enumerate() {
            let (b, e) = (self.rbeg[i], self.rbeg[i + 1]);
            *o = red8(e - b, b & 1, |k| self.rval[b + k] * v[self.rind[b + k]]);
        }
    }
    /// `3565e5`: out = Aᵀ·v over the CSC arrays.
    pub fn aty(&self, v: &[f64], out: &mut [f64]) {
        for (j, o) in out.iter_mut().enumerate() {
            let (b, e) = (self.cbeg[j], self.cbeg[j + 1]);
            *o = red8(e - b, b & 1, |k| self.cval[b + k] * v[self.cind[b + k]]);
        }
    }
}

/// x86 `maxsd a, b` / `minsd a, b` as compiled here: the first operand only when strictly better (NaN, ±0 → b).
#[inline]
fn maxsd(a: f64, b: f64) -> f64 {
    if a > b { a } else { b }
}
#[inline]
fn minsd(a: f64, b: f64) -> f64 {
    if a < b { a } else { b }
}

/// Iterate vectors of the PDHG state P (offsets in docs/m3_pdhg.md §4).
#[derive(Clone, Debug, Default)]
pub struct Vecs {
    pub x: Vec<f64>,    // P+0x28
    pub y: Vec<f64>,    // P+0x30
    pub xhat: Vec<f64>, // P+0xb8 (saved on evaluation iterations)
    pub yhat: Vec<f64>, // P+0xc0
    pub xdel: Vec<f64>, // P+0xc8 (x̂ − t)/τ (saved on evaluation iterations)
    pub xref: Vec<f64>, // P+0xd0
    pub yref: Vec<f64>, // P+0xd8
    pub x0: Vec<f64>,   // P+0x128 anchor
    pub y0: Vec<f64>,   // P+0x130 anchor
    pub ax: Vec<f64>,   // P+0x138
    pub aty: Vec<f64>,  // P+0x140
    pub lb: Vec<f64>,   // P+0x148
    pub ub: Vec<f64>,   // P+0x150
    pub dz: Vec<f64>,   // P+0x198 scratch [dx (n) | dy (m)]
}

/// `35207d`: x̂ = proj(x − (c − Aᵀy)·τ), x_ref = (x̂ + x̂) − x; on evaluation iterations save x̂ and (x̂ − t)/τ.
pub fn primal_step(a: &Mat, c: &[f64], v: &mut Vecs, eta: f64, omega: f64, save: bool, work: &mut f64) {
    let tau = eta / omega;
    a.aty(&v.y, &mut v.aty);
    *work += a.nnz() as f64 * 3.0;
    for j in 0..a.n {
        let xj = v.x[j];
        let t = xj - (c[j] - v.aty[j]) * tau;
        let xh = minsd(maxsd(t, v.lb[j]), v.ub[j]);
        if save {
            v.xhat[j] = xh;
            v.xdel[j] = (xh - t) / tau;
        }
        v.xref[j] = (xh + xh) - xj;
    }
    *work += a.n as f64 * 6.0;
}

/// `3524ec`: t = y − (A x_ref)·σ, ŷ = t − p with p = (−σ)·b on '=' rows and maxsd(t, (−σ)·b) otherwise (y ≤ 0),
/// y_ref = (ŷ − y) + ŷ.
pub fn dual_step(a: &Mat, b: &[f64], eq: &[bool], v: &mut Vecs, eta: f64, omega: f64, work: &mut f64) {
    let sigma = eta * omega;
    a.ax(&v.xref, &mut v.ax);
    *work += a.nnz() as f64 * 3.0;
    let nsig = -sigma;
    for i in 0..a.m {
        let yi = v.y[i];
        let t = yi - v.ax[i] * sigma;
        let lo = nsig * b[i];
        let yh = t - if eq[i] { lo } else { maxsd(t, lo) };
        v.yhat[i] = yh;
        v.yref[i] = (yh - yi) + yh;
    }
    *work += a.m as f64 * 6.0;
}

/// `352bcb`: ‖z − z_ref‖ in the metric ω‖dx‖² + ‖dy‖²/ω + 2η⟨dx, Aᵀdy⟩ (0 when negative or NaN). Overwrites aty.
pub fn fixed_point_norm(a: &Mat, v: &mut Vecs, eta: f64, omega: f64, work: &mut f64) -> f64 {
    let n = a.n;
    for j in 0..n {
        v.dz[j] = v.x[j] - v.xref[j];
    }
    for i in 0..a.m {
        v.dz[n + i] = v.y[i] - v.yref[i];
    }
    let (dx, dy) = v.dz.split_at(n);
    a.aty(dy, &mut v.aty);
    *work += a.nnz() as f64 * 3.0;
    let (pa, pb, pc) = (dot(dx, dx, 0), dot(dy, dy, n & 1), dot(dx, &v.aty, 0));
    let s = (pc * 2.0) * eta + (pa * omega + pb / omega);
    if s >= 0.0 { s.sqrt() } else { 0.0 }
}

/// `3528a6`: Halpern step z = z_ref·w + z⁰·(1 − w), w = (k+1)/(k+2), k = iterations since restart.
pub fn halpern(v: &mut Vecs, k: i64, work: &mut f64) {
    let w = (k + 1) as f64 / (k + 2) as f64;
    let w1 = 1.0 - w;
    for (x, (r, z)) in v.x.iter_mut().zip(v.xref.iter().zip(&v.x0)) {
        *x = r * w + z * w1;
    }
    for (y, (r, z)) in v.y.iter_mut().zip(v.yref.iter().zip(&v.y0)) {
        *y = r * w + z * w1;
    }
    *work += (v.x.len() as f64 + v.y.len() as f64) * 3.0;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn red8_order() {
        // 11 terms, peel 1: t0 | lanes over 1..9 | tail 9, 10; with values chosen so that order matters
        let v = [1e16, 1.0, -1e16, 1.0, 3.0, 5.0, 7.0, 9.0, 11.0, 0.5, 0.25];
        let s = red8(v.len(), 1, |k| v[k]);
        let a = [[1e16 + 1.0, -1e16], [1.0, 3.0], [5.0, 7.0], [9.0, 11.0]]; // peel term seeds lane A[0]
        let lane = |l: usize| (a[0][l] + a[1][l]) + (a[2][l] + a[3][l]);
        assert_eq!(s, ((lane(0) + lane(1)) + 0.5) + 0.25);
        assert_eq!(red8(5, 0, |k| v[k]), (((1e16 + 1.0) + -1e16) + 1.0) + 3.0);
    }
}
