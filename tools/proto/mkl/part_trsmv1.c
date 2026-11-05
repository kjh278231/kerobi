/* DTRSM('L','U','T','U') v1 path of MKL AVX2 (m <= 12, or m % 4 == 0 && m <= 88): mkl_blas_avx2_dtrsm_left_ker.
   X overwrites B; U = upper triangle of A (unit diagonal, A's diagonal ignored); L = U^T, L[i][p] = A[p + i*lda]. */
#include <math.h>

/* t - sum_{p<i} u[p]*x[p] in mkl_blas_avx2_dtrsm_pst's order (Left/Upper/Trans branch); u, x contiguous */
static double dot_trsm_pst(double t, const double *u, const double *x, int i) {
    int nb16 = i & ~15;
    if (nb16) {
        double a[4][4] = {{t, 0, 0, 0}, {0, 0, 0, 0}, {0, 0, 0, 0}, {0, 0, 0, 0}};
        for (int b = 0; b < nb16; b += 16)
            for (int q = 0; q < 4; q++)
                for (int l = 0; l < 4; l++) { int p = b + 4 * q + l; a[q][l] = fma(-u[p], x[p], a[q][l]); }
        double s[4];
        for (int l = 0; l < 4; l++) s[l] = (a[0][l] + a[1][l]) + (a[2][l] + a[3][l]);
        t = (s[0] + s[2]) + (s[1] + s[3]);
    }
    int nb4 = (i - nb16) & ~3;
    if (nb4) {
        double v[4] = {t, 0, 0, 0};
        for (int g = nb16; g < nb16 + nb4; g += 4)
            for (int l = 0; l < 4; l++) v[l] = v[l] - u[g + l] * x[g + l];
        t = (v[0] + v[2]) + (v[1] + v[3]);
    }
    for (int p = nb16 + nb4; p < i; p++) t = t - u[p] * x[p];
    return t;
}

/* mkl_blas_avx2_dtrsm_pst('L','U','T','U', m, n, 1.0, A, lda, B, ldb) */
static void trsm_pst_lutu(int m, int n, const double *A, int lda, double *B, int ldb) {
    for (int j = 0; j < n; j++) {
        double *x = B + (long)j * ldb;
        for (int i = 1; i < m; i++) x[i] = dot_trsm_pst(x[i], A + (long)i * lda, x, i);
    }
}

void emu_trsm_v1(int m, int n, const double *A, int lda, double *B, int ldb) {
    int m4 = m & ~3, mr = m - m4, n4 = n & ~3;
    if (m4 > 0 && n4 > 0) {
        /* ker_llu_a4_b8 / b4: per element, one fused chain x_i = b_i - u_0 x_0 - u_1 x_1 - ... in p order */
        for (int j = 0; j < n4; j++) {
            double *x = B + (long)j * ldb;
            for (int i = 0; i < m4; i++) {
                double acc = x[i];
                for (int p = 0; p < i; p++) acc = fma(-A[p + (long)i * lda], x[p], acc);
                x[i] = acc;
            }
        }
        if (mr > 0) {
            /* dgemm_pst('T','N', mr, n4, m4, -1, A + m4*lda, lda, B, ldb, 1, B + m4, ldb): k = m4 < 16 here,
               4 lanes of mul+add over k, lanes reduced (l0+l2)+(l1+l3), then C - s */
            for (int j = 0; j < n4; j++) {
                double *x = B + (long)j * ldb;
                for (int i = m4; i < m; i++) {
                    const double *u = A + (long)i * lda;
                    double v[4] = {0, 0, 0, 0};
                    for (int g = 0; g < m4; g += 4)
                        for (int l = 0; l < 4; l++) v[l] = u[g + l] * x[g + l] + v[l];
                    double s = (v[0] + v[2]) + (v[1] + v[3]);
                    x[i] = fma(1.0, x[i], -1.0 * s);
                }
            }
            trsm_pst_lutu(mr, n4, A + m4 + (long)m4 * lda, lda, B + m4, ldb);
        }
    }
    if (m4 == 0) n4 = 0;
    if (n > n4) trsm_pst_lutu(m, n - n4, A, lda, B + (long)n4 * ldb, ldb);
}
