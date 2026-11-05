/* Emulation of MKL (libgurobi130.so, AVX2) DGEMM('T','N', alpha=1, beta=1) nocopy path:
   mkl_blas_avx2_xdgemm -> mkl_blas_avx2_xdgemm_nocopy_driver -> mkl_blas_avx2_dgemm_kernel_nocopy_TN_b1.
   C[i,j] += sum_p A[p + i*lda] * B[p + j*ldb]. Build: gcc -O2 -ffp-contract=off, explicit fma() only. */
#include <math.h>

/* nocopy vs fullacopybcopy choice in mkl_blas_avx2_xdgemm for transa='T', transb='N', alpha != 0
   (m == 1 or n == 1 never get here: mkl_blas_dgemm sends them to dgemv). */
int mkl_gemm_is_nocopy(int m, int n, int k, int lda, int ldb, int ldc) {
    (void)lda; (void)ldb; (void)ldc;
    if (m <= 1 || n <= 1 || k <= 1) return 0; /* dgemv (m or n == 1) / dgemmger (k == 1) paths */
    if (k <= 128 && (long)m * n / (m + n) <= 10) return m <= 100 || (n <= 3 && k <= 3);
    return m < 56 || n < 56;
}

/* the three per-element accumulation patterns of the TN_b1 kernel; accumulators start at +0.0 */
static double dot_seq_fma(const double *a, const double *b, int k) { /* one fma chain */
    double s = 0.0;
    for (int p = 0; p < k; p++) s = fma(a[p], b[p], s);
    return s;
}
static double dot_eo_fma(const double *a, const double *b, int k) { /* even/odd fma chains, odd tail fma'd last */
    double e = 0.0, o = 0.0;
    int ke = k & ~1;
    for (int p = 0; p < ke; p += 2) { e = fma(a[p], b[p], e); o = fma(a[p + 1], b[p + 1], o); }
    double s = e + o;
    if (k & 1) s = fma(a[ke], b[ke], s);
    return s;
}
static double dot_eo_muladd(const double *a, const double *b, int k) { /* same, with separate mul and add */
    double e = 0.0, o = 0.0;
    int ke = k & ~1;
    for (int p = 0; p < ke; p += 2) { e = e + a[p] * b[p]; o = o + a[p + 1] * b[p + 1]; }
    double s = e + o;
    if (k & 1) s = s + a[ke] * b[ke];
    return s;
}

/* one kernel call (TN_b1): rows in blocks of 8 (a tail of 5..7 rows is a masked 8-block, 1..4 rows a 4-block),
   columns in groups of 6 then one group of n%6. */
static void kernel_tn_b1(int m, int n, int k, const double *A, long lda, const double *B, long ldb, double *C, long ldc) {
    for (int i0 = 0; i0 < m; i0 += 8) {
        int r = m - i0, rows = r >= 8 ? 8 : r, wide = r >= 5;
        for (int j0 = 0; j0 < n; j0 += 6) {
            int w = n - j0 >= 6 ? 6 : n - j0;
            double (*dot)(const double *, const double *, int);
            if (w <= 2) dot = dot_eo_muladd;
            else if (w == 3 || !wide) dot = dot_eo_fma;
            else dot = dot_seq_fma;
            for (int j = j0; j < j0 + w; j++)
                for (int i = i0; i < i0 + rows; i++)
                    C[i + j * ldc] = C[i + j * ldc] + dot(A + i * lda, B + j * ldb, k);
        }
    }
}

/* driver blocking (TN branch of mkl_blas_avx2_xdgemm_nocopy_driver): k outer, then m, then n */
static int blk(int rem, int big, int cap, int lim) { return rem >= big ? cap : rem > lim ? (rem + 1) / 2 : rem; }

void emu_gemm_nocopy_tn(int m, int n, int k, const double *A, int lda, const double *B, int ldb, double *C, int ldc) {
    for (int p0 = 0, kb; p0 < k; p0 += kb) {
        kb = blk(k - p0, 384, 192, 192);
        for (int i0 = 0, mb; i0 < m; i0 += mb) {
            mb = blk(m - i0, 8064, 4032, 6048);
            for (int j0 = 0, nb; j0 < n; j0 += nb) {
                nb = blk(n - j0, 192, 96, 144);
                kernel_tn_b1(mb, nb, kb, A + p0 + (long)i0 * lda, lda, B + p0 + (long)j0 * ldb, ldb,
                             C + i0 + (long)j0 * ldc, ldc);
            }
        }
    }
}
