//! Bit-exact emulation of the two MKL calls of Gurobi's barrier factor (MKL statically linked in libgurobi130.so,
//! AVX2 code path, alpha = beta = 1): DGEMM('T','N') and DTRSM('L','U','T','U'). fma = `f64::mul_add`, every other
//! operation rounds separately. Checked against the real MKL on all 2660 DGEMM and 746 DTRSM shapes of the 64 bench
//! LPs (random data, any ld and alignment; docs/m3/m3_chol.md §3). Results do not depend on ld or buffer alignment.
//! shortcut: AVX2 kernels only; on an AVX-512 machine Gurobi's MKL takes other kernels (and the work estimate
//! constant differs too, mod.rs `cpu_divisor`). Port the AVX-512 paths when such a machine has to match.
#![allow(clippy::too_many_arguments, clippy::needless_range_loop, clippy::assign_op_pattern)] // kernels as written

// ======== part_gemv.rs ========
// Port of part_gemv.c: MKL AVX2 dgemv_t (alpha = beta = 1, incx = 1) and the m == 1 / n == 1 DGEMM('T','N') routed to it.
// Per column j, 4 lanes (start +0): 8-blocks acc[l] = fma(x[p+l], a[p+l], x[p+4+l]*a[p+4+l]) + acc[l]; one 4-block if
// k%8 >= 4 (fma for columns in 4-column groups, mul+add for the n&2 / n&1 tail columns); k%4 tail masked fma
// (lanes >= k%4: fma(0,0,acc)); y[j] += (acc0 + acc1) + (acc2 + acc3).

/// y[j*incy] += sum_p a[p + j*lda] * x[p], for j < n
pub fn emu_gemv_t(k: usize, n: usize, a: &[f64], lda: usize, x: &[f64], y: &mut [f64], incy: usize) {
    let (k8, r, n4) = (k & !7, k & 3, n & !3);
    let k4 = (k - k8) & 4 != 0;
    for j in 0..n {
        let a = &a[j * lda..];
        let mut acc = [0.0f64; 4];
        let mut p = 0;
        while p < k8 {
            for l in 0..4 {
                acc[l] = x[p + l].mul_add(a[p + l], x[p + 4 + l] * a[p + 4 + l]) + acc[l];
            }
            p += 8;
        }
        if k4 {
            for l in 0..4 {
                acc[l] = if j < n4 { x[p + l].mul_add(a[p + l], acc[l]) } else { x[p + l] * a[p + l] + acc[l] };
            }
            p += 4;
        }
        if r != 0 {
            for l in 0..4 {
                acc[l] = if l < r { x[p + l].mul_add(a[p + l], acc[l]) } else { 0.0f64.mul_add(0.0, acc[l]) };
            }
        }
        y[j * incy] += (acc[0] + acc[1]) + (acc[2] + acc[3]);
    }
}

/// DGEMM('T','N', m, n, k, 1, A, lda, B, ldb, 1, C, ldc) for m == 1 or n == 1 (m == 1 checked first)
pub fn emu_gemm_tn_gemv(m: usize, n: usize, k: usize, a: &[f64], lda: usize, b: &[f64], ldb: usize, c: &mut [f64], ldc: usize) {
    if m == 1 {
        emu_gemv_t(k, n, b, ldb, a, c, ldc)
    } else {
        emu_gemv_t(k, m, a, lda, b, c, 1)
    }
}

// ======== part_nocopy.rs ========
// Emulation of MKL (libgurobi130.so, AVX2) DGEMM('T','N', alpha=1, beta=1) nocopy path
// (mkl_blas_avx2_xdgemm_nocopy_driver -> mkl_blas_avx2_dgemm_kernel_nocopy_TN_b1), port of part_nocopy.c.
// C[i + j*ldc] += sum_p A[p + i*lda] * B[p + j*ldb].

/// nocopy vs fullacopybcopy choice of mkl_blas_avx2_xdgemm for TN, alpha != 0.
/// m == 1 / n == 1 go to dgemv and k == 1 to dgemmger earlier (mkl_blas_dgemm), so they are not nocopy.
pub fn mkl_gemm_is_nocopy(m: usize, n: usize, k: usize) -> bool {
    if m <= 1 || n <= 1 || k <= 1 {
        return false;
    }
    if k <= 128 && m * n / (m + n) <= 10 {
        return m <= 100 || (n <= 3 && k <= 3);
    }
    m < 56 || n < 56
}

// accumulators start at +0.0 (vxorpd), which matters for signed zeros
fn dot_seq_fma(a: &[f64], b: &[f64], k: usize) -> f64 {
    let mut s = 0.0f64;
    for p in 0..k {
        s = a[p].mul_add(b[p], s);
    }
    s
}

fn dot_eo_fma(a: &[f64], b: &[f64], k: usize) -> f64 {
    let (mut e, mut o) = (0.0f64, 0.0f64);
    let ke = k & !1;
    for p in (0..ke).step_by(2) {
        e = a[p].mul_add(b[p], e);
        o = a[p + 1].mul_add(b[p + 1], o);
    }
    let mut s = e + o;
    if k & 1 == 1 {
        s = a[ke].mul_add(b[ke], s);
    }
    s
}

fn dot_eo_muladd(a: &[f64], b: &[f64], k: usize) -> f64 {
    let (mut e, mut o) = (0.0f64, 0.0f64);
    let ke = k & !1;
    for p in (0..ke).step_by(2) {
        e = e + a[p] * b[p];
        o = o + a[p + 1] * b[p + 1];
    }
    let mut s = e + o;
    if k & 1 == 1 {
        s = s + a[ke] * b[ke];
    }
    s
}

// one kernel call: rows in blocks of 8 (5..7 leftover rows = masked 8-block, 1..4 = 4-block), columns in groups of 6
#[allow(clippy::too_many_arguments)]
fn kernel_tn_b1(m: usize, n: usize, k: usize, a: &[f64], lda: usize, b: &[f64], ldb: usize, c: &mut [f64], ldc: usize) {
    for i0 in (0..m).step_by(8) {
        let r = m - i0;
        let rows = r.min(8);
        let wide = r >= 5;
        for j0 in (0..n).step_by(6) {
            let w = (n - j0).min(6);
            let dot: fn(&[f64], &[f64], usize) -> f64 = if w <= 2 {
                dot_eo_muladd
            } else if w == 3 || !wide {
                dot_eo_fma
            } else {
                dot_seq_fma
            };
            for j in j0..j0 + w {
                for i in i0..i0 + rows {
                    let s = dot(&a[i * lda..], &b[j * ldb..], k);
                    c[i + j * ldc] = c[i + j * ldc] + s;
                }
            }
        }
    }
}

fn blk(rem: usize, big: usize, cap: usize, lim: usize) -> usize {
    if rem >= big {
        cap
    } else if rem > lim {
        rem.div_ceil(2)
    } else {
        rem
    }
}

/// driver blocking: k outer (blocks of <= 192), then m, then n (n > 144 split)
#[allow(clippy::too_many_arguments)]
pub fn emu_gemm_nocopy_tn(m: usize, n: usize, k: usize, a: &[f64], lda: usize, b: &[f64], ldb: usize, c: &mut [f64], ldc: usize) {
    let mut p0 = 0;
    while p0 < k {
        let kb = blk(k - p0, 384, 192, 192);
        let mut i0 = 0;
        while i0 < m {
            let mb = blk(m - i0, 8064, 4032, 6048);
            let mut j0 = 0;
            while j0 < n {
                let nb = blk(n - j0, 192, 96, 144);
                kernel_tn_b1(mb, nb, kb, &a[p0 + i0 * lda..], lda, &b[p0 + j0 * ldb..], ldb, &mut c[i0 + j0 * ldc..], ldc);
                j0 += nb;
            }
            i0 += mb;
        }
        p0 += kb;
    }
}

// ======== part_packed.rs ========
// Port of part_packed.c: MKL AVX2 packed DGEMM('T','N') path (fullacopybcopy + dgemm_kernel_0), alpha = beta = 1.
// Column blocks of nc = 24 (k < 48) or 192; per block 4-column tiles then 2, 1. Rows: 12-row tiles, then m%12 as 8,4,2,1.
// Per element q accumulators by p mod q (p < 4*(k/4), fma, start +0); combine when k >= 4: q=2 a0+a1,
// q=4 (a0+a1)+(a2+a3); k%4 tail fma'd into the sum; C = C + sum. 12x1 tiles add an extra +0.0 after the loop.

fn dot_q(a: &[f64], b: &[f64], k: usize, q: usize, zadd: bool) -> f64 {
    let k4 = k & !3;
    let mut acc = [0.0f64; 4];
    for p in 0..k4 {
        acc[p % q] = a[p].mul_add(b[p], acc[p % q]);
    }
    let mut s = acc[0];
    if k4 != 0 {
        if q == 2 {
            s = acc[0] + acc[1];
        } else if q == 4 {
            s = (acc[0] + acc[1]) + (acc[2] + acc[3]);
        } else if zadd {
            s = acc[0] + 0.0;
        }
    }
    for p in k4..k {
        s = a[p].mul_add(b[p], s);
    }
    s
}

// tile size along a dimension of length len: full tiles of `full`, then the rest by its bits (largest first)
fn tile_of(idx: usize, len: usize, full: usize) -> usize {
    let nf = len - len % full;
    if idx < nf {
        return full;
    }
    let (mut t, mut off) = (if full == 12 { 8 } else { 2 }, nf);
    while t >= 1 {
        if (len - nf) & t != 0 {
            if idx < off + t {
                return t;
            }
            off += t;
        }
        t >>= 1;
    }
    unreachable!()
}

fn q_of(mr: usize, nr: usize) -> usize {
    match (mr, nr) {
        (8, 1) | (4, 1) => 4,
        (4, 4) | (2, 1) | (1, _) => 2,
        _ => 1,
    }
}

/// C[i + j*ldc] += sum_p A[p + i*lda] * B[p + j*ldb]. shortcut: assumes k <= 256 (MKL splits k above), extend if k grows
pub fn emu_gemm_packed_tn(m: usize, n: usize, k: usize, a: &[f64], lda: usize, b: &[f64], ldb: usize, c: &mut [f64], ldc: usize) {
    let nc = if k < 48 { 24 } else { 192 };
    let mut j0 = 0;
    while j0 < n {
        let nb = (n - j0).min(nc);
        for jj in 0..nb {
            let (j, nr) = (j0 + jj, tile_of(jj, nb, 4));
            for i in 0..m {
                let mr = tile_of(i, m, 12);
                c[i + j * ldc] += dot_q(&a[i * lda..], &b[j * ldb..], k, q_of(mr, nr), mr == 12 && nr == 1);
            }
        }
        j0 += nc;
    }
}

// ======== part_trsmv1.rs ========
// DTRSM('L','U','T','U') v1 path (m <= 12, or m % 4 == 0 && m <= 88). Port of part_trsmv1.c.

/// t - sum_{p<i} u[p]*x[p] in mkl_blas_avx2_dtrsm_pst's order (Left/Upper/Trans branch)
fn dot_trsm_pst(mut t: f64, u: &[f64], x: &[f64], i: usize) -> f64 {
    let nb16 = i & !15;
    if nb16 > 0 {
        let mut a = [[0.0f64; 4]; 4];
        a[0][0] = t;
        for b in (0..nb16).step_by(16) {
            for q in 0..4 {
                for l in 0..4 {
                    let p = b + 4 * q + l;
                    a[q][l] = (-u[p]).mul_add(x[p], a[q][l]);
                }
            }
        }
        let s: Vec<f64> = (0..4).map(|l| (a[0][l] + a[1][l]) + (a[2][l] + a[3][l])).collect();
        t = (s[0] + s[2]) + (s[1] + s[3]);
    }
    let nb4 = (i - nb16) & !3;
    if nb4 > 0 {
        let mut v = [t, 0.0, 0.0, 0.0];
        for g in (nb16..nb16 + nb4).step_by(4) {
            for l in 0..4 {
                v[l] = v[l] - u[g + l] * x[g + l];
            }
        }
        t = (v[0] + v[2]) + (v[1] + v[3]);
    }
    for p in nb16 + nb4..i {
        t = t - u[p] * x[p];
    }
    t
}

/// mkl_blas_avx2_dtrsm_pst('L','U','T','U', m, n, 1.0, A, lda, B, ldb); a and b start at the block's (0,0)
fn trsm_pst_lutu(m: usize, n: usize, a: &[f64], lda: usize, b: &mut [f64], ldb: usize) {
    for j in 0..n {
        let x = &mut b[j * ldb..];
        for i in 1..m {
            x[i] = dot_trsm_pst(x[i], &a[i * lda..], x, i);
        }
    }
}

pub fn trsm_v1(m: usize, n: usize, a: &[f64], lda: usize, b: &mut [f64], ldb: usize) {
    let (m4, mr) = (m & !3, m & 3);
    let mut n4 = n & !3;
    if m4 > 0 && n4 > 0 {
        // ker_llu_a4_b8/b4: one fused chain per element, p ascending
        for j in 0..n4 {
            let x = &mut b[j * ldb..];
            for i in 0..m4 {
                let mut acc = x[i];
                for p in 0..i {
                    acc = (-a[p + i * lda]).mul_add(x[p], acc);
                }
                x[i] = acc;
            }
        }
        if mr > 0 {
            // dgemm_pst('T','N', mr, n4, m4, -1, ..., 1, ...): 4 lanes of mul+add, (l0+l2)+(l1+l3), then C - s
            for j in 0..n4 {
                let x = &mut b[j * ldb..];
                for i in m4..m {
                    let u = &a[i * lda..];
                    let mut v = [0.0f64; 4];
                    for g in (0..m4).step_by(4) {
                        for l in 0..4 {
                            v[l] = u[g + l] * x[g + l] + v[l];
                        }
                    }
                    let s = (v[0] + v[2]) + (v[1] + v[3]);
                    x[i] = 1.0f64.mul_add(x[i], -s);
                }
            }
            trsm_pst_lutu(mr, n4, &a[m4 + m4 * lda..], lda, &mut b[m4..], ldb);
        }
    }
    if m4 == 0 {
        n4 = 0;
    }
    if n > n4 {
        trsm_pst_lutu(m, n - n4, a, lda, &mut b[n4 * ldb..], ldb);
    }
}

// ======== part_trsmbig.rs ========
// Bit-exact emulation of MKL (AVX2, as linked in libgurobi130.so) for
//   DTRSM('L','U','T','U', m, n, 1.0, A, lda, B, ldb) on the "fullacopybcopy" strategy path
//   (m > 12 and !(m % 4 == 0 && m <= 88), n >= 2), kernel mkl_blas_avx2_dtrsm_kernel_ll_0,
// and DTRSV('U','T','U') with incx = 1 (mkl_blas_avx2_dtrsv_utu; DTRSM with n == 1 goes there).
// U^T is lower: L[i][p] = A[p + i*lda] for p < i. The diagonal of A is ignored. Port of part_trsmbig.c.

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

// ======== emu_dispatch.rs ========
// ---- public entry points: the dispatch of the MKL in libgurobi130.so on AVX2, alpha = beta = 1 ----

/// DGEMM('T','N', m, n, k, 1, A, lda, B, ldb, 1, C, ldc): c[i + j*ldc] += sum_p a[p + i*lda] * b[p + j*ldb]
#[allow(clippy::too_many_arguments)]
pub fn gemm_tn(m: usize, n: usize, k: usize, a: &[f64], lda: usize, b: &[f64], ldb: usize, c: &mut [f64], ldc: usize) {
    if m == 0 || n == 0 || k == 0 {
        return; // beta = 1: nothing to do
    }
    if m == 1 || n == 1 {
        emu_gemm_tn_gemv(m, n, k, a, lda, b, ldb, c, ldc); // mkl_blas_dgemm -> dgemv
    } else if k == 1 {
        // mkl_blas_dgemmger: rank-1 update, one fma per element
        for j in 0..n {
            for i in 0..m {
                c[i + j * ldc] = a[i * lda].mul_add(b[j * ldb], c[i + j * ldc]);
            }
        }
    } else if mkl_gemm_is_nocopy(m, n, k) {
        emu_gemm_nocopy_tn(m, n, k, a, lda, b, ldb, c, ldc);
    } else {
        // shortcut: the packed path is exact for k <= 256 (block widths are <= 240, so always); MKL splits larger k
        emu_gemm_packed_tn(m, n, k, a, lda, b, ldb, c, ldc);
    }
}

/// DTRSM('L','U','T','U', m, n, 1, A, lda, B, ldb): solve U^T X = B in place, U = strict upper triangle of A
pub fn trsm_lutu(m: usize, n: usize, a: &[f64], lda: usize, b: &mut [f64], ldb: usize) {
    if m == 0 || n == 0 {
        return;
    }
    if n == 1 {
        // shortcut: dtrsv is exact for m <= 32 (bench LPs: m <= 7); larger m runs the same kernel without MKL's
        // 32-row dgemv blocking (rounding-level difference). Port the blocking when a model hits it.
        emu_trsv_utu(m, a, lda, b); // mkl_blas_dtrsm -> dtrsv
    } else if m <= 12 || (m.is_multiple_of(4) && m <= 88) {
        trsm_v1(m, n, a, lda, b, ldb); // mkl_blas_avx2_xdtrsm_v1
    } else {
        // shortcut: exact for m <= 254 (block widths are <= 240, so always); MKL uses 192-row panels from m = 256
        emu_trsm_big(m, n, a, lda, b, ldb); // xdtrsm_left_lower_fullacopybcopy
    }
}
