"""Barrier-state oracle (docs/m3_barrier.md §3): read Gurobi's barrier workspace B (cbdata+0x68, layout in
docs/m3_barrier.md §3) from inside BARRIER callbacks and dump it per iteration, so a port can be compared array by
array at the first divergence.

usage: python tools/bar_oracle.py <model.mps.gz> <out.jsonl> [--iters 0,1,2|all] [--vec] [--chol] [--param Name=Value ...]
Default params: Threads=1, Method=2, Presolve=0, Crossover=0. One JSON object per recorded iteration. Doubles are
float.hex() so they compare bit for bit. The first recorded iteration also carries the raw B block (0xa00 bytes as
doubles), the factor structure F, found on the stack frame of the main loop 2bf1ea, and the model data D as the
barrier sees it ("D": scaled, rows permuted, docs/m3_ipm.md §1). --vec adds per iteration the raw B block, the
history arrays and the work vectors in VEC (residuals, theta, directions of the previous iteration). --chol adds per
iteration the numeric factor in F (chol_dump: the factor of the theta recorded one iteration earlier; at iteration 0 the
factor of the starting theta) and, in the first record, the supernode structure it lives in.
Field meanings: docs/m3_ipm.md, docs/m3_chol.md.
"""
import argparse, ctypes, gzip, json

import gurobipy as gp

api = ctypes.pythonapi
api.PyCapsule_GetName.restype = ctypes.c_char_p
api.PyCapsule_GetName.argtypes = [ctypes.py_object]
api.PyCapsule_GetPointer.restype = ctypes.c_void_p
api.PyCapsule_GetPointer.argtypes = [ctypes.py_object, ctypes.c_char_p]
I = lambda a: ctypes.c_int.from_address(a).value
L = lambda a: ctypes.c_long.from_address(a).value
D = lambda a: ctypes.c_double.from_address(a).value

# B offsets: name -> (offset, kind). Scalars read every iteration.
SCALARS = {"iter": (0x40, "i"), "homog": (0x2c, "i"), "pobj": (0x130, "d"), "dobj": (0x140, "d"), "sense": (0x158, "d"),
           "ctx": (0x138, "d"), "bty": (0x148, "d"), "rp_rel": (0x1d8, "d"), "rd_rel": (0x1e0, "d"), "compl": (0x1f0, "d"),
           "mu": (0x1f8, "d"), "tau": (0x690, "d"), "kappa": (0x698, "d"), "alpha_p": (0x5b8, "d"), "alpha_d": (0x5c8, "d"),
           "reg": (0x280, "d"), "ncorr": (0x298, "i"), "work_pred": (0x5b0, "d"),
           "rp2": (0x198, "d"), "rd2": (0x1a8, "d"), "ru2": (0x1b8, "d"), "ap_cand": (0x5c0, "d"), "ad_cand": (0x5d0, "d"),
           "shift": (0x38, "d"), "freepert": (0x248, "d"), "corr_lim": (0x29c, "i"), "restart_it": (0xf8, "i"),
           "best_it": (0x88, "i"), "dobj_best": (0x1e8, "d"), "pres_inf": (0x770, "d"), "dres_inf": (0x780, "d")}
# B vectors (docs/m3_ipm.md §1): name -> (pointer offset, length kind). mn = m+n ordered [columns, slacks].
VEC = {"w": (0x670, "n"), "s": (0x678, "n"), "rp": (0x7a0, "m"), "rd": (0x7a8, "mn"), "ru": (0x7b0, "n"),
       "rxz": (0x7c8, "mn"), "rws": (0x7d0, "n"), "rhsx": (0x7e0, "mn"), "theta_inv": (0x800, "mn"), "theta": (0x808, "mn"),
       "dx0": (0x6a0, "mn"), "dy0": (0x6b0, "m"), "dz0": (0x6c0, "mn"), "dw0": (0x6d0, "mn"), "ds0": (0x6e0, "mn"),
       "dx1": (0x6a8, "mn"), "dy1": (0x6b8, "m"), "dz1": (0x6c8, "mn"), "dw1": (0x6d8, "mn"), "ds1": (0x6e8, "mn")}
HIST = {"h48": 0x48, "h50": 0x50, "h58": 0x58, "h60": 0x60, "h68": 0x68, "h70": 0x70, "h78": 0x78, "h80": 0x80}


def arr(p, k, kind="d"):
    if not p or k <= 0:
        return None
    t = {"d": ctypes.c_double, "i": ctypes.c_int, "l": ctypes.c_long}[kind]
    return [x.hex() if kind == "d" else x for x in (t * k).from_address(p)]


def readable():
    out = []
    for ln in open("/proc/self/maps"):
        f = ln.split()
        if f[1][0] == "r":
            a, b = f[0].split("-")
            out.append((int(a, 16), int(b, 16), f[-1] if len(f) > 5 else ""))
    return out


def find_F(B, m):
    """F (2cd7cb, factor structure) is a local of 2bf1ea, not stored in B. Its frame holds local_98 = B+0x5c8,
    local_90 = B+0x5b8, local_88 = F next to each other (2bf1ea L104-106); scan the main-thread stack for that triple."""
    lo, hi, _ = next(r for r in readable() if r[2] == "[stack]")
    words = (ctypes.c_long * ((hi - lo) // 8)).from_address(lo)
    for k in range(len(words) - 2):
        if words[k] == B + 0x5c8 and words[k + 1] == B + 0x5b8:
            F = words[k + 2]
            if F and I(F + 4) == m:
                return F
    return 0


def snap_D(Dm):
    """Model data D (user model +0xe8) as barrier sees it: scaled in place, rows in ordering permutation (D+0x3e0 scale
    array, columns then rows, stays in original row order)."""
    m, n, nnz = I(Dm + 8), I(Dm + 0xc), L(Dm + 0x10)
    return {"m": m, "n": n, "nnz": nnz, "beg": arr(L(Dm + 0xf8), n, "l"), "len": arr(L(Dm + 0x100), n, "i"),
            "ind": arr(L(Dm + 0x108), nnz, "i"), "val": arr(L(Dm + 0x110), nnz), "obj": arr(L(Dm + 0x50), n),
            "rhs": arr(L(Dm + 0x380), m), "lb": arr(L(Dm + 0x390), n), "ub": arr(L(Dm + 0x398), n),
            "sense": bytes((ctypes.c_char * m).from_address(L(Dm + 0x388))).decode(),
            "scale": arr(L(Dm + 0x3e0), m + n), "objscale": D(Dm + 0x3e8).hex(), "objcon": D(Dm + 0x60).hex()}


_F = {}


def chol_dump(F, m, static):
    """Factor storage (docs/m3_chol.md §1): per block start c (F+0x78[c] = w > 0) the doubles at F+0xb8[c], packed lower
    triangle of the w x w diagonal block by rows then the (len-w) x w rectangle by rows (len = F+0xa8[c]);
    F+0xe0 pivots d, F+0xf0 assembled diagonal, F+0x100/0x114/0x118/0x178 scalars."""
    blocks = arr(L(F + 0x78), m, "i")
    clen = arr(L(F + 0xa8), m, "i")
    vp = (ctypes.c_long * m).from_address(L(F + 0xb8))
    out = {"vals": {c: arr(vp[c], blocks[c] * (blocks[c] + 1) // 2 + (clen[c] - blocks[c]) * blocks[c])
                    for c in range(m) if blocks[c] > 0},
           "d": arr(L(F + 0xe0), m), "e": arr(L(F + 0xe8), m), "diag0": arr(L(F + 0xf0), m), "f8": arr(L(F + 0xf8), m),
           "F100": D(F + 0x100).hex(), "F114": I(F + 0x114), "F118": I(F + 0x118), "F178": I(F + 0x178)}
    if static:
        rp = (ctypes.c_long * m).from_address(L(F + 0xb0))
        f88 = arr(L(F + 0x88), m + 1, "i")
        out.update({"snode": arr(L(F + 0x70), m, "i"), "blocks": blocks, "col_len": clen, "f88": f88,
                    "f90": arr(L(F + 0x90), f88[m], "i"), "f98": arr(L(F + 0x98), f88[m], "i"),
                    "rows": {c: arr(rp[c], clen[c], "i") for c in range(m) if blocks[c] > 0}})
    return out


def dump(B, it, full, vec=False, Dm=0, chol=False):
    m, n = I(B + 0x18), I(B + 0x20)
    out = {k: (I(B + o) if t == "i" else D(B + o).hex()) for k, (o, t) in SCALARS.items()}
    out.update({"m": m, "n": n, "ncomp": L(B + 0x210),
                "x": arr(L(B + 0x658), m + n), "y": arr(L(B + 0x660), m), "z": arr(L(B + 0x668), m + n),
                "a670": arr(L(B + 0x670), n), "a678": arr(L(B + 0x678), n)})
    if vec:
        out["B"] = arr(B, 0xa00 // 8)
        k = {"m": m, "n": n, "mn": m + n}
        out.update({nm: arr(L(B + o), k[kd]) for nm, (o, kd) in VEC.items()})
        out.update({nm: arr(L(B + o), min(it + 1, I(B + 0x44))) for nm, o in HIST.items()})
    if chol:
        if "F" not in _F:
            _F["F"] = find_F(B, m)
        out["theta"] = arr(L(B + 0x808), m + n)
        out["chol"] = chol_dump(_F["F"], m, full)
    if full:
        out["B"] = arr(B, 0xa00 // 8)
        out["lb"], out["ub"], out["obj"] = arr(L(B + 0x5f8), n), arr(L(B + 0x600), n), arr(L(B + 0x608), n)
        out["rhs"] = arr(L(B + 0x610), m)
        out["type"] = list((ctypes.c_ubyte * (m + n)).from_address(L(B + 0x640))) if L(B + 0x640) else None
        if Dm:
            out["D"] = snap_D(Dm)
        F = find_F(B, m)
        out["F_addr"] = F
        if F:
            out["F"] = arr(F, 0x200 // 8)
            out["F_nnzL"], out["F_ops"], out["F_ops2"] = L(F + 0x38), D(F + 0x60), D(F + 0x48)
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("model")
    ap.add_argument("out")
    ap.add_argument("--iters", default="0")
    ap.add_argument("--vec", action="store_true")
    ap.add_argument("--chol", action="store_true")
    ap.add_argument("--param", action="append", default=[])
    a = ap.parse_args()
    params = {"OutputFlag": 0, "Threads": 1, "Method": 2, "Presolve": 0, "Crossover": 0}
    for p in a.param:
        k, v = p.split("=", 1)
        params[k] = int(v) if v.lstrip("-").isdigit() else float(v)
    want = None if a.iters == "all" else {int(x) for x in a.iters.split(",")}
    env = gp.Env(params=params)
    model = gp.read(a.model, env)
    mp = api.PyCapsule_GetPointer(model._cmodel, api.PyCapsule_GetName(model._cmodel))
    snaps = {}

    def cb(mdl, where):
        if where != gp.GRB.Callback.BARRIER:
            return
        it = int(mdl.cbGet(gp.GRB.Callback.BARRIER_ITRCNT))
        if want is None or it in want:
            c = mdl._callback._cbdata
            B = L(api.PyCapsule_GetPointer(c, api.PyCapsule_GetName(c)) + 0x68)
            snaps[it] = dump(B, it, full=not snaps, vec=a.vec, Dm=L(mp + 0xe8), chol=a.chol)

    model.optimize(cb)
    with (gzip.open(a.out, "wt") if a.out.endswith(".gz") else open(a.out, "w")) as f:
        for it in sorted(snaps):
            f.write(json.dumps(snaps[it]) + "\n")
    print(f"recorded iterations {sorted(snaps)}; BarIterCount {model.BarIterCount}, status {model.Status}, "
          f"ObjVal {model.ObjVal if model.Status == 2 else None!r}")


if __name__ == "__main__":
    main()
