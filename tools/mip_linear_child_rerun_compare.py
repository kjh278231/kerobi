#!/usr/bin/env python3
"""Compare a pre-merge re-run (tools/mip_linear_child_premerge.sh OUTDIR) with the committed M2-03c-1 captures."""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
O = ROOT / 'bench/oracle'


def load(p):
    return json.loads(Path(p).read_text())


def main(out):
    out = Path(out)
    rows, fails = [], 0

    def check(group, name, ok, note=''):
        nonlocal fails
        rows.append((group, name, 'PASS' if ok else 'FAIL', note))
        fails += not ok

    for model in ('fractional_branch', 'root_unbounded'):
        ref = load(O / f'mip_linear_child_trace/{model}_1.json')
        for r in (1, 2, 3):
            check('trace', f'{model}_{r}', load(out / f'trace/{model}_{r}.json') == ref)
    for mode in ('natural', 'inject_reclass'):
        ref = load(O / f'mip_linear_child_experiments/{mode}_1.json')
        for r in (1, 2, 3):
            check('experiment', f'{mode}_{r}', load(out / f'experiments/{mode}_{r}.json') == ref)
    ref = load(O / 'mip_linear_child_experiments/inject_branch_1.json')
    cut = lambda e: e[:next(i for i, x in enumerate(e) if x['event'] == 'inject_branch') + 1]
    for r in (1, 2, 3):
        j = load(out / f'experiments/inject_branch_{r}.json')
        check('experiment', f'inject_branch_{r}', j['ok'] is False and cut(j['events']) == cut(ref['events']), 'synthetic crash: prefix up to injection')
    for case in ('neos-3754480-nidda_stress', 'gmu-35-50_default'):
        ref = load(O / f'mip_linear_child_scan/repro/{case}_1.json')
        for r in (1, 2, 3):
            j = load(out / f'scan/{case}_{r}.json')
            check('scan', f'{case}_{r}', j['ok'] and j['counts'] == ref['counts'] and j['output'] == ref['output'])
    for r in (1, 2, 3):  # TimeLimit run: informational only, never counted as pass/fail
        j = load(out / f'scan/neos-4338804-snowy_default_{r}.json')
        rows.append(('scan', f'neos-4338804-snowy_default_{r}', 'INFO', f'TimeLimit run, set:3={j["counts"].get("set:3", 0)}, {j["output"]}'))
    for name in ('nidda_stress_retry', 'gmu-35-50_default_retry', 'snowy_default_set3'):
        ref = load(O / f'mip_linear_child_retry/{name}_1.json')
        for r in (1, 2, 3):
            j = load(out / f'retry/{name}_{r}.json')
            check('retry', f'{name}_{r}', all(j.get(k) == ref.get(k) for k in ('ok', 'output', 'params', 'model_sha256', 'binary_sha256', 'events', 'fallback_counts')))
    for g, n, s, note in rows:
        print(f'{s:4} {g:10} {n:36} {note}')
    print(f'compared {sum(s != "INFO" for _, _, s, _ in rows)}, failed {fails}, info {sum(s == "INFO" for _, _, s, _ in rows)}')
    return 1 if fails else 0


if __name__ == '__main__':
    try:
        sys.exit(main(sys.argv[1]))
    except (OSError, KeyError, StopIteration, json.JSONDecodeError, IndexError) as e:
        print(f'FAIL: {e}', file=sys.stderr)
        sys.exit(1)
