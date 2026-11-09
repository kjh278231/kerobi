"""Root LP regressions against the stored Gurobi Threads=1 logs."""
import json
import re
import sys
from pathlib import Path

import pytest
from rsolver import _core

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT.parent / "re/bench/data/miplib"
sys.path.insert(0, str(ROOT / 'tools'))
from mip_root_replay import rebuild, replay  # noqa: E402


def test_objective_lattice_and_user_cutoff_reach_python_root_bounds():
    model = _core.build_model([2.0, 3.0], [], [0.0, 0.0], [20.0, 20.0], 'II')
    assert model.mip_root_bounds([], [2.0, 2.0]) == ([0.0, 0.0], [4.0, 3.0])
    assert model.mip_root_bounds([('GURO_PAR_OBJGCD', 0.0)], [2.0, 2.0]) == ([0.0, 0.0], [5.0, 3.0])
    # A stricter user cutoff is used directly, without lattice rounding or an incumbent.
    params = [('Cutoff', 6.0), ('Heuristics', 0.0)]
    assert model.mip_root_bounds(params) == ([0.0, 0.0], [3.0, 2.0])
    assert model.mip_root_bounds(params, [2.0, 2.0]) == ([0.0, 0.0], [3.0, 2.0])
    inc, (status, p0, root, bounds) = model.mip_presolve_root(params + [('GURO_PAR_PRELIMIT', 0.0)])
    assert inc is None and status == 0 and root[1] == 2
    assert bounds == ([0.0, 0.0], [3.0, 2.0])
    assert p0.ub == model.ub == [20.0, 20.0]


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


def test_mad_incumbent_tightens_root_domain():
    x = json.loads((ROOT / 'bench/oracle/mip_root/mad/incumbent.json').read_text())
    model = _core.read_model(str(DATA / 'mad.mps.gz')).presolve([('Threads', 1.0)])
    captured = _core.read_model(str(ROOT / 'bench/oracle/mip_root/mad/0.mps.gz'))
    lb, ub = model.mip_root_bounds([('Threads', 1.0)], x)
    assert lb == captured.lb
    assert ub == captured.ub
    assert model.mip_root([('Threads', 1.0)], x)[3] == 34
    assert model.mip_root([('Threads', 1.0), ('Heuristics', 0.0)])[3] == 32
    x[0] = 0.5
    with pytest.raises(ValueError):
        model.mip_root([('Threads', 1.0)], x)


def test_original_pipeline_finds_mad_and_preserves_presolve():
    params = [('Threads', 1.0)]
    raw = _core.read_model(str(DATA / 'mad.mps.gz'))
    before = raw.to_mps()
    incumbent, (status, p0, root, bounds) = raw.mip_presolve_root(params)
    assert incumbent[0] == pytest.approx(6.964)
    assert incumbent[1] == json.loads((ROOT / 'bench/oracle/mip_root/mad/incumbent.json').read_text())
    assert status == 0 and p0.to_mps() == raw.presolve(params).to_mps()
    captured = _core.read_model(str(ROOT / 'bench/oracle/mip_root/mad/0.mps.gz'))
    assert bounds == (captured.lb, captured.ub)
    assert root == (1, 2, 0.0, 34.0)
    assert raw.to_mps() == before
    for disabled in [('Heuristics', 0.0), ('GURO_PAR_PREPREHEUR', 0.0)]:
        inc, (st, pre, lp, actual_bounds) = raw.mip_presolve_root(params + [disabled])
        assert inc is None and st == 0
        assert actual_bounds == pre.mip_root_bounds(params + [('Heuristics', 0.0)])
        assert lp[3] == 32


def test_original_pipeline_reports_presolve_and_unsupported_models():
    impossible = _core.build_model([0.0], [], [2.5], [2.5], 'I')
    assert impossible.mip_presolve_root([]) == (None, (3, None, None, None))
    unbounded = _core.build_model([-1.0], [], [0.0], [1e30], 'I')
    inc, outcome = unbounded.mip_presolve_root([])
    assert inc is not None and outcome == (5, None, None, None)
    empty = _core.build_model([], [], [], [], '')
    inc, (status, p0, root, bounds) = empty.mip_presolve_root([])
    assert inc is None and status == 0 and p0.obj == [] and root is None and bounds == ([], [])
    infeasible_empty = _core.build_model([], [([], 1.0, 1e30)], [], [], '')
    assert infeasible_empty.mip_presolve_root([]) == (None, (3, None, None, None))
    empty_domain = _core.build_model([1.0], [], [2.0], [3.0], 'B')
    inc, (status, p0, root, bounds) = empty_domain.mip_presolve_root([('GURO_PAR_PRELIMIT', 0.0)])
    assert inc is None and status == 0 and p0 is not None and bounds == ([2.0], [1.0])
    assert root[:2] == (1, 3) and root[2] != root[2] and root[3] == 0
    for typ in ['C', 'S', 'N']:
        model = _core.build_model([1.0], [], [0.0], [1.0], typ)
        with pytest.raises(NotImplementedError, match='linear MIP'):
            model.mip_presolve_root([])


def test_original_pipeline_does_not_retry_heuristic_after_presolve():
    params = [('Threads', 1.0)]
    raw = _core.read_model(str(DATA / 'neos-4338804-snowy.mps.gz'))
    inc, (status, p0, _, bounds) = raw.mip_presolve_root(params)
    assert inc is None and status == 0
    assert p0.mip_initial_incumbent(params) is not None
    assert bounds == p0.mip_root_bounds(params + [('Heuristics', 0.0)])
    assert bounds != p0.mip_root_bounds(params)


def test_mad_finds_and_applies_an_incumbent_without_an_external_solution():
    params = [('Threads', 1.0)]
    model = _core.read_model(str(DATA / 'mad.mps.gz')).presolve(params)
    before = (model.lb, model.ub, model.vtype)
    objective, x = model.mip_initial_incumbent(params)
    assert len(x) == len(model.obj)
    assert objective < 7.0
    assert objective == pytest.approx(model.objcon + sum(c * v for c, v in zip(model.obj, x)))
    assert all(lo - 1e-6 <= v <= hi + 1e-6 for lo, hi, v in zip(model.lb, model.ub, x))
    assert all(t == 'C' or abs(v - round(v)) <= 1e-5 for t, v in zip(model.vtype, x))
    activity = [0.0] * len(model.rhs)
    for col, v in zip(model.cols, x):
        for i, a in col:
            activity[i] += a * v
    for a, rhs, sense in zip(activity, model.rhs, model.row_sense):
        violation = a - rhs if sense == '<' else rhs - a if sense == '>' else abs(a - rhs)
        assert violation <= 1e-6
    assert model.mip_root_bounds(params) == model.mip_root_bounds(params, x)
    ub = model.mip_root_bounds(params)[1]
    assert all(ub[j] < before[1][j] for j in range(200, 220))
    assert model.mip_root(params)[3] == 34
    assert model.mip_initial_incumbent(params) == (objective, x)
    assert model.mip_initial_incumbent(params + [('Heuristics', 0.0)]) is None
    assert model.mip_initial_incumbent(params + [('GURO_PAR_PREPREHEUR', 0.0)]) is None
    assert (model.lb, model.ub, model.vtype) == before


@pytest.mark.parametrize("lo, hi, cost, objective", [(0.0, 2.0, -1.0, -1.0), (-1.0, 1.0, 1.0, 0.0)])
def test_raw_binary_type_domain_applies_to_bounds_incumbents_and_root(lo, hi, cost, objective):
    model = _core.build_model([cost], [], [lo], [hi], 'B')
    for params in [[], [('Heuristics', 0.0)]]:
        for invalid in [-1.0, 2.0]:
            with pytest.raises(ValueError):
                model.mip_root_bounds(params, [invalid])
            with pytest.raises(ValueError):
                model.mip_root(params, [invalid])
        lb, ub = model.mip_root_bounds(params)
        assert 0.0 <= lb[0] <= ub[0] <= 1.0
        assert model.mip_root(params)[1:3] == (2, objective)
    assert (model.lb, model.ub) == ([lo], [hi])
    integer = _core.build_model([1.0], [], [0.0], [2.0], 'I')
    assert integer.mip_root_bounds([], [2.0]) == ([0.0], [1.0])
    assert integer.mip_root_bounds([('GURO_PAR_OBJGCD', 0.0)], [2.0]) == ([0.0], [2.0])


def test_binary_empty_intersection_is_reported_before_the_root_lp():
    import math
    model = _core.build_model([1.0], [], [2.0], [3.0], 'B')
    for params in [[], [('Heuristics', 0.0)]]:
        with pytest.raises(ValueError, match="empty column domain"):
            model.mip_root_bounds(params)
        with pytest.raises(ValueError):
            model.mip_root(params, [2.0])
        method, status, objective, iterations = model.mip_root(params)
        assert (method, status, iterations) == (1, 3, 0.0)
        assert math.isnan(objective)
        assert model.mip_initial_incumbent(params) is None
    assert (model.lb, model.ub) == ([2.0], [3.0])


@pytest.mark.parametrize("lo, hi, endpoint, invalid", [
    (1.0 + 0.5e-6, 2.0, 1.0, 1.0 - 0.75e-6),
    (-1.0, -0.5e-6, 0.0, 0.75e-6),
])
def test_binary_endpoint_within_bound_tolerance_remains_feasible(lo, hi, endpoint, invalid):
    model = _core.build_model([1.0], [], [lo], [hi], 'B')
    for params in [[], [('Heuristics', 0.0)]]:
        assert model.mip_root_bounds(params) == ([endpoint], [endpoint])
        assert model.mip_root_bounds(params, [endpoint]) == ([endpoint], [endpoint])
        assert model.mip_root(params)[1:3] == (2, endpoint)
        with pytest.raises(ValueError):
            model.mip_root(params, [invalid])
    assert model.mip_initial_incumbent([]) == (endpoint, [endpoint])
    assert (model.lb, model.ub) == ([lo], [hi])


def test_automatic_incumbent_does_not_use_relaxed_implied_integer_bounds():
    model = _core.build_model([1.0, 0.0], [([(0, 1.0), (1, -1.0)], 0.0, 0.0)],
                              [0.0, 1.0], [0.999995, 1.0], 'CB')
    assert model.mip_initial_incumbent([]) is None
    assert model.mip_root([]) is not None
    with pytest.raises(ValueError):
        model.mip_root([], [1.0, 1.0])
    assert model.ub == [0.999995, 1.0]


def test_progress_distinguishes_fractional_root_from_mip_completion():
    fractional = _core.build_model([1.0, 1.0], [([(0, 2.0), (1, 2.0)], 1.0, 1e30)], [0.0]*2, [1.0]*2, 'BB')
    status, bound, objective, x, root, bounds, complete = fractional.mip_root_progress([('Heuristics', 0.0)])
    assert status == 1 and bound == 1.0 and objective is None and x is None
    assert root[1:3] == (2, 0.5) and complete
    # Submitting a feasible incumbent closes the lattice gap without confusing the LP objective with the bound.
    result = fractional.mip_root_progress([('Heuristics', 0.0)], [1.0, 0.0])
    assert result[0:3] == (2, 1.0, 1.0)
    assert result[3] == [1.0, 0.0]


def test_progress_propagates_objective_through_a_chain_and_preserves_model():
    model = _core.build_model([1.0, 0.0, 0.0], [
        ([(0, 1.0), (1, -1.0)], 0.0, 0.0),
        ([(1, 1.0), (2, -1.0)], 0.0, 0.0),
    ], [0.0]*3, [10.0]*3, 'III')
    before = model.to_mps()
    result = model.mip_root_progress([('Cutoff', 3.0), ('Heuristics', 0.0)])
    assert result[0:4] == (2, 0.0, 0.0, [0.0]*3)
    assert result[5][1] == [3.0]*3 and result[6]
    assert model.to_mps() == before


def test_progress_cutoff_conflict_and_original_infeasibility():
    model = _core.build_model([1.0], [], [2.0], [10.0], 'I')
    result = model.mip_root_progress([('Cutoff', 1.0), ('Heuristics', 0.0)])
    assert result[0] == 6 and result[2] is None and result[4] is None
    # A feasible supplied point outside Cutoff must not become an accepted incumbent.
    assert model.mip_root_progress([('Cutoff', 1.0)], [2.0])[0] == 6
    impossible = _core.build_model([0.0], [([(0, 1.0)], 2.0, 1e30)], [0.0], [1.0], 'B')
    assert impossible.mip_root_progress([('Heuristics', 0.0)])[0] == 3
    assert _core.build_model([], [], [], [], '').mip_root_progress([])[0:4] == (2, 0.0, 0.0, [])
    empty = _core.build_model([], [([], 1.0, 1e30)], [], [], '')
    assert empty.mip_root_progress([])[0] == 3
    unbounded = _core.build_model([-1.0], [], [0.0], [1e30], 'I')
    result = unbounded.mip_root_progress([('Heuristics', 0.0)])
    assert result[0] == 1 and result[4][1] == 5  # LP unbounded alone leaves the MIP unresolved


@pytest.mark.parametrize('limit,code', [('TimeLimit', 9), ('IterationLimit', 7), ('SolutionLimit', 10)])
def test_progress_limits_do_not_report_false_optimality(limit, code):
    model = _core.build_model([1.0], [], [0.0], [2.0], 'I')
    result = model.mip_root_progress([(limit, 0.0), ('Heuristics', 0.0)])
    assert result[0] == code and result[4] is None


def test_original_progress_keeps_original_incumbent_and_p0_separate():
    raw = _core.read_model(str(DATA / 'mad.mps.gz'))
    params = [('Threads', 1.0)]
    incumbent, (status, p0, result) = raw.mip_presolve_progress(params)
    assert status == 0 and p0.to_mps() == raw.presolve(params).to_mps()
    assert incumbent[0] == pytest.approx(6.964) and len(incumbent[1]) == len(raw.obj)
    assert result[0] in (1, 2) and result[2] <= incumbent[0]
    if result[3] is not None:
        assert len(result[3]) == len(p0.obj)  # new root solutions stay in P0 coordinates
    impossible = _core.build_model([0.0], [], [2.5], [2.5], 'I')
    assert impossible.mip_presolve_progress([]) == (None, (3, None, None))
