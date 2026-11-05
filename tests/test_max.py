"""Maximization through the driver (sense handling of presolve, engines, uncrush and attributes): six Netlib LPs with
the objective negated and ModelSense = -1 (bench/gurobi_max, written by gurobipy), Presolve -1, against Gurobi.
Method 2/6 (Crossover 0): status, BarIterCount/PDHGIterCount and X, RC, Slack, Pi bit for bit; Method 1: status and
objective. No Gurobi."""
import json
from pathlib import Path

import pytest
import rsolver

FIX = Path(__file__).resolve().parents[1] / "bench/gurobi_max"
REF = json.loads((FIX / "ref.json").read_text())


@pytest.mark.parametrize("key", sorted(REF))
def test_max_model(key):
    name, method = key.split("/")
    r = REF[key]
    m = rsolver.read(FIX / f"{name}.mps")
    for k, v in {"Method": int(method), "Crossover": 0, "OutputFlag": 0, "Threads": 1, "WorkLimit": 60}.items():
        m.setParam(k, v)
    m.optimize()
    assert m.Status == r["Status"] and abs(m.ObjVal - r["ObjVal"]) <= 1e-6 * max(1.0, abs(r["ObjVal"]))
    if method != "1":
        it = "BarIterCount" if method == "2" else "PDHGIterCount"
        assert getattr(m, it) == r[it]
        for a in ("X", "RC", "Slack", "Pi"):
            assert [v.hex() for v in m.getAttr(a)] == r[a], a
