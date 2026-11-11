#!/usr/bin/env python3
"""M2-06g-2 저장 원본 재생. 실행 계약은 docs/m2/mip_sb_restore.md."""
import argparse
import collections
import copy
import gzip
import hashlib
import json
import math
from pathlib import Path
import struct
import sys

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / 'bench/oracle/m2_06g2/final'
BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
CASES = [f'{m}_{k}' for k in ('default', 'iters10') for m in ('pk1', 'neos5', 'tr12-30')]


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def value(s):
    assert isinstance(s, str) and len(s) == 16, f'hex field: {s!r}'
    return struct.unpack('>d', bytes.fromhex(s))[0]


def bits(x):
    assert math.isfinite(x), f'non-finite: {x}'
    return struct.pack('>d', x).hex()


def same(a, b, at):
    if a == b:
        return
    if isinstance(a, dict) and isinstance(b, dict) and a.keys() == b.keys():
        for k in a:
            same(a[k], b[k], f'{at}.{k}')
    if isinstance(a, list) and isinstance(b, list) and len(a) == len(b):
        for i, (x, y) in enumerate(zip(a, b)):
            same(x, y, f'{at}[{i}]')
    raise AssertionError(f'{at}: expected={str(a)[:100]} actual={str(b)[:100]}')


def partition(e, at, mutation=None):
    a = copy.deepcopy(e['before'])
    n, extra = e['n'], e['extra']
    assert n >= 0 and extra >= 0, at
    same([n + extra] * 3, [len(x) for x in a], at + '.length')
    qs = iter(e['q'])

    def ask(i):
        j, r = next(qs)
        same(a[0][i], j, at + '.query.j')
        assert r in (0, 1), at + '.query.r'
        return r

    def swap(i, j):
        for row in a[:1] if mutation == 'omit_vectors' else a:
            row[i], row[j] = row[j], row[i]

    if extra == 0:
        end = 0
        for i in range(n):
            if ask(i):
                swap(i, end)
                end += 1
        start, count = 0, end
    else:
        start = n
        for i in reversed(range(n)):
            if ask(i):
                start -= 1
                swap(i, start)
        end = n
        for i in range(n, n + extra):
            if ask(i):
                swap(i, end)
                end += 1
        count = end - start
    same([], list(qs), at + '.queries.remaining')
    same(a, e['after'], at + '.after')
    same([start, count], [e['start'], e['count']], at + '.range')
    work = (n + extra) * 3.0 + count * 6.0
    if mutation == 'omit_swap_work':
        work = (n + extra) * 3.0
    same(bits(work + value(e['work0'])), e['work1'], at + '.work')


def state(s, at):
    assert set(s) == {'dim', 'status', 'stat', 'flags', 'vec', 'basic'}, at
    m, n = s['dim']
    assert m > 0 and n >= 0, at + '.dim'
    for k, size in [('stat', (m+n)*4), ('flags', m+n), ('vec', (m+n)*8), ('basic', m*4)]:
        assert isinstance(s[k], str) and len(s[k]) == 2*size, at + '.' + k
        bytes.fromhex(s[k])


def kernel(e, at, mutation=None):
    same(0, e['rc'], at + '.rc')
    same(e['p0'], e['p1'], at + '.params')
    same(e['bounds0'], e['bounds1'], at + '.bounds')
    ncol, lb, ub = e['bounds0']
    assert ncol > 0 and isinstance(lb, str) and isinstance(ub, str), at + '.bounds missing'
    same([16*ncol]*2, [len(lb), len(ub)], at + '.bounds length')
    bytes.fromhex(lb)
    bytes.fromhex(ub)
    assert e['n'] > 0 and e['stride'] == 1 and len(e['j']) == e['n'], at
    same([e['n'], e['n']], [len(e['a']), len(e['z'])], at + '.outputs.length')
    if e['e'] == 'kernel':
        same([], e['steps'], at + '.fallback.steps')
        return
    steps = e['steps']
    assert steps and steps[0]['kind'] == 'save', at + '.save missing'
    saved = steps[0]['src']
    state(saved, at + '.saved')
    base = None
    iters = 0.0
    current = None
    restores = 0
    for i, s in enumerate(steps):
        where = f'{at}.steps[{i}]'
        kind = s['kind']
        if kind in ('save', 'restore'):
            state(s['src'], where + '.src')
            state(s['dst'], where + '.dst')
            same(s['src'], s['dst'], where + '.copy')
            same(saved, s['src'], where + '.saved_source')
            if kind == 'restore':
                restores += 1
                assert current is not None and base is not None, where + '.solve missing'
                assert s['branch'] is not None and s['result'] is not None, where + '.result missing'
                j, direction = s['branch']
                assert j in e['j'] and direction in (60, 62), where + '.branch'
                ncol = e['bounds0'][0]
                original = [struct.unpack('<' + 'd'*ncol, bytes.fromhex(x)) for x in e['bounds0'][1:]]
                active = [struct.unpack('<' + 'd'*ncol, bytes.fromhex(x)) for x in current['before'][1:]]
                assert all(lb >= old for lb, old in zip(active[0], original[0])), where + '.LB loosened'
                assert all(ub <= old for ub, old in zip(active[1], original[1])), where + '.UB loosened'
                assert (active[1][j] < original[1][j] if direction == 60 else active[0][j] > original[0][j]), where + '.branch bound missing'
                if current['status'] == 2:
                    assert 'obj' in current, where + '.ObjVal missing'
                    expected = max(current['obj'], base)
                elif current['status'] in (3, 6):
                    expected = 1e100
                elif current['status'] in (7, 11, -1000):
                    expected = max(base, value(current['dual']))
                else:
                    raise AssertionError(where + f'.uncovered status {current["status"]}')
                if mutation in ('no_base_clamp', 'no_status7_clamp'):
                    if current['status'] == 7:
                        expected = value(current['dual'])
                    if mutation == 'no_base_clamp' and current['status'] == 2:
                        expected = current['obj']
                same(bits(expected), s['result'][0 if direction == 60 else 1], where + '.consumed_result')
                assert current.get('it') is not None, where + '.IterCount missing'
                current = None
        elif kind == 'solve':
            same(0, s['rc'], where + '.rc')
            same(s['before'], s['after'], where + '.solve.bounds')
            assert current is None, where + '.restore missing'
            current = s.copy()
        elif kind == 'attr':
            same(0, s['rc'], where + '.rc')
            x = value(s['value'])
            if s['attr'] == 'ObjVal':
                if current is None:
                    assert base is None, where + '.duplicate base'
                    base = x
                else:
                    current['obj'] = x
            elif s['attr'] == 'IterCount':
                assert current is not None and x >= 0, where
                current['it'] = x
                iters += x
            else:
                raise AssertionError(where + '.unknown attr')
        else:
            raise AssertionError(where + '.unknown kind')
    assert restores > 0, at + '.restore count is zero'
    assert current and current['status'] == 2 and 'it' not in current, at + '.final parent solve'
    same(e['bounds0'], current['before'], at + '.parent.bounds')
    same(bits(iters), e['cost1'], at + '.iteration_sum')
    same('0000000000000000', e['cost0'], at + '.iteration_sum.initial')


def rules(d, case, mutation=None):
    es = d['events']
    assert es, case + '.empty events'
    counts = collections.Counter(e['e'] for e in es)
    for tag in ('partition', 'parent', 'budget'):
        assert counts[tag] > 0, case + '.missing ' + tag
    if case.endswith('iters10'):
        for tag in ('batch', 'kernel', 'double', 'cost'):
            assert counts[tag] > 0, case + '.missing ' + tag
    budget = None
    pending = None
    kernels, doubles = {}, {}
    seen = set()
    for i, e in enumerate(es):
        at = f'{case}.events[{i}]({e["e"]})'
        if 'id' in e:
            assert e['id'] not in seen, at + '.duplicate id'
            seen.add(e['id'])
        if e['e'] == 'partition':
            partition(e, at, mutation)
        elif e['e'] == 'parent':
            same(0, e['rc'], at + '.rc')
            same(e['p0'], e['p1'], at + '.params')
            assert e['classification'] in (0, 1, 2, 3, -1), at
        elif e['e'] == 'budget':
            budget = e
            same(e['nodeobj'], e['base'], at + '.base')
            cut = value(e['cut'])
            assert cut <= 5e99 and e['depth'] >= 0 and e['total'] > 0, at
            exponent = 0.1 if cut < 5e99 else 0.2
            if mutation == 'wrong_exponent':
                exponent = 0.2 if exponent == 0.1 else 0.1
            same(bits(math.pow(1.0/(float(e['depth'])+1.0), exponent)), e['weight'], at + '.weight')
        elif e['e'] in ('kernel', 'double'):
            kernel(e, at, mutation)
            (kernels if e['e'] == 'kernel' else doubles)[e['batch']] = e
        elif e['e'] == 'batch':
            assert budget is not None and pending is None, at
            same('542623', e['ra'], at + '.callsite')
            same(0, e['rc'], at + '.rc')
            same(e['p0'], e['p1'], at + '.params')
            cap = 100 if budget['cap'] == -1 else budget['cap']
            same(min(budget['total'], cap), e['n'], at + '.cap')
            same(1 if budget['cap'] == -1 else budget['threads'], e['threads'], at + '.threads')
            same(2 if budget['cap'] == -1 else 2000000000, e['stall'], at + '.stall')
            k, f = kernels.pop(e['id']), doubles.pop(e['id'])
            for field in ('j', 'a', 'z', 'cost1'):
                same(f[field], k[field], at + '.fallback.' + field)
            same(e['iters'], k['cost1'], at + '.returned_iteration_sum')
            reps, positions, groups = [], [], {}
            same(e['n'], len(e['groups']), at + '.groups.length')
            for j, g in zip(e['j'], e['groups']):
                if g is None or g not in groups:
                    if g is not None:
                        groups[g] = len(reps)
                    positions.append(len(reps))
                    reps.append(j)
                else:
                    positions.append(groups[g])
            same(reps, k['j'], at + '.symmetry.representatives')
            for field in ('a', 'z'):
                same([k[field][i] for i in positions], e[field], at + '.symmetry.' + field)
            pending = e
        elif e['e'] == 'cost':
            assert pending is not None, at + '.batch missing'
            total = value(pending['cost0']) + value(pending['iters'])
            if mutation == 'omit_cost':
                total = value(pending['cost0'])
            same(bits(total), e['cost1'], at + '.accumulation')
            same(pending['limitSaved'], e['limit1'], at + '.IterationLimit restoration')
            pending = None
        else:
            raise AssertionError(at + '.unknown event')
    assert pending is None and not kernels and not doubles, case + '.unmatched batch'
    return dict(counts)


def identity(d, case, control, manifest):
    same(1, d['schema'], case + '.schema')
    same('mip_sb_restore', d['kind'], case + '.kind')
    same(control, d['control'], case + '.control')
    same(case.rsplit('_', 1)[0] + '.mps.gz', d['model'], case + '.model_name')
    same(True, d['ok'], case + '.ok')
    same([], d['errors'], case + '.errors')
    same(BINARY, d['binary_sha256'], case + '.binary')
    same(manifest['models'][d['model']], d['model_sha256'], case + '.model')
    same(manifest['tool_sha256'], d['tool_sha256'], case + '.tool')
    params = dict(Threads=1, Seed=0, LogToConsole=0, NodeLimit=300)
    if case.endswith('iters10'):
        params['GURO_PAR_SBMAXITERS'] = 10
    same(params, d['params'], case + '.params')
    assert d['runtime']['arch'] == 'x64', case + '.arch'
    assert d['output'].startswith('DONE 8 300 '), case + '.output'


def self_test(data, manifest):
    case = 'pk1_iters10'
    original = data[case][0]
    tests = []

    def event(d, tag):
        return next(e for e in d['events'] if e['e'] == tag)

    def changed(tag, field, val):
        def f(d):
            event(d, tag)[field] = val
        return f

    def scalar(d):
        e = event(d, 'partition')
        e['after'][1][0] = '0000000000000001'

    def restore(d):
        e = event(d, 'double')
        next(s for s in e['steps'] if s['kind'] == 'restore')['dst']['basic'] = '00'

    def result(d):
        e = event(d, 'double')
        next(s for s in e['steps'] if s['kind'] == 'restore')['result'] = ['0000000000000001']*2

    def missing_restore(d):
        e = event(d, 'double')
        e['steps'] = [s for s in e['steps'] if s['kind'] != 'restore']

    def bounds(d):
        event(d, 'double')['bounds1'][1] = '00'

    def branch_bounds(d):
        e = event(d, 'double')
        for step in e['steps']:
            if step['kind'] == 'solve':
                step['before'] = copy.deepcopy(e['bounds0'])
                step['after'] = copy.deepcopy(e['bounds0'])

    for name, mutate in [
        ('binary', lambda d: d.update(binary_sha256='0'*64)),
        ('model', lambda d: d.update(model_sha256='0'*64)),
        ('empty', lambda d: d.update(events=[])),
        ('zero_batch', lambda d: d.update(events=[e for e in d['events'] if e['e'] not in ('batch','kernel','double','cost')])),
        ('partition_vector', scalar), ('partition_work', changed('partition','work1','0000000000000000')),
        ('basis_copy', restore), ('consumed_result', result), ('missing_restore', missing_restore), ('bounds', bounds),
        ('cost', changed('cost','cost1','0000000000000000')),
        ('limit', changed('cost','limit1','0000000000000000')),
        ('branch_bounds', branch_bounds),
        ('weight', changed('budget','weight','0000000000000000')),
        ('query', changed('partition','q',[])),
    ]:
        d = copy.deepcopy(original)
        mutate(d)
        try:
            identity(d, case, False, manifest)
            rules(d, case)
        except (AssertionError, KeyError, StopIteration, TypeError):
            tests.append(name)
        else:
            raise AssertionError('NOT REJECTED: ' + name)
    mutations = {}
    for mutation in ('omit_vectors', 'omit_swap_work', 'wrong_exponent', 'omit_cost', 'no_base_clamp', 'no_status7_clamp'):
        rejected = []
        for c in CASES:
            try:
                rules(data[c][0], c, mutation)
            except AssertionError:
                rejected.append(c)
        mutations[mutation] = rejected
    # Coverage is fixed from the saved runs; surviving variants are documented as unverified.
    expected = {
        'omit_vectors': [],
        'omit_swap_work': CASES,
        'wrong_exponent': CASES,
        'omit_cost': [c for c in CASES if c.endswith('iters10')],
        'no_base_clamp': ['neos5_iters10'],
        'no_status7_clamp': [],
    }
    same(expected, mutations, 'mutation coverage')
    print('SELF-TEST rejected', len(tests), 'tampers;', json.dumps(mutations))


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--data', type=Path, default=DATA)
    ap.add_argument('--write-manifest', action='store_true')
    ap.add_argument('--self-test', action='store_true')
    args = ap.parse_args()
    files = {p.name: p for p in args.data.glob('*.json.gz')}
    expected_files = {f'{c}_{r}.json.gz' for c in CASES for r in (1,2,3,'control')}
    same(expected_files, set(files), 'capture file set')
    load = lambda p: json.loads(gzip.decompress(p.read_bytes()))
    if args.write_manifest:
        assert not (args.data/'manifest.json').exists(), 'manifest exists'
        first = [load(files[f'{c}_1.json.gz']) for c in CASES]
        sources = list((ROOT.parent/'re/decomp/c').glob('*'))
        suffixes = ('691453.c','691b56.c','266b6a.c','695bd7.c','62ec1d.c','62f909.c','68fa4c..0.c','68fa4c..1.c','68d6ec.c','68f0c1.c')
        manifest = dict(task='M2-06g-2',schema=1,binary_sha256=BINARY,tool_sha256=sha(ROOT/'tools/mip_sb_restore.js'),
            files={n:sha(p) for n,p in sorted(files.items())},models={d['model']:d['model_sha256'] for d in first},
            sources={str(p.relative_to(ROOT.parent)):sha(p) for p in sources if p.name.endswith(suffixes)})
    else:
        manifest = json.loads((args.data/'manifest.json').read_text())
    same('M2-06g-2', manifest['task'], 'manifest.task')
    same(BINARY, manifest['binary_sha256'], 'manifest.binary')
    same(sha(ROOT/'tools/mip_sb_restore.js'), manifest['tool_sha256'], 'capture tool.sha')
    same({n:sha(p) for n,p in sorted(files.items())}, manifest['files'], 'manifest.files')
    for name, h in manifest['sources'].items():
        p = ROOT.parent/name
        if p.exists():
            same(h, sha(p), 'source ' + name)
    data = {}
    for case in CASES:
        runs = [load(files[f'{case}_{r}.json.gz']) for r in (1,2,3)]
        control = load(files[f'{case}_control.json.gz'])
        identity(control, case, True, manifest)
        same([], control['events'], case + '.control.events')
        for i,d in enumerate(runs):
            identity(d, case, False, manifest)
            same(runs[0], d, case + f'.repeat[{i}]')
            same(control['output'], d['output'], case + '.noninterference')
        print(case, json.dumps(rules(runs[0], case)))
        data[case] = runs
    if args.self_test:
        self_test(data, manifest)
    if args.write_manifest:
        (args.data/'manifest.json').write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
    print('ACCEPTANCE PASS' + (' (+ self-test)' if args.self_test else ''))


if __name__ == '__main__':
    try:
        main()
    except (AssertionError, KeyError, StopIteration, TypeError, ValueError) as exc:
        print('FAIL:', str(exc) or type(exc).__name__, file=sys.stderr)
        sys.exit(1)
