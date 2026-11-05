"""check part_trsmbig.so (emu_trsm_big / emu_trsv_utu) against the real MKL DTRSM('L','U','T','U') bitwise.
usage: python test_trsmbig.py [trials]   (shapes: tshapes.json entries on the strategy path or with n == 1, plus extras)"""
import ctypes, json, sys
import numpy as np
import mk
D = __file__.rsplit('/', 1)[0]
emu = ctypes.CDLL(D + '/part_trsmbig.so')
VP = ctypes.c_void_p
def big_path(m, n): return n > 1 and m > 12 and not (m % 4 == 0 and m <= 88)
def make(rng, m, n, lda, ldb, sparse=False):
    U = rng.uniform(-1, 1, lda * m) * 2.0 ** rng.integers(-6, 1, lda * m) / max(1, m) ** 0.5
    for i in range(m): U[i + i * lda] = rng.uniform(0.5, 3)  # diagonal must be ignored
    B = rng.uniform(-1, 1, ldb * n) * 2.0 ** rng.integers(-6, 7, ldb * n)
    if sparse:  # exact zeros (both signs) as in sparse Cholesky blocks
        U[rng.random(U.size) < 0.6] = 0.0
        z = rng.random(B.size); B[z < 0.3] = 0.0; B[z > 0.85] = -0.0
    return U, B
def emu_run(m, n, U, lda, B, ldb):
    u, b = np.ascontiguousarray(U), np.array(B)
    if n == 1: emu.emu_trsv_utu(m, u.ctypes.data_as(VP), lda, b.ctypes.data_as(VP))
    else: emu.emu_trsm_big(m, n, u.ctypes.data_as(VP), lda, b.ctypes.data_as(VP), ldb)
    return b
def check(m, n, rng, trials):
    for t in range(trials):
        lda = m + (t % 3) * 5; ldb = m + ((t + 1) % 3) * 3
        al = (8 * (t % 8), 8 * ((t * 3 + 1) % 8))
        U, B = make(rng, m, n, lda, ldb, sparse=t % 2 == 1)
        R = mk.trsm_lutu(m, n, U, lda, B, ldb, al)
        E = emu_run(m, n, U, lda, B, ldb)
        if not np.array_equal(R.view(np.int64), E.view(np.int64)):
            bad = np.nonzero(R.view(np.int64) != E.view(np.int64))[0]
            return f"trial {t} lda={lda} ldb={ldb} al={al}: {len(bad)} bad, first (row,col) {[(int(k % ldb), int(k // ldb)) for k in bad[:6]]}"
    return None
def replay(fn):
    import gzip
    fh = float.fromhex; okc = tot = 0
    for r in json.load(gzip.open(fn, "rt"))["calls"]:
        if r["f"] != "DTRSM" or not (big_path(r["m"], r["n"]) or r["n"] == 1): continue
        m, n, lda, ldb = r["m"], r["n"], r["lda"], r["ldb"]
        E = emu_run(m, n, np.array([fh(v) for v in r["A"]]), lda, np.array([fh(v) for v in r["B0"]]), ldb)
        tot += 1; okc += np.array_equal(E.view(np.int64), np.array([fh(v) for v in r["B1"]]).view(np.int64))
    print(f"recorded {fn.rsplit('/', 1)[-1]}: {okc}/{tot} calls on these paths bit-equal")
def rust_check(lst, rng):
    """run the Rust port (part_trsmbig_check, built from part_trsmbig.rs) on 2 cases per shape vs the real MKL"""
    import os, subprocess
    exe = D + "/part_trsmbig_check"
    if not os.path.exists(exe):
        print("rust: part_trsmbig_check not built (rustc -O --edition 2021 -o part_trsmbig_check part_trsmbig_check.rs)"); return
    cases, ref = [], []
    for m, n in lst:
        for t in range(2):
            lda, ldb = m + 2 * t, m + t
            U, B = make(rng, m, n, lda, ldb, sparse=t == 1)
            cases.append(np.concatenate([np.array([m, n, lda, ldb], np.uint64).view(np.float64), U, B]))
            ref.append(mk.trsm_lutu(m, n, U, lda, B, ldb))
    np.concatenate(cases).tofile(D + "/rtmp_trsmbig_in.bin")
    subprocess.run([exe, D + "/rtmp_trsmbig_in.bin", D + "/rtmp_trsmbig_out.bin"], check=True)
    out = np.fromfile(D + "/rtmp_trsmbig_out.bin", np.float64)
    ref = np.concatenate(ref)
    print(f"rust: {'all' if np.array_equal(out.view(np.int64), ref.view(np.int64)) else 'NOT all'} {len(cases)} cases bit-equal to MKL")

if __name__ == "__main__":
    trials = int(sys.argv[1]) if len(sys.argv) > 1 else 6
    rng = np.random.default_rng(12345)
    shapes = sorted({tuple(s) for s in json.load(open(D + '/tshapes.json')) if big_path(*s) or (s[1] == 1 and s[0] <= 32)})
    extra = [(m, n) for m in (13, 14, 15, 16 + 76, 23, 35, 47, 61, 100, 133, 191, 193, 250) for n in (2, 3, 4, 5, 6, 7, 9, 190, 193, 200, 389)]
    extra += [(m, 1) for m in range(2, 33)]
    ok = bad = 0
    for tag, lst in (("tshapes", shapes), ("extra", sorted(set(extra) - set(shapes)))):
        okc = 0
        for m, n in lst:
            r = check(m, n, rng, trials)
            if r: print(f"  MISMATCH {tag} m={m} n={n}: {r}")
            else: okc += 1
        print(f"{tag}: {okc}/{len(lst)} shapes bit-equal ({trials} trials each, varied lda/ldb/alignment, odd trials sparse with +-0)")
    for fn in ("mkl_afiro.json.gz", "mkl_agg.json.gz"): replay(D + "/../" + fn)
    rust_check(shapes + sorted(set(extra) - set(shapes)), rng)
