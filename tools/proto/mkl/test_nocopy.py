"""check part_nocopy.so (emu_gemm_nocopy_tn) bitwise against the real MKL DGEMM('T','N') on every gshapes.json shape
that takes the nocopy path; also checks mkl_gemm_is_nocopy against the traced paths (paths_G.json).
usage: python test_nocopy.py [trials] [shapes.json]"""
import ctypes, json, os, sys
import numpy as np
import mk
D = os.path.dirname(os.path.abspath(__file__))
so = ctypes.CDLL(D + '/part_nocopy.so')
VP = ctypes.c_void_p
rng = np.random.default_rng(12345)
def rnd(sz):  # mixed exponents, so products are inexact and fma vs mul+add differ
    return rng.uniform(-1, 1, sz) * 2.0 ** rng.integers(-6, 7, sz)
def emu(m, n, k, A, lda, B, ldb, C, ldc):
    c = C.copy()
    so.emu_gemm_nocopy_tn(m, n, k, A.ctypes.data_as(VP), lda, B.ctypes.data_as(VP), ldb, c.ctypes.data_as(VP), ldc)
    return c
def check(m, n, k, trials):
    for t in range(trials):
        pad = (0, 0, 0) if t == 0 else tuple(int(x) for x in rng.integers(0, 4, 3))
        lda, ldb, ldc = k + pad[0], k + pad[1], m + pad[2]
        al = (0, 0, 0) if t == 0 else tuple(int(x) * 8 for x in rng.integers(0, 8, 3))
        A, B, C = rnd(lda * m), rnd(ldb * n), rnd(ldc * n)
        R = mk.gemm_tn(m, n, k, A, lda, B, ldb, C, ldc, al)
        E = emu(m, n, k, A, lda, B, ldb, C, ldc)
        if not np.array_equal(R.view(np.int64), E.view(np.int64)):
            return False, (t, lda, ldb, ldc, al, int((R != E).sum()))
    return True, None
if __name__ == '__main__':
    args = [a for a in sys.argv[1:] if a != "--rust"]
    trials = int(args[0]) if args else 6
    shapes = json.load(open(args[1] if len(args) > 1 else D + "/gshapes.json"))
    paths = json.load(open(D + '/paths_G.json'))
    traced = {tuple(s): ('nocopy' in p) for p, v in paths.items() for s in v}
    is_nc = lambda m, n, k: m > 1 and n > 1 and so.mkl_gemm_is_nocopy(m, n, k, k, k, m)
    disagree = [s for s in map(tuple, shapes) if s in traced and traced[s] != bool(is_nc(*s))]
    print('dispatch predicate disagreements with trace:', disagree)
    nc = [s for s in shapes if is_nc(*s)]
    bad = []
    for m, n, k in nc:
        ok, info = check(m, n, k, trials)
        if not ok: bad.append(((m, n, k), info))
    print(f'nocopy shapes fully bit-equal: {len(nc) - len(bad)} / {len(nc)}  ({trials} trials each)')
    for b in bad[:40]: print('  FAIL', b)

def check_rust(shapes, trials=2):
    """run the Rust port (part_nocopy_check binary) on the same kind of inputs and compare bitwise with MKL"""
    import subprocess
    cases, refs = [], []
    for m, n, k in shapes:
        for t in range(trials):
            pad = tuple(int(x) for x in rng.integers(0, 3, 3))
            lda, ldb, ldc = k + pad[0], k + pad[1], m + pad[2]
            A, B, C = rnd(lda * m), rnd(ldb * n), rnd(ldc * n)
            refs.append(mk.gemm_tn(m, n, k, A, lda, B, ldb, C, ldc))
            cases.append(' '.join(map(str, (m, n, k, lda, ldb, ldc))) + ' ' + ' '.join('%x' % x for x in np.concatenate([A, B, C]).view(np.uint64)))
    out = subprocess.run([D + '/part_nocopy_check'], input='\n'.join(cases) + '\n', capture_output=True, text=True, check=True).stdout.split('\n')
    bad = sum(not np.array_equal(np.array([int(h, 16) for h in o.split()], dtype=np.uint64), r.view(np.uint64)) for o, r in zip(out, refs))
    print(f'rust port: {len(refs) - bad} / {len(refs)} cases bit-equal to MKL')

if __name__ == '__main__' and '--rust' in sys.argv:
    check_rust(nc)
