#!/usr/bin/env python3
"""M2-06h-2 first-candidate control: original inputs -> Rust -> original commands/results."""
import argparse
import collections
import copy
import gzip
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile

import mip_sb_snapshot_check as S
from mip_sb_restore_check import BINARY, bits, value, same, sha, rules

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / 'bench/oracle/m2_06h2/capture'
WORK = ROOT / 'target/mip_sb_control'
SOURCE = 'crates/mip/src/strong_branch.rs'
TEST = 'crates/mip/tests/strong_branch_l1.rs'
OLD = ROOT / 'bench/oracle/m2_06h1/verified/1.json.gz'
TOOLS = ['tools/mip_sb_control.js', 'tools/mip_sb_snapshot.js', 'tools/mip_sb_control_runs.sh',
         'tools/mip_sb_control_rust.py', 'tools/mip_sb_snapshot_check.py', 'tools/mip_sb_restore_check.py', SOURCE, TEST]


def load(p):
    with gzip.open(p) as f:
        return json.load(f)


def project(d):
    d = copy.deepcopy(d)
    for e in d['events']:
        e.pop('types', None)
        if 'steps' in e:
            e['steps'] = [s for s in e['steps'] if s['kind'] not in ('input', 'reset')]
    return d


def controls(ds, manifest):
    for d in ds:
        same(BINARY, d['binary_sha256'], 'binary')
        same(manifest['model'], d['model_sha256'], 'model')
        same(sha(ROOT/'tools/mip_sb_control.js'), d['tool_sha256'], 'tool')
        same('mip_sb_control', d['kind'], 'kind')
        same(True, d['ok'], 'ok')
        same([], d['errors'], 'errors')
        same(dict(Threads=1, Seed=0, LogToConsole=0, NodeLimit=300, GURO_PAR_SBMAXITERS=10), d['params'], 'params')
    same([], ds[3]['events'], 'control.events')
    same(ds[0]['output'], ds[3]['output'], 'control.output')
    for d in ds[1:3]:
        same(ds[0], d, 'repeat')
    d = project(ds[0])
    S.acceptance(d)
    rules(S.projection(d), 'pk1_iters10')
    old = load(OLD)
    for k in ('events', 'output', 'counts', 'params', 'model_sha256', 'binary_sha256'):
        same(old[k], d[k], 'projection.'+k)


def first_candidate(d):
    """Inputs come from solution/bounds/LP returns; expected commands never feed Rust."""
    inputs, traces = [], []
    totals = collections.Counter()
    for e in d['events']:
        if e['e'] != 'double':
            continue
        at = f'double[{e["id"]}]'
        steps = e['steps']
        ins = [s for s in steps if s['kind'] == 'input']
        same(1, len(ins), at+'.input.count')
        inp = ins[0]
        same(0, inp['rc'], at+'.input.rc')
        n = e['bounds0'][0]
        same([n, n, 2*n], [len(inp['x']), len(inp['scales']), len(inp['flips'])], at+'.input.length')
        j = e['j'][0]
        assert 0 <= j < n and e['stride'] == 1, at
        same(n, len(bytes.fromhex(e['types'])), at+'.types.length')
        assert bytes.fromhex(e['types'])[j] in (66, 73), at+'.noninteger type'
        assert bytes.fromhex(inp['flips'])[j] == 0, at+'.uncovered flip'
        scale = value(inp['scales'][j])
        assert scale > 0, at+'.scale'
        # Inverse of the original model-bound transform, before any request is observed.
        lb, ub = [struct.unpack('<d', bytes.fromhex(row)[j*8:j*8+8])[0] * scale for row in e['bounds0'][1:]]
        base = next(s['value'] for s in steps if s['kind']=='attr' and s['attr']=='ObjVal')
        args = [str(e['id']), str(j), inp['x'][j], bits(lb), bits(ub), inp['tol'], base]
        expected = [f'case {e["id"]}']
        begin = next(i for i,s in enumerate(steps) if s['kind']=='boundwrite')
        restores = 0
        consumed = None
        # End immediately after the second restoration request, before any next candidate.
        for si in range(begin, len(steps)):
            s = steps[si]
            k = s['kind']
            if k == 'boundwrite':
                same(0, s['rc'], at+'.bound.rc')
                same([j], s['j'], at+'.additional propagation')
                expected.append(f'bound {j} {bytes.fromhex(s["dir"])[0]} {s["val"][0]}')
                if restores == 2:
                    break
            elif k in ('flush', 'reset'):
                same(0, s['rc'], at+'.'+k+'.rc')
                expected.append(k)
            elif k == 'solve':
                same(0, s['rc'], at+'.solve.rc')
                assert s['status'] in (2, 7), at+'.uncovered status'
                totals[f'status{s["status"]}'] += 1
                expected.append('solve')
                following = steps[si+1:]
                it = following[0]
                same(['attr', 'IterCount', 0], [it['kind'], it['attr'], it['rc']], at+'.iters')
                if s['status'] == 2:
                    obj = following[1]
                    same(['attr', 'ObjVal', 0], [obj['kind'], obj['attr'], obj['rc']], at+'.obj')
                    v = obj['value']
                else:
                    v = s['dual']
                args += [str(s['status']), v, it['value']]
            elif k == 'attr':
                pass
            elif k == 'restore':
                same([j, 62 if restores == 0 else 60], s['branch'], at+'.branch')
                expected.append('restore')
                restores += 1
                consumed = s['result']
            else:
                raise AssertionError(at+'.unexpected '+k)
        same(2, restores, at+'.restores')
        same(13, len(args), at+'.args')
        args.append(e['cost0'])
        # Independent sum of original IterCount attributes (not Rust-generated).
        cost = bits(value(e['cost0']) + value(args[9]) + value(args[12]))
        if e['n'] == 1:
            same(e['cost1'], cost, at+'.batch cost')  # one-candidate batch: original total
        totals['cost1'] += e['n'] == 1
        expected.append(f'result {consumed[0]} {consumed[1]} {cost}')
        for x in args[2:7] + [args[8], args[9], args[11], args[12]]:
            bits(value(x))  # reject IEEE NaN/inf rather than accepting both
        inputs.append(' '.join(args))
        traces.extend(expected)
        totals['candidates'] += 1
    same(24, totals['candidates'], 'first candidate count')
    same([6, 42], [totals['status2'], totals['status7']], 'branch coverage')
    return '\n'.join(inputs)+'\n', '\n'.join(traces)+'\n', dict(totals)


def synthetic(root, label):
    env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT/'target/mip_sb_control_mutants'))
    r = subprocess.run(['cargo', 'test', '--release', '-p', 'rsolver-mip', '--test', 'strong_branch_l1'],
                       cwd=root, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    (WORK/(label+'.synthetic.log')).write_text(r.stdout)
    return r.returncode


def run(root, input_text, label):
    WORK.mkdir(parents=True, exist_ok=True)
    ip, op = WORK/(label+'.input'), WORK/(label+'.trace')
    ip.write_text(input_text)
    if op.exists():
        op.unlink()
    env = dict(os.environ, SB_CONTROL_INPUT=str(ip), SB_CONTROL_OUTPUT=str(op))
    if root != ROOT:
        env['CARGO_TARGET_DIR'] = str(ROOT/'target/mip_sb_control_mutants')
    result = subprocess.run(['cargo', 'test', '--release', '-p', 'rsolver-mip', '--test',
                             'strong_branch_l1', '--', '--ignored'], cwd=root, env=env,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    (WORK/(label+'.log')).write_text(result.stdout)
    assert result.returncode == 0, f'{label}: cargo failed; {WORK/(label+".log")}'
    return op.read_text()


def compare(expected, actual, at):
    same(expected.splitlines(), actual.splitlines(), at)


def self_test(ds, manifest, expected, actual, inputs):
    def reject(name, fn):
        try:
            fn()
        except (AssertionError, KeyError, StopIteration):
            print('REJECT', name)
        else:
            raise AssertionError('NOT REJECTED '+name)
    def input_field(d, field, v):
        s = next(s for e in d['events'] if e['e']=='double' for s in e['steps'] if s['kind']=='input')
        s[field] = v
    def change_x(d):
        e = next(e for e in d['events'] if e['e']=='double')
        s = next(s for s in e['steps'] if s['kind']=='input')
        s['x'][e['j'][0]] = bits(2.0)
    tampers = [
        ('binary', lambda d:d.__setitem__('binary_sha256', '0'*64)),
        ('model', lambda d:d.__setitem__('model_sha256', '0'*64)),
        ('empty', lambda d:d.__setitem__('events', [])),
        ('solution', change_x),
        ('tolerance', lambda d:input_field(d, 'tol', bits(1.0))),
        ('null input', lambda d:input_field(d, 'x', [])),
        ('no target', lambda d:d.__setitem__('events', [e for e in d['events'] if e['e']!='double'])),
        ('missing input', lambda d:next(e for e in d['events'] if e['e']=='double').__setitem__('steps',
             [s for s in next(e for e in d['events'] if e['e']=='double')['steps'] if s['kind']!='input'])),
        ('missing reset', lambda d:next(e for e in d['events'] if e['e']=='double').__setitem__('steps',
             [s for s in next(e for e in d['events'] if e['e']=='double')['steps'] if s['kind']!='reset'])),
    ]
    for name, mutate in tampers:
        bad = copy.deepcopy(ds)
        for d in bad[:3]:
            mutate(d)
        def check_bad():
            controls(bad, manifest)
            ip, ex, _ = first_candidate(bad[0])
            compare(ex, actual if ip == inputs else run(ROOT, ip, 'tamper-'+name.replace(' ', '-')), name)
        reject(name, check_bad)
    # Direct hits on this tool's own count and batch-cost checks (controls() would reject first).
    def drop_one(d):
        i = next(i for i,e in enumerate(d['events']) if e['e']=='double')
        del d['events'][i]
    def batch_cost(d):
        next(e for e in d['events'] if e['e']=='double' and e['n']==1)['cost1'] = bits(1e9)
    for name, mutate in [('count', drop_one), ('batch cost', batch_cost)]:
        bad = copy.deepcopy(ds[0])
        mutate(bad)
        reject(name, lambda bad=bad:first_candidate(bad))
    for name, old, new in [('order', 'flush\nsolve', 'solve\nflush'),
                           ('restore', 'restore\n', ''), ('bound', 'bound ', 'wrong '),
                           ('result', 'result ', 'wrong '), ('cost', actual.splitlines()[-1], 'result 0 0 0')]:
        assert old in actual
        reject('trace '+name, lambda old=old,new=new:compare(expected, actual.replace(old,new,1), name))
    mutants = [
        ('reverse', '(0, Side::Lower, split + 1.0, c.lower),\n        (1, Side::Upper, split, c.upper)',
         '(1, Side::Upper, split, c.upper),\n        (0, Side::Lower, split + 1.0, c.lower)'),
        ('cost', '*iterations += result.iterations;', '*iterations = result.iterations;'),
        ('restore', 'lp.restore().map_err(Error::Backend)?;', ''),
        ('split', 'split + 1.0', 'split + 2.0'),
        ('clamp2', 'if obj > c.parent_obj { obj } else { c.parent_obj }', 'obj'),
        ('clamp7', 'if c.parent_obj > dual { c.parent_obj } else { dual }', 'dual'),
    ]
    with tempfile.TemporaryDirectory(prefix='sb-control-') as td:
        root = Path(td)
        shutil.copytree(ROOT/'crates', root/'crates')
        for p in ('Cargo.toml', 'Cargo.lock'):
            shutil.copy2(ROOT/p, root/p)
        original = (ROOT/SOURCE).read_text()
        for name, old, new in mutants:
            assert old in original, name
            (root/SOURCE).write_text(original.replace(old,new,1))
            trace = run(root, inputs, 'mutant-'+name)
            if trace == expected:
                # Original data has no branch for it; report whether the synthetic tests catch it.
                caught = synthetic(root, 'mutant-'+name) != 0
                print('SURVIVED', name, '(original unverified; synthetic', 'rejects)' if caught else 'misses)')
                assert name in ('clamp2', 'clamp7') and caught, 'unexpected surviving mutant '+name
            else:
                print('REJECT Rust', name)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--self-test', action='store_true')
    ap.add_argument('--write-manifest', action='store_true')
    a = ap.parse_args()
    files = [DATA/f'{i}.json.gz' for i in (1,2,3)] + [DATA/'control.json.gz']
    ds = [load(p) for p in files]
    if a.write_manifest:
        paths = files + [ROOT/p for p in TOOLS] + [OLD]
        manifest = dict(binary=BINARY, model=ds[0]['model_sha256'],
                        files={str(p.relative_to(ROOT)):sha(p) for p in paths})
        (DATA/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
    manifest = json.loads((DATA/'manifest.json').read_text())
    same(BINARY, manifest['binary'], 'manifest.binary')
    for p,h in manifest['files'].items():
        same(h, sha(ROOT/p), 'manifest.'+p)
    controls(ds, manifest)
    inputs, expected, totals = first_candidate(ds[0])
    actual = run(ROOT, inputs, 'original')
    compare(expected, actual, 'Rust')
    if a.self_test:
        self_test(ds, manifest, expected, actual, inputs)
    print('PASS', totals)


if __name__ == '__main__':
    main()
