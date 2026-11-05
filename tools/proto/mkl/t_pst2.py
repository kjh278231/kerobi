import math, sys, itertools
import numpy as np
from pst import trsm_pst
from t_pst import rnd, U_rand
fma = math.fma
def dot_pst(t, u, x):
    """t - sum u[p]*x[p] in the dtrsm_pst LUT order"""
    i = len(u); nb16 = i & ~15
    if nb16:
        a = [[t, 0.0, 0.0, 0.0], [0.0] * 4, [0.0] * 4, [0.0] * 4]
        for b in range(0, nb16, 16):
            for q in range(4):
                for l in range(4):
                    p = b + 4 * q + l; a[q][l] = fma(-u[p], x[p], a[q][l])
        s = [(a[0][l] + a[1][l]) + (a[2][l] + a[3][l]) for l in range(4)]
        t = (s[0] + s[2]) + (s[1] + s[3])
    r = i - nb16; nb4 = r & ~3
    if nb4:
        v = [t, 0.0, 0.0, 0.0]
        for g in range(0, nb4, 4):
            for l in range(4):
                p = nb16 + g + l; v[l] = v[l] - u[p] * x[p]
        t = (v[0] + v[2]) + (v[1] + v[3])
    for p in range(nb16 + nb4, i): t = t - u[p] * x[p]
    return t
def trsm_pst_model(m, n, U, lda, B, ldb):
    X = B.copy()
    for j in range(n):
        for i in range(m):
            X[i + j * ldb] = dot_pst(X[i + j * ldb], [U[p + i * lda] for p in range(i)], [X[p + j * ldb] for p in range(i)])
    return X
if __name__ == "__main__":
    bad = 0
    for m, n in itertools.product(list(range(1, 41)) + [57, 64, 70], [1, 2, 3]):
        lda, ldb = m + 1, m + 2
        for t in range(3):
            U = U_rand(m, lda); B = rnd(ldb * n)
            R = trsm_pst(b'L', b'U', b'T', b'U', m, n, 1.0, U, lda, B, ldb)
            if not (trsm_pst_model(m, n, U, lda, B, ldb) == R).all(): bad += 1; print('bad', m, n); break
    print('bad', bad)
