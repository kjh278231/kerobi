"""direct ctypes access to mkl_blas_avx2_dtrsm_pst / dgemm_pst (Fortran-style pointer args)"""
import ctypes
import numpy as np
import mk, argtrace
VP = mk.VP
TRSM_PST = ctypes.CFUNCTYPE(None, *[VP] * 11)(mk.BASE + argtrace.SYM['mkl_blas_avx2_dtrsm_pst'])
GEMM_PST = ctypes.CFUNCTYPE(None, *[VP] * 13)(mk.BASE + argtrace.SYM['mkl_blas_avx2_dgemm_pst'])
P = mk._p
def trsm_pst(side, uplo, tr, diag, m, n, alpha, A, lda, B, ldb, al=(0, 0)):
    a, b = mk.aligned(A, al[0]), mk.aligned(B, al[1]); mk._keep.clear()
    TRSM_PST(P(ctypes.c_char(side)), P(ctypes.c_char(uplo)), P(ctypes.c_char(tr)), P(ctypes.c_char(diag)), P(ctypes.c_long(m)), P(ctypes.c_long(n)),
             P(ctypes.c_double(alpha)), a.ctypes.data_as(VP), P(ctypes.c_long(lda)), b.ctypes.data_as(VP), P(ctypes.c_long(ldb)))
    return b.copy()
def gemm_pst(ta, tb, m, n, k, alpha, A, lda, B, ldb, beta, C, ldc, al=(0, 0, 0)):
    a, b, c = mk.aligned(A, al[0]), mk.aligned(B, al[1]), mk.aligned(C, al[2]); mk._keep.clear()
    GEMM_PST(P(ctypes.c_char(ta)), P(ctypes.c_char(tb)), P(ctypes.c_long(m)), P(ctypes.c_long(n)), P(ctypes.c_long(k)), P(ctypes.c_double(alpha)),
             a.ctypes.data_as(VP), P(ctypes.c_long(lda)), b.ctypes.data_as(VP), P(ctypes.c_long(ldb)), P(ctypes.c_double(beta)), c.ctypes.data_as(VP), P(ctypes.c_long(ldc)))
    return c.copy()
