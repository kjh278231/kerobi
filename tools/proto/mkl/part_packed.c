/* Emulation of the packed DGEMM('T','N') path of MKL AVX2 (alpha = beta = 1):
   mkl_blas_avx2_xdgemm -> d_generic_fullacopybcopy -> dgemm_kernel_0 (12x4 micro-kernel).
   Columns: blocks of nc = 24 (k < 48) or 192 (k >= 48); inside a block, 4-column tiles, then a 2- and a 1-column tile
   (block & 2, block & 1). Rows: 12-row tiles from row 0, then the m % 12 rest as 8-, 4-, 2-, 1-row tiles (its bits).
   Per element: q accumulators (p mod q, for p < 4*(k/4)), all start at +0 and use fma; after that loop (only when k >= 4)
   q=2: a0 + a1, q=4: (a0 + a1) + (a2 + a3); the k % 4 tail is fma'd sequentially into that sum; then C = C + sum.
   q by tile (rows x cols): 1 for 12x4 12x2 8x4 8x2 4x2 2x4 2x2; 2 for 4x4 2x1 1x4 1x2 1x1; 4 for 8x1 4x1;
   12x1 is q=1 plus an extra "+ 0.0" after the loop (only visible on signed zeros). No dependence on ld or alignment. */
#include <math.h>

static double dot_q(const double *a, const double *b, int k, int q, int zadd) {
    int k4 = k & ~3;
    double acc[4] = {0.0, 0.0, 0.0, 0.0}, s;
    for (int p = 0; p < k4; p++) acc[p % q] = fma(a[p], b[p], acc[p % q]);
    s = acc[0];
    if (k4) {
        if (q == 2) s = acc[0] + acc[1];
        else if (q == 4) s = (acc[0] + acc[1]) + (acc[2] + acc[3]);
        else if (zadd) s = acc[0] + 0.0;
    }
    for (int p = k4; p < k; p++) s = fma(a[p], b[p], s);
    return s;
}

/* tile size along a dimension of length len: full tiles of `full`, then the rest by its bits (largest first) */
static int tile_of(int idx, int len, int full) {
    int nf = len - len % full;
    if (idx < nf) return full;
    for (int t = full == 12 ? 8 : 2, off = nf; t >= 1; t >>= 1)
        if ((len - nf) & t) { if (idx < off + t) return t; off += t; }
    return 0; /* unreachable */
}

/* shortcut: assumes k <= 256 (MKL splits k above that: 300 -> 150+150, 512 -> 192+192+128), extend if k grows */
void emu_gemm_packed_tn(int m, int n, int k, const double *A, int lda, const double *B, int ldb, double *C, int ldc) {
    static const int Q[13][5] = {[12] = {0, 1, 1, 0, 1}, [8] = {0, 4, 1, 0, 1}, [4] = {0, 4, 1, 0, 2},
                                 [2] = {0, 2, 1, 0, 1}, [1] = {0, 2, 2, 0, 2}};
    int nc = k < 48 ? 24 : 192;
    for (int j0 = 0; j0 < n; j0 += nc) {
        int nb = n - j0 < nc ? n - j0 : nc;
        for (int jj = 0; jj < nb; jj++) {
            int j = j0 + jj, nr = tile_of(jj, nb, 4);
            for (int i = 0; i < m; i++) {
                int mr = tile_of(i, m, 12);
                C[i + (long)j * ldc] += dot_q(A + (long)i * lda, B + (long)j * ldb, k, Q[mr][nr], mr == 12 && nr == 1);
            }
        }
    }
}
