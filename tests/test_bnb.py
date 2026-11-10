"""End-to-end B&B checks against exhaustive enumeration of small bounded MIPs."""
import itertools
import random

import pytest
from rsolver import _core


def binary_cases():
    rng = random.Random(62026)
    for case in range(200):
        n = 4
        obj = [rng.randrange(-4, 5) for _ in range(n)]
        rows = []
        for _ in range(rng.randrange(1, 6)):
            coeff = [rng.randrange(-3, 4) for _ in range(n)]
            rhs = rng.randrange(-4, 6)
            rows.append((coeff, rhs))
        yield case, obj, rows


@pytest.mark.parametrize('case,obj,rows', list(binary_cases()), ids=lambda x: str(x) if isinstance(x, int) else None)
@pytest.mark.parametrize('presolve', [False, True])
def test_binary_search_matches_enumeration(case, obj, rows, presolve):
    feasible = [x for x in itertools.product((0, 1), repeat=len(obj))
                if all(sum(a * b for a, b in zip(coeff, x)) <= rhs for coeff, rhs in rows)]
    expected = min((sum(a * b for a, b in zip(obj, x)) for x in feasible), default=None)
    core_rows = [([(j, a) for j, a in enumerate(coeff) if a], -1e30, rhs) for coeff, rhs in rows]
    model = _core.build_model(obj, core_rows, [0.0] * len(obj), [1.0] * len(obj), 'B' * len(obj))
    params = [('Heuristics', 0), ('MIPGap', 0), ('MIPGapAbs', 0)]
    status, bound, objective, x, _, _ = (model.mip_solve_original(params) if presolve else model.mip_solve(params))
    assert status == (2 if feasible else 3), (case, obj, rows, status)
    if feasible:
        assert objective == pytest.approx(expected), (case, obj, rows)
        assert bound == pytest.approx(expected)
        assert all(abs(v - round(v)) <= 1e-5 for v in x)
        assert all(sum(a * b for a, b in zip(coeff, x)) <= rhs + 1e-6 for coeff, rhs in rows)
    else:
        assert objective is None and x is None


def test_cutoff_accepts_a_solution_at_the_boundary():
    rows = [([(i, 1.0), (j, 1.0)], -1e30, 1.0) for i, j in [(0, 1), (1, 2), (0, 2)]]
    model = _core.build_model([-1.0] * 3, rows, [0.0] * 3, [1.0] * 3, 'BBB')
    params = [('Heuristics', 0), ('GURO_PAR_OBJGCD', 0)]
    assert model.mip_solve(params + [('Cutoff', -2)])[0] == 6
    status, bound, objective, x, _, _ = model.mip_solve(params + [('Cutoff', -1)])
    assert (status, bound, objective) == (2, -1.0, -1.0)
    assert sum(x) == pytest.approx(1.0)


def test_optimize_restores_original_variables_after_presolve():
    import rsolver
    m = rsolver.Model()
    fixed = m.addVar(lb=2, ub=2, vtype=rsolver.GRB.INTEGER)
    x = [m.addVar(vtype=rsolver.GRB.BINARY) for _ in range(3)]
    for i, j in [(0, 1), (1, 2), (0, 2)]:
        m.addConstr(x[i] + x[j] <= 1)
    m.setObjective(3 * fixed - rsolver.quicksum(x))
    m.optimize()
    assert m.Status == rsolver.GRB.OPTIMAL
    assert m.ObjVal == pytest.approx(5)
    assert fixed.X == pytest.approx(2)
    assert sum(v.X for v in x) == pytest.approx(1)


def test_expired_time_limit_and_presolve_path():
    model = _core.build_model([-1.0] * 3,
        [([(i, 1.0), (j, 1.0)], -1e30, 1.0) for i, j in [(0, 1), (1, 2), (0, 2)]],
        [0.0] * 3, [1.0] * 3, 'BBB')
    result, path = model.mip_solve_diagnostics([('TimeLimit', 0), ('Presolve', 0)])
    assert result[0] == 9
    assert path == 'original'


def test_restore_candidate_validates_input_and_returns_original_solution():
    model = _core.build_model([3.0], [], [2.0], [2.0], 'I')
    x, objective = model.mip_restore_candidate([], [])
    assert x == pytest.approx([2.0])
    assert objective == pytest.approx(6.0)
    with pytest.raises(ValueError, match='wrong reduced solution length'):
        model.mip_restore_candidate([], [0.0])


def test_time_limit_interrupts_long_root_lp():
    import os
    from pathlib import Path
    import subprocess
    import sys

    path = Path(__file__).resolve().parents[2] / 're/bench/data/miplib/cod105.mps.gz'
    if not path.exists():
        pytest.skip('external MIPLIB fixture unavailable')
    code = '''from rsolver import _core
import sys
m = _core.read_model(sys.argv[1])
r = m.mip_solve([("Presolve", 0), ("Heuristics", 0), ("TimeLimit", 0.25)])
assert r[0] == 9 and r[5] > 0, r
'''
    subprocess.run([sys.executable, '-c', code, str(path)], check=True, timeout=6, env=os.environ.copy())
