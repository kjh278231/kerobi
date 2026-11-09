"""Check follow-up MIP root progress on MIPLIB; requires no Gurobi.

PYTHONPATH=python ../.venv/bin/python tools/mip_progress_check.py [name ...]
The original incumbent and P0 solutions have distinct coordinate systems.
"""
import csv
import math
import sys
from collections import Counter
from pathlib import Path

from rsolver import _core

DATA = Path(__file__).resolve().parents[2] / 're/bench/data'


def validate(model, objective, x):
    assert len(x) == len(model.obj)
    assert all(math.isfinite(v) for v in x)
    for lo, hi, typ, v in zip(model.lb, model.ub, model.vtype, x):
        if typ == 'B':
            lo, hi = max(0.0, lo), min(1.0, hi)
        assert lo - 1e-6 <= v <= hi + 1e-6
        assert typ == 'C' or abs(v - round(v)) <= 1e-5
    activity = [0.0] * len(model.rhs)
    for v, col in zip(x, model.cols):
        for row, coefficient in col:
            activity[row] += coefficient * v
    for a, rhs, sense in zip(activity, model.rhs, model.row_sense):
        violation = a - rhs if sense == '<' else rhs - a if sense == '>' else abs(a - rhs)
        assert violation <= 1e-6
    expected = model.objcon + sum(c * v for c, v in zip(model.obj, x))
    assert math.isclose(expected, objective, rel_tol=1e-12, abs_tol=1e-8)


def main():
    with (DATA / 'miplib_small.csv').open() as f:
        cases = list(csv.DictReader(f))
    if sys.argv[1:]:
        names = set(sys.argv[1:])
        cases = [case for case in cases if case['name'] in names]
        assert len(cases) == len(names), 'unknown case'
    counts = Counter()
    for case in cases:
        name = case['name']
        raw = _core.read_model(str(DATA / 'miplib' / f'{name}.mps.gz'))
        params = [('Threads', 1.0)]
        original, (pre_status, p0, progress) = raw.mip_presolve_progress(params)
        assert pre_status == 0, (name, pre_status)
        assert p0.to_mps() == raw.presolve(params).to_mps(), name
        status, bound, objective, x, _, _, complete = progress
        assert status in (1, 2, 3, 6), (name, status)
        if original is not None:
            validate(raw, *original)
        if x is not None:
            validate(p0, objective, x)
        if objective is not None:
            assert p0.sense * bound <= p0.sense * objective + 1e-6 * max(1.0, abs(objective)), name
        if case['best_obj'] == 'Infeasible':
            assert objective is None and original is None, name
        elif case['best_obj']:
            best = raw.sense * float(case['best_obj'])
            assert p0.sense * bound <= best + 1e-6 * max(1.0, abs(best)), (name, bound, best)
        counts[status] += 1
        print(f'{name}: status={status} bound={bound:.12g} incumbent={objective} complete={complete}')
    print(f'{len(cases)} checked; statuses={dict(counts)}')


if __name__ == '__main__':
    main()
