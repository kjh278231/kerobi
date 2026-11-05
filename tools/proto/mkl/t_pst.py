import math, sys, itertools
import numpy as np
from pst import trsm_pst, gemm_pst
fma = math.fma
rng = np.random.default_rng(3)
def rnd(n, lo=-6, hi=7): return rng.uniform(-1, 1, n) * 2.0 ** rng.integers(lo, hi, n)
def U_rand(m, lda):
    U = rnd(lda * m, -6, 0)
    for i in range(m): U[i + i * lda] = rng.uniform(2, 3)
    return U
def seq_fma(m, n, U, lda, B, ldb):
    X = B.copy()
    for j in range(n):
        for i in range(m):
            acc = X[i + j * ldb]
            for p in range(i): acc = fma(-U[p + i * lda], X[p + j * ldb], acc)
            X[i + j * ldb] = acc
    return X
def seq_add(m, n, U, lda, B, ldb):
    X = B.copy()
    for j in range(n):
        for i in range(m):
            acc = X[i + j * ldb]
            for p in range(i): acc = acc - U[p + i * lda] * X[p + j * ldb]
            X[i + j * ldb] = acc
    return X
def colwise_fma(m, n, U, lda, B, ldb):  # right-looking: after x_p known, update all later rows
    X = B.copy()
    for j in range(n):
        for p in range(m):
            for i in range(p + 1, m): X[i + j * ldb] = fma(-U[p + i * lda], X[p + j * ldb], X[i + j * ldb])
    return X
def colwise_add(m, n, U, lda, B, ldb):
    X = B.copy()
    for j in range(n):
        for p in range(m):
            for i in range(p + 1, m): X[i + j * ldb] = X[i + j * ldb] - U[p + i * lda] * X[p + j * ldb]
    return X
models = dict(seq_fma=seq_fma, seq_add=seq_add, colwise_fma=colwise_fma, colwise_add=colwise_add)
for m, n in itertools.product(range(1, 14), [1, 2, 3, 5, 8]):
    lda, ldb = m + 1, m + 2
    ok = {k: True for k in models}
    for t in range(4):
        U = U_rand(m, lda); B = rnd(ldb * n)
        R = trsm_pst(b'L', b'U', b'T', b'U', m, n, 1.0, U, lda, B, ldb)
        for k, f in models.items(): ok[k] &= bool((f(m, n, U, lda, B, ldb) == R).all())
    print(m, n, [k for k, v in ok.items() if v])
