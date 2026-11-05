"""log the (m, n, k, alpha*, Apack, Bpack, C, ldc) args of every mkl_blas_avx2_dgemm_kernel_0 call for a TN gemm"""
import ctypes, sys, os
import numpy as np
import mk
at = ctypes.CDLL(os.path.dirname(os.path.abspath(__file__)) + "/part_packed_argtrace.so")
def kargs(m, n, k, lda=None, ldb=None, ldc=None):
    lda, ldb, ldc = lda or k, ldb or k, ldc or m
    rng = np.random.default_rng(0)
    at.at_set(ctypes.c_uint64(mk.BASE + 0x245f300))
    mk.gemm_tn(m, n, k, rng.random(lda * m), lda, rng.random(ldb * n), ldb, rng.random(ldc * n), ldc)
    at.at_off()
    buf = (ctypes.c_int64 * 800000)()
    cnt = at.at_get(buf)
    return [list(buf[8 * i:8 * i + 8]) for i in range(cnt)]
if __name__ == "__main__":
    m, n, k = map(int, sys.argv[1:4])
    for r in kargs(m, n, k): print("m=%d n=%d k=%d  C=%x ldc=%d" % (r[0], r[1], r[2], r[6], r[7]))
