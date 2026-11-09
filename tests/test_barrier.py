"""M3 barrier: Method=2, Presolve=0, Crossover=0 against Gurobi (bench/gurobi_bar_nopre, docs/m3/m3_barrier.md)."""
import json
from pathlib import Path

import rsolver

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT.parent / "re/bench/data/netlib"


def test_barrier_statistics_equal_gurobi_log():
    """Ordering result: AA' NZ, Factor NZ, Factor Ops as printed under "Barrier statistics" (%.3e)."""
    files = sorted((ROOT / "bench/gurobi_bar_nopre").glob("*/summary.json"))
    assert len(files) == 64
    for f in files:
        g = json.load(open(f))["log"]["barstats"]
        _, aat, nz, ops, free = rsolver.read(DATA / f"{f.parent.name}.mps.gz")._core.barrier_symbolic()
        assert (f"{aat / 2:.3e}", f"{nz:.3e}", f"{ops:.3e}") == (g["AA' NZ"], g["Factor NZ"], g["Factor Ops"]), f.parent.name
        assert str(free) == g.get("Free vars", "0"), f.parent.name


def test_barrier_solves_64_lps_like_gurobi():
    """Method=2 path of Model.optimize: status and objective (rel. err <= 1e-6) on all 64, BarIterCount equal on at
    on all 64 (measured 2026-11-06 with the bit-exact factor/solve and sums, docs/m3/m3_chol.md; 55/64 traces bit-identical), one BARRIER callback per iteration including the last."""
    same = 0
    for f in sorted((ROOT / "bench/gurobi_bar_nopre").glob("*/summary.json")):
        g = json.load(open(f))
        m = rsolver.read(DATA / f"{f.parent.name}.mps.gz")
        m.setParam("Method", 2)
        m.setParam("Presolve", 0)
        its = []
        m.optimize(lambda model, where: its.append((where, model.cbGet(rsolver.GRB.Callback.BARRIER_ITRCNT))))
        assert m.Status == g["Status"], f.parent.name
        assert abs(m.ObjVal - g["ObjVal"]) <= 1e-6 * max(1.0, abs(g["ObjVal"])), f.parent.name
        assert its == [(rsolver.GRB.Callback.BARRIER, k) for k in range(m.BarIterCount + 1)], f.parent.name
        same += m.BarIterCount == g["BarIterCount"]
    assert same == 64
