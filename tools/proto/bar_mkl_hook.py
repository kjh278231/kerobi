"""Log the MKL DGEMM/DTRSM calls Gurobi's barrier makes (docs/m3/m3_chol.md §3).

The MKL inside libgurobi130.so is static and has only local symbols, so the entries are patched in memory with
`movabs rax, hook; jmp rax`. The hook copies the inputs, puts the original bytes back, calls the original, re-patches
and copies the output. Every call is recorded with its arguments, inputs and output as float.hex strings.

usage: python tools/proto/bar_mkl_hook.py <model.mps.gz> <out.json.gz> [--iters -1,0,1] [--shapes]
  records the calls made before the BARRIER callback of iteration 0 (tag -1: starting factor and its solves) and
  between callbacks (tag k: the factor and solves of iteration k) for the listed tags (default -1). --shapes records
  no data, only the count of each call shape up to the last listed tag.
"""
import ctypes
import gzip
import json
import sys

import gurobipy as gp

BASE = next(int(ln.split("-")[0], 16) for ln in open("/proc/self/maps")
            if "libgurobi130.so" in ln and ln.split()[2] == "00000000")
OFF = {"DGEMM": 0x9ffc30, "DTRSM": 0x9ff5e0}
libc = ctypes.CDLL(None)
libc.mprotect.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int]
VP = ctypes.c_void_p
T_GEMM = ctypes.CFUNCTYPE(None, *[VP] * 13)
T_TRSM = ctypes.CFUNCTYPE(None, *[VP] * 11)

ch = lambda p: ctypes.c_char.from_address(p).value.decode()
iv = lambda p: ctypes.c_int.from_address(p).value
dv = lambda p: ctypes.c_double.from_address(p).value
hx = lambda p, k: [x.hex() for x in (ctypes.c_double * k).from_address(p)] if k > 0 else []


class Patch:
    def __init__(self, name, hook):
        self.addr = BASE + OFF[name]
        self.orig = ctypes.string_at(self.addr, 12)
        page = self.addr & ~0xfff
        if libc.mprotect(page, 0x2000, 7) != 0:
            raise OSError("mprotect failed")
        self.jmp = b"\x48\xb8" + ctypes.cast(hook, VP).value.to_bytes(8, "little") + b"\xff\xe0"
        self.on()

    def on(self):
        ctypes.memmove(self.addr, self.jmp, 12)

    def off(self):
        ctypes.memmove(self.addr, self.orig, 12)


LOG = {"tag": -1, "want": None, "calls": []}


def _keep():
    return LOG["want"] is None or LOG["tag"] in LOG["want"]


def _shape(key):
    sh = LOG.setdefault("shapes", {})
    sh[key] = sh.get(key, 0) + 1


def gemm(ta, tb, m, n, k, al, A, lda, B, ldb, be, C, ldc):
    ta_, tb_, m_, n_, k_, lda_, ldb_, ldc_ = ch(ta), ch(tb), iv(m), iv(n), iv(k), iv(lda), iv(ldb), iv(ldc)
    _shape(f"G {ta_}{tb_} {m_} {n_} {k_} {dv(al)} {dv(be)}")
    rec = None
    if _keep():
        rec = {"f": "DGEMM", "tag": LOG["tag"], "ta": ta_, "tb": tb_, "m": m_, "n": n_, "k": k_, "alpha": dv(al).hex(),
               "beta": dv(be).hex(), "lda": lda_, "ldb": ldb_, "ldc": ldc_,
               "A": hx(A, lda_ * (m_ if ta_ in "Tt" else k_)), "B": hx(B, ldb_ * (k_ if tb_ in "Tt" else n_)),
               "C0": hx(C, ldc_ * n_), "Aal": A % 64, "Bal": B % 64, "Cal": C % 64}
    P["DGEMM"].off()
    ORIG["DGEMM"](ta, tb, m, n, k, al, A, lda, B, ldb, be, C, ldc)
    P["DGEMM"].on()
    if rec is not None:
        rec["C1"] = hx(C, ldc_ * n_)
        LOG["calls"].append(rec)


def trsm(side, uplo, ta, diag, m, n, al, A, lda, B, ldb):
    s_, u_, t_, d_, m_, n_, lda_, ldb_ = ch(side), ch(uplo), ch(ta), ch(diag), iv(m), iv(n), iv(lda), iv(ldb)
    _shape(f"T {s_}{u_}{t_}{d_} {m_} {n_} {dv(al)}")
    rec = None
    if _keep():
        rec = {"f": "DTRSM", "tag": LOG["tag"], "side": s_, "uplo": u_, "ta": t_, "diag": d_, "m": m_, "n": n_,
               "alpha": dv(al).hex(), "lda": lda_, "ldb": ldb_, "A": hx(A, lda_ * (m_ if s_ in "Ll" else n_)),
               "B0": hx(B, ldb_ * n_), "Aal": A % 64, "Bal": B % 64}
    P["DTRSM"].off()
    ORIG["DTRSM"](side, uplo, ta, diag, m, n, al, A, lda, B, ldb)
    P["DTRSM"].on()
    if rec is not None:
        rec["B1"] = hx(B, ldb_ * n_)
        LOG["calls"].append(rec)


ORIG = {"DGEMM": T_GEMM(BASE + OFF["DGEMM"]), "DTRSM": T_TRSM(BASE + OFF["DTRSM"])}
HOOKS = {"DGEMM": T_GEMM(gemm), "DTRSM": T_TRSM(trsm)}
P = {}


def install():
    for nm in OFF:
        if nm not in P:
            P[nm] = Patch(nm, HOOKS[nm])


def main():
    want = {int(x) for x in sys.argv[sys.argv.index("--iters") + 1].split(",")} if "--iters" in sys.argv else {-1}
    last = max(want)
    LOG["want"] = set() if "--shapes" in sys.argv else want
    env = gp.Env(params={"OutputFlag": 0, "Threads": 1, "Method": 2, "Presolve": 0, "Crossover": 0})
    model = gp.read(sys.argv[1], env)

    def cb(mdl, where):
        if where == gp.GRB.Callback.BARRIER:
            LOG["tag"] = int(mdl.cbGet(gp.GRB.Callback.BARRIER_ITRCNT))
            if LOG["tag"] > last:
                mdl.terminate()
    install()
    model.optimize(cb)
    with gzip.open(sys.argv[2], "wt") as f:
        json.dump({"calls": LOG["calls"], "shapes": LOG.get("shapes", {})}, f)
    print(f"{len(LOG['calls'])} calls recorded, {sum(LOG.get('shapes', {}).values())} calls seen")


if __name__ == "__main__":
    main()
