#!/usr/bin/env python3
"""M2-06h-3/06h-4/06h-5 strong-branching candidate loop: original backend responses -> Rust -> original command trace."""
import argparse
import collections
import copy
import gzip
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

from mip_sb_restore_check import BINARY, bits, value, same, sha

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / 'bench/oracle/m2_06h3/final'
# M2-06h-4: natural cases of branches the 06h-3 models never took; the manifest here covers both sets.
DATA4 = ROOT / 'bench/oracle/m2_06h4/final'
WORK = ROOT / 'target/mip_sb_loop'
SOURCE = 'crates/mip/src/strong_branch.rs'
TEST = 'crates/mip/tests/strong_branch_l1.rs'
# Smallest explore captures covering status 3/-1000, down >= 1e100, up prefill (markshare_4_0), dual < parent (kasai),
# timer expiry (ran14x18-disj-8, 5 of 94 batches; replayed since M2-06h-5).
NEW_MODELS = ['markshare_4_0', 'neos-3627168-kasai', 'ran14x18-disj-8']
OLD_MODELS = ['pk1', 'neos5', 'tr12-30']
MODELS = {**{m: DATA for m in OLD_MODELS}, **{m: DATA4 for m in NEW_MODELS}}
OLD_06H2 = ROOT / 'bench/oracle/m2_06h2/capture/1.json.gz'
OLD_06G2 = str(ROOT / 'bench/oracle/m2_06g2/final') + '/{}_iters10_1.json.gz'
TOOLS = ['tools/mip_sb_loop.js', 'tools/mip_sb_snapshot.js', 'tools/mip_sb_loop_runs.sh', 'tools/mip_sb_census.py',
         'tools/mip_sb_loop_rust.py', 'tools/mip_sb_restore_check.py', SOURCE, TEST]
PARAMS = dict(Threads=1, Seed=0, LogToConsole=0, NodeLimit=300, GURO_PAR_SBMAXITERS=10)
NEW_KINDS = {'origLower', 'origUpper', 'propEnter', 'prop', 'values', 'poll', 'timer', 'input', 'reset'}
NEG, INF = bits(-1e100), bits(1e100)


def load(p):
    with gzip.open(p) as f:
        return json.load(f)


def digest(s):
    """Same length + FNV-1a32 summary as tools/mip_sb_loop.js."""
    if s is None:
        return None
    h = 0x811c9dc5
    for ch in s.encode():
        h = ((h ^ ch) * 0x01000193) & 0xffffffff
    return f'{len(s)}:{h:x}'


def summarize(d):
    """Apply the capture's summaries to an older raw capture."""
    d = copy.deepcopy(d)
    for e in d['events']:
        for s in e.get('steps', []):
            for k in ('bounds', 'before', 'after'):
                if k in s:
                    s[k] = [s[k][0], digest(s[k][1]), digest(s[k][2])]
            for k in ('src', 'dst', 'factor', 'factorDst'):
                v = s.get(k)
                if isinstance(v, dict) and 'stat' in v:
                    for f in ('stat', 'flags', 'vec', 'basic'):
                        v[f] = digest(v[f])
                if isinstance(v, dict) and 'arrays' in v:
                    v['arrays'] = [digest(a) for a in v['arrays']]
    return d


def project(d, kinds=NEW_KINDS):
    d = copy.deepcopy(d)
    for e in d['events']:
        if e.get('e') in ('double', 'kernel'):
            for k in ('stall', 'work0', 'work1', 'types'):
                e.pop(k, None)
        if 'steps' in e:
            e['steps'] = [s for s in e['steps'] if s['kind'] not in kinds]
            for s in e['steps']:
                s.pop('w', None)
    return d


def kernel_view(d):
    """Fields both the 06g-2 tool and this capture record for the double kernel."""
    out = []
    for e in d['events']:
        if e.get('e') != 'double':
            continue
        steps = []
        for s in e['steps']:
            if s['kind'] == 'solve':
                steps.append(['solve', s['status'], s['dual'], s['before'], s['after']])
            elif s['kind'] == 'attr':
                steps.append(['attr', s['attr'], s['value']])
            elif s['kind'] in ('save', 'restore'):
                steps.append([s['kind'], s['src'], s['dst']])
        out.append([e[k] for k in ('n', 'j', 'stride', 'a', 'z', 'cost0', 'cost1', 'rc', 'bounds0', 'bounds1')] + [steps])
    return out


def controls(cases, manifest):
    for m, ds in cases.items():
        for d in ds:
            same(BINARY, d['binary_sha256'], m+'.binary')
            same(manifest['models'][m], d['model_sha256'], m+'.model')
            same(sha(ROOT/'tools/mip_sb_loop.js'), d['tool_sha256'], m+'.tool')
            same('mip_sb_loop', d['kind'], m+'.kind')
            same(True, d['ok'], m+'.ok')
            same([], d['errors'], m+'.errors')
            same(PARAMS, d['params'], m+'.params')
        same([], ds[3]['events'], m+'.control.events')
        same(ds[0]['output'], ds[3]['output'], m+'.control.output')
        for d in ds[1:3]:
            same(ds[0], d, m+'.repeat')
        if m not in OLD_MODELS:
            continue
        # Non-interference: earlier captures of the same setting, with this capture's additions removed.
        d = project(ds[0])
        old = summarize(load(Path(OLD_06G2.format(m))))
        same(kernel_view(old), kernel_view(d), m+'.06g2 kernel')
        for k in ('output', 'counts', 'params', 'model_sha256', 'binary_sha256'):
            same(old[k], d[k], m+'.06g2.'+k)
        if m == 'pk1':
            old = project(summarize(load(OLD_06H2)))
            for k in ('events', 'output', 'counts', 'params', 'model_sha256', 'binary_sha256'):
                same(old[k], project(ds[0])[k], 'pk1.06h2.'+k)


def batches(m, d, cover):
    """Inputs and backend responses come from the original only; expected commands never feed Rust."""
    inputs, traces = [], []
    for e in d['events']:
        if e['e'] != 'double':
            continue
        at = f'{m}.double[{e["id"]}]'
        steps = e['steps']
        same(1, e['stride'], at+'.stride')
        same(0, e['rc'], at+'.rc')
        assert e['n'] == len(e['j']) > 0, at+'.n'
        first = next(i for i, s in enumerate(steps) if s['kind'] == 'poll')
        pre = {s['kind']: s for s in steps[:first]}
        same(['factorSave', 'save', 'input', 'attr', 'origLower', 'origUpper'], [s['kind'] for s in steps[:first]], at+'.prefix')
        inp, ncol = pre['input'], e['bounds0'][0]
        same(0, inp['rc'], at+'.input.rc')
        same(['ObjVal', 0], [pre['attr']['attr'], pre['attr']['rc']], at+'.parent')
        for k in ('origLower', 'origUpper'):
            # Getter return is unused by the caller (C:368-369), so rc is not a status.
            same(ncol, len(pre[k]['v']), at+'.'+k+'.length')
        same([ncol, ncol], [len(inp['x']), len(bytes.fromhex(e['types']))], at+'.input.length')
        types, flips = bytes.fromhex(e['types']), bytes.fromhex(inp['flips'])
        for j in e['j']:
            assert types[j] in (66, 73), at+'.noninteger candidate'
            assert flips[j] == 0, at+'.flipped candidate'
        for x in inp['x'] + pre['origLower']['v'] + pre['origUpper']['v'] + [inp['tol'], pre['attr']['value']]:
            bits(value(x))  # reject IEEE NaN/inf
        head = ['batch', e['id'], e['stall'], inp['tol'], pre['attr']['value'], e['cost0'], steps[first-1]['w']]
        lines = [' '.join(map(str, head)), 'cols '+' '.join(map(str, e['j'])), 'x '+' '.join(inp['x']),
                 'lower '+' '.join(pre['origLower']['v']), 'upper '+' '.join(pre['origUpper']['v'])]
        exp = [f'case {e["id"]}']
        i, prev = first, steps[first-1]
        while i < len(steps):
            s, k = steps[i], steps[i]['kind']
            if k == 'poll':
                exp.append('poll')
            elif k == 'timer':
                assert s['r'] in (0, 1), at+'.timer result'
                exp.append(f'timer {prev["w"]}')
                lines.append(f'timer {s["r"]} {s["w"]}')
                if s['r']:  # C:1095-1100 expiry leaves the loop: closing flush and re-solve only
                    same(['flush', 'solve'], [t['kind'] for t in steps[i+1:]], at+'.after expiry')
                    cover['timer expired'] += 1
            elif k == 'propEnter':
                assert s['value'] is not None and s['dir'] in (60, 62), at+'.prop call site'
                bits(value(s['value']))
                exp.append(f'prop {s["col"]} {s["dir"]} {s["value"]} prop {s["w"]}')
                i += 1
                while steps[i]['kind'] == 'boundwrite':  # requests made inside the propagation
                    same(0, steps[i]['rc'], at+'.prop bound rc')
                    i += 1
                p = steps[i]
                same(['prop', 0, 0], [p['kind'], p['rc'], p['out']], at+'.prop')
                assert p['count'] == len(p['list']) > 0, at+'.prop list'
                cover['prop'] += 1
                cover['extra propagation'] += p['count'] > 1
                lines.append(' '.join(['prop', str(p['out']), str(p['count'])] +
                                      [f'{c} {f} {lo} {up}' for c, f, lo, up in zip(p['list'], p['flags'], p['lower'], p['upper'])] + [p['w']]))
                s = p
            elif k == 'flush':
                same(0, s['rc'], at+'.flush.rc')
                exp.append('flush')
            elif k == 'solve':
                same(0, s['rc'], at+'.solve.rc')
                assert s['status'] in (2, 3, 6, 7, 11, -1000), at+'.uncovered status'
                exp.append(f'solve {prev["w"]}')
                v, it = '-', '-'
                if i+1 < len(steps) and steps[i+1]['kind'] == 'attr':
                    a = steps[i+1]
                    same(['IterCount', 0], [a['attr'], a['rc']], at+'.iters')
                    it = a['value']
                    if s['status'] == 2:
                        o = steps[i+2]
                        same(['attr', 'ObjVal', 0], [o['kind'], o['attr'], o['rc']], at+'.obj')
                        v = o['value']
                        cover['optimal below parent'] += value(v) < value(pre['attr']['value'])
                    elif s['status'] in (7, 11, -1000):  # C:562-569 one clamp path
                        v = s['dual']
                        cover['limited below parent'] += value(v) < value(pre['attr']['value'])
                    cover[f'status {s["status"]}'] += 1
                else:
                    same(len(steps)-1, i, at+'.final solve is last')
                for x in (v, it):
                    if x != '-':
                        bits(value(x))
                lines.append(f'solve {s["status"]} {v} {it} {s["w"]}')
            elif k == 'attr':
                pass
            elif k == 'values':
                same(0, s['rc'], at+'.values.rc')
                same(len(s['j']), len(s['v']), at+'.values.length')
                for x in s['v']:
                    bits(value(x))
                exp.append('values '+' '.join(map(str, s['j'])))
                lines.append('values '+' '.join(s['v']))
                cover['values'] += 1
            elif k == 'reset':
                same(0, s['rc'], at+'.reset.rc')
                exp.append('reset')
            elif k == 'restore':
                exp.append(f'restore {prev["w"]}')
                lines.append(f'restore {s["w"]}')
            elif k == 'boundwrite':
                same(0, s['rc'], at+'.bound.rc')
                same(1, len(s['j']), at+'.bound.count')
                exp.append(f'bound {s["j"][0]} {bytes.fromhex(s["dir"])[0]} {s["val"][0]}')
                cover['restore request'] += 1
            else:
                raise AssertionError(at+'.unexpected '+k)
            prev = s
            i += 1
        exp.append(f'result {" ".join(e["a"])} | {" ".join(e["z"])} | {e["cost1"]}')
        lines.append('end')
        inputs.extend(lines)
        traces.extend(exp)
        branches(e, cover)
    return '\n'.join(inputs)+'\n', '\n'.join(traces)+'\n'


def branches(e, cover):
    """Branch evidence from the original command order and results (not from Rust)."""
    steps, cols = e['steps'], e['j']
    dirs = collections.defaultdict(list)
    for s in steps:
        if s['kind'] == 'propEnter':
            dirs[cols.index(s['col'])].append(s['dir'])
    last = max(dirs) if dirs else -1
    cover['batches'] += 1
    cover['candidates evaluated'] += len(dirs)
    cover['whole candidate skipped'] += sum(1 for k in range(last) if k not in dirs)
    for k, d in dirs.items():
        if d == [62]:
            cover['up >= 1e100' if e['z'][k] == INF else 'down gate' if e['a'][k] == NEG else 'down prefilled'] += 1
        elif d == [60]:
            cover['up prefilled'] += 1
        else:
            same([62, 60], d, 'direction order')
            cover['down >= 1e100'] += e['a'][k] == INF
    cover['early exit'] += last < len(cols) - 1
    parent = next(s['value'] for s in steps if s['kind'] == 'attr')
    cover['prefilled value'] += sum(1 for k, v in enumerate(e['a'] + e['z'])
                                    if v == parent and (k % len(cols)) not in dirs)


REQUIRED = ['prop', 'extra propagation', 'values', 'restore request', 'status 2', 'status 6', 'status 7',
            'optimal below parent', 'whole candidate skipped', 'down gate', 'up >= 1e100', 'early exit', 'prefilled value',
            # M2-06h-4 natural cases.
            'status 3', 'status -1000', 'limited below parent', 'down >= 1e100', 'up prefilled',
            # M2-06h-5.
            'timer expired']


def run(root, text, label):
    WORK.mkdir(parents=True, exist_ok=True)
    ip, op = WORK/(label+'.input'), WORK/(label+'.trace')
    ip.write_text(text)
    if op.exists():
        op.unlink()
    env = dict(os.environ, SB_LOOP_INPUT=str(ip), SB_LOOP_OUTPUT=str(op))
    if root != ROOT:
        env['CARGO_TARGET_DIR'] = str(ROOT/'target/mip_sb_loop_mutants')
    r = subprocess.run(['cargo', 'test', '--release', '-p', 'rsolver-mip', '--test', 'strong_branch_l1', '--', '--ignored'],
                       cwd=root, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    (WORK/(label+'.log')).write_text(r.stdout)
    assert r.returncode == 0, f'{label}: cargo failed; {WORK/(label+".log")}'
    return op.read_text()


def compare(expected, actual, at):
    e, a = expected.splitlines(), actual.splitlines()
    for i, (x, y) in enumerate(zip(e, a)):
        if x != y:
            case = next((l for l in reversed(e[:i]) if l.startswith('case ')), '?')
            raise AssertionError(f'{at}: first difference at line {i} ({case}): expected {x[:200]!r}, got {y[:200]!r}')
    same(len(e), len(a), at+'.lines')


def build(cases):
    cover = collections.Counter()
    inputs, expected = '', ''
    for m in cases:
        i, e = batches(m, cases[m][0], cover)
        inputs, expected = inputs + i, expected + e
    for k in REQUIRED:
        assert cover[k] > 0, 'no original case for '+k
    return inputs, expected, cover


def self_test(cases, manifest, inputs, expected, actual):
    def reject(name, fn):
        try:
            fn()
        except (AssertionError, KeyError, StopIteration, ValueError, IndexError):
            print('REJECT', name)
        else:
            raise AssertionError('NOT REJECTED '+name)

    def kernels(d):
        return [e for e in d['events'] if e['e'] == 'double']

    def step(d, kind, n=0):
        return [s for e in kernels(d) for s in e['steps'] if s['kind'] == kind][n]

    def drop(kind, pred=lambda s: True):
        def f(d):
            for e in kernels(d):
                e['steps'] = [s for s in e['steps'] if not (s['kind'] == kind and pred(s))]
        return f

    def set_step(kind, field, v, n=0):
        return lambda d: step(d, kind, n).__setitem__(field, v)

    tampers = [
        ('binary', lambda d: d.__setitem__('binary_sha256', '0'*64)),
        ('model', lambda d: d.__setitem__('model_sha256', '0'*64)),
        ('empty', lambda d: d.__setitem__('events', [])),
        ('no kernel', lambda d: d.__setitem__('events', [e for e in d['events'] if e['e'] != 'double'])),
        ('no propagation', drop('prop')),
        ('no extra propagation', lambda d: [s.update(count=1, list=s['list'][:1], flags=s['flags'][:1], lower=s['lower'][:1],
                                                     upper=s['upper'][:1]) for e in kernels(d) for s in e['steps'] if s['kind'] == 'prop']),
        ('no values', drop('values')),
        ('solution', lambda d: step(d, 'input')['x'].__setitem__(kernels(d)[0]['j'][0], bits(2.0))),
        ('tolerance', set_step('input', 'tol', bits(1.0))),
        ('original bound', lambda d: step(d, 'origUpper')['v'].__setitem__(step(d, 'prop')['list'][0], bits(1e9))),
        ('later values', lambda d: [s.__setitem__('v', [bits(0.5)]*len(s['v'])) for e in kernels(d) for s in e['steps'] if s['kind'] == 'values']),
        ('work', set_step('restore', 'w', bits(-1.0))),
        ('stall', lambda d: kernels(d)[0].__setitem__('stall', 1)),
        ('missing reset', drop('reset', lambda s: True)),
        ('timer', set_step('timer', 'r', 1)),
        ('timer clear', lambda d: [s.__setitem__('r', 0) for e in kernels(d) for s in e['steps'] if s['kind'] == 'timer']),
        ('contradiction', set_step('prop', 'out', 1)),
        ('status', set_step('solve', 'status', 3)),
    ]
    for name, mutate in tampers:
        bad = copy.deepcopy(cases)
        for m in MODELS:
            for d in bad[m][:3]:
                mutate(d)

        def check(bad=bad, name=name):
            controls(bad, manifest)
            i, e, _ = build(bad)
            compare(e, actual if i == inputs else run(ROOT, i, 'tamper-'+name.replace(' ', '-')), name)
        reject(name, check)
    # The stored repeats are identical, so also hit the builder directly (controls() would stop first).
    for name, mutate in [('direct values', lambda d: [s.__setitem__('v', [bits(0.5)]*len(s['v'])) for e in kernels(d) for s in e['steps'] if s['kind'] == 'values']),
                         ('direct stall', lambda d: [e.__setitem__('stall', 1) for e in kernels(d)]),
                         ('direct work', set_step('restore', 'w', bits(-1.0)))]:
        bad = {m: [copy.deepcopy(cases[m][0])] for m in MODELS}
        for m in MODELS:
            mutate(bad[m][0])

        def check(bad=bad, name=name):
            i, e, _ = build(bad)
            compare(e, run(ROOT, i, 'direct-'+name.replace(' ', '-')), name)
        reject(name, check)
    for name, old, new in [('order', 'flush\nsolve', 'solve\nflush'), ('bound', '\nbound ', '\nwrong '),
                           ('values', '\nvalues ', '\nwrong '), ('result', '\nresult ', '\nwrong ')]:
        assert old in actual, name
        reject('trace '+name, lambda old=old, new=new, name=name: compare(expected, actual.replace(old, new, 1), name))
    mutants = [
        ('prefill floor', 'if x.floor() + 1e-10 >= v {', 'if x.floor() + 1e-10 > v + 1.0 {'),
        ('prefill ceil', 'if v >= x.ceil() - 1e-10 {', 'if v > x.ceil() + 1.0 {'),
        ('prefill nil', 'let nil = self.eps > obj - parent;', 'let nil = false;'),
        ('prefill min', 'self.down_est[kk] = if self.down_est[kk] < obj { self.down_est[kk] } else { obj };', ''),
        ('eps', '(parent.abs() + 1.0) * 1e-15', '(parent.abs() + 1.0) * 1e-6'),
        ('gate', '&& best >= score(s.down_est[k] - parent, s.up_est[k] - parent);', '&& false;'),
        ('down gate', '&& best >= score(s.down_est[k] - parent, s.up[k] - parent)) {', '&& false) {'),
        ('stall', 'if stall >= b.stall_limit {', 'if stall > b.stall_limit {'),
        ('best', 'if v > best {', 'if v >= best {'),
        ('work', '*self.work += p.changes.len() as f64 * 6.0;', '*self.work += p.changes.len() as f64 * 5.0;'),
        ('restore strict', 'if c.flags & 1 != 0 && lo < c.lower {', 'if c.flags & 1 != 0 && lo <= c.lower {'),
        ('restore upper', 'if c.flags & 2 != 0 && c.upper < up {', 'if c.flags & 2 != 0 && false {'),
        ('up skip', 'if s.up[k] >= INF {\n                judge = false;', 'if false {\n                judge = false;'),
        ('score shift', '((a + 1e-5) * (b + 1e-5)).sqrt() - 1e-5', '((a + 1e-5) * (b + 1e-5)).sqrt()'),
        ('score floor', 'let a = if a > 1e-10 { a } else { 1e-10 };', 'let a = if a > 1e-2 { a } else { 1e-2 };'),
        ('clamp2', 'LpValue::Optimal(obj) => if obj > parent { obj } else { parent },', 'LpValue::Optimal(obj) => obj,'),
        ('clamp7', 'LpValue::Limited(dual) => if parent > dual { parent } else { dual },', 'LpValue::Limited(dual) => dual,'),
        ('split', 's.direction(k, Side::Lower, split + 1.0)?;', 's.direction(k, Side::Lower, split + 2.0)?;'),
        ('timer ignored', 'if s.lp.expired(s.work).map_err(Error::Backend)? {', 'if s.lp.expired(s.work).map_err(Error::Backend)? && false {'),
        ('timer return', 'if s.lp.expired(s.work).map_err(Error::Backend)? {\n            break;',
         'if s.lp.expired(s.work).map_err(Error::Backend)? {\n            return Ok(Evaluation { down: s.down, up: s.up });'),
    ]
    survivors = []
    with tempfile.TemporaryDirectory(prefix='sb-loop-') as td:
        root = Path(td)
        shutil.copytree(ROOT/'crates', root/'crates')
        for p in ('Cargo.toml', 'Cargo.lock'):
            shutil.copy2(ROOT/p, root/p)
        original = (ROOT/SOURCE).read_text()
        for name, old, new in mutants:
            assert old in original, name
            (root/SOURCE).write_text(original.replace(old, new, 1))
            if run(root, inputs, 'mutant-'+name.replace(' ', '-')) == expected:
                caught = synthetic(root, 'mutant-'+name.replace(' ', '-')) != 0
                survivors.append((name, caught))
                print('SURVIVED', name, '(original unverified; synthetic', 'rejects)' if caught else 'misses)')
            else:
                print('REJECT Rust', name)
    # No original case: a set flag on an unchanged bound (0 in the 06h-4 census too).
    # A constant score shift keeps every comparison against best (initially 0) unchanged.
    # clamp7 is rejected since 06h-4 (neos-3627168-kasai status 7 dual below parent).
    same([('restore strict', True), ('score shift', False)], survivors, 'surviving mutants')


def synthetic(root, label):
    env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT/'target/mip_sb_loop_mutants'))
    r = subprocess.run(['cargo', 'test', '--release', '-p', 'rsolver-mip', '--test', 'strong_branch_l1'],
                       cwd=root, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    (WORK/(label+'.synthetic.log')).write_text(r.stdout)
    # Only a failing assertion counts; a mutant that does not compile proves nothing.
    assert r.returncode == 0 or 'panicked' in r.stdout, f'{label}: synthetic did not run'
    return r.returncode


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--self-test', action='store_true')
    ap.add_argument('--write-manifest', action='store_true')
    ap.add_argument('--explore', help='replay every capture in DIR (exploration, not a baseline)')
    a = ap.parse_args()
    if a.explore:
        tot = collections.Counter()
        for p in sorted(Path(a.explore).glob('*.json.gz')):
            m, cover = p.name[:-8], collections.Counter()
            try:
                i, e = batches(m, load(p), cover)
                if cover['batches']:
                    compare(e, run(ROOT, i, 'explore-'+m), m)
                    tot += cover
                print(m, 'MATCH' if cover['batches'] else 'no batches', cover['batches'])
            except AssertionError as x:
                print(m, 'REJECT', str(x)[:200])
        print('TOTAL', dict(tot))
        return
    files = {m: [d/f'{m}_{i}.json.gz' for i in (1, 2, 3)] + [d/f'{m}_control.json.gz']
             for m, d in MODELS.items()}
    cases = {m: [load(p) for p in ps] for m, ps in files.items()}
    if a.write_manifest:
        paths = [p for ps in files.values() for p in ps] + [ROOT/p for p in TOOLS] + [OLD_06H2] + [Path(OLD_06G2.format(m)) for m in OLD_MODELS]
        manifest = dict(binary=BINARY, models={m: cases[m][0]['model_sha256'] for m in files},
                        files={str(p.relative_to(ROOT)): sha(p) for p in paths})
        (DATA4/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
    manifest = json.loads((DATA4/'manifest.json').read_text())
    same(BINARY, manifest['binary'], 'manifest.binary')
    for p, h in manifest['files'].items():
        same(h, sha(ROOT/p), 'manifest.'+p)
    controls(cases, manifest)
    inputs, expected, cover = build(cases)
    actual = run(ROOT, inputs, 'original')
    compare(expected, actual, 'Rust')
    if a.self_test:
        self_test(cases, manifest, inputs, expected, actual)
    print('PASS', dict(cover))


if __name__ == '__main__':
    main()
