"""Simplex-state oracle (docs/10 §6): read Gurobi's simplex workspace (model+0xe0, layout in re/notes/13 §4) from
inside SIMPLEX callbacks and dump it per iteration, so a port can be compared array by array at the first divergence.

usage: python tools/spx_oracle.py <model.mps.gz> <out.jsonl> [--iters 0,1,2,10] [--param Name=Value ...]
Default params: Threads=1, Method=1, Presolve=0. One JSON object per recorded iteration, taken at the LAST callback with
that count: dual initialization fires SIMPLEX callbacks mid-way (free-column pivots), before x_B exists.
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


def arr(p, k, kind="d"):
    if not p or k <= 0:
        return None
    t = {"d": ctypes.c_double, "i": ctypes.c_int, "l": ctypes.c_long}[kind]
    return [x.hex() if kind == "d" else x for x in (t * k).from_address(p)]  # doubles as hex: exact bits


def sparse(p, m):
    """{int cnt; int* idx (+8); double* val (+0x10)}: (cnt, idx[:cnt], val[:cnt]); cnt = -1 is dense: (-1, None, val[:m])."""
    if not p:
        return None
    k = I(p)
    if k == -1:
        return [k, None, arr(L(p + 0x10), m)]
    return [k, arr(L(p + 8), k, "i"), arr(L(p + 0x10), k)] if 0 < k < 10**7 else [k, None, None]


def dump(spx, it, full):
    m, n = I(spx + 0x64), I(spx + 0x68)
    P = L(spx + 0x478)
    out = {"iter": it, "m": m, "n": n, "status": I(spx + 0xf0), "obj": D(spx + 0x118).hex(), "objscale": D(spx + 0xd8).hex(),
           "pert": D(spx + 0x348).hex(), "pert_level": I(spx + 0x338), "mode": I(spx + 0x3ec), "quad": I(spx + 0x28),
           "r": I(spx + 0x2f8),
           # perturbation trigger counters (48d5f): refactor count, degenerate count and its mark, iteration mark, flag
           # 0x110, objective record 0x140, 0x3e0, restarts 0x35c
           "pt": {"rc": I(spx + 0x2c0), "deg": I(spx + 0x330), "deg0": I(spx + 0x334), "it0": D(spx + 0x2b0),
                  "f110": I(spx + 0x110), "rec": D(spx + 0x140), "x3e0": D(spx + 0x3e0), "sw": I(spx + 0x35c),
                  "x2a8": D(spx + 0x2a8)}, "q": I(spx + 0x2fc), "refactor_reason": I(spx + 0x294),
           # last pivot: FTRAN column α_q (spx+0x210) and "tau" = spx+0x218 as {cnt, idx*, val*}, α_r, β = ‖ρ‖². spx+0x218 holds ρ_r
           # inside 28c93/8b473; at the callback it is a reused, mixed buffer (docs/m1/m1_02a_beta.md), not a clean ρ or τ
           "aq": sparse(L(spx + 0x210), m), "tau": sparse(L(spx + 0x218), m), "ar": D(spx + 0x300).hex(), "aq308": D(spx + 0x308).hex(),
           "beta": D(spx + 0x310).hex(),
           "vstat": arr(L(spx + 0x168), n + m, "i"), "d": arr(L(spx + 0x1e0), n + m), "xB": arr(L(spx + 0x1e8), m),
           "c": arr(L(spx + 0x98), n + m), "l": arr(L(spx + 0xa8), n + m), "u": arr(L(spx + 0xb0), n + m),
           "w": arr(L(P + 0x08), m) if P else None,
           # CHUZR state (2896a3): mode P+0x40 (0 = ordered list), list P+0x10 of length P+0x3c, cursor spx+0x2c4,
           # per-position skip flags spx+0x280, rebuild budget P+0x38
           "price": {"mode": I(P + 0x40), "list": arr(L(P + 0x10), max(I(P + 0x3c), I(P + 0x28) if I(P + 0x40) == 1 else 0), "i"), "cursor": I(spx + 0x2c4),
                     "flags": list(ctypes.string_at(L(spx + 0x280), m)), "budget": I(P + 0x38), "allow": I(P + 0x28), "score": arr(L(P + 0x18), m), "cost_mode": I(P + 0x44),
                     "row_cost": arr(L(P + 0x68), m), "var_cost": arr(L(P + 0x70), n + m),
                     "avg": D(P + 0x48), "thr": D(P + 0x30).hex()} if P else None,
           # LU counters (spx+0x470): updates done, refactor interval, row-eta count, Σ nnz of update columns
           "lu": {"k": I(L(spx + 0x470) + 0x18c), "interval": I(L(spx + 0x470) + 0x188),
                  "eta": I(L(spx + 0x470) + 0x184), "cols": L(L(spx + 0x470) + 0x190),
                  # U position of each LU index (LU+0x40) and the prefix of empty U columns (LU+0x100)
                  "upos": arr(L(L(spx + 0x470) + 0x40), m, "i"), "t": I(L(spx + 0x470) + 0x100)}
           if L(spx + 0x470) else None}
    if full:
        beg = arr(L(spx + 0x70), n + 1, "l")
        nnz = beg[n] if beg else 0
        out.update({"colscale": arr(L(spx + 0xe0), n), "rowscale": arr(L(spx + 0xe8), m), "b": arr(L(spx + 0xa0), m),
                    "sense": bytes((ctypes.c_char * m).from_address(L(spx + 0x90))).decode() if L(spx + 0x90) else None,
                    "beg": beg, "len": arr(L(spx + 0x78), n, "i"), "ind": arr(L(spx + 0x80), nnz, "i"),
                    "val": arr(L(spx + 0x88), nnz)})
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("model")
    ap.add_argument("out")
    ap.add_argument("--iters", default="0")
    ap.add_argument("--param", action="append", default=[])
    a = ap.parse_args()
    params = {"OutputFlag": 0, "Threads": 1, "Method": 1, "Presolve": 0}
    for p in a.param:
        k, v = p.split("=", 1)
        params[k] = int(v) if v.lstrip("-").isdigit() else float(v)
    want = {int(x) for x in a.iters.split(",")}
    env = gp.Env(params=params)
    model = gp.read(a.model, env)
    ptr = api.PyCapsule_GetPointer(model._cmodel, api.PyCapsule_GetName(model._cmodel))
    snaps = {}

    def cb(mdl, where):
        if where != gp.GRB.Callback.SIMPLEX:
            return
        it = int(mdl.cbGet(gp.GRB.Callback.SPX_ITRCNT))
        if it in want and L(ptr + 0xe0):
            snaps[it] = dump(L(ptr + 0xe0), it, full=it == min(want))  # overwrite: keep the last callback per count

    model.optimize(cb)
    with open(a.out, "w") as f:
        for it in sorted(snaps):
            f.write(json.dumps(snaps[it]) + "\n")
    print(f"recorded iterations {sorted(snaps)}; IterCount {model.IterCount:.0f}, status {model.Status}")


if __name__ == "__main__":
    main()
