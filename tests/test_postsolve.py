"""M1 LP postsolve: rsolver's uncrush (postsolve.rs, Gurobi's 363daa) reproduces Gurobi's output arrays bit for bit.
bench/gurobi_uncrush/<model>.json is a capture of tools/uncrush_oracle.py at default parameters: the record kinds
(oldest first), the presolved solution 363daa reads (`pre_after`: x, dj, status of the columns then rows) and its
output (basis int[n+m], x[n+m] = columns then slack, dual[n+m] = d then y). No Gurobi is needed.
Covered record types: afiro 1/2/7/11, scfxm1 4/6/8/11, forplan 24, fit1d 4, gfrd-pnc 21."""
import json
from pathlib import Path

import pytest
import rsolver

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT.parent / "re/bench/data"
FIX = ROOT / "bench/gurobi_uncrush"


def hexes(v):
    return [float.fromhex(s) for s in v]


@pytest.mark.parametrize("name", ["afiro", "scfxm1", "forplan", "fit1d", "gfrd-pnc"])
def test_uncrush_matches_gurobi(name):
    cap = json.loads((FIX / f"{name}.json").read_text())
    core = rsolver.read(DATA / "netlib" / f"{name}.mps.gz")._core
    assert [k for k, _, _, _ in core.post_records(cap["params"])] == cap["kinds"]
    pa = cap["pre_after"]
    basis, x, dual = core.uncrush(cap["params"], hexes(pa["x"]), hexes(pa["dj"]), pa["st"])
    out = cap["out"]
    assert basis == out["basis"]
    assert [v.hex() for v in x] == out["x"]
    assert [v.hex() for v in dual] == out["dual"]


@pytest.mark.parametrize("name", ["scsd1", "grow7"])
def test_no_uncrush_without_removals(name):
    """Presolve removes no row or column: Gurobi never calls 363daa, the solution is passed through."""
    core = rsolver.read(DATA / "netlib" / f"{name}.mps.gz")._core
    nrec, counts = core.post_counts([])
    assert nrec == 0 and counts[3] == 0 and counts[4] == 0
    n, m = len(core.obj), len(core.rhs)
    x, dj, st = [float(i) for i in range(n + m)], [0.5 * i for i in range(n + m)], [-1] * (n + m)
    assert core.uncrush([], x, dj, st) == (st, x, dj)
