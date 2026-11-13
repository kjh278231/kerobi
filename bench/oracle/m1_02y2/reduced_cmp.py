"""M1-02y-2: original and Rust cold on the same reduced pilot files (Method 1, Presolve 0, Threads 1): IterCount,
objective and relative gap to the reduced optimum given as name=z.
usage: PYTHONPATH=$PWD/python python bench/oracle/m1_02y2/reduced_cmp.py file.mps=z ...
"""
import json, sys
import gurobipy as gp
import rsolver

prm = {"Method": 1, "Presolve": 0, "Threads": 1}
for arg in sys.argv[1:]:
    path, z = arg.rsplit("=", 1)
    z = float(z)
    g = gp.read(path, gp.Env(params={"OutputFlag": 0}))
    for k, v in prm.items():
        g.setParam(k, v)
    g.optimize()
    r = rsolver.read(path)
    r.Params.update({**prm, "OutputFlag": 0})
    r.optimize()
    row = {"file": path.rsplit("/", 1)[-1], "z": z}
    for name, m in (("gurobi", g), ("rust", r)):
        row[name] = {"status": int(m.Status), "iter": float(m.IterCount), "obj": m.ObjVal, "rel_gap": (m.ObjVal - z) / abs(z)}
    print(json.dumps(row), flush=True)
