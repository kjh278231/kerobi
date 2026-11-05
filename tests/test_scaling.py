"""M1 stage A: LP simplex scaling equals Gurobi's (bench/gurobi_spx0, read at iteration 0 by tools/spx_oracle.py)."""
import json
from pathlib import Path

import rsolver

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT.parent / "re/bench/data/netlib"


def test_scale_factors_bitwise_equal():
    files = sorted((ROOT / "bench/gurobi_spx0").glob("*.json"))
    assert len(files) == 64
    for f in files:
        g = json.load(open(f))
        col, row, obj = rsolver.read(DATA / f"{f.stem}.mps.gz")._core.scaling()
        assert [x.hex() for x in col] == g["colscale"], f.stem
        assert [x.hex() for x in row] == g["rowscale"], f.stem
        assert obj.hex() == g["objscale"], f.stem


def test_iteration0_basic_solution_bitwise_equal():
    """M1 stage C: singleton LU + FTRAN give Gurobi's x_B for its iteration-0 basis."""
    n_checked = 0
    for f in sorted((ROOT / "bench/gurobi_spx0").glob("*.json")):
        g = json.load(open(f))
        if not g["xB_valid"]:
            continue
        xb = rsolver.read(DATA / f"{f.stem}.mps.gz")._core.basic_solution(g["vstat"])
        assert [x.hex() for x in xb] == g["xB"], f.stem
        n_checked += 1
    assert n_checked == 60
