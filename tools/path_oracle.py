"""Pivot-path oracle (P1 first divergence): run Gurobi (Presolve=0, Threads=1, given Method) and at every SIMPLEX
callback diff the status array spx+0x168 against the previous callback. One record per callback:
[iteration, entering vars, leaving vars, nonbasic status changes (bound flips), objective].
usage: python tools/path_oracle.py <out_dir> [--method 1] name ...   (Gurobi: one process at a time)
writes <out_dir>/<name>.json.gz
"""
import argparse, ctypes, gzip, json
from pathlib import Path

import gurobipy as gp

DATA = Path(__file__).resolve().parents[2] / "re/bench/data/netlib"
api = ctypes.pythonapi
api.PyCapsule_GetName.restype = ctypes.c_char_p
api.PyCapsule_GetName.argtypes = [ctypes.py_object]
api.PyCapsule_GetPointer.restype = ctypes.c_void_p
api.PyCapsule_GetPointer.argtypes = [ctypes.py_object, ctypes.c_char_p]
L = lambda a: ctypes.c_long.from_address(a).value
I = lambda a: ctypes.c_int.from_address(a).value


def run(name, method, out):
    env = gp.Env(params={"OutputFlag": 0, "Threads": 1, "Method": method, "Presolve": 0})
    model = gp.read(str(DATA / f"{name}.mps.gz"), env)
    ptr = api.PyCapsule_GetPointer(model._cmodel, api.PyCapsule_GetName(model._cmodel))
    prev, rows = [None], []

    def cb(mdl, where):
        if where != gp.GRB.Callback.SIMPLEX:
            return
        spx = L(ptr + 0xe0)
        if not spx or not L(spx + 0x168):
            return
        k = I(spx + 0x64) + I(spx + 0x68)
        vs = list((ctypes.c_int * k).from_address(L(spx + 0x168)))
        it = int(mdl.cbGet(gp.GRB.Callback.SPX_ITRCNT))
        p = prev[0]
        if p is None:
            rows.append([it, [], [], 0, mdl.cbGet(gp.GRB.Callback.SPX_OBJVAL)])
        else:
            ent = [j for j in range(k) if p[j] < 0 <= vs[j]]
            lev = [j for j in range(k) if vs[j] < 0 <= p[j]]
            flips = sum(1 for j in range(k) if p[j] < 0 and vs[j] < 0 and p[j] != vs[j])
            rows.append([it, ent, lev, flips, mdl.cbGet(gp.GRB.Callback.SPX_OBJVAL)])
        prev[0] = vs

    model.optimize(cb)
    with gzip.open(out / f"{name}.json.gz", "wt") as f:
        json.dump({"IterCount": model.IterCount, "Status": model.Status, "path": rows}, f)
    print(f"{name}: {len(rows)} callbacks, IterCount {model.IterCount:.0f}")


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("out", type=Path)
    ap.add_argument("--method", type=int, default=1)
    ap.add_argument("names", nargs="+")
    a = ap.parse_args()
    a.out.mkdir(parents=True, exist_ok=True)
    for n in a.names:
        run(n, a.method, a.out)
