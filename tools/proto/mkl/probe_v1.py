import ctypes, sys
import numpy as np
import mk, argtrace
m, n = int(sys.argv[1]), int(sys.argv[2])
lda, ldb = m + 3, m + 5
U = np.arange(lda * m, dtype=float); B = np.zeros(ldb * n)
u = mk.aligned(U, 0); b = mk.aligned(B, 0)
ua, ba = u.ctypes.data, b.ctypes.data
rd = lambda p, t: getattr(ctypes, t).from_address(p).value
def f():
    P = mk._p
    mk.DTRSM(P(ctypes.c_char(b"L")), P(ctypes.c_char(b"U")), P(ctypes.c_char(b"T")), P(ctypes.c_char(b"U")),
          P(ctypes.c_int(m)), P(ctypes.c_int(n)), P(ctypes.c_double(1.0)), u.ctypes.data_as(mk.VP), P(ctypes.c_int(lda)),
          b.ctypes.data_as(mk.VP), P(ctypes.c_int(ldb)))
def desc(x):
    if ua <= x < ua + 8 * len(U): return f"A+{(x - ua) // 8}"
    if ba <= x < ba + 8 * len(B): return f"B+{(x - ba) // 8}"
    return hex(x)
res = argtrace.run(['mkl_blas_avx2_dgemm_pst', 'mkl_blas_avx2_dtrsm_pst', 'mkl_blas_avx2_dtrsm_ker_llu_a4_b8', 'mkl_blas_avx2_dtrsm_ker_llu_a4_b4', 'mkl_blas_avx2_dtrsm_left'], f)
print('lda', lda, 'ldb', ldb)
for nm, a in res:
    print(nm, [desc(x) for x in a])
