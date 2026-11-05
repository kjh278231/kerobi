"""Barrier-state oracle (docs/m3_barrier.md §3): read Gurobi's barrier workspace B (cbdata+0x68, layout in
docs/m3_barrier.md §3) from inside BARRIER callbacks and dump it per iteration, so a port can be compared array by
array at the first divergence.

usage: python tools/bar_oracle.py <model.mps.gz> <out.jsonl> [--iters 0,1,2|all] [--param Name=Value ...]
Default params: Threads=1, Method=2, Presolve=0, Crossover=0. One JSON object per recorded iteration. Doubles are
float.hex() so they compare bit for bit. The first recorded iteration also carries the raw B block (0xa00 bytes as
doubles) and the factor structure F, found on the stack frame of the main loop 2bf1ea.
"""
import argparse, ctypes, json

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
           "reg": (0x280, "d"), "ncorr": (0x298, "i"), "work_pred": (0x5b0, "d")}


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


def dump(B, it, full):
    m, n = I(B + 0x18), I(B + 0x20)
    out = {k: (I(B + o) if t == "i" else D(B + o).hex()) for k, (o, t) in SCALARS.items()}
    out.update({"m": m, "n": n,
                "x": arr(L(B + 0x658), m + n), "y": arr(L(B + 0x660), m), "z": arr(L(B + 0x668), m + n),
                "a670": arr(L(B + 0x670), n), "a678": arr(L(B + 0x678), n)})
    if full:
        out["B"] = arr(B, 0xa00 // 8)
        out["lb"], out["ub"], out["obj"] = arr(L(B + 0x5f8), n), arr(L(B + 0x600), n), arr(L(B + 0x608), n)
        out["rhs"] = arr(L(B + 0x610), m)
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
    ap.add_argument("--param", action="append", default=[])
    a = ap.parse_args()
    params = {"OutputFlag": 0, "Threads": 1, "Method": 2, "Presolve": 0, "Crossover": 0}
    for p in a.param:
        k, v = p.split("=", 1)
        params[k] = int(v) if v.lstrip("-").isdigit() else float(v)
    want = None if a.iters == "all" else {int(x) for x in a.iters.split(",")}
    env = gp.Env(params=params)
    model = gp.read(a.model, env)
    snaps = {}

    def cb(mdl, where):
        if where != gp.GRB.Callback.BARRIER:
            return
        it = int(mdl.cbGet(gp.GRB.Callback.BARRIER_ITRCNT))
        if want is None or it in want:
            c = mdl._callback._cbdata
            B = L(api.PyCapsule_GetPointer(c, api.PyCapsule_GetName(c)) + 0x68)
            snaps[it] = dump(B, it, full=not snaps)

    model.optimize(cb)
    with open(a.out, "w") as f:
        for it in sorted(snaps):
            f.write(json.dumps(snaps[it]) + "\n")
    print(f"recorded iterations {sorted(snaps)}; BarIterCount {model.BarIterCount}, status {model.Status}")


if __name__ == "__main__":
    main()
