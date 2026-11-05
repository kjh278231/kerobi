"""Rust ports (part_gemv.rs, part_packed.rs via test_packed_rust.rs) vs the C emulators and the real MKL on random inputs"""
import ctypes, json, subprocess, os
import numpy as np
import mk
D = os.path.dirname(os.path.abspath(__file__))
VP = ctypes.c_void_p
cg, cp = ctypes.CDLL(D + "/part_gemv.so"), ctypes.CDLL(D + "/part_packed.so")
rng = np.random.default_rng(17)
def rnd(n): return rng.uniform(-1, 1, n) * 2.0 ** rng.integers(-6, 7, n)
gs = json.load(open(D + "/gshapes.json"))
P = [[122,2,5],[102,6,4],[106,10,7],[129,9,7],[126,6,11],[127,7,12],[125,5,19],[129,9,11],[127,7,28],[66,61,7],
     [95,109,4],[75,126,11],[120,110,9],[116,110,15],[192,243,192],[240,240,240]]
cases = [("G", s) for s in gs if s[0] == 1 or s[1] == 1] + [("P", s) for s in P] + [("P", [131, 197, 99]), ("P", [143, 30, 3])]
inp, ref = [], []
for kind, (m, n, k) in cases:
    lda, ldb, ldc = k + int(rng.integers(0, 3)), k + int(rng.integers(0, 3)), m + int(rng.integers(0, 4))
    A, B, C = rnd(lda * m), rnd(ldb * n), rnd(ldc * n)
    E = C.copy()
    (cg.emu_gemm_tn_gemv if kind == "G" else cp.emu_gemm_packed_tn)(m, n, k, A.ctypes.data_as(VP), lda, B.ctypes.data_as(VP), ldb, E.ctypes.data_as(VP), ldc)
    R = mk.gemm_tn(m, n, k, A, lda, B, ldb, C, ldc)
    ref.append((E, R))
    inp.append(f"{kind} {m} {n} {k} {lda} {ldb} {ldc}")
    inp += [" ".join(str(x) for x in v.view(np.uint64)) for v in (A, B, C)]
out = subprocess.run([D + "/test_packed_rust"], input="\n".join(inp) + "\n", capture_output=True, text=True, check=True).stdout.split("\n")
ok_c = ok_m = 0
for (E, R), line in zip(ref, out):
    Rs = np.array([int(x) for x in line.split()], dtype=np.uint64)
    ok_c += np.array_equal(Rs, E.view(np.uint64)); ok_m += np.array_equal(Rs, R.view(np.uint64))
print(f"rust == C: {ok_c}/{len(ref)}, rust == MKL: {ok_m}/{len(ref)}")
