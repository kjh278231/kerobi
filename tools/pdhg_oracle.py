"""PDHG-state oracle (docs/m3_pdhg.md §3): read Gurobi's PDHG state P (callback cbdata+0x68 = &local_878 of 34c004,
layout in docs/m3_pdhg.md) inside PDHG callbacks (where = 10, every 100 iterations) and dump it, so a port can be
compared field by field and vector by vector at the first divergence.

usage: python tools/pdhg_oracle.py <model.mps.gz> <out.jsonl[.gz]> [--iters 0,93|all] [--vec] [--param Name=Value ...]
Default params: Threads=1, Method=6, Presolve=0, Crossover=0. One JSON object per recorded callback, keyed by the
callback's PDHG_ITRCNT. Doubles are float.hex() so they compare bit for bit. Every record has the raw P block (0x730
bytes as doubles); the first one also has the model data D as PDHG sees it (scaled in place, bar_oracle.snap_D).
--vec adds every heap vector P points to: {offset: [addr % 64, doubles...]}, length = malloc_usable_size / 8 (the
tail past the real length is allocator slack).
"""
import argparse, ctypes, gzip, json

import gurobipy as gp

from bar_oracle import L, api, arr, readable, snap_D

libc = ctypes.CDLL(None)
libc.malloc_usable_size.restype = ctypes.c_size_t
libc.malloc_usable_size.argtypes = [ctypes.c_void_p]
P_SIZE = 0x730


def heap_vectors(P):
    regions = [(a, b) for a, b, name in readable() if name in ("", "[heap]")]  # malloc arenas, not stack or files
    out = {}
    for k in range(P_SIZE // 8):
        v = L(P + 8 * k)
        if v % 8 == 0 and any(a <= v < b for a, b in regions):
            out[f"{8 * k:#x}"] = [v % 64, *arr(v, libc.malloc_usable_size(v) // 8)]
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("model")
    ap.add_argument("out")
    ap.add_argument("--iters", default="all")
    ap.add_argument("--vec", action="store_true")
    ap.add_argument("--param", action="append", default=[])
    a = ap.parse_args()
    params = {"OutputFlag": 0, "Threads": 1, "Method": 6, "Presolve": 0, "Crossover": 0}
    for p in a.param:
        k, v = p.split("=", 1)
        params[k] = int(v) if v.lstrip("-").isdigit() else float(v)
    want = None if a.iters == "all" else {int(x) for x in a.iters.split(",")}
    env = gp.Env(params=params)
    model = gp.read(a.model, env)
    mp = api.PyCapsule_GetPointer(model._cmodel, api.PyCapsule_GetName(model._cmodel))
    C = gp.GRB.Callback
    recs = []

    def cb(mdl, where):
        if where != C.PDHG:
            return
        it = int(mdl.cbGet(C.PDHG_ITRCNT))
        if want is not None and it not in want:
            return
        c = mdl._callback._cbdata
        P = L(api.PyCapsule_GetPointer(c, api.PyCapsule_GetName(c)) + 0x68)
        r = {"it": it, "cb": [mdl.cbGet(w).hex() for w in (C.PDHG_PRIMOBJ, C.PDHG_DUALOBJ, C.PDHG_PRIMINF,
                                                            C.PDHG_DUALINF, C.PDHG_COMPL)],
             "P": arr(P, P_SIZE // 8)}
        if not recs:
            r["D"] = snap_D(L(mp + 0xe8))
            r["D_objsense"] = ctypes.c_int.from_address(L(mp + 0xe8) + 4).value
        if a.vec:
            r["vec"] = heap_vectors(P)
        recs.append(r)

    model.optimize(cb)
    with (gzip.open(a.out, "wt") if a.out.endswith(".gz") else open(a.out, "w")) as f:
        for r in recs:
            f.write(json.dumps(r) + "\n")
    print(f"recorded callbacks {[r['it'] for r in recs][:8]}... ({len(recs)}); PDHGIterCount {model.PDHGIterCount}, "
          f"status {model.Status}, ObjVal {model.ObjVal if model.SolCount else None!r}")


if __name__ == "__main__":
    main()
