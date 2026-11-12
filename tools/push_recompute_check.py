"""M3-04c-17: 독립3회/선행15 비간섭과 재계산 제어 재생, 오류 검출."""
import argparse
from collections import Counter
import copy
import hashlib
import json
import math
from pathlib import Path
import struct
import sys

from push_supplier_check import Fail, equal, require
from push_recompute_runs import CASES
from push_recompute_synthetic import BINARY_SHA, SCHEMA, SUBSTITUTION, bits, specs, sha_specs

ROOT = Path(__file__).resolve().parents[1]
D = Path('bench/oracle/m3_push_recompute')
TOOLS = ('tools/push_recompute_agent.js', 'tools/push_recompute_capture.js',
         'tools/push_recompute_synthetic.py', 'tools/push_recompute_runs.py', 'tools/push_recompute_check.py',
         'tools/push_factor_synthetic.py', 'tools/push_supplier_check.py',
         'tools/frida_resolve.cjs', 'tools/gurun.sh')
SOURCES = ('../re/decomp/c/001c0070_PRIVATE0000000000134a87.c',
           '../re/decomp/c/001205d0_PRIVATE000000000009cf9b.c')


def sha(p):
    return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()


def load(p):
    return json.loads((ROOT/p).read_text())


def records():
    return {n: [load(D/f'{n}-{i}.json') for i in (1, 2, 3)] for n in (*CASES, 'synthetic')}


def files():
    return dict(tools=set(TOOLS),
                inputs={p for model, extra in CASES.values() for p in (model, extra) if p},
                baselines={f'bench/oracle/m3_push_factor/{n}-1.json' for n in CASES} |
                          {'bench/oracle/m3_push_factor/manifest.json'},
                sources=set(SOURCES), assembly={str(D/'source.asm')},
                captures={str(D/f'{n}-{i}.json') for n in (*CASES, 'synthetic') for i in (1, 2, 3)},
                history={str(p.relative_to(ROOT)) for p in (ROOT/D/'history').glob('*') if p.is_file()})


def manifest():
    return dict(task='M3-04c-17', binary_sha256=BINARY_SHA, specs_sha256=sha_specs(),
                **{k: {p: sha(p) for p in sorted(paths)} for k, paths in files().items()})


def check_files(m):
    equal('M3-04c-17', m['task'], 'manifest.task')
    equal(BINARY_SHA, m['binary_sha256'], 'manifest.binary')
    equal(sha_specs(), m['specs_sha256'], 'manifest.specs')
    for field, paths in files().items():
        equal(paths, set(m[field]), 'manifest.'+field+'.keys')
        for p in paths:
            if field == 'sources' and not (ROOT/p).exists():
                continue
            equal(m[field][p], sha(p), 'manifest.'+p)


def controls(data, m):
    counts = {}
    for name in (*CASES, 'synthetic'):
        rs = data[name]
        equal(3, len(rs), name+'.runs')
        equal(3, len({r['run_id'] for r in rs}), name+'.run_id')
        equal(3, len({r['pid'] for r in rs}), name+'.pid')
        for i, r in enumerate(rs):
            label = f'{name}/{i+1}'
            equal(BINARY_SHA, r['binary_sha256'], label+'.binary')
            tool = 'tools/push_recompute_synthetic.py' if name == 'synthetic' else 'tools/push_recompute_capture.js'
            equal(m['tools'][tool], r['tool_sha256'], label+'.tool')
            equal(rs[0]['events'], r['events'], label+'.events')
            equal(rs[0]['environment'], r['environment'], label+'.environment')
            if name == 'synthetic':
                equal(SCHEMA, r['schema'], label+'.schema')
                equal(SUBSTITUTION, r['substitution'], label+'.substitution')
                equal('synthetic-original-slice', r['origin'], label+'.origin')
                equal(sha_specs(), r['input_sha256'], label+'.input')
            else:
                model, extra = CASES[name]
                baseline = load(f'bench/oracle/m3_push_factor/{name}-1.json')
                equal('M3-04c-17-v1', r['schema'], label+'.schema')
                equal('natural', r['origin'], label+'.origin')
                equal('control' if name=='control' else 'primal', r['kind'], label+'.kind')
                equal(m['tools']['tools/push_recompute_agent.js'], r['agent_sha256'], label+'.agent')
                equal(sha(model), r['model_sha256'], label+'.model')
                ex = load(extra) if extra else {}
                params = dict(Threads=1, Seed=0, OutputFlag=0, Presolve=0, Method=2,
                              Crossover=0 if name=='control' else 3, LPWarmStart=2)
                params.update(ex.get('params', {}))
                equal(params, r['params'], label+'.params')
                require(ex.get('starts')==r['starts'], label+'.input-starts')
                equal(baseline['starts'], r['starts'], label+'.starts')
                equal([], r['errors'], label+'.errors')
                equal('process-terminated', r['reason'], label+'.reason')
                equal(baseline['output'], r['output'], label+'.output')
                equal(baseline['result'], r['result'], label+'.result')
                projected = [e for e in r['events'] if not e['event'].startswith('recompute:')]
                equal(baseline['events'], projected, label+'.projection')
        counts[name] = len(rs[0]['events'])
    return counts


def f64(s):
    require(isinstance(s, str) and len(s)==16, 'f64 비트 필수: '+repr(s))
    value = struct.unpack('<d', bytes.fromhex(s))[0]
    require(math.isfinite(value), '유한 f64 계약: '+s)
    return value


def recompute(x, raw):
    m = x['m']
    require(type(m) is int and m>=0, 'm 전제')
    equal(m, len(x['header']), 'header 길이')
    equal(m, len(x['rhs']), 'rhs 길이')
    equal(m, len(raw), 'raw 길이')
    require(len(set(x['header']))==m, 'header 중복')
    equal(len(x['lb']), len(x['ub']), 'bounds 길이')
    I, h = f64(x['I']), f64(x['h'])
    require(I>0. and 0.<h<I, 'I/h 전제')
    equal(bits(-I), x['minus_I'], '-I')
    equal(bits(-h), x['minus_h'], '-h')
    basic, width, marks, pinf, violations = [], [], [], 0., 0
    for p, j in enumerate(x['header']):
        require(type(j) is int and 0<=j<len(x['lb']), f'header[{p}]')
        l, u, v = f64(x['lb'][j]), f64(x['ub'][j]), f64(raw[p])
        require(l<=u, f'bounds[{j}]')
        if l<=-I:
            w = I
        else:
            w = I if u>=I else u-l
            v -= l
        if l<=-I and u>=I:
            mark = 'F'
        elif v < -h:
            mark = 'B'; violations += 1; pinf -= v
        elif v < h:
            mark = 'X' if w==0. else 'L'
        elif v < w-h or w>=I:
            mark = 'N'
        elif w+h <= v:
            mark = 'A'; violations += 1; pinf += v-w
        else:
            mark = 'U'
        basic.append(bits(v)); width.append(bits(w)); marks.append(mark.encode().hex())
    retry = int(x['model_status']==2 and x['attempt']==0 and pinf>1e10)
    return dict(exit='retry' if retry else 'normal', basic=basic, width=width, marks=marks,
                pinf=bits(pinf), violations=violations, retry=retry, request=x['request'],
                aliases=x['aliases'])


def natural(r, name):
    totals = Counter()
    current = None
    push = last_factor = None
    for e in r['events']:
        tag = e['event']
        if tag == 'push:enter':
            require(push is None, name+'.nested push')
            push = e; last_factor = None
        if tag == 'factor:leave':last_factor = e
        if tag == 'push:leave':
            require(current is None, name+'.unfinished recompute')
            push = None
        if not tag.startswith('recompute:'):continue
        label = name+f'.recompute[{e["id"]}].'+tag
        require(push is not None, label+'.push required')
        equal(push['call'], e['call'], label+'.call')
        require(e['site'] in ('pivot', 'round'), label+'.site')
        if tag == 'recompute:enter':
            require(current is None, label+'.nested')
            x = e['input']
            equal(push['attempt'], x['attempt'], label+'.attempt')
            equal(push['model_status'], x['model_status'], label+'.model_status')
            equal(push['state']['m'], x['m'], label+'.m')
            equal(push['model']['lb'], x['lb'], label+'.lb')
            equal(push['model']['ub'], x['ub'], label+'.ub')
            equal(push['state']['tolerances']['370'], x['I'], label+'.I')
            equal(push['state']['tolerances']['380'], x['h'], label+'.h')
            equal(dict(factor=True, rhs=True, input=True, basic=True, marks=True), x['frame_matches'], label+'.frame')
            require(last_factor is not None, label+'.factor required')
            equal(0, last_factor['ret'], label+'.factor.ret')
            equal(last_factor['header'], x['header'], label+'.new header')
            equal(last_factor['request'], x['request'], label+'.request')
            require(x['request']!=0, label+'.request nonzero')
            current = dict(enter=e, stage=0)
            continue
        require(current is not None, label+'.enter required')
        start = current['enter']; x = start['input']
        for key in ('id', 'site', 'call', 'step'):
            equal(start[key], e[key], label+'.'+key)
        if tag == 'recompute:ftran:enter':
            equal(0, current['stage'], label+'.stage')
            equal(dict(count=-1, values=x['rhs']), e['input'], label+'.input')
            equal(dict(factor=True, input=True, output=True, flag=0, work=True), e['args'], label+'.args')
            current['stage'] = 1
        elif tag == 'recompute:ftran:leave':
            equal(1, current['stage'], label+'.stage')
            equal(x['m'], len(e['raw']), label+'.raw length')
            equal(x['m'], len(e['input_after']), label+'.input length')
            equal(x['m'], len(e['rhs_after']), label+'.rhs length')
            if x['aliases']['rhs_input']:
                equal(e['input_after'], e['rhs_after'], label+'.rhs alias')
            else:
                equal(x['rhs'], e['rhs_after'], label+'.rhs preserved by ftran')
            current.update(stage=2, raw=e['raw'], rhs_after=e['rhs_after'])
        elif tag == 'recompute:leave':
            equal(2, current['stage'], label+'.stage')
            want = recompute(x, current['raw'])
            for key, value in want.items():equal(value, e[key], label+'.'+key)
            equal(dict(header=x['header'], lb=x['lb'], ub=x['ub'], rhs=current['rhs_after']), e['preserved'], label+'.preserved')
            totals[e['site']] += 1
            totals['rows'] += x['m']
            totals.update('mark_'+bytes.fromhex(mark).decode() for mark in e['marks'])
            current = None
        else:
            raise Fail(label+'.unknown tag')
    require(current is None and push is None, name+'.closed')
    if name == 'control':equal(0, totals['pivot']+totals['round'], name+'.no target')
    else:require(totals['pivot']+totals['round']>0, name+'.target required')
    return totals


def acceptance(data):
    totals = Counter()
    for name in CASES:
        counts = natural(data[name][0], name)
        totals.update(counts)
    require(totals['pivot']>0 and totals['round']>0, '자연 두 진입점 필수')
    expected = specs()
    actual = data['synthetic'][0]['events']
    equal(len(expected), len(actual), 'synthetic cases')
    for case, event in zip(expected, actual):
        label = 'synthetic.'+case['name']
        equal(case, {k: event[k] for k in ('name', 'site', 'input')}, label+'.input')
        x = case['input']
        want = recompute(x, x['raw'])
        want.update(calls=[dict(input=dict(count=-1, values=x['rhs']),
                               args=dict(factor=True, input=True, output=True, flag=0, work=True))],
                    preserved=dict(header=x['header'], lb=x['lb'], ub=x['ub'], rhs=x['rhs']))
        equal(want, event['result'], label+'.result')
        totals['synthetic'] += 1
        totals['synthetic_rows'] += x['m']
        totals['synthetic_retry'] += want['retry']
        totals.update('synthetic_mark_'+bytes.fromhex(mark).decode() for mark in want['marks'])
    return dict(totals)


def self_test(data, m):
    rejected = []
    def test(name, mutate, fn):
        d, mm = copy.deepcopy(data), copy.deepcopy(m)
        mutate(d, mm)
        try:
            fn(d, mm)
        except (Fail, KeyError, TypeError, IndexError, ValueError) as exc:
            rejected.append(dict(name=name, error=str(exc)))
            return
        raise Fail('NOT REJECTED '+name)
    for key in ('binary_sha256', 'model_sha256', 'agent_sha256', 'tool_sha256'):
        test(key, lambda d, m, k=key: d['arms'][0].update({k:'0'*64}), controls)
    for field in ('inputs', 'baselines', 'tools', 'captures', 'assembly'):
        test('manifest-empty-'+field, lambda d, m, f=field: m.update({f:{}}), lambda d, m: check_files(m))
    test('manifest-binary-zero', lambda d, m: m.update(binary_sha256='0'*64), lambda d, m: check_files(m))
    test('empty-events', lambda d, m: d['arms'][0].update(events=[]), lambda d, m: acceptance(d))
    def zero(d, m):
        for n in ('arms', 'retry', 'interior'):
            for r in d[n]:r['events'] = [e for e in r['events'] if not e['event'].startswith('recompute:')]
    test('all-target-zero', zero, lambda d, m: acceptance(d))
    test('independent-difference', lambda d, m: d['arms'][1]['events'].pop(), controls)
    test('projection-difference', lambda d, m: d['arms'][0]['events'][0].update(first=99), controls)
    test('null-null', lambda d, m: d['synthetic'][0]['events'][0].update(result=None), lambda d, m: acceptance(d))
    def change_event(d, tag, key, value):
        e = next(e for e in d['interior'][0]['events'] if e['event']==tag)
        e[key] = value
    for tag, key, value in (
        ('recompute:enter', 'input', None), ('recompute:enter', 'id', 99),
        ('recompute:ftran:enter', 'input', dict(count=0, values=[])),
        ('recompute:ftran:enter', 'args', dict(factor=True, input=True, output=True, flag=1, work=True)),
        ('recompute:ftran:leave', 'raw', []), ('recompute:leave', 'basic', []),
        ('recompute:leave', 'width', []), ('recompute:leave', 'marks', []),
        ('recompute:leave', 'pinf', bits(1.)), ('recompute:leave', 'violations', 1),
        ('recompute:leave', 'retry', 1), ('recompute:leave', 'request', 0),
        ('recompute:leave', 'aliases', dict(rhs_input=True, raw_basic=True))):
        test('natural-'+tag+'.'+key, lambda d, m, t=tag, k=key, v=value: change_event(d,t,k,v), lambda d, m: acceptance(d))
    def alter_input(d, key, value):
        next(e for e in d['interior'][0]['events'] if e['event']=='recompute:enter')['input'][key] = value
    for key, value in (('attempt', 1), ('model_status', 1), ('header', list(range(8))), ('minus_h', bits(0.)), ('I', bits(1e20))):
        test('natural-input-'+key, lambda d,m,k=key,v=value: alter_input(d,k,v), lambda d,m: acceptance(d))
    def raw_change(d, m):
        e = next(e for e in d['interior'][0]['events'] if e['event']=='recompute:ftran:leave')
        e['raw'][0] = bits(f64(e['raw'][0])+1.)
    test('natural-raw-bit', raw_change, lambda d,m: acceptance(d))
    for suffix, key, value in (
        ('free', 'marks', ['4e']), ('fixed-h-exact', 'marks', ['58']),
        ('bounded-minus-h-exact', 'marks', ['42']), ('bounded-h-exact', 'marks', ['4c']),
        ('bounded-width-minus-h-exact', 'marks', ['4e']), ('bounded-width-plus-h-exact', 'marks', ['55']),
        ('shifted-h-exact', 'basic', [bits(3.125)]), ('upper-infinite', 'width', [bits(0.), bits(0.)]),
        ('accumulation', 'pinf', bits(float(2**54+4))), ('mixed', 'violations', 0),
        ('alias-1-1', 'aliases', dict(rhs_input=False,raw_basic=False)),
        ('empty', 'pinf', bits(1.))):
        for site in ('pivot', 'round'):
            def mutate(d,m,n=site+'-'+suffix,k=key,v=value):
                next(e for e in d['synthetic'][0]['events'] if e['name']==n)['result'][k] = v
            test(site+'-'+suffix+'.'+key, mutate, lambda d,m: acceptance(d))
    for site in ('pivot', 'round'):
        for case in specs():
            if case['site']!=site or '-retry-' not in case['name']:continue
            def mutate(d,m,n=case['name']):
                event = next(e for e in d['synthetic'][0]['events'] if e['name']==n)
                event['result']['retry'] ^= 1
            test(case['name']+'.retry', mutate, lambda d,m: acceptance(d))
    return dict(rejected=len(rejected), details=rejected)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for flag in ('controls', 'acceptance', 'self-test', 'write-manifest'):
        p.add_argument('--'+flag, action='store_true')
    p.add_argument('--report', type=Path)
    a = p.parse_args()
    if a.write_manifest:
        (ROOT/D/'manifest.json').write_text(json.dumps(manifest(), ensure_ascii=False, indent=2)+'\n')
    m = load(D/'manifest.json'); check_files(m); data = records()
    out = {}
    if a.controls:out['controls'] = controls(data, m)
    if a.acceptance:out['acceptance'] = acceptance(data)
    if a.self_test:out['self_test'] = self_test(data, m)
    if a.report:a.report.write_text(json.dumps(out, ensure_ascii=False, indent=2)+'\n')
    print(json.dumps({k: v if k!='self_test' else dict(rejected=v['rejected']) for k,v in out.items()}, ensure_ascii=False))


if __name__ == '__main__':
    try:main()
    except (Fail, KeyError, TypeError, IndexError, ValueError) as exc:
        print('FAIL', exc, file=sys.stderr); sys.exit(1)
