/* Emulation of mkl_blas_avx2_xdgemv_t (alpha = beta = 1, incx = 1) and of the m==1 / n==1 DGEMM('T','N') that
   mkl_blas_dgemm routes to it. Per column j, 4 lanes l = 0..3 (one ymm accumulator, starts at +0):
     8-blocks p: acc[l] = fma(x[p+l], a[p+l], x[p+4+l]*a[p+4+l]) + acc[l]
     one 4-block if k%8 >= 4: fma into acc (columns in the 4-column groups) or mul + add (2- and 1-column tail groups)
     k%4 tail: masked fma, lanes >= k%4 get fma(0,0,acc) (= acc + 0)
     y[j] += (acc0 + acc1) + (acc2 + acc3)
   Columns are processed in groups of 4, then 2, then 1 (n&~3, n&2, n&1). No dependence on ld or alignment. */
#include <math.h>

void emu_gemv_t(int k, int n, const double *A, int lda, const double *x, double *y, int incy) {
    int k8 = k & ~7, k4 = (k - k8) & 4, r = k & 3, n4 = n & ~3;
    for (int j = 0; j < n; j++) {
        const double *a = A + (long)j * lda;
        double acc[4] = {0.0, 0.0, 0.0, 0.0};
        int p = 0;
        for (; p < k8; p += 8)
            for (int l = 0; l < 4; l++) acc[l] = fma(x[p + l], a[p + l], x[p + 4 + l] * a[p + 4 + l]) + acc[l];
        if (k4) {
            for (int l = 0; l < 4; l++) acc[l] = j < n4 ? fma(x[p + l], a[p + l], acc[l]) : x[p + l] * a[p + l] + acc[l];
            p += 4;
        }
        if (r)
            for (int l = 0; l < 4; l++) acc[l] = l < r ? fma(x[p + l], a[p + l], acc[l]) : fma(0.0, 0.0, acc[l]);
        y[(long)j * incy] += (acc[0] + acc[1]) + (acc[2] + acc[3]);
    }
}

/* DGEMM('T','N', m, n, k, 1, A, lda, B, ldb, 1, C, ldc) for m == 1 or n == 1 (m == 1 is checked first) */
void emu_gemm_tn_gemv(int m, int n, int k, const double *A, int lda, const double *B, int ldb, double *C, int ldc) {
    if (m == 1) emu_gemv_t(k, n, B, ldb, A, C, ldc);
    else emu_gemv_t(k, m, A, lda, B, C, 1);
}
