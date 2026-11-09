"""Root LP regressions against the stored Gurobi Threads=1 logs."""
import re
import sys
from pathlib import Path

import pytest
from rsolver import _core

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT.parent / "re/bench/data/miplib"
sys.path.insert(0, str(ROOT / 'tools'))
from mip_root_replay import rebuild, replay  # noqa: E402


@pytest.mark.parametrize("name", [
    "gen-ip002", "markshare2", "tr12-30", "mik-250-20-75-4",
    "neos-4338804-snowy", "fhnw-binpack4-4", "markshare_4_0", "glass4",
    "ic97_potential", "graphdraw-domain", "timtab1",
])
def test_integer_column_rules_match_root_objective_and_iterations(name):
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


def test_mad_captured_root_bounds_explain_remaining_iteration_difference():
    model, typed, reference, iterations = replay(ROOT / 'bench/oracle/mip_root/mad')
    assert iterations == reference['iter'] == 34
    pre = _core.read_model(str(DATA / 'mad.mps.gz')).presolve([('Threads', 1.0)])
    assert (pre.cols, pre.obj, pre.rhs, pre.row_sense, pre.lb, pre.vtype) == (
        model.cols, model.obj, model.rhs, model.row_sense, model.lb, typed.vtype,
    )
    changed = [j for j, (a, b) in enumerate(zip(pre.ub, model.ub)) if a != b]
    assert changed == list(range(200, 220))
    assert all(pre.ub[j] == 1e30 and model.ub[j] == 6.9639999992036 for j in changed)
    # Change only these bounds: the solver and all matrix/column-type inputs stay the same.
    status, obj, _, iterations, *_ = rebuild(model, typed.vtype, pre.lb, pre.ub).solve(1, 1e18, False)
    assert (status, obj, iterations) == (2, 0.0, 32)
