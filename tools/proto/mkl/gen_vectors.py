"""Test vectors from the real MKL (libgurobi130.so, AVX2) for offline checking of the emulator ports.
Writes vectors.jsonl: one JSON object per line, all doubles as float.hex strings.
  gemm: C1 = C0 + A^T B  (DGEMM 'T','N', alpha = beta = 1), A is lda x m, B is ldb x n, C is ldc x n (column-major)
  trsm: B1 = U^{-T} B0   (DTRSM 'L','U','T','U', alpha = 1), A is lda x m (only the strict upper triangle is used)"""
import gzip, json, os
import numpy as np
import mk
D = os.path.dirname(os.path.abspath(__file__))
rng = np.random.default_rng(2024)
def rnd(n, lo=-6, hi=7): return rng.uniform(-1, 1, n) * 2.0 ** rng.integers(lo, hi, n)
def hx(a): return [float(x).hex() for x in a]
def gemm_case(m, n, k, tag, lda=None, ldb=None, ldc=None, al=(0, 0, 0)):
    lda, ldb, ldc = lda or k, ldb or k, ldc or m
    A, B, C = rnd(lda * m), rnd(ldb * n), rnd(ldc * n)
    C1 = mk.gemm_tn(m, n, k, A, lda, B, ldb, C, ldc, al)
    return dict(f="gemm", tag=tag, m=m, n=n, k=k, lda=lda, ldb=ldb, ldc=ldc, al=list(al), A=hx(A), B=hx(B), C0=hx(C), C1=hx(C1))
def trsm_case(m, n, tag, lda=None, ldb=None, al=(0, 0)):
    lda, ldb = lda or m, ldb or m
    U = rnd(lda * m, -7, -1); U[[i + i * lda for i in range(m)]] = rng.uniform(2, 3, m)
    B = rnd(ldb * n)
    B1 = mk.trsm_lutu(m, n, U, lda, B, ldb, al)
    return dict(f="trsm", tag=tag, m=m, n=n, lda=lda, ldb=ldb, al=list(al), A=hx(U), B0=hx(B), B1=hx(B1))
cases = []
for nm in ["degen2", "perold", "pilot4"]:
    sh = json.load(gzip.open(f"{D}/../shapes/{nm}.json.gz", "rt"))["shapes"]
    for key in sorted(sh):
        p = key.split()
        if p[0] == "G": cases.append(gemm_case(int(p[2]), int(p[3]), int(p[4]), nm))
        elif int(p[3]) > 0: cases.append(trsm_case(int(p[2]), int(p[3]), nm))
# every code path with ld > minimum and shifted alignment
al8 = lambda: 8 * int(rng.integers(0, 8))
for m, n, k in [(1, 1, 5), (1, 7, 9), (1, 12, 30), (5, 1, 17), (13, 1, 3), (2, 2, 4), (3, 3, 7), (7, 6, 3), (8, 8, 16), (9, 13, 31),
                (24, 24, 5), (45, 21, 4), (16, 77, 8), (33, 40, 21), (95, 109, 4), (122, 2, 5), (66, 61, 7), (192, 243, 192)]:
    cases.append(gemm_case(m, n, k, "path", lda=k + 3, ldb=k + 1, ldc=m + 4, al=(al8(), al8(), al8())))
for m, n in [(2, 1), (7, 1), (13, 1), (2, 5), (3, 9), (5, 30), (8, 17), (11, 23), (12, 4), (40, 13), (88, 9), (13, 7), (19, 25), (37, 50), (90, 11), (192, 243)]:
    cases.append(trsm_case(m, n, "path", lda=m + 2, ldb=m + 3, al=(al8(), al8())))
with open(f"{D}/vectors.jsonl", "w") as f:
    for c in cases: f.write(json.dumps(c) + "\n")
print(len(cases), "cases")
