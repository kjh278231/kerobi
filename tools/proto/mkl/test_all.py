"""Check mkl_emu.c (libmkl_emu.so) against the real MKL DGEMM/DTRSM in libgurobi130.so, bit for bit.
usage: python test_all.py [trials=4]
  every [m,n,k] of gshapes.json and [m,n] of tshapes.json, `trials` random inputs each (mixed exponents), cycling through
  minimal ld, ldc > m, ld > minimum, and buffer alignments 0..56 mod 64; plus a replay of the recorded Gurobi calls."""
import ctypes, gzip, json, os, sys
import numpy as np
import mk
D = os.path.dirname(os.path.abspath(__file__))
emu = ctypes.CDLL(os.environ.get("EMU_LIB", D + "/libmkl_emu.so"))
dp = lambda a: a.ctypes.data_as(ctypes.c_void_p)
rng = np.random.default_rng(int(os.environ.get("SEED", 1)))
def rnd(n, lo=-6, hi=7): return rng.uniform(-1, 1, n) * 2.0 ** rng.integers(lo, hi, n)
same = lambda x, y: bool((x.view(np.int64) == y.view(np.int64)).all())
al8 = lambda: 8 * int(rng.integers(0, 8))
def lds(t, m, k):  # (lda, ldb, ldc) per trial
    return [(k, k, m), (k, k, m + 3), (k + 1, k + 2, m + 5), (k + 3, k, m)][t % 4]
def gemm_ok(m, n, k, lda, ldb, ldc, al):
    A, B, C = rnd(lda * m), rnd(ldb * n), rnd(ldc * n)
    R = mk.gemm_tn(m, n, k, A, lda, B, ldb, C, ldc, al)
    E = C.copy(); emu.emu_gemm_tn(m, n, k, dp(A), lda, dp(B), ldb, dp(E), ldc)
    return same(R, E)
def trsm_ok(m, n, lda, ldb, al):
    U = rnd(lda * m, -7, -1); U[[i + i * lda for i in range(m)]] = rng.uniform(2, 3, m)
    B = rnd(ldb * n)
    R = mk.trsm_lutu(m, n, U, lda, B, ldb, al)
    E = B.copy(); emu.emu_trsm_lutu(m, n, dp(U), lda, dp(E), ldb)
    return same(R, E)
def replay(fn):
    fh = float.fromhex; ok = bad = 0
    for r in json.load(gzip.open(fn, "rt"))["calls"]:
        if r["f"] == "DGEMM":
            A, B, E = (np.array([fh(x) for x in r[f]]) for f in ("A", "B", "C0"))
            emu.emu_gemm_tn(r["m"], r["n"], r["k"], dp(A), r["lda"], dp(B), r["ldb"], dp(E), r["ldc"]); W = r["C1"]
        else:
            A, E = (np.array([fh(x) for x in r[f]]) for f in ("A", "B0"))
            emu.emu_trsm_lutu(r["m"], r["n"], dp(A), r["lda"], dp(E), r["ldb"]); W = r["B1"]
        good = len(W) == 0 or same(E, np.array([fh(x) for x in W]))
        ok += good; bad += not good
    return ok, bad
if __name__ == "__main__":
    trials = int(sys.argv[1]) if len(sys.argv) > 1 else 4
    G = json.load(open(D + "/gshapes.json")); T = json.load(open(D + "/tshapes.json"))
    gbad = [s for s in G if not all(gemm_ok(*s, *lds(t, s[0], s[2]), (al8(), al8(), al8())) for t in range(trials))]
    print(f"DGEMM shapes fully bit-equal: {len(G) - len(gbad)}/{len(G)}  bad: {gbad[:20]}")
    tbad = [s for s in T if not all(trsm_ok(*s, s[0] + (t % 3) * 2, s[0] + (t // 2) % 2 * 3, (al8(), al8())) for t in range(trials))]
    print(f"DTRSM shapes fully bit-equal: {len(T) - len(tbad)}/{len(T)} ({sum(1 for s in T if s[1] == 0)} have n=0)  bad: {tbad[:20]}")
    for fn in ("mkl_afiro.json.gz", "mkl_agg.json.gz"):  # recorded by tools/proto/bar_mkl_hook.py; skipped if absent
        if os.path.exists(D + "/../" + fn):
            print(f"replay {fn} (recorded Gurobi calls) ok/bad:", replay(D + "/../" + fn))
