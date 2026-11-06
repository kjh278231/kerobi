"""Root LP regressions against the stored Gurobi Threads=1 logs."""
import re
from pathlib import Path

import pytest
from rsolver import _core

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT.parent / "re/bench/data/miplib"


@pytest.mark.parametrize("name", [
    "gen-ip002", "markshare2", "tr12-30", "mik-250-20-75-4",
    "neos-4338804-snowy", "fhnw-binpack4-4",
])
def test_integer_column_weights_match_root_objective_and_iterations(name):
    ref = re.search(
        r"Root relaxation: objective (\S+), (\d+) iterations",
        (ROOT / "bench/gurobi" / name / "log.txt").read_text(),
    )
    assert ref is not None
    model = _core.read_model(str(DATA / f"{name}.mps.gz")).presolve([("Threads", 1.0)])
    method, status, obj, iterations = model.mip_root([("Threads", 1.0)])
    assert (method, status) == (1, 2)
    assert f"{obj:.6e}" == ref[1]
    assert iterations == int(ref[2])


def test_presolve_removing_all_columns_has_no_root_lp():
    model = _core.read_model(str(DATA / "enlight_hard.mps.gz")).presolve([("Threads", 1.0)])
    assert model.mip_root([("Threads", 1.0)]) is None
