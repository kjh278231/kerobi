#include <assert.h>
/* ---- public entry points: the dispatch of MKL 2025 (libgurobi130.so) on AVX2, alpha = beta = 1 ---- */

/* DGEMM('T','N', m, n, k, 1, A, lda, B, ldb, 1, C, ldc): C[i + j*ldc] += sum_p A[p + i*lda] * B[p + j*ldb] */
void emu_gemm_tn(int m, int n, int k, const double *A, int lda, const double *B, int ldb, double *C, int ldc) {
    if (m <= 0 || n <= 0 || k <= 0) return;                    /* beta = 1: nothing to do */
    if (m == 1 || n == 1) { emu_gemm_tn_gemv(m, n, k, A, lda, B, ldb, C, ldc); return; } /* mkl_blas_dgemm -> dgemv */
    if (k == 1) {                                               /* mkl_blas_dgemmger: rank-1 update, one fma */
        for (int j = 0; j < n; j++)
            for (int i = 0; i < m; i++) C[i + (long)j * ldc] = fma(A[(long)i * lda], B[(long)j * ldb], C[i + (long)j * ldc]);
        return;
    }
    if (mkl_gemm_is_nocopy(m, n, k, lda, ldb, ldc)) emu_gemm_nocopy_tn(m, n, k, A, lda, B, ldb, C, ldc);
    else { assert(k <= 256); emu_gemm_packed_tn(m, n, k, A, lda, B, ldb, C, ldc); } /* verified range only */
}

/* DTRSM('L','U','T','U', m, n, 1, A, lda, B, ldb): solve U^T X = B in place, U = strict upper triangle of A */
void emu_trsm_lutu(int m, int n, const double *A, int lda, double *B, int ldb) {
    if (m <= 0 || n <= 0) return;
    if (n == 1) { assert(m <= 32); emu_trsv_utu(m, A, lda, B); return; } /* mkl_blas_dtrsm -> dtrsv */
    if (m <= 12 || (m % 4 == 0 && m <= 88)) emu_trsm_v1(m, n, A, lda, B, ldb); /* mkl_blas_avx2_xdtrsm_v1 */
    else { assert(m <= 254); emu_trsm_big(m, n, A, lda, B, ldb); } /* xdtrsm_left_lower_fullacopybcopy */
}
