"""M3-04c-18: 17의 원본 명령 실행기로 누적·sentinel 판별10사례를 추가한다."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import sys

import push_recompute_check as oracle
import push_recompute_synthetic as native

ROOT = Path(__file__).resolve().parents[1]
D = ROOT / 'bench/oracle/m3_push_recompute_rust'
SCHEMA = 'M3-04c-18-extra-v1'


def specs():
    out = []
    I, h = 1e30, 0.125
    for site in ('pivot', 'round'):
        def add(name, rows):
            m = len(rows)
            x = dict(m=m, header=list(reversed(range(m))),
                     lb=[native.bits(r[0]) for r in rows], ub=[native.bits(r[1]) for r in rows],
                     raw=[native.bits(r[2]) for r in reversed(rows)],
                     rhs=[native.bits(17.25+i) for i in range(m)],
                     I=native.bits(I), minus_I=native.bits(-I), h=native.bits(h), minus_h=native.bits(-h),
                     attempt=0, model_status=1, request=10, aliases=dict(rhs_input=False, raw_basic=False))
            out.append(dict(name=site+'-'+name, site=site, input=x))
        add('lower-sentinel-wide', [(-I, I/2, 1.)])
        add('upper-sentinel-wide', [(I/2, I, I*0.75)])
        add('infinite-width-large-value', [(0., I, 2*I)])
        # Header reverses these: B contributes1, then A contributes(2^54-(2^54-4))=4.
        add('above-association', [(0., float(2**54-4), float(2**54)), (0., I, -1.)])
        # Original order: 2^54, 1, 1, 1 -> 2^54. Reverse -> 2^54+4.
        add('basis-accumulation', [(0., I, -1.)]*3+[(0., I, -float(2**54))])
    return out


def input_sha():
    return hashlib.sha256(json.dumps(specs(), sort_keys=True).encode()).hexdigest()


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def manifest():
    paths = [Path(__file__).relative_to(ROOT), Path('tools/push_recompute_synthetic.py'),
             Path('tools/push_recompute_check.py'), Path('bench/oracle/m3_push_recompute/manifest.json'),
             Path('bench/oracle/m3_push_recompute/source.asm')]
    paths += [Path('bench/oracle/m3_push_recompute_rust')/f'extra-{i}.json' for i in (1, 2, 3)]
    return dict(schema=SCHEMA, binary_sha256=native.BINARY_SHA, input_sha256=input_sha(),
                files={str(p): sha(ROOT/p) for p in paths})


def check(data, m):
    oracle.equal(manifest(), m, 'extra manifest files/inputs/binary')
    oracle.equal(3, len(data), 'extra independent runs')
    oracle.equal(3, len({r['pid'] for r in data}), 'extra independent pids')
    oracle.equal(3, len({r['run_id'] for r in data}), 'extra independent ids')
    for i, r in enumerate(data):
        label = f'extra[{i}]'
        for key, value in dict(schema=SCHEMA, binary_sha256=native.BINARY_SHA,
                               tool_sha256=sha(Path(__file__)), input_sha256=input_sha(),
                               slice_tool_sha256=sha(ROOT/'tools/push_recompute_synthetic.py'),
                               origin='synthetic-original-slice', substitution=native.SUBSTITUTION).items():
            oracle.equal(value, r[key], label+'.'+key)
        oracle.equal(data[0]['events'], r['events'], label+'.events')
        oracle.equal(data[0]['environment'], r['environment'], label+'.environment')
        oracle.equal(10, len(r['events']), label+'.nonempty count')
        for case, event in zip(specs(), r['events']):
            label2 = label+'.'+case['name']
            oracle.equal(case, {k: event[k] for k in ('name', 'site', 'input')}, label2+'.input')
            x = case['input']
            want = oracle.recompute(x, x['raw'])
            want.update(calls=[dict(input=dict(count=-1, values=x['rhs']),
                                   args=dict(factor=True, input=True, output=True, flag=0, work=True))],
                        preserved=dict(header=x['header'], lb=x['lb'], ub=x['ub'], rhs=x['rhs']))
            oracle.equal(want, event['result'], label2+'.result')
    return dict(cases=10, rows=18, independent_runs=3)


def load_checked():
    data = [json.loads((D/f'extra-{i}.json').read_text()) for i in (1, 2, 3)]
    m = json.loads((D/'extra-manifest.json').read_text())
    check(data, m)
    return data


def self_test(data, manifest_data):
    out = []
    def reject(name, mutate):
        rs, m = copy.deepcopy(data), copy.deepcopy(manifest_data)
        mutate(rs, m)
        try:
            check(rs, m)
        except (oracle.Fail, TypeError, KeyError, ValueError) as exc:
            out.append(dict(name=name, first_difference=str(exc)))
            return
        raise oracle.Fail('NOT REJECTED extra '+name)
    reject('binary-zero', lambda d, m: d[0].update(binary_sha256='0'*64))
    reject('input-hash-zero', lambda d, m: d[0].update(input_sha256='0'*64))
    reject('tool-hash-zero', lambda d, m: d[0].update(tool_sha256='0'*64))
    reject('slice-hash-zero', lambda d, m: d[0].update(slice_tool_sha256='0'*64))
    reject('manifest-empty', lambda d, m: m.update(files={}))
    reject('empty-events', lambda d, m: [r.update(events=[]) for r in d])
    reject('repeat-difference', lambda d, m: d[1]['events'].pop())
    reject('null-result', lambda d, m: [r['events'][0].update(result=None) for r in d])
    reject('input-change', lambda d, m: [r['events'][0]['input'].update(header=[]) for r in d])
    for field, value in (('pinf', native.bits(0.)), ('violations', 0), ('basic', []), ('width', []),
                         ('marks', []), ('request', 0), ('retry', 1), ('exit', 'retry'),
                         ('calls', []), ('preserved', {}), ('aliases', {})):
        reject('result-'+field, lambda d, m, k=field, v=value: [r['events'][3]['result'].update({k:v}) for r in d])
    reject('case-order', lambda d, m: [r['events'].reverse() for r in d])
    return out


def main():
    p = argparse.ArgumentParser(description=__doc__)
    g = p.add_mutually_exclusive_group(required=True)
    g.add_argument('--capture', type=Path)
    g.add_argument('--check', action='store_true')
    g.add_argument('--write-manifest', action='store_true')
    a = p.parse_args()
    oracle.check_files(oracle.load(oracle.D/'manifest.json'))
    if a.capture:
        oracle.require(not a.capture.exists(), 'capture output already exists')
        # Existing slice executor, ABI, patched FTRAN/log, and exits are unchanged.
        native.specs = specs
        result = native.capture()
        result.update(schema=SCHEMA, tool_sha256=sha(Path(__file__)),
                      slice_tool_sha256=sha(ROOT/'tools/push_recompute_synthetic.py'))
        oracle.equal(input_sha(), result['input_sha256'], 'capture input hash')
        with a.capture.open('x') as f:
            f.write(json.dumps(result, ensure_ascii=False, indent=2)+'\n')
        print(json.dumps(dict(captured=len(result['events']), path=str(a.capture))))
    else:
        if a.write_manifest:
            (D/'extra-manifest.json').write_text(json.dumps(manifest(), ensure_ascii=False, indent=2)+'\n')
        data = load_checked()
        tests = self_test(data, json.loads((D/'extra-manifest.json').read_text()))
        (D/'extra-self-test.json').write_text(json.dumps(tests, ensure_ascii=False, indent=2)+'\n')
        print(json.dumps(dict(cases=10, rows=18, independent_runs=3, rejected=len(tests))))


if __name__ == '__main__':
    try:
        main()
    except (oracle.Fail, KeyError, TypeError, IndexError, ValueError) as exc:
        sys.exit('FAIL '+str(exc))
