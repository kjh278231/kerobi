"""ctypes access to the MKL DGEMM/DTRSM statically linked in libgurobi130.so, with controlled buffer alignment."""
import ctypes
import os
import numpy as np
LIBP = os.path.join(os.path.dirname(os.path.abspath(__file__)), '../../../../.venv/lib/python3.13/site-packages/gurobipy/.libs/libgurobi130.so')
_lib = ctypes.CDLL(LIBP)
BASE = next(int(ln.split("-")[0], 16) for ln in open("/proc/self/maps") if "libgurobi130.so" in ln and ln.split()[2] == "00000000")
VP = ctypes.c_void_p
DGEMM = ctypes.CFUNCTYPE(None, *[VP] * 13)(BASE + 0x9ffc30)
DTRSM = ctypes.CFUNCTYPE(None, *[VP] * 11)(BASE + 0x9ff5e0)
_keep = []
def _p(x):
    _keep.append(x); return ctypes.cast(ctypes.pointer(x), VP)
def aligned(vals, al):
    """copy of vals (float64) in a buffer whose address is == al (mod 64)"""
    vals = np.asarray(vals, dtype=np.float64)
    raw = np.zeros(len(vals) + 16, dtype=np.float64)
    off = ((al - raw.ctypes.data) % 64) // 8
    a = raw[off:off + len(vals)]
    a[:] = vals
    assert len(vals) == 0 or a.ctypes.data % 64 == al
    return a
def gemm_tn(m, n, k, A, lda, B, ldb, C, ldc, al=(0, 0, 0)):
    a, b, c = aligned(A, al[0]), aligned(B, al[1]), aligned(C, al[2])
    _keep.clear()
    DGEMM(_p(ctypes.c_char(b"T")), _p(ctypes.c_char(b"N")), _p(ctypes.c_int(m)), _p(ctypes.c_int(n)), _p(ctypes.c_int(k)),
          _p(ctypes.c_double(1.0)), a.ctypes.data_as(VP), _p(ctypes.c_int(lda)), b.ctypes.data_as(VP), _p(ctypes.c_int(ldb)),
          _p(ctypes.c_double(1.0)), c.ctypes.data_as(VP), _p(ctypes.c_int(ldc)))
    return c.copy()
def trsm_lutu(m, n, U, lda, B, ldb, al=(0, 0)):
    u, b = aligned(U, al[0]), aligned(B, al[1])
    _keep.clear()
    DTRSM(_p(ctypes.c_char(b"L")), _p(ctypes.c_char(b"U")), _p(ctypes.c_char(b"T")), _p(ctypes.c_char(b"U")),
          _p(ctypes.c_int(m)), _p(ctypes.c_int(n)), _p(ctypes.c_double(1.0)), u.ctypes.data_as(VP), _p(ctypes.c_int(lda)),
          b.ctypes.data_as(VP), _p(ctypes.c_int(ldb)))
    return b.copy()
