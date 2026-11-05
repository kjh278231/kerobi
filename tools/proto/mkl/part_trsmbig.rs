//! Bit-exact emulation of MKL (AVX2, as linked in libgurobi130.so) for
//!   DTRSM('L','U','T','U', m, n, 1.0, A, lda, B, ldb) on the "fullacopybcopy" strategy path
//!   (m > 12 and !(m % 4 == 0 && m <= 88), n >= 2), kernel mkl_blas_avx2_dtrsm_kernel_ll_0,
//! and DTRSV('U','T','U') with incx = 1 (mkl_blas_avx2_dtrsv_utu; DTRSM with n == 1 goes there).
//! U^T is lower: L[i][p] = A[p + i*lda] for p < i. The diagonal of A is ignored. Port of part_trsmbig.c.

/// dot modes per (row block R = 12, 8, 4, 2, 1; column group C = 4, 2, 1), see `neg_dot`.
const MODE: [[usize; 3]; 5] = [[1, 1, 0], [1, 1, 4], [2, 1, 4], [1, 1, 2], [2, 2, 2]];

/// -sum l[p]*x[p], p < k, by fma. mode 1: one chain from +0; 2 / 4: for p < 4*(k/4) chain p % mode into its own
/// accumulator, combine (a0 + a1) or ((a0 + a1) + (a2 + a3)), then the last k % 4 terms; mode 0: one chain with
/// "+ 0.0" after the first 4*(k/4) terms. k < 4: one chain.
fn neg_dot(mode: usize, l: &[f64], x: &[f64], k: usize) -> f64 {
    let k4 = k & !3;
    let mut p = 0;
    let mut acc = 0.0f64;
    if k4 > 0 && mode >= 2 {
        let mut a = [0.0f64; 4];
        while p < k4 {
            a[p % mode] = (-l[p]).mul_add(x[p], a[p % mode]);
            p += 1;
        }
        acc = if mode == 2 { a[0] + a[1] } else { (a[0] + a[1]) + (a[2] + a[3]) };
    } else if k4 > 0 && mode == 0 {
        while p < k4 {
            acc = (-l[p]).mul_add(x[p], acc);
            p += 1;
        }
        acc += 0.0;
    }
    while p < k {
        acc = (-l[p]).mul_add(x[p], acc);
        p += 1;
    }
    acc
}

/// Rows in blocks of 12, then m % 12 as 8, 4, 2, 1; columns in groups of 4, then 2 (n & 2), then 1 (n & 1).
/// Row i of a block starting at r0: t = B[i] + neg_dot(..., k = r0); then t = fma(x[p], -L[i][p], t) for
/// p = r0..i-1 in order. shortcut: verified for m <= 254 (MKL splits rows into 192-row panels from about m = 256).
pub fn emu_trsm_big(m: usize, n: usize, a: &[f64], lda: usize, b: &mut [f64], ldb: usize) {
    for j in 0..n {
        let x = &mut b[j * ldb..j * ldb + m];
        let r = n & 3;
        let cg = if j < n - r { 0 } else if r & 2 != 0 && j < n - r + 2 { 1 } else { 2 };
        let (mut r0, mut rb, mut ri) = (0usize, 12usize, 0usize);
        while r0 < m {
            let left = m - r0;
            if left < 12 {
                rb = if left & 8 != 0 { 8 } else if left & 4 != 0 { 4 } else if left & 2 != 0 { 2 } else { 1 };
                ri = match rb { 8 => 1, 4 => 2, 2 => 3, _ => 4 };
            }
            for i in r0..r0 + rb {
                let d = neg_dot(MODE[ri][cg], &a[i * lda..], x, r0);
                x[i] += d;
            }
            for p in r0..r0 + rb {
                for i in p + 1..r0 + rb {
                    x[i] = x[p].mul_add(-a[p + i * lda], x[i]);
                }
            }
            r0 += rb;
        }
    }
}

/// (l0 + l2) + (l1 + l3), MKL's horizontal sum of a 4-lane accumulator.
fn hsum4(v: &[f64; 4]) -> f64 {
    (v[0] + v[2]) + (v[1] + v[3])
}

/// dtrsv_utu, incx = 1. shortcut: m <= 32 only (larger m: MKL splits into 32-row blocks with a dgemv update).
pub fn emu_trsv_utu(m: usize, a: &[f64], lda: usize, x: &mut [f64]) {
    let mut i0 = 0;
    while i0 + 1 < m {
        let (c0, c1) = (&a[i0 * lda..], &a[(i0 + 1) * lda..]);
        let (mut s0, mut s1) = (x[i0], x[i0 + 1]);
        let mut q = 0;
        if i0 >= 16 {
            let (mut va, mut vb, mut vc) = ([s0, 0.0, 0.0, 0.0], [0.0f64; 4], [s1, 0.0, 0.0, 0.0]);
            while q + 16 <= i0 {
                for l in 0..4 {
                    va[l] = (-x[q + l]).mul_add(c0[q + l], va[l]);
                    va[l] = (-x[q + 8 + l]).mul_add(c0[q + 8 + l], va[l]);
                    vb[l] = (-x[q + 4 + l]).mul_add(c0[q + 4 + l], vb[l]);
                    vb[l] = (-x[q + 12 + l]).mul_add(c0[q + 12 + l], vb[l]);
                    for u in [0, 4, 8, 12] {
                        vc[l] -= x[q + u + l] * c1[q + u + l];
                    }
                }
                q += 16;
            }
            for l in 0..4 {
                va[l] += vb[l];
            }
            s0 = hsum4(&va);
            s1 = hsum4(&vc);
        }
        if i0 - q >= 4 {
            let (mut va, mut vc) = ([s0, 0.0, 0.0, 0.0], [s1, 0.0, 0.0, 0.0]);
            while q + 4 <= i0 {
                for l in 0..4 {
                    va[l] -= x[q + l] * c0[q + l];
                    vc[l] -= x[q + l] * c1[q + l];
                }
                q += 4;
            }
            s0 = hsum4(&va);
            s1 = hsum4(&vc);
        }
        while q < i0 {
            s0 -= x[q] * c0[q];
            s1 -= x[q] * c1[q];
            q += 1;
        }
        x[i0] = s0;
        x[i0 + 1] = (-s0).mul_add(c1[i0], s1);
        i0 += 2;
    }
    if m & 1 == 1 {
        let i = m - 1;
        let c0 = &a[i * lda..];
        let mut s = x[i];
        let mut q = 0;
        if i >= 16 {
            let mut v = [[0.0f64; 4]; 4];
            v[0][0] = s;
            while q + 16 <= i {
                for u in 0..4 {
                    for l in 0..4 {
                        v[u][l] = (-x[q + 4 * u + l]).mul_add(c0[q + 4 * u + l], v[u][l]);
                    }
                }
                q += 16;
            }
            let mut t = [0.0f64; 4];
            for l in 0..4 {
                t[l] = (v[0][l] + v[1][l]) + (v[2][l] + v[3][l]);
            }
            s = hsum4(&t);
        }
        if i - q >= 4 {
            let mut va = [s, 0.0, 0.0, 0.0];
            while q + 4 <= i {
                for l in 0..4 {
                    va[l] -= x[q + l] * c0[q + l];
                }
                q += 4;
            }
            s = hsum4(&va);
        }
        while q < i {
            s -= x[q] * c0[q];
            q += 1;
        }
        x[i] = s;
    }
}
