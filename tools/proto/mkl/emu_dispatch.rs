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
        assert!(k <= 256, "packed DGEMM emulated for k <= 256 only");
        emu_gemm_packed_tn(m, n, k, a, lda, b, ldb, c, ldc);
    }
}

/// DTRSM('L','U','T','U', m, n, 1, A, lda, B, ldb): solve U^T X = B in place, U = strict upper triangle of A
pub fn trsm_lutu(m: usize, n: usize, a: &[f64], lda: usize, b: &mut [f64], ldb: usize) {
    if m == 0 || n == 0 {
        return;
    }
    if n == 1 {
        assert!(m <= 32, "DTRSV emulated for m <= 32 only");
        emu_trsv_utu(m, a, lda, b); // mkl_blas_dtrsm -> dtrsv
    } else if m <= 12 || (m % 4 == 0 && m <= 88) {
        trsm_v1(m, n, a, lda, b, ldb); // mkl_blas_avx2_xdtrsm_v1
    } else {
        assert!(m <= 254, "DTRSM strategy path emulated for m <= 254 only");
        emu_trsm_big(m, n, a, lda, b, ldb); // xdtrsm_left_lower_fullacopybcopy
    }
}
