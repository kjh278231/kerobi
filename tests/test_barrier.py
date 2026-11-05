"""M3 barrier: Method=2, Presolve=0, Crossover=0 against Gurobi (bench/gurobi_bar_nopre, docs/m3_barrier.md)."""
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
        _, aat, nz, ops = rsolver.read(DATA / f"{f.parent.name}.mps.gz")._core.barrier_symbolic()
        assert (f"{aat / 2:.3e}", f"{nz:.3e}", f"{ops:.3e}") == (g["AA' NZ"], g["Factor NZ"], g["Factor Ops"]), f.parent.name
