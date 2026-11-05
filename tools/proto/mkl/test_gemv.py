"""check part_gemv.c against the real MKL on every m==1 or n==1 shape of gshapes.json (plus ld/ldc/alignment variants)"""
import ctypes, json, sys, os
import numpy as np
import mk
D = os.path.dirname(os.path.abspath(__file__))
so = ctypes.CDLL(D + "/part_gemv.so")
VP = ctypes.c_void_p
rng = np.random.default_rng(int(sys.argv[1]) if len(sys.argv) > 1 else 5)
def rnd(n): return rng.uniform(-1, 1, n) * 2.0 ** rng.integers(-6, 7, n)
def one(m, n, k, lda, ldb, ldc, al):
    A, B, C = rnd(lda * m), rnd(ldb * n), rnd(ldc * n)
    R = mk.gemm_tn(m, n, k, A, lda, B, ldb, C, ldc, al)
    E = C.copy()
    so.emu_gemm_tn_gemv(m, n, k, A.ctypes.data_as(VP), lda, B.ctypes.data_as(VP), ldb, E.ctypes.data_as(VP), ldc)
    return np.array_equal(R.view(np.int64), E.view(np.int64))
shapes = [s for s in json.load(open(D + "/gshapes.json")) if s[0] == 1 or s[1] == 1]
ng = len(shapes)
shapes += [[1, n, k] for n in range(1, 10) for k in range(1, 40)] + [[m, 1, k] for m in range(2, 10) for k in range(1, 40)]
good = 0
bad = []
for m, n, k in shapes:
    ok = True
    for t in range(6):
        lda, ldb = k + (t % 3) * int(rng.integers(0, 5)), k + (t % 2) * int(rng.integers(0, 5))
        ldc = m + (t % 3 == 2) * int(rng.integers(1, 13))
        al = tuple(int(x) for x in rng.integers(0, 8, 3) * 8)
        ok &= one(m, n, k, lda, ldb, ldc, al)
    good += ok
    if not ok: bad.append((m, n, k))
gb = sum(1 for s in bad if list(s) in shapes[:ng])
print(f"gemv path: gshapes {ng - gb}/{ng} bit-equal, all incl. sweep {good}/{len(shapes)}", "bad:", bad[:20])
