"""`365159` (uncrush without a basis, the barrier/PDHG path of `36578f`) against Gurobi, bit for bit.
bench/gurobi_uncrush159/<model>.json holds what Gurobi passed to 365159 (presolved x, slack, d, y, columns marked in
D+0x408 still flipped) and what it returned (x, slack, d, y of the original model), captured with
tools/uncrush_oracle.py at Method 2, Crossover 0. The set covers record types 1 2 3 4 6 7 8 11 14 21 24 and a presolve
that removes nothing (grow7). No Gurobi is needed."""
import json
from pathlib import Path

import pytest
import rsolver

ROOT = Path(__file__).resolve().parents[1]
FIX = ROOT / "bench/gurobi_uncrush159"
H = lambda v: [float.fromhex(s) for s in v]


@pytest.mark.parametrize("name", sorted(p.stem for p in FIX.glob("*.json")))
def test_uncrush_nonbasic_matches_gurobi(name):
    cap = json.loads((FIX / f"{name}.json").read_text())
    cap["params"] = [tuple(p) for p in cap["params"]]
    core = rsolver.read(ROOT.parent / "re/bench/data/netlib" / f"{name}.mps.gz")._core
    assert [k for k, _, _, _ in core.post_records(cap["params"])] == cap["kinds"]
    q = core.presolve(cap["params"])
    x, d = H(cap["in"]["x"]), H(cap["in"]["d"])
    for j, (lo, up) in enumerate(zip(q.lb, q.ub)):
        if lo <= -1e30 and up < 1e30:  # 07d3ce flip mark: 365159 flips these back first
            x[j], d[j] = -x[j], -d[j]
    gx, gd = core.uncrush_nonbasic(cap["params"], x + H(cap["in"]["slack"]), d + H(cap["in"]["y"]))
    n, o = cap["n_m"][0], cap["out"]
    assert [v.hex() for v in gx[:n]] == o["x"]
    assert [v.hex() for v in gx[n:]] == o["slack"]
    assert [v.hex() for v in gd[:n]] == o["d"]
    assert [v.hex() for v in gd[n:]] == o["y"]
