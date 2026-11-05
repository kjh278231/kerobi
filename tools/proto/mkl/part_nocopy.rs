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
        (rem + 1) / 2
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
