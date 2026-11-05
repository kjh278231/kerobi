"""Simplex-state oracle (docs/10 §6): read Gurobi's simplex workspace (model+0xe0, layout in re/notes/13 §4) from
inside SIMPLEX callbacks and dump it per iteration, so a port can be compared array by array at the first divergence.

usage: python tools/spx_oracle.py <model.mps.gz> <out.jsonl> [--iters 0,1,2,10] [--param Name=Value ...]
Default params: Threads=1, Method=1, Presolve=0. One JSON object per recorded iteration (first callback of that count).
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


def dump(spx, it, full):
    m, n = I(spx + 0x64), I(spx + 0x68)
    P = L(spx + 0x478)
    out = {"iter": it, "m": m, "n": n, "status": I(spx + 0xf0), "obj": D(spx + 0x118).hex(), "objscale": D(spx + 0xd8).hex(),
           "pert": D(spx + 0x348).hex(), "pert_level": I(spx + 0x338), "mode": I(spx + 0x3ec), "quad": I(spx + 0x28),
           "r": I(spx + 0x2f8), "q": I(spx + 0x2fc), "refactor_reason": I(spx + 0x294),
           "vstat": arr(L(spx + 0x168), n + m, "i"), "d": arr(L(spx + 0x1e0), n + m), "xB": arr(L(spx + 0x1e8), m),
           "c": arr(L(spx + 0x98), n + m), "l": arr(L(spx + 0xa8), n + m), "u": arr(L(spx + 0xb0), n + m),
           "w": arr(L(P + 0x08), m) if P else None}
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
    done, f = set(), open(a.out, "w")

    def cb(mdl, where):
        if where != gp.GRB.Callback.SIMPLEX:
            return
        it = int(mdl.cbGet(gp.GRB.Callback.SPX_ITRCNT))
        if it in want and it not in done and L(ptr + 0xe0):
            done.add(it)
            f.write(json.dumps(dump(L(ptr + 0xe0), it, full=not (done - {it}))) + "\n")

    model.optimize(cb)
    f.close()
    print(f"recorded iterations {sorted(done)}; IterCount {model.IterCount:.0f}, status {model.Status}")


if __name__ == "__main__":
    main()
