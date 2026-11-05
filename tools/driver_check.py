"""G5 (4): rsolver optimize() with presolve against a Gurobi capture of tools/parity.py (summary.json per model).
Prints one line per model whose status, objective (relative 1e-6) or iteration count (--iters) differs, then counts.
usage: PYTHONPATH=$PWD/python python tools/driver_check.py <capture_dir> [--iters IterCount|BarIterCount|PDHGIterCount]
       [--param Name=Value ...] [name ...]      (no Gurobi; parameters as in the capture's log)
"""
import argparse, json, sys, time
from pathlib import Path

import rsolver

DATA = Path(__file__).resolve().parents[2] / "re/bench/data/netlib"
ap = argparse.ArgumentParser()
ap.add_argument("ref")
ap.add_argument("--iters")
ap.add_argument("--param", action="append", default=[])
ap.add_argument("names", nargs="*")
a = ap.parse_args()
names = a.names or sorted(p.name for p in Path(a.ref).iterdir() if (p / "summary.json").exists())
ok = {"status": 0, "obj": 0, "iters": 0}
for name in names:
    ref = json.loads((Path(a.ref) / name / "summary.json").read_text())
    m = rsolver.read(DATA / f"{name}.mps.gz")
    m.setParam("OutputFlag", 0)
    m.setParam("Threads", 1)
    m.setParam("WorkLimit", 60)
    for kv in a.param:
        k, v = kv.split("=")
        m.setParam(k, float(v))
    t = time.perf_counter()
    print(f"{name:12s} ...", end="\r", flush=True)
    m.optimize()
    obj = getattr(m, "_objval", None)
    good = {"status": m.Status == ref["Status"],
            "obj": obj is not None and ref["ObjVal"] is not None and abs(obj - ref["ObjVal"]) <= 1e-6 * max(1.0, abs(ref["ObjVal"])),
            "iters": not a.iters or getattr(m, a.iters) == ref[a.iters]}
    for k in ok:
        ok[k] += good[k]
    if not all(good.values()):
        it = f" {a.iters} {getattr(m, a.iters)} vs {ref[a.iters]}" if a.iters else ""
        print(f"{name:12s} status {m.Status} vs {ref['Status']} obj {obj} vs {ref['ObjVal']}{it} ({time.perf_counter() - t:.1f}s)", flush=True)
print(" ".join(f"{k} {v}/{len(names)}" for k, v in ok.items()))
