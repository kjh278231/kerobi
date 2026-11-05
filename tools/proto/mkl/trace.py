"""which MKL functions run for a given call: python trace.py G m n k [lda ldb ldc] | T m n [lda ldb]"""
import ctypes, sys, re
import numpy as np
import mk
syms = {}
for ln in open(__file__.rsplit('/', 1)[0] + '/syms.txt'):
    p = ln.split()
    if len(p) == 4 and p[2] in 'tT' and int(p[1], 16) > 0:
        a = int(p[0], 16)
        if 0x9f0000 <= a < 0x2500000 and a not in (0x9ff5e0, 0x9ffc30):
            syms.setdefault(a, p[3])
tr = ctypes.CDLL(__file__.rsplit('/', 1)[0] + '/tracer.so')
addrs = sorted(syms)
arr = (ctypes.c_uint64 * len(addrs))(*[mk.BASE + a for a in addrs])
def run(f):
    tr.trace_set(arr, len(addrs))
    f()
    tr.trace_off()
    buf = (ctypes.c_uint64 * 4000000)()
    n = tr.trace_get(buf, 4000000)
    seq = [syms[buf[i] - mk.BASE] for i in range(n)]
    out = []
    for s in seq:
        if out and out[-1][0] == s: out[-1][1] += 1
        else: out.append([s, 1])
    return out
if __name__ == "__main__":
    rng = np.random.default_rng(1)
    a = sys.argv[1:]
    if a[0] == 'G':
        m, n, k = map(int, a[1:4]); lda, ldb, ldc = (list(map(int, a[4:7])) + [k, k, m])[len(a[4:7]):][:0] or (map(int, a[4:7]) if len(a) > 4 else (k, k, m))
        lda, ldb, ldc = (int(x) for x in a[4:7]) if len(a) > 4 else (k, k, m)
        f = lambda: mk.gemm_tn(m, n, k, rng.random(lda * m), lda, rng.random(ldb * n), ldb, rng.random(ldc * n), ldc)
    else:
        m, n = map(int, a[1:3]); lda, ldb = (int(x) for x in a[3:5]) if len(a) > 3 else (m, m)
        f = lambda: mk.trsm_lutu(m, n, rng.random(lda * m), lda, rng.random(ldb * n), ldb)
    for s, c in run(f): print(f"  {s} x{c}" if c > 1 else f"  {s}")
