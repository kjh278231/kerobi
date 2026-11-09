"""Replay a captured root LP with its column types, without Gurobi.

usage: PYTHONPATH=python python tools/mip_root_replay.py <capture directory> [--k 0]
       [--presolved <original MIP.mps.gz>]
Capture with SPX_K=<k> tools/gurun.sh node tools/mip_root_trace.js <model> <directory>.
RSOLVER_SNAP=<file> enables the existing per-iteration workspace dump.
"""
import argparse
import json
from pathlib import Path

from rsolver import _core


def rebuild(model, vtype, lb=None, ub=None):
    """Keep the captured matrix and row order while restoring MIP column types."""
    rows = [[] for _ in model.rhs]
    for j, col in enumerate(model.cols):
        for i, value in col:
            rows[i].append((j, value))
    constraints = [
        (row, rhs if sense != '<' else -1e30, rhs if sense != '>' else 1e30)
        for row, sense, rhs in zip(rows, model.row_sense, model.rhs)
    ]
    result = _core.build_model([model.sense * value for value in model.obj], constraints,
                               model.lb if lb is None else lb,
                               model.ub if ub is None else ub, vtype)
    assert result.cols == model.cols and result.rhs == model.rhs
    assert result.row_sense == model.row_sense
    return result


def replay(directory, k=0):
    path = directory / f'{k}.mps'
    model = _core.read_model(str(path if path.exists() else path.with_suffix('.mps.gz')))
    setup = json.loads((directory / f'spx_{k}.json').read_text())
    assert (setup['n'], setup['m']) == (len(model.obj), len(model.rhs))
    vtype = bytes(setup['vtype'][:setup['n']]).decode('ascii')
    reference = next(
        event for line in (directory / 'events.jsonl').read_text().splitlines()
        if (event := json.loads(line)).get('k') == k
        and event['fn'] == 'cda5b' and event['ev'] == '<'
    )
    typed = rebuild(model, vtype)
    status, obj, _, iterations, *_ = typed.solve(1, 1e18, False)
    # build_model uses minimization and zero objective constant; restore the captured objective convention.
    obj = model.sense * obj + model.objcon
    assert status == reference['status'], (status, reference['status'])
    assert f'{obj:.6e}' == f"{reference['obj']:.6e}", (obj, reference['obj'])
    return model, typed, reference, iterations


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('directory', type=Path)
    ap.add_argument('--k', type=int, default=0)
    ap.add_argument('--presolved', type=Path)
    args = ap.parse_args()
    model, typed, reference, iterations = replay(args.directory, args.k)
    print(f"captured bounds: iterations {iterations:g}, Gurobi {reference['iter']:g}")
    if args.presolved:
        pre = _core.read_model(str(args.presolved)).presolve([('Threads', 1.0)])
        assert (pre.cols, pre.obj, pre.rhs, pre.row_sense) == (model.cols, model.obj, model.rhs, model.row_sense)
        changes = {key: [[j, a, b] for j, (a, b) in enumerate(zip(getattr(pre, key), getattr(model, key))) if a != b]
                   for key in ['lb', 'ub']}
        print(json.dumps(changes))
        result = rebuild(model, typed.vtype, pre.lb, pre.ub).solve(1, 1e18, False)
        print(f'presolved bounds: status {result[0]}, iterations {result[3]:g}')


if __name__ == '__main__':
    main()
