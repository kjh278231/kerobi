"""G5 (1): the parameter table (crates/model/src/param_table.rs, generated from libgurobi130.so) matches gurobipy.
bench/gurobi_params.tsv is gurobipy's getParamInfo for all 548 public and hidden parameters (tools/param_oracle.py).
No Gurobi is needed."""
import ast
import csv
from pathlib import Path

import rsolver

FIX = Path(__file__).resolve().parents[1] / "bench/gurobi_params.tsv"


def lit(s):
    return float(s) if s in ("inf", "-inf") else ast.literal_eval(s)


def test_param_table_matches_gurobipy():
    m = rsolver.Model()
    rows = list(csv.DictReader(FIX.open(), delimiter="\t"))
    assert len(rows) == 548
    for r in rows:
        name, typ, _cur, lo, hi, df = m.getParamInfo(r["name"].upper())
        got = (name, typ.__name__, lo, hi, df)
        want = (r["name"], r["type"], lit(r["min"]), lit(r["max"]), lit(r["default"]))
        assert got == want and all(type(a) is type(b) for a, b in zip(got, want)), (got, want)
    assert m.getParamInfo("NoSuchParam") is None
