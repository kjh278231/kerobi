"""check part_packed.c against the real MKL on the packed-path gshapes (dgemm_kernel_0 used) and a sweep of other shapes"""
import ctypes, json, sys, os
import numpy as np
import mk
from part_packed_args import kargs
D = os.path.dirname(os.path.abspath(__file__))
so = ctypes.CDLL(D + "/part_packed.so")
VP = ctypes.c_void_p
rng = np.random.default_rng(int(sys.argv[1]) if len(sys.argv) > 1 else 3)
def rnd(n): return rng.uniform(-1, 1, n) * 2.0 ** rng.integers(-6, 7, n)
def one(m, n, k, lda, ldb, ldc, al):
    A, B, C = rnd(lda * m), rnd(ldb * n), rnd(ldc * n)
    R = mk.gemm_tn(m, n, k, A, lda, B, ldb, C, ldc, al)
    E = C.copy()
    so.emu_gemm_packed_tn(m, n, k, A.ctypes.data_as(VP), lda, B.ctypes.data_as(VP), ldb, E.ctypes.data_as(VP), ldc)
    return np.array_equal(R.view(np.int64), E.view(np.int64))
def check(shapes, trials=4):
    bad = []
    for m, n, k in shapes:
        assert kargs(m, n, k), (m, n, k)  # must really take the packed path
        ok = True
        for t in range(trials):
            lda, ldb = k + (t % 2) * int(rng.integers(1, 5)), k + (t % 3 == 1) * int(rng.integers(1, 5))
            ldc = m + (t % 2) * int(rng.integers(1, 13))
            al = tuple(int(x) for x in rng.integers(0, 8, 3) * 8)
            ok &= one(m, n, k, lda, ldb, ldc, al)
        if not ok: bad.append((m, n, k))
    return bad
G = [[122,2,5],[102,6,4],[106,10,7],[129,9,7],[126,6,11],[127,7,12],[125,5,19],[129,9,11],[127,7,28],[66,61,7],
     [95,109,4],[75,126,11],[120,110,9],[116,110,15],[192,243,192],[240,240,240]]
G = [s for s in json.load(open(D + "/gshapes.json")) if s in G]
bad = check(G)
print(f"packed path gshapes: {len(G) - len(bad)}/{len(G)} bit-equal", bad)
sweep = [[m, n, k] for m in (97, 121, 128, 129, 130, 131, 133, 135, 139, 143, 200, 241) for n in (2, 3, 5, 7, 25, 30, 51, 197)
         for k in (1, 2, 3, 5, 8, 13, 47, 48, 64, 99, 256)]
sweep = [s for s in sweep if kargs(*s)]
bad = check(sweep, 2)
print(f"packed path sweep: {len(sweep) - len(bad)}/{len(sweep)} bit-equal", bad[:20])
