"""M3-04c-18: 원본 FTRAN 반환값만 Rust에 주고 원본 후상태와 직접 대조한다."""
import argparse
from collections import Counter
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys

import push_recompute_check as oracle

ROOT = Path(__file__).resolve().parents[1]
D = ROOT / 'bench/oracle/m3_push_recompute_rust'
INPUT = D / 'input.txt'
TRACE = ROOT / 'target/m3_push_recompute_rust/trace.txt'
SOURCE = ROOT / 'crates/lp/src/crossover_recompute.rs'
SCHEMA = 'M3-04c-18-v1'
RESULT_KEYS = ('exit', 'basic', 'width', 'marks', 'pinf', 'violations', 'retry', 'request')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def cases():
    data, manifest = oracle.records(), oracle.load(oracle.D / 'manifest.json')
    oracle.check_files(manifest)
    oracle.controls(data, manifest)
    oracle.acceptance(data)
    out = []
    for name in ('arms', 'retry', 'interior'):
        current = None
        for e in data[name][0]['events']:
            if e['event'] == 'recompute:enter':
                current = dict(name=f'{name}/{e["call"]}/{e["id"]}/{e["site"]}',
                               site=e['site'], origin='natural', input=copy.deepcopy(e['input']))
            elif e['event'] == 'recompute:ftran:leave':
                current['input']['raw'] = e['raw']
            elif e['event'] == 'recompute:leave':
                current['result'] = e
                out.append(current)
                current = None
    for e in data['synthetic'][0]['events']:
        out.append(dict(name='synthetic/'+e['name'], site=e['site'], origin='synthetic',
                        input=e['input'], result=e['result']))
    oracle.equal(128, len(out), '8 natural + 120 synthetic')
    oracle.equal(len(out), len({c['name'] for c in out}), 'unique cases')
    return out


def project(cs):
    lines, expected = [], {}
    for case in cs:
        x, r = case['input'], case['result']
        fields = [case['name'], x['m'], len(x['lb']), x['I'], x['h'], x['model_status'],
                  x['attempt'], x['request'], int(x['aliases']['raw_basic']),
                  *x['header'], *x['lb'], *x['ub'], *x['raw']]
        lines.append(' '.join(map(str, fields)))
        # Native poststates, never the Python recompute model or Rust-produced answers.
        want = {key: r[key] for key in RESULT_KEYS}
        want['raw_basic'] = r['aliases']['raw_basic']
        want['preserved'] = {key: r['preserved'][key] for key in ('header', 'lb', 'ub')}
        expected[case['name']] = want
    body = '\n'.join(lines)+'\n'
    header = SCHEMA+' '+hashlib.sha256(body.encode()).hexdigest()
    return header+'\n'+body, expected


def no_duplicates(pairs):
    out = {}
    for key, value in pairs:
        oracle.require(key not in out, 'duplicate JSON key: '+key)
        out[key] = value
    return out


def compare(txt, expected, header):
    lines = txt.splitlines()
    oracle.require(lines and lines[0] == header, 'trace schema/input hash')
    oracle.equal(len(expected), len(lines)-1, 'trace case count')
    fields = 0
    def check(want, got, label):
        nonlocal fields
        oracle.require(type(want) is type(got), label+': type mismatch')
        if isinstance(want, dict):
            oracle.equal(set(want), set(got), label+'.keys')
            for key in want:
                check(want[key], got[key], label+'.'+key)
        elif isinstance(want, list):
            oracle.equal(len(want), len(got), label+'.length')
            for i, (w, g) in enumerate(zip(want, got)):
                check(w, g, f'{label}[{i}]')
        else:
            oracle.equal(want, got, label)
            fields += 1
    for line, (name, want) in zip(lines[1:], expected.items()):
        tag, body = line.split(';', 1)
        oracle.equal(name, tag, 'trace case order')
        check(want, json.loads(body, object_pairs_hook=no_duplicates), name)
    return dict(cases=len(expected), fields=fields, rule='ordered typed exact; f64 little-endian bits incl signed zero')


def self_test(txt, expected, header):
    results = []
    def reject(name, lines):
        try:
            compare('\n'.join(lines)+'\n', expected, header)
        except (oracle.Fail, ValueError, TypeError, KeyError) as exc:
            results.append(dict(name=name, first_difference=str(exc)))
            return
        raise oracle.Fail('NOT REJECTED '+name)
    lines = txt.splitlines()
    reject('empty-targets', lines[:1])
    reject('missing-case', lines[:-1])
    reject('case-order', lines[:1]+[lines[2], lines[1]]+lines[3:])
    reject('input-hash-zero', [SCHEMA+' '+'0'*64]+lines[1:])
    for case in ('arms/1/1/round', 'synthetic/pivot-mixed', 'synthetic/round-alias-1-1',
                 'synthetic/pivot-accumulation', 'synthetic/round-empty'):
        index = list(expected).index(case)+1
        for key in expected[case]:
            altered = lines.copy()
            tag, body = altered[index].split(';', 1)
            value = json.loads(body)
            old = value[key]
            value[key] = (not old if isinstance(old, bool) else old+1 if isinstance(old, int)
                          else [] if isinstance(old, list) else {} if isinstance(old, dict) else 'invalid')
            if value[key] == old:
                value[key] = ['invalid']
            altered[index] = tag+';'+json.dumps(value)
            reject(case+'/'+key, altered)
        altered = lines.copy()
        altered[index] = altered[index][:-1]+',"request":999}'
        reject(case+'/duplicate', altered)
    # One-bit change must fail even if the numerical difference is tiny.
    for key in ('basic', 'width', 'pinf'):
        altered = lines.copy()
        tag, body = altered[1].split(';', 1)
        value = json.loads(body)
        old = value[key][0] if isinstance(value[key], list) else value[key]
        b = bytearray.fromhex(old); b[0] ^= 1
        if isinstance(value[key], list): value[key][0] = b.hex()
        else: value[key] = b.hex()
        altered[1] = tag+';'+json.dumps(value)
        reject('f64-one-bit/'+key, altered)
    return results


# Fix expected classifications before executing Rust mutants. Four predicates cannot
# be distinguished by this captured corpus (see contract); retain those limitations.
SURVIVORS = {'lower-sentinel-strict', 'upper-sentinel-strict', 'normal-no-infinite', 'reassociate-above'}
MUTATIONS = [
    ('no-raw-copy', 'basic.copy_from_slice(raw);', 'let _ = raw;'),
    ('no-shift', 'basic[p] -= lower;', 'basic[p] += 0.0;'),
    ('wrong-shift-sign', 'basic[p] -= lower;', 'basic[p] += lower;'),
    ('wrong-column', 'let j = column as usize;', 'let j = p; let _ = column;'),
    ('lower-sentinel-strict', 'lower <= -inf {', 'lower < -inf {'),
    ('shift-sentinel-equality', 'if lower > -inf {', 'if lower >= -inf {'),
    ('upper-sentinel-strict', 'if upper >= inf {', 'if upper > inf {'),
    ('width-is-upper', 'upper - lower', 'upper'),
    ('free-as-normal', "b'F'", "b'N'"),
    ('free-ignore-upper', 'lower <= -inf && upper >= inf', 'lower <= -inf'),
    ('free-ignore-lower', 'lower <= -inf && upper >= inf', 'upper >= inf'),
    ('below-inclusive', 'v < -h', 'v <= -h'),
    ('lower-inclusive', 'v < h', 'v <= h'),
    ('fixed-as-lower', "if w == 0.0 { b'X' } else { b'L' }", "if false { b'X' } else { b'L' }"),
    ('lower-as-fixed', 'w == 0.0', 'true'),
    ('normal-inclusive', 'v < w - h', 'v <= w - h'),
    ('normal-no-h', 'v < w - h', 'v < w'),
    ('normal-no-infinite', '|| w >= inf', '|| false'),
    ('above-strict', 'w + h <= v', 'w + h < v'),
    ('above-no-h', 'w + h <= v', 'w <= v'),
    ('no-below-count', "violations += 1;\n            pinf -= v;", "violations += 0;\n            pinf -= v;"),
    ('no-above-count', "violations += 1;\n            pinf += v - w;", "violations += 0;\n            pinf += v - w;"),
    ('below-wrong-sign', 'pinf -= v;', 'pinf += v;'),
    ('above-no-width', 'pinf += v - w;', 'pinf += v;'),
    ('reassociate-above', 'pinf += v - w;', 'pinf = (pinf + v) - w;'),
    ('reverse-basis', 'input.header.iter().enumerate()', 'input.header.iter().enumerate().rev()'),
    ('pinf-not-reset', 'let mut pinf = 0.0;', 'let mut pinf = state.pinf;'),
    ('count-not-reset', 'let mut violations = 0;', 'let mut violations = state.violations;'),
    ('retry-inclusive', 'pinf > 1e10', 'pinf >= 1e10'),
    ('retry-ignore-status', 'input.model_status == 2', 'true'),
    ('retry-ignore-attempt', 'input.attempt == 0', 'true'),
    ('retry-no-write', 'state.retry = 1;', 'state.retry = 0;'),
    ('request-cleared', 'state.pinf = pinf;', 'state.request = 0; state.pinf = pinf;'),
]


def mutate_rust(path, expected, header):
    original, results = SOURCE.read_text(), []
    command = ['cargo', 'test', '-p', 'rsolver-lp', '--test', 'push_recompute']
    try:
        for name, old, new in MUTATIONS:
            oracle.equal(1, original.count(old), 'unique mutation target '+name)
            SOURCE.write_text(original.replace(old, new, 1))
            TRACE.unlink(missing_ok=True)
            proc = subprocess.run(command, cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            if proc.returncode or not TRACE.exists():
                results.append(dict(name=name, result='test-failed', log=proc.stdout))
                continue
            try:
                compare(TRACE.read_text(), expected, header)
            except (oracle.Fail, ValueError, TypeError) as exc:
                results.append(dict(name=name, result='rejected', first_difference=str(exc)))
            else:
                results.append(dict(name=name, result='survived'))
    finally:
        SOURCE.write_text(original)
        proc = subprocess.run(command, cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        oracle.require(proc.returncode == 0, proc.stdout)
        compare(TRACE.read_text(), expected, header)
        path.write_text(json.dumps(results, ensure_ascii=False, indent=2)+'\n')
    oracle.equal(len(MUTATIONS), len(results), 'all mutants executed')
    for result in results:
        oracle.equal('survived' if result['name'] in SURVIVORS else 'rejected', result['result'],
                     'fixed mutation expectation '+result['name'])
    return dict(rejected=sum(x['result']=='rejected' for x in results),
                unverified=[x['name'] for x in results if x['result']=='survived'])


def manifest():
    paths = ['crates/lp/src/crossover_recompute.rs', 'crates/lp/src/lib.rs', 'crates/lp/tests/push_recompute.rs',
             'tools/push_recompute_rust.py', 'bench/oracle/m3_push_recompute/manifest.json']
    paths += [str(p.relative_to(ROOT)) for p in sorted(D.iterdir()) if p.is_file() and p.name != 'manifest.json']
    return dict(task='M3-04c-18', baseline='484243d', binary_sha256=oracle.BINARY_SHA,
                oracle_manifest_sha256=sha(ROOT/oracle.D/'manifest.json'), files={p: sha(ROOT/p) for p in paths})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    g = parser.add_mutually_exclusive_group(required=True)
    for flag in ('write', 'compare', 'self-test', 'write-manifest'):
        g.add_argument('--'+flag, action='store_true')
    g.add_argument('--mutate-rust', type=Path)
    args = parser.parse_args()
    cs = cases()
    source, expected = project(cs)
    header = source.splitlines()[0]
    D.mkdir(parents=True, exist_ok=True)
    if args.write:
        INPUT.write_text(source)
        result = dict(written=len(cs))
    else:
        oracle.equal(source, INPUT.read_text(), 'fixed input projection')
        txt = TRACE.read_text()
        result = compare(txt, expected, header)
        result.update(natural=8, synthetic=120, rows=sum(c['input']['m'] for c in cs),
                      sites=dict(Counter(c['site'] for c in cs)))
        if args.mutate_rust:
            result['mutations'] = mutate_rust(args.mutate_rust, expected, header)
        elif args.self_test:
            tests = self_test(txt, expected, header)
            (D/'self-test.json').write_text(json.dumps(tests, ensure_ascii=False, indent=2)+'\n')
            result['rejected'] = len(tests)
        else:
            (D/'comparison.json').write_text(json.dumps(result, ensure_ascii=False, indent=2)+'\n')
        if args.write_manifest:
            (D/'trace.txt').write_text(txt)
            (D/'manifest.json').write_text(json.dumps(manifest(), ensure_ascii=False, indent=2)+'\n')
    print(json.dumps(result, ensure_ascii=False))


if __name__ == '__main__':
    try:
        main()
    except (oracle.Fail, KeyError, ValueError, TypeError, IndexError) as exc:
        sys.exit('FAIL '+str(exc))
