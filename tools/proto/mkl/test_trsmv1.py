"""check emu_trsm_v1 against the real MKL DTRSM on every tshapes.json shape on the v1 path"""
import ctypes, json, sys, os
import numpy as np
import mk
D = os.path.dirname(os.path.abspath(__file__))
lib = ctypes.CDLL(D + '/libtrsmv1.so')
rng = np.random.default_rng(11)
def rnd(n, lo=-6, hi=7): return rng.uniform(-1, 1, n) * 2.0 ** rng.integers(lo, hi, n)
def U_rand(m, lda):
    U = rnd(lda * m, -7, -1)
    U[[i + i * lda for i in range(m)]] = rng.uniform(2, 3, m)
    return U
def is_v1(m): return m <= 12 or (m % 4 == 0 and m <= 88)
def check(m, n, lda, ldb, al):
    U = U_rand(m, lda); B = rnd(ldb * n)
    R = mk.trsm_lutu(m, n, U, lda, B, ldb, al)
    E = B.copy()
    dp = lambda a: a.ctypes.data_as(ctypes.c_void_p)
    lib.emu_trsm_v1(m, n, dp(U), lda, dp(E), ldb)
    return (R.view(np.int64) == E.view(np.int64)).all()
if __name__ == "__main__":
    shapes = [tuple(s) for s in json.load(open(D + '/tshapes.json'))] + [(m, n) for m in list(range(1, 13)) + list(range(16, 89, 4)) for n in (1, 2, 3, 4, 5, 7, 8, 9, 12, 13, 17, 31, 300)]
    shapes = sorted(set(s for s in shapes if is_v1(s[0]) and s[1] >= 2))
    good = 0; bad = []
    for m, n in shapes:
        ok = True
        for t in range(6):
            lda = m + (t % 3) * 3; ldb = m + (t // 3) * 5
            ok &= check(m, n, lda, ldb, (8 * rng.integers(0, 8), 8 * rng.integers(0, 8)))
        good += ok
        if not ok: bad.append((m, n))
    print(f"v1 shapes fully bit-equal: {good}/{len(shapes)}"); print('bad', bad[:40])
