"""M1-02y-2: unscaled dual violation of the final LP solution, from the public X/RC/Slack/Pi attributes only.
Both solvers read the same MPS; bounds, senses and ModelSense come from gurobipy's read (allowed outside the size
limit). A column at a bound (|x - bound| <= 1e-9 max(1,|bound|)) is dual-infeasible by the wrong-signed part of RC,
a column strictly between its bounds by |RC|; a tight row (|slack| <= FeasTol 1e-6: Rust leaves rounding up to ~1e-8
in the slack of nonbasic rows where Gurobi reports 0, and a basic row has Pi ~ 0 either way) by the wrong-signed part
of Pi, any other row by |Pi|. Minimize convention (RC, Pi multiplied by ModelSense).
usage: python tools/m1_end_dualvio.py {gurobi|rust} [--method 1] [--out f.jsonl] name ...   (names: Netlib files)
       python tools/m1_end_dualvio.py --self-test
"""
import argparse, json, sys
from pathlib import Path

DATA = Path(__file__).resolve().parents[2] / "re/bench/data/netlib"
INF = 1e30


def at(x, b):
    return abs(b) < INF and abs(x - b) <= 1e-9 * max(1.0, abs(b))


def dual_vio(lb, ub, x, rc, sense, slack, pi, s):
    """(max column violation, max row violation, count over 1e-9) in minimize convention."""
    cv = []
    for l, u, xj, d in zip(lb, ub, x, rc):
        d *= s
        lo, up = at(xj, l), at(xj, u)
        cv.append(0.0 if lo and up else max(-d, 0.0) if lo else max(d, 0.0) if up else abs(d))
    rv = []
    for sn, sl, y in zip(sense, slack, pi):
        y *= s
        if sn == "=" or abs(sl) > 1e-6:
            rv.append(0.0 if sn == "=" else abs(y))
        else:  # min: y <= 0 on a tight <= row, y >= 0 on a tight >= row
            rv.append(max(y, 0.0) if sn == "<" else max(-y, 0.0))
    return max(cv, default=0.0), max(rv, default=0.0), sum(v > 1e-9 for v in cv + rv)


def structure(path):
    import gurobipy as gp
    env = gp.Env(params={"OutputFlag": 0})
    g = gp.read(str(path), env)
    v, c = g.getVars(), g.getConstrs()
    return g, g.getAttr("LB", v), g.getAttr("UB", v), g.getAttr("Sense", c), g.ModelSense


def run(solver, name, method):
    path = DATA / f"{name}.mps.gz"
    g, lb, ub, sense, s = structure(path)
    if solver == "gurobi":
        m = g
        for k, val in {"Method": method, "Presolve": 0, "Threads": 1}.items():
            m.setParam(k, val)
        m.optimize()
        v, c = m.getVars(), m.getConstrs()
        get = lambda a, o: m.getAttr(a, o)
    else:
        import rsolver
        m = rsolver.read(str(path))
        m.Params.update({"Method": method, "Presolve": 0, "Threads": 1, "OutputFlag": 0})
        m._driver(None)  # the Rust driver with Presolve=0 (same engine as optimize, which hands out no RC/Pi)
        v = c = None
        get = lambda a, o: m.getAttr(a)
    row = {"name": name, "solver": solver, "method": method, "status": int(m.Status), "iter": float(m.IterCount)}
    if m.Status == 2:
        x, rc, sl, pi = get("X", v), get("RC", v), get("Slack", c), get("Pi", c)
        if solver == "rust":  # shortcut: Rust hands out Pi negated on '>' rows (M1-02y-2 §6), undo until it is fixed
            pi = [-y if sn == ">" else y for y, sn in zip(pi, sense)]
        assert len(x) == len(lb) and len(pi) == len(sense), (len(x), len(lb), len(pi), len(sense))
        cmax, rmax, cnt = dual_vio(lb, ub, x, rc, sense, sl, pi, s)
        row.update(obj=m.ObjVal, dvio_col=cmax, dvio_row=rmax, dvio_n=cnt)
    return row


def self_test():
    # min x0 + x1 at lower 0 with RC = (1, -2e-7): column violation 2e-7; a <= row with slack 1e-8 is tight
    # (rounding), so its Pi = +3e-7 violates
    c, r, n = dual_vio([0, 0], [INF, INF], [0, 0], [1, -2e-7], ["<", ">"], [1e-8, 5], [3e-7, 0], 1)
    assert (c, r, n) == (2e-7, 3e-7, 2), (c, r, n)
    # maximize flips signs; a column between bounds counts |RC|
    c, r, n = dual_vio([0], [10], [4], [-1e-8], ["="], [0], [7], -1)
    assert (c, r, n) == (1e-8, 0.0, 1), (c, r, n)
    print("self-test PASS")


def main():
    if "--self-test" in sys.argv:
        return self_test()
    ap = argparse.ArgumentParser()
    ap.add_argument("solver", choices=["gurobi", "rust"])
    ap.add_argument("--method", type=int, default=1)
    ap.add_argument("--out")
    ap.add_argument("names", nargs="+")
    a = ap.parse_args()
    out = open(a.out, "a") if a.out else None
    for name in a.names:
        try:
            row = run(a.solver, name, a.method)
        except Exception as e:  # e.g. Model too large: record and go on
            row = {"name": name, "solver": a.solver, "method": a.method, "error": str(e)[:200]}
        print(json.dumps(row), flush=True)
        if out:
            out.write(json.dumps(row) + "\n")


if __name__ == "__main__":
    main()
