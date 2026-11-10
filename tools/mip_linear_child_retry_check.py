#!/usr/bin/env python3
"""M2-03c-1 revised acceptance (user decision 2026-11-10).

natural  : status-12 numeric retry in 5ce67b, from saved original captures (3 independent runs each).
synthetic: 5->12 reclass boundary, delegated to `mip_linear_child_accept.py controls` (inject_reclass).
status 3 : node-status-3 caller 682037 (limited-iteration fallback) and what follows for that node.
Exit 0 only if every required check passes. Unverified items are printed, never counted as passes.
"""
import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DIR = ROOT / 'bench/oracle/mip_linear_child_retry'
MANIFEST = ROOT / 'bench/oracle/mip_linear_child_retry_manifest.json'
BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
BASE = {'Threads': 1, 'Seed': 0, 'LogToConsole': 0}
STRESS = {'Presolve': 0, 'ScaleFlag': 0, 'Quad': 0, 'NumericFocus': 0, 'MarkowitzTol': 0.0001}
CASES = {
    'nidda_stress_retry': ('neos-3754480-nidda.mps.gz', dict(BASE, NodeLimit=6000, **STRESS), 'child'),
    'gmu-35-50_default_retry': ('gmu-35-50.mps.gz', dict(BASE, NodeLimit=20000), 'root'),
    'snowy_default_set3': ('neos-4338804-snowy.mps.gz', dict(BASE, NodeLimit=20000), 'set3'),
}
PARAM_KEYS = ('presolve', 'method', 'concurrent_method', 'threads')
SET3_RET = 0x566c48  # inside 682037 (file+0x566030), after its second limited-iteration cda5b call


def require(cond, msg):
    if not cond:
        raise ValueError(msg)


def load(name):
    return [json.loads((DIR / f'{name}_{r}.json').read_text()) for r in (1, 2, 3)]


def check_manifest():
    m = json.loads(MANIFEST.read_text())
    require(m['binary_sha256'] == BINARY, 'manifest binary mismatch')
    for path, digest in m['files'].items():
        require(hashlib.sha256((ROOT / path).read_bytes()).hexdigest() == digest, f'manifest hash mismatch: {path}')
    return m


def check_identity(runs, name, manifest):
    model, params, _ = CASES[name]
    want = manifest['external_models'][f're/bench/data/miplib/{model}']
    require(len(runs) == 3, f'{name}: need 3 runs')
    for i, r in enumerate(runs, 1):
        require(r['ok'] is True and not r['errors'], f'{name} run {i}: not ok')
        require(r['binary_sha256'] == BINARY, f'{name} run {i}: binary hash')
        require(r['model'] == model and r['model_sha256'] == want, f'{name} run {i}: model identity')
        require(r['params'] == params, f'{name} run {i}: params')
        require(r['events'], f'{name} run {i}: no events')
    for i in (1, 2):
        require(runs[i]['events'] == runs[0]['events'] and runs[i]['output'] == runs[0]['output']
                and runs[i].get('fallback_counts') == runs[0].get('fallback_counts'), f'{name}: run {i + 1} differs from run 1')


def params(lp):
    return {k: lp[k] for k in PARAM_KEYS}


def header(lp, name, where):
    # ws+0x168: one int per (row+col); basic entries hold their basis position (a permutation of 0..rows-1), nonbasic hold negative codes.
    v = lp['vstat']
    require(v is not None and len(v) == lp['ws_rows'] + lp['ws_cols'], f'{name}: basis header missing at {where}')
    require(sorted(x for x in v if x >= 0) == list(range(lp['ws_rows'])), f'{name}: basis header not a valid basis at {where}')
    return v


def check_retry(runs, name):
    kind = CASES[name][2]
    require(len(runs[0]['events']) == 1, f'{name}: expected exactly one retry solve')
    ev = runs[0]['events'][0]['events']
    names = [e['event'] for e in ev]
    require(names[:4] == ['solve:enter', 'normalize_before', 'normalize_after', 'retry'], f'{name}: prefix order {names[:4]}')
    require(names[-3:] == ['finish:enter', 'finish:leave', 'solve:leave'], f'{name}: suffix order {names[-3:]}')
    require(ev[1]['lp_status'] == 12 and ev[2]['lp_status'] == 12 and ev[3]['lp_status'] == 12, f'{name}: LP status 12 before retry')
    node = ev[3]['node']
    if kind == 'child':
        require(node['root'] is False and node['cache'] == 2, f'{name}: expected non-root node with root cache 2')
    else:
        require(node['root'] is True, f'{name}: expected root node')
    calls = [e for e in ev[4:-3] if e['event'].endswith(':enter')]
    require(calls, f'{name}: no LP call after retry entry')
    enter, leave = ev[0], ev[-1]
    require(params(enter['lp']) == params(leave['lp']), f'{name}: parameters not restored {params(enter["lp"])} -> {params(leave["lp"])}')
    require(leave['lp']['status'] != 12, f'{name}: retry left LP status 12')
    drivers = [e for e in ev if e['event'] == 'lpdriver:enter']
    if drivers:  # C: retry frees the workspace, then 0c3d0d with Presolve=1 (RELAXPREP<0), Threads=1
        require(drivers[0]['lp']['presolve'] == 1 and drivers[0]['lp']['threads'] == 1, f'{name}: lpdriver retry params')
        require(drivers[0]['lp']['ws_rows'] is None, f'{name}: workspace not freed before lpdriver')
    before, after = header(ev[3]['lp'], name, 'retry entry'), header(leave['lp'], name, 'solve leave')
    basis = [e for e in ev if e['event'] == 'simplex:enter' and e['vbasis'] is not None]
    for e in basis:
        require(len(e['vbasis']) == e['lp']['ws_cols'] and len(e['cbasis']) == e['lp']['ws_rows'], f'{name}: basis length')
        require(sum(1 for x in e['vbasis'] + e['cbasis'] if x == 0) == e['lp']['ws_rows'], f'{name}: retry input basis has wrong basic count')
    out = [e for e in ev if e['event'] == 'simplex:leave' and e.get('vbasis_out') is not None]
    return {'node': node, 'path': 'lpdriver' if drivers else 'simplex', 'final_lp_status': leave['lp']['status'],
            'final_node_status': leave['node']['status'], 'params': params(enter['lp']), 'basis_inputs': len(basis), 'basis_arrays_after': len(out),
            'basic_set_changes': sum((x >= 0) != (y >= 0) for x, y in zip(before, after)),
            'input_valid': [sum(1 for x in e['vbasis'] + e['cbasis'] if x == 0) == e['lp']['ws_rows'] for e in basis]}


def check_set3(runs):
    ev = runs[0]['events']
    sets = [i for i, e in enumerate(ev) if e['event'] == 'set3']
    require(sets, 'status 3: no 5ff754(node, 3) call')
    found = []
    for i in sets:
        e = ev[i]
        require(e['ret'] == SET3_RET, f'status 3: unexpected caller {hex(e["ret"])}')
        j = i
        while ev[j]['event'] != 'fallback:enter':
            j -= 1
        block = ev[j:i]
        sx = [x for x in block if x['event'] == 'fallback_simplex:leave']
        require(len(sx) == 2, f'status 3 node {e["node"]["id"]}: expected two limited LPs, got {len(sx)}')
        require(all(x['lp']['status'] != 2 for x in sx), f'status 3 node {e["node"]["id"]}: a limited LP was optimal')
        require(sx[1]['iteration_limit'] == 100.0, f'status 3 node {e["node"]["id"]}: second limit {sx[1]["iteration_limit"]}')
        leave = next(x for x in ev[i:] if x['event'] == 'fallback:leave')
        require(leave['node']['status'] == 3 and leave['node']['id'] == e['node']['id'], 'status 3: node status after fallback')
        branch = [x for x in ev if x['event'] == 'branch:enter' and x['node']['id'] == e['node']['id']]
        kids = [x['child'] for x in ev if x['event'] == 'create:leave' and x['branching_node'] == e['node']['id']]
        found.append({'node': e['node']['id'], 'depth': e['node']['depth'], 'limits': [x['iteration_limit'] for x in sx],
                      'lp_status': [x['lp']['status'] for x in sx], 'branched': len(branch), 'children_status': [k['status'] for k in kids]})
    return found, sum(e['event'] == 'set10' for e in ev)


def synthetic():
    r = subprocess.run([sys.executable, str(ROOT / 'tools/mip_linear_child_accept.py'), 'controls'], capture_output=True, text=True)
    require(r.returncode == 0, f'synthetic reclass controls failed: {r.stderr.strip()}')


def run_all(data, manifest):
    for name in CASES:
        check_identity(data[name], name, manifest)
    retry = {name: check_retry(data[name], name) for name in ('nidda_stress_retry', 'gmu-35-50_default_retry')}
    set3, set10 = check_set3(data['snowy_default_set3'])
    return retry, set3, set10


def selftest(data, manifest):
    def mut(name, f):
        d = copy.deepcopy(data)
        f(d)
        try:
            run_all(d, manifest)
        except (ValueError, KeyError, IndexError, StopIteration, TypeError) as e:
            print(f'mutation rejected {name}: {e}')
            return
        raise ValueError(f'mutation accepted: {name}')
    nid, gmu, sn = 'nidda_stress_retry', 'gmu-35-50_default_retry', 'snowy_default_set3'
    mut('zero_binary_all', lambda d: [r.__setitem__('binary_sha256', '0' * 64) for n in d for r in d[n]])
    mut('empty_events', lambda d: [r.__setitem__('events', []) for r in d[nid]])
    mut('param_not_restored', lambda d: [r['events'][0]['events'][-1]['lp'].__setitem__('presolve', 1) for r in d[gmu]])
    mut('retry_left_12', lambda d: [r['events'][0]['events'][-1]['lp'].__setitem__('status', 12) for r in d[nid]])
    mut('child_marked_root', lambda d: [r['events'][0]['events'][3]['node'].__setitem__('root', True) for r in d[nid]])
    mut('retry_removed', lambda d: [r['events'][0]['events'].pop(3) for r in d[nid]])
    mut('header_invalid', lambda d: [r['events'][0]['events'][-1]['lp']['vstat'].__setitem__(0, 9999) for r in d[gmu]])
    mut('workspace_not_freed', lambda d: [next(e for e in r['events'][0]['events'] if e['event'] == 'lpdriver:enter')['lp'].__setitem__('ws_rows', 110) for r in d[gmu]])
    mut('basis_basic_count', lambda d: [next(e for e in r['events'][0]['events'] if e['event'] == 'simplex:enter')['vbasis'].__setitem__(0, -1 if next(e for e in r['events'][0]['events'] if e['event'] == 'simplex:enter')['vbasis'][0] == 0 else 0) for r in d[nid]])
    mut('basis_truncated', lambda d: [next(e for e in r['events'][0]['events'] if e['event'] == 'simplex:enter')['vbasis'].pop() for r in d[nid]])
    mut('run_differs', lambda d: d[gmu][2]['events'][0]['events'][0]['node'].__setitem__('id', 999))
    mut('wrong_params', lambda d: [r['params'].__setitem__('NodeLimit', 1) for r in d[sn]])
    mut('set3_caller', lambda d: [next(e for e in r['events'] if e['event'] == 'set3').__setitem__('ret', 0) for r in d[sn]])
    mut('no_set3', lambda d: [r.__setitem__('events', [e for e in r['events'] if e['event'] != 'set3']) for r in d[sn]])


def main():
    manifest = check_manifest()
    data = {name: load(name) for name in CASES}
    retry, set3, set10 = run_all(data, manifest)
    synthetic()
    for name, r in retry.items():
        print(f'natural retry {name}: node {r["node"]["id"]} root={r["node"]["root"]} cache={r["node"]["cache"]}, path={r["path"]}, '
              f'final LP {r["final_lp_status"]}, node status {r["final_node_status"]}, params restored {r["params"]}, '
              f'basis inputs {r["basis_inputs"]} (basic count == rows: {r["input_valid"]}), '
              f'header valid before/after, basic-set changes {r["basic_set_changes"]}, 3/3 identical')
    for s in set3:
        print(f'status 3 via 682037@0x566c48: node {s["node"]} depth {s["depth"]}, limits {s["limits"]}, LP {s["lp_status"]}, '
              f'branched {s["branched"]}, children status {s["children_status"]}, 3/3 identical')
    print('synthetic 5->12 reclass: inject_reclass 3/3 (accept.py controls exit 0)')
    print(f'counts: natural retry=2 cases, synthetic reclass=1, status3 nodes={len(set3)}, natural status-10 setter={set10}')
    print('unverified: natural 5->12 (impossible in exact arithmetic, unobserved in scan), basis after retry in VBasis/CBasis form '
          '(simplex input buffers are unchanged on return; only the ws+0x168 header is captured, negative codes uninterpreted), '
          'source of the retry input basis (differs from the failing header; assumed node-saved basis, not read), factor/norm contents, 682037 entry predicates '
          '5eda00/115e51/ws+0x3f0 meaning, status 10 creation')
    if '--self-test' in sys.argv:
        selftest(data, manifest)
    return 0


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (ValueError, OSError, KeyError, IndexError, StopIteration, TypeError, json.JSONDecodeError) as exc:
        print(f'FAIL: {exc}', file=sys.stderr)
        sys.exit(1)
