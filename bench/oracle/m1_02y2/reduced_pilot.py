"""M1-02y-2: does the original accept Rust's final pilot basis as optimal?

pilot (1441 x 3652) is over the size-limited license, so the original cannot run it. This removes nonbasic columns
of Rust's final basis (fixed at their values: rhs and objective constant adjusted) until the column count fits, keeps
every basic column and the nonbasic columns with the smallest |RC| (so all of Rust's dual-violating columns stay),
then runs Gurobi 13.0.3 (Method 1, Presolve 0, Threads 1) on the reduced model cold and warm-started from Rust's
basis, and cold with OptimalityTol = FeasibilityTol = 1e-9 for the reduced optimum. Rust's basis stays a basis of the reduced model and its x stays primal feasible. The reduced model is not the
same input state as pilot (Gurobi scales a different matrix), so this tests the end-point acceptance only.

needs: basis.json (Rust final basis, Gurobi convention, n + m entries) and rust_x_rc.json (x, RC of the same run),
written by the commands in docs/m1/m1_02y2_pilot.md §5.
usage: python bench/oracle/m1_02y2/reduced_pilot.py [--keep 2000]
"""
import argparse, json
from pathlib import Path
import gurobipy as gp

D = Path(__file__).parent
MPS = Path(__file__).resolve().parents[4] / "re/bench/data/netlib/pilot.mps.gz"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--keep", type=int, default=2000)
    a = ap.parse_args()
    basis = json.load(open(D / "basis.json"))
    xr = json.load(open(D / "rust_x_rc.json"))
    x, rc = xr["x"], xr["rc"]
    env = gp.Env(params={"OutputFlag": 0})
    g = gp.read(str(MPS), env)
    v, c = g.getVars(), g.getConstrs()
    n, m = len(v), len(c)
    assert len(basis) == n + m and len(x) == n
    assert sum(b == 0 for b in basis) == m
    nb = sorted((j for j in range(n) if basis[j] != 0), key=lambda j: abs(rc[j]))
    drop = set(nb[max(0, a.keep - (n - len(nb))):])
    # fix dropped columns at Rust's value: move a_ij x_j to the rhs and c_j x_j to the objective constant
    rhs = g.getAttr("RHS", c)
    const = g.ObjCon
    for j in drop:
        if x[j] != 0.0:
            col = g.getCol(v[j])
            for k in range(col.size()):
                rhs[col.getConstr(k).index] -= col.getCoeff(k) * x[j]
            const += v[j].Obj * x[j]
    g.setAttr("RHS", c, rhs)
    g.ObjCon = const
    keep = [j for j in range(n) if j not in drop]
    g.remove([v[j] for j in drop])
    g.update()
    kv = g.getVars()
    vio = [j for j in keep if basis[j] == -1 and rc[j] < -1e-9 or basis[j] == -2 and rc[j] > 1e-9]
    out = {"cols": len(kv), "rows": m, "dropped": len(drop), "rust_obj": xr["obj"],
           "rust_dual_violating_kept": len(vio), "min_dropped_abs_rc": min((abs(rc[j]) for j in drop), default=None)}
    for mode in ("cold", "warm", "tight"):
        h = g.copy()
        h.setParam("Method", 1)
        h.setParam("Presolve", 0)
        h.setParam("Threads", 1)
        if mode == "tight":  # the reduced model's optimum: tolerances 1e-9
            h.setParam("OptimalityTol", 1e-9)
            h.setParam("FeasibilityTol", 1e-9)
        if mode == "warm":
            hv = h.getVars()
            h.setAttr("VBasis", hv, [basis[j] for j in keep])
            h.setAttr("CBasis", h.getConstrs(), [0 if basis[n + i] == 0 else -1 for i in range(m)])
        h.optimize()
        out[mode] = {"status": h.Status, "iter": h.IterCount, "obj": h.ObjVal if h.Status == 2 else None}
    z = out["tight"]["obj"]
    for mode in ("cold", "warm"):
        out[mode]["rel_gap"] = (out[mode]["obj"] - z) / abs(z) if z is not None and out[mode]["obj"] is not None else None
    print(json.dumps(out, indent=1))


if __name__ == "__main__":
    main()
