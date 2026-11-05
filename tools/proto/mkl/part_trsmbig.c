/* Bit-exact emulation of MKL (AVX2, as linked in libgurobi130.so) for
 *   DTRSM('L','U','T','U', m, n, 1.0, A, lda, B, ldb)  on the "fullacopybcopy" strategy path
 *   (mkl_blas_avx2_xdtrsm: m > 12 and !(m % 4 == 0 && m <= 88)), kernel mkl_blas_avx2_dtrsm_kernel_ll_0,
 * and DTRSV('U','T','U') with incx = 1 (mkl_blas_avx2_dtrsv_utu; DTRSM with n == 1 goes there).
 * U^T is lower: L[i][p] = A[p + i*lda] for p < i. The diagonal of A is ignored (the packed diagonal is set to 1.0). */
#include <math.h>

/* ll_0 (mkl_blas_avx2_dtrsm_kernel_ll_0, A packed negated, packed diagonal set to 1.0).
 * Rows: blocks of R = 12 from the top, then the remainder m % 12 as R = 8, 4, 2, 1 (in that order).
 * Columns: groups of C = 4 from the left, then C = 2 if n & 2, then C = 1 if n & 1 (the driver's column chunks of
 * 132 (m < 49) or 192 columns are multiples of 4, so they do not change the grouping).
 * Row i of a block starting at r0 (k = r0 solved rows above):
 *   acc = dot(-L[i][0..k), x[0..k)) by fma, mode by (R, C) below; t = B[i] + acc;
 *   then for p = r0..i-1 (right-looking inside the block): t = fma(x[p], -L[i][p], t); x[i] = t * 1.0.
 * dot modes: 1 = one fma chain from +0; 2 / 4 = for p < 4*(k/4) chain p % 2 (p % 4) into its own accumulator from +0,
 * then combine (a0 + a1) or ((a0 + a1) + (a2 + a3)), then the last k % 4 terms chain onto that; k < 4: one chain.
 * Mode 0 = one chain plus "+ 0.0" after the first 4*(k/4) terms (R = 12, C = 1; only differs for an underflowed -0). */
static const int MODE[5][3] = { /* R = 12, 8, 4, 2, 1 ; C = 4, 2, 1 */
    {1, 1, 0}, {1, 1, 4}, {2, 1, 4}, {1, 1, 2}, {2, 2, 2}};

static double negdot(int mode, const double *l, const double *x, int k) {
    int k4 = k & ~3, p = 0;
    double acc = 0.0;
    if (k4 && mode >= 2) {
        double a[4] = {0.0, 0.0, 0.0, 0.0};
        for (; p < k4; p++) a[p % mode] = fma(-l[p], x[p], a[p % mode]);
        acc = mode == 2 ? a[0] + a[1] : (a[0] + a[1]) + (a[2] + a[3]);
    } else if (k4 && mode == 0) {
        for (; p < k4; p++) acc = fma(-l[p], x[p], acc);
        acc = acc + 0.0;
    }
    for (; p < k; p++) acc = fma(-l[p], x[p], acc);
    return acc;
}

void emu_trsm_big(int m, int n, const double *A, int lda, double *B, int ldb) {
    /* shortcut: verified for m <= 254; from about m = 256 MKL splits rows into 192-row panels (not emulated) */
    for (int j = 0; j < n; j++) {
        double *x = B + (long)j * ldb;
        int r = n & 3, cg = j < n - r ? 0 : ((r & 2) && j < n - r + 2) ? 1 : 2;
        for (int r0 = 0, R = 12, ri = 0; r0 < m; r0 += R) {
            int left = m - r0;
            if (left < 12) { R = left & 8 ? 8 : left & 4 ? 4 : left & 2 ? 2 : 1; ri = R == 8 ? 1 : R == 4 ? 2 : R == 2 ? 3 : 4; }
            for (int i = r0; i < r0 + R; i++) x[i] = x[i] + negdot(MODE[ri][cg], A + (long)i * lda, x, r0);
            for (int p = r0; p < r0 + R; p++)
                for (int i = p + 1; i < r0 + R; i++) x[i] = fma(x[p], -A[p + (long)i * lda], x[i]);
        }
    }
}

/* horizontal sum of a 4-lane accumulator as MKL does it: (l0 + l2) + (l1 + l3) */
static double hsum4(const double v[4]) { return (v[0] + v[2]) + (v[1] + v[3]); }

/* dtrsv_utu, incx = 1, m <= 32 (larger m: xdtrsv splits into 32-row blocks with a dgemv update, not emulated).
 * Rows in pairs (i0, i0+1), then a last odd row. s = x[i] - sum_{p<i0} A[p + i*lda] * x[p]. */
void emu_trsv_utu(int m, const double *A, int lda, double *x) {
    /* shortcut: m <= 32 only (all recorded n == 1 calls have m <= 7); larger m needs the dgemv_t emulation */
    int i0 = 0;
    for (; i0 + 1 < m; i0 += 2) {
        const double *c0 = A + (long)i0 * lda, *c1 = A + (long)(i0 + 1) * lda;
        double s0 = x[i0], s1 = x[i0 + 1];
        int q = 0;
        if (i0 >= 16) {
            /* col i0: two fma accumulators (blocks 0,2 -> a; 1,3 -> b) of 4 lanes, s0 in lane 0 of a;
             * col i0+1: one accumulator, mul then sub, s1 in lane 0 */
            double a[4] = {s0, 0, 0, 0}, b[4] = {0, 0, 0, 0}, c[4] = {s1, 0, 0, 0};
            for (; q + 16 <= i0; q += 16)
                for (int l = 0; l < 4; l++) {
                    a[l] = fma(-x[q + l], c0[q + l], a[l]);
                    a[l] = fma(-x[q + 8 + l], c0[q + 8 + l], a[l]);
                    b[l] = fma(-x[q + 4 + l], c0[q + 4 + l], b[l]);
                    b[l] = fma(-x[q + 12 + l], c0[q + 12 + l], b[l]);
                    for (int u = 0; u < 16; u += 4) c[l] = c[l] - x[q + u + l] * c1[q + u + l];
                }
            for (int l = 0; l < 4; l++) a[l] = a[l] + b[l];
            s0 = hsum4(a); s1 = hsum4(c);
        }
        if (i0 - q >= 4) {
            double a[4] = {s0, 0, 0, 0}, c[4] = {s1, 0, 0, 0};
            for (; q + 4 <= i0; q += 4)
                for (int l = 0; l < 4; l++) { a[l] = a[l] - x[q + l] * c0[q + l]; c[l] = c[l] - x[q + l] * c1[q + l]; }
            s0 = hsum4(a); s1 = hsum4(c);
        }
        for (; q < i0; q++) { s0 = s0 - x[q] * c0[q]; s1 = s1 - x[q] * c1[q]; }
        x[i0] = s0;
        x[i0 + 1] = fma(-s0, c1[i0], s1);
    }
    if (m & 1) {
        int i = m - 1, q = 0;
        const double *c0 = A + (long)i * lda;
        double s = x[i];
        if (i >= 16) {
            double a[4][4] = {{s, 0, 0, 0}};
            for (; q + 16 <= i; q += 16)
                for (int u = 0; u < 4; u++)
                    for (int l = 0; l < 4; l++) a[u][l] = fma(-x[q + 4 * u + l], c0[q + 4 * u + l], a[u][l]);
            double t[4];
            for (int l = 0; l < 4; l++) t[l] = (a[0][l] + a[1][l]) + (a[2][l] + a[3][l]);
            s = hsum4(t);
        }
        if (i - q >= 4) {
            double a[4] = {s, 0, 0, 0};
            for (; q + 4 <= i; q += 4)
                for (int l = 0; l < 4; l++) a[l] = a[l] - x[q + l] * c0[q + l];
            s = hsum4(a);
        }
        for (; q < i; q++) s = s - x[q] * c0[q];
        x[i] = s;
    }
}
