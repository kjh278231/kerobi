"""Reproducible, isolated MIPLIB B&B comparison; run with the project Python.

Each solver gets TimeLimit seconds and one thread. A wall-clock watchdog also
covers model loading and operations that do not poll TimeLimit. Output is JSONL.
"""
import argparse
import csv
import json
import math
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT.parent / 're/bench/data'
STATUSES = {2: 'optimal', 3: 'infeasible', 4: 'inf_or_unbounded', 5: 'unbounded',
            7: 'iteration_limit', 8: 'node_limit', 9: 'time_limit', 10: 'solution_limit', 12: 'numeric'}


def worker(solver, name, limit):
    path = DATA / 'miplib' / f'{name}.mps.gz'
    started = time.monotonic()
    if solver == 'records':
        from collections import Counter
        from rsolver import _core
        model = _core.read_model(str(path))
        counts = Counter(r[0] for r in model.mip_post_records([('Threads', 1.0)]))
        print('RESULT ' + json.dumps(dict(record_types=dict(counts), elapsed=time.monotonic() - started)))
        return
    if solver == 'gurobi':
        import gurobipy as gp
        model = gp.read(str(path))
        model.Params.OutputFlag = 0
        model.Params.Threads = 1
        model.Params.TimeLimit = limit
        model.optimize()
        result = dict(status=STATUSES.get(model.Status, str(model.Status)),
                      objective=model.ObjVal if model.SolCount else None,
                      bound=model.ObjBound, nodes=model.NodeCount)
    else:
        from rsolver import _core
        from mip_progress_check import validate
        model = _core.read_model(str(path))
        (status, bound, objective, x, nodes, iterations), path = model.mip_solve_diagnostics(
            [('Threads', 1.0), ('TimeLimit', limit)])
        result = dict(status=STATUSES.get(status, str(status)), objective=objective,
                      bound=bound, nodes=nodes, iterations=iterations, path=path)
        if x is not None:
            try:
                validate(model, objective, x)
                result['solution_valid'] = True
            except AssertionError:
                result['solution_valid'] = False
    for key in ('objective', 'bound'):
        if result[key] is not None and not math.isfinite(result[key]):
            result[key] = None
    result['elapsed'] = time.monotonic() - started
    print('RESULT ' + json.dumps(result, allow_nan=False))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--limit', type=float, default=2)
    parser.add_argument('--watchdog', type=float, default=6)
    parser.add_argument('--output', type=Path, default=ROOT / 'bench/mip_bnb_baseline.jsonl')
    parser.add_argument('--records', action='store_true', help='rerun presolve record inventory only')
    parser.add_argument('--solver', choices=('gurobi', 'rsolver'), help='run only one solver')
    parser.add_argument('--worker', nargs=2, metavar=('SOLVER', 'NAME'))
    args = parser.parse_args()
    if args.worker:
        worker(*args.worker, args.limit)
        return
    assert args.limit > 0 and args.watchdog > args.limit
    cases = list(csv.DictReader((DATA / 'miplib_small.csv').open()))
    env = dict(os.environ, PYTHONPATH=str(ROOT / 'python'))
    if args.records and args.output == ROOT / 'bench/mip_bnb_baseline.jsonl':
        args.output = ROOT / 'bench/mip_bnb_records.jsonl'
    with args.output.open('w') as out:
        for case in cases:
            for solver in (('records',) if args.records else ((args.solver,) if args.solver else ('gurobi', 'rsolver'))):
                name = case['name']
                cmd = [sys.executable, __file__, '--worker', solver, name, '--limit', str(args.limit)]
                try:
                    p = subprocess.run(cmd, capture_output=True, text=True, timeout=args.watchdog, env=env)
                    lines = [line[7:] for line in p.stdout.splitlines() if line.startswith('RESULT ')]
                    if p.returncode == 0 and lines:
                        result = json.loads(lines[-1])
                    else:
                        result = dict(status='error', error=(p.stderr or p.stdout)[-1200:], returncode=p.returncode)
                except subprocess.TimeoutExpired:
                    result = dict(status='watchdog_timeout')
                result.update(name=name, solver=solver, limit=args.limit, watchdog=args.watchdog)
                out.write(json.dumps(result) + '\n')
                out.flush()
                print(name, solver, result.get('status', 'ok'), flush=True)


if __name__ == '__main__':
    main()
