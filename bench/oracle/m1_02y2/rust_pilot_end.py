"""Writes rust_x_rc.json (x, RC, obj of Rust pilot Method 1 Presolve 0 through the driver, which hands out RC) and
checks that the plain optimize run (which wrote basis.json via the temporary RSOLVER_TMP02Y2_BASIS probe) ended at
the same x."""
import json, sys
from pathlib import Path
import rsolver

D = Path(__file__).parent
MPS = str(Path(__file__).resolve().parents[4] / "re/bench/data/netlib/pilot.mps.gz")
prm = {"Method": 1, "Presolve": 0, "Threads": 1, "OutputFlag": 0}
a = rsolver.read(MPS); a.Params.update(prm); a.optimize()
b = rsolver.read(MPS); b.Params.update(prm); b._driver(None)
xa, xb = a.getAttr("X"), b.getAttr("X")
assert a.IterCount == b.IterCount and xa == xb, (a.IterCount, b.IterCount, max(abs(p - q) for p, q in zip(xa, xb)))
json.dump({"x": xb, "rc": b.getAttr("RC"), "obj": b.ObjVal, "iter": b.IterCount}, open(D / "rust_x_rc.json", "w"))
print("rust pilot", b.ObjVal, b.IterCount)
