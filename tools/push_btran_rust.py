"""M3-04c-21d: 지원 BTRAN Rust adapter를 21c 원본 캡처의 커널 경계·raw·입력 후상태·cache·F1d8·scratch·bucket·work와 비트 비교한다.

입력 평문에는 `btran:enter`의 factor·입력·out count·cache·F1d8·F+0x200·bucket만 넣는다. 원본 커널/반환 사건은 이 비교기만 읽는다.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

import push_btran_check as oracle
from push_ftran_check import ZERO_LE, f64
from push_supplier_check import Fail, equal, require

ROOT = Path(__file__).resolve().parents[1]
D = ROOT / 'bench/oracle/m3_push_btran_rust'
TRACE = ROOT / 'target/m3_push_btran_rust/trace.txt'
SOURCE = Path('crates/lp/src/crossover_btran.rs')
SCHEMA = 'M3-04c-21d-v1'
MODELS = ('arms', 'retry', 'interior', 'adlittle', 'scsd1', 'capri')
KERNEL_KEYS = ('kernel', 'phase', 'input', 'output_count', 'output', 'alias', 'scratch', 'cache', 'x1d8', 'x200')


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def grouped(r):
    """id → (enter, [커널 사건], leave), 캡처 순서."""
    out = {}
    for e in r['events']:
        if not oracle.is_new(e):
            continue
        k = e['id']
        if e['event'] == 'btran:enter':
            require(k not in out, f'중복 enter {k}')
            out[k] = (e, [], None)
        elif e['event'] == 'btran:kernel':
            out[k][1].append(e)
        else:
            require(out[k][2] is None, f'중복 leave {k}')
            out[k] = (out[k][0], out[k][1], e)
    for k, (en, ks, lv) in out.items():
        require(lv is not None, f'leave 누락 {k}')
    return out


def cases():
    m = oracle.load(oracle.D / 'manifest.json')
    oracle.check_files(m)
    data = oracle.records()
    oracle.controls(data, m)
    out = []
    for name in MODELS:
        g = grouped(data[name][0])
        equal(oracle.EXPECTED[name], len(g), name + '.calls')
        for k, (en, ks, lv) in g.items():
            out.append(dict(name=f'{name}/{k}', model=name, enter=en, kernels=ks, leave=lv))
    return out


def ints(xs):
    return [str(len(xs))] + [str(int(x)) for x in xs]


def hexes(xs):
    for h in xs:
        f64(h)
    return [str(len(xs))] + list(xs)


def line(c):
    e = c['enter']; f = e['factor']; a = e['args']; s = f['scalars']; m = f['m']; n = c['name']
    require(a['factor_s470'] and a['factor_frame'] and a['input_frame'] and a['output_frame'], n + '.args')
    require(not a['cache_null'] and a['cache_f1d0'], n + '.cache arg')
    equal(ZERO_LE, f['zero'], n + '.zero')
    inp = e['input']
    fields = [n, str(m), f['zero'], str(inp['count'])] + ints(inp['idx'] or []) + hexes(inp['val']) + [str(e['output_count'])]
    cache = f['cache']
    if cache is None:
        fields.append('0')
    else:
        fields += ['1', str(cache['count'])] + ints(cache['idx']) + hexes(cache['val'])
    fields += [str(f['x1d8']), e['x200']]
    p = f['perm']
    for key in ('x20', 'x30', 'x40', 'x48'):
        fields += ints(p[key])
    fields += hexes(f['diag'])
    U = f['U']
    for i in range(m):
        row = U['row'][i]
        equal(U['row_end'][i] - U['row_start'][i], len(row['cols']), f'{n}.Urow{i}')
        fields += ints(row['cols']) + hexes(row['vals'])
    fields += ints(f['prefix']['flag'])
    fields += [str(s['x100']), str(s['x104']), s['x108'], s['x110'], str(s['xd8']), str(s['xe4'])]
    lb = f['L']['beg']
    fields.append(str(lb[m] - lb[s['xe4']]))
    R = f['Lrow']; rb = R['beg']
    for i in range(m):
        lo, hi = rb[i] - rb[0], rb[i + 1] - rb[0]
        fields += ints(R['cols'][lo:hi]) + hexes(R['vals'][lo:hi])
    equal(s['x140'], R['levels'], n + '.x140')
    fields.append(str(R['levels']))
    if R['levels'] >= 0:
        fields += ints(R['lev']) + ints(R['lst']) + ints(R['bucket'])
    fields += [str(s['x1e0']), str(s['x184'])]
    equal(s['x180'], len(f['etas']), n + '.etas')
    fields.append(str(len(f['etas'])))
    for t in f['etas']:
        fields += [str(t['r'])] + ints(t['idx']) + hexes(t['val'])
    return ' '.join(fields)


def expected(c):
    """원본 커널 경계·반환(비교 대상만)."""
    ks = [{k: e[k] for k in KERNEL_KEYS if k in e} for e in c['kernels']]
    for k in ks:
        equal(k['phase'] == 'leave', 'output' in k, c['name'] + '.kernel output 유무')
    lv = c['leave']; f = lv['factor']
    return dict(kernels=ks, leave=dict(output=lv['output'], input_after=lv['input_after'], alias=lv['alias'], cache=f['cache'],
                                       x1d8=f['x1d8'], scratch=f['scratch'], bucket=f['Lrow']['bucket'], x200=lv['x200']))


def project(cs):
    body = '\n'.join(line(c) for c in cs) + '\n'
    header = SCHEMA + ' ' + hashlib.sha256(body.encode()).hexdigest()
    return header + '\n' + body, {c['name']: expected(c) for c in cs}


def first_diff(a, b, path=''):
    if type(a) is not type(b):
        return f'{path}: 형식 {a!r} != {b!r}'
    if isinstance(a, dict):
        for k in sorted(set(a) | set(b)):
            if k not in a or k not in b:
                return f'{path}.{k}: 키 누락'
            d = first_diff(a[k], b[k], f'{path}.{k}')
            if d:
                return d
        return None
    if isinstance(a, list):
        for i, (x, y) in enumerate(zip(a, b)):
            d = first_diff(x, y, f'{path}[{i}]')
            if d:
                return d
        return None if len(a) == len(b) else f'{path}: 길이 {len(a)} != {len(b)}'
    return None if a == b else f'{path}: 원본 {a!r} != Rust {b!r}'


def count_leaves(x):
    if isinstance(x, dict):
        return sum(count_leaves(v) for v in x.values())
    if isinstance(x, list):
        return sum(count_leaves(v) for v in x) or 1
    return 1


def compare(txt, want, header):
    lines = txt.splitlines()
    require(lines, '빈 trace')
    equal(header, lines[0], 'trace 입력 해시')
    got = {}
    for ln in lines[1:]:
        name, body = ln.split(';', 1)
        require(name not in got, '중복 사례 ' + name)
        got[name] = json.loads(body)
    equal(list(want), list(got), '사례 목록·순서')
    branches = Counter()
    fields = 0
    for name, w in want.items():
        g = got[name]
        require(g.get('error') is None, f'{name}: Rust 오류 {g.get("error")}')
        d = first_diff(w, {k: g[k] for k in ('kernels', 'leave')}, name)
        if d:
            raise Fail('불일치 ' + d)
        for k, v in g['branches'].items():
            branches[f'{k}:{v}'] += 1
        fields += count_leaves(w)
    return dict(calls=len(want), compared_leaves=fields, branches=dict(sorted(branches.items())),
                kernel_events=sum(len(w['kernels']) for w in want.values()))


def flip(h):
    b = bytearray.fromhex(h); b[0] ^= 1
    return b.hex()


def self_test(txt, want, header):
    """trace 변조가 모두 첫 차이로 거부되는지 확인한다."""
    results = []
    lines = txt.splitlines()

    def reject(name, new_lines):
        try:
            compare('\n'.join(new_lines) + '\n', want, header)
        except (Fail, KeyError, TypeError, ValueError) as exc:
            results.append(dict(name=name, first_difference=str(exc)))
            return
        raise Fail('NOT REJECTED ' + name)

    def edit(i, fn):
        name, body = lines[i].split(';', 1)
        g = json.loads(body); fn(g)
        out = lines.copy(); out[i] = name + ';' + json.dumps(g)
        return out

    reject('zero-target', lines[:1])
    reject('missing-case', lines[:-1])
    reject('input-hash-zero', [SCHEMA + ' ' + '0' * 64] + lines[1:])
    reject('case-order', lines[:1] + [lines[2], lines[1]] + lines[3:])
    for i, name in enumerate(want, 1):
        g0 = json.loads(lines[i].split(';', 1)[1]); lv = g0['leave']
        if lv['output']['val']:
            reject(name + '/leave.output.val', edit(i, lambda g: g['leave']['output']['val'].__setitem__(0, flip(g['leave']['output']['val'][0]))))
        if lv['output']['idx']:
            reject(name + '/leave.output.idx', edit(i, lambda g: g['leave']['output']['idx'].__setitem__(-1, g['leave']['output']['idx'][-1] + 1)))
        if len(lv['output']['idx'] or []) > 1:
            reject(name + '/leave.output.order', edit(i, lambda g: (g['leave']['output']['idx'].insert(0, g['leave']['output']['idx'].pop(1)),
                                                                    g['leave']['output']['val'].insert(0, g['leave']['output']['val'].pop(1)))))
        reject(name + '/leave.output.count', edit(i, lambda g: g['leave']['output'].__setitem__('count', g['leave']['output']['count'] + 1)))
        if lv['input_after']['val']:
            reject(name + '/leave.input_after.val', edit(i, lambda g: g['leave']['input_after']['val'].__setitem__(-1, flip(g['leave']['input_after']['val'][-1]))))
        reject(name + '/leave.input_after.count', edit(i, lambda g: g['leave']['input_after'].__setitem__('count', g['leave']['input_after']['count'] + 1)))
        reject(name + '/leave.alias', edit(i, lambda g: g['leave']['alias']['out'].__setitem__('val', 'in' if g['leave']['alias']['out']['val'] == 'out' else 'out')))
        reject(name + '/leave.cache.count', edit(i, lambda g: g['leave']['cache'].__setitem__('count', g['leave']['cache']['count'] + 1)))
        if lv['cache']['val']:
            reject(name + '/leave.cache.val', edit(i, lambda g: g['leave']['cache']['val'].__setitem__(0, flip(g['leave']['cache']['val'][0]))))
        reject(name + '/leave.x1d8', edit(i, lambda g: g['leave'].__setitem__('x1d8', g['leave']['x1d8'] + 1)))
        reject(name + '/leave.scratch', edit(i, lambda g: g['leave']['scratch']['x1b8'].append([0, '000000000000f03f'])))
        reject(name + '/leave.x200', edit(i, lambda g: g['leave'].__setitem__('x200', flip(g['leave']['x200']))))
        if lv['bucket']:
            reject(name + '/leave.bucket', edit(i, lambda g: g['leave']['bucket'].__setitem__(-1, g['leave']['bucket'][-1] + 1)))
        reject(name + '/unknown-cell', edit(i, lambda g: g['leave']['output']['val'].__setitem__(0, None)
                                         if g['leave']['output']['val'] else g['leave']['output'].__setitem__('count', None)))
        reject(name + '/error', edit(i, lambda g: g.__setitem__('error', 'injected')))
        if g0['kernels']:
            reject(name + '/kernel-missing', edit(i, lambda g: g['kernels'].pop()))
            reject(name + '/kernel-phase', edit(i, lambda g: g['kernels'][0].__setitem__('phase', 'leave')))
            reject(name + '/kernel-x1d8', edit(i, lambda g: g['kernels'][0].__setitem__('x1d8', g['kernels'][0]['x1d8'] - 1)))
            reject(name + '/kernel-x200', edit(i, lambda g: g['kernels'][-1].__setitem__('x200', flip(g['kernels'][-1]['x200']))))
            last = g0['kernels'][-1]
            if last['output']['val']:
                reject(name + '/kernel-output-val', edit(i, lambda g: g['kernels'][-1]['output']['val'].__setitem__(0, flip(g['kernels'][-1]['output']['val'][0]))))
            reject(name + '/kernel-scratch', edit(i, lambda g: g['kernels'][-1]['scratch']['x1b0'].append([0, 1])))
    return results


# (이름, 원래 문자열, 변이) — 원본과 다른 동작이 저장 자료 차이로 거부되어야 한다.
MUTATIONS = [
    ('cache-skip-empty-check', 'if f.urow[r].cols.is_empty() {', 'if false {'),
    ('cache-always-empty', 'if f.urow[r].cols.is_empty() {', 'if true {'),
    ('cache-no-x1d8', 's.x1d8 = r as i32;', 'let _ = r;'),
    ('cache-no-copy', 'n = oc;\n        }\n        s.cache.as_mut().unwrap().count = n;', 'n = oc;\n        }\n        s.cache.as_mut().unwrap().count = 0;'),
    ('general-dense-u', 'kernel_path = c >= 0 && ((f.f108 + f.f110) as i32).wrapping_mul(c) <= m as i32;', 'kernel_path = false;'),
    ('kernel-strict-direct', 'if f.upos[r] < f.t {', 'if f.upos[r] <= f.t {'),
    ('kernel-direct-nodiv', 'let y = v / f.diag[r];\n                if zero < y.abs() {\n                    emit(s, oi, ov, &mut n, r, y);\n                    let row = &f.urow[r];\n                    for (&col, &u) in row.cols.iter().zip(&row.vals) {\n                        let mk',
     'let y = v;\n                if zero < y.abs() {\n                    emit(s, oi, ov, &mut n, r, y);\n                    let row = &f.urow[r];\n                    for (&col, &u) in row.cols.iter().zip(&row.vals) {\n                        let mk'),
    ('kernel-max-heap', 'if child + 1 < hn && h(s, child + 1) < h(s, child) {', 'if child + 1 < hn && h(s, child) < h(s, child + 1) {'),
    ('kernel-never-seq', 'if ((m as i32 - top) as f64) <= hn as f64 * 6.0 {', 'if false {'),
    ('kernel-always-seq', 'if ((m as i32 - top) as f64) <= hn as f64 * 6.0 {', 'if true {'),
    ('kernel-plus', 's.vals[col] -= u * y;\n                    if mk == 0 {\n                        s.mark[col] = 1;\n                        let x', 's.vals[col] += u * y;\n                    if mk == 0 {\n                        s.mark[col] = 1;\n                        let x'),
    ('kernel-nonstrict', 'if f.zero < y.abs() {\n            let row = &f.urow[r];', 'if y != 0.0 {\n            let row = &f.urow[r];'),
    ('kernel-work-off', 's.work += acc;', 'let _ = acc;'),
    ('kernel-no-clear-mark', '            s.mark[r] = 0;\n            s.vals[r] = 0.0;\n        }\n        kernel_seq', '            s.vals[r] = 0.0;\n        }\n        kernel_seq'),
    ('eta0-no-swap', 'let i16 = s.output.idx;\n    s.output = Header { count: s.input.count, idx: i37, val: v44 };\n    s.input = Header { count: Some(count), idx: i16, val: v26 };',
     'let i16 = s.input.idx;\n    let (i37, v44, v26) = (i37, v44, v26);\n    let _ = (i37, v44, v26);'),
    ('lt-dense-always', 'if count >= 0 && f.mult * count <= m as i32 && (f.levels as f64) < m as f64 * 0.1 {', 'if false {'),
    ('lt-sparse-boundary', 'if count >= 0 && f.mult * count <= m as i32 &&', 'if count >= 0 && f.mult * count < m as i32 &&'),
    ('lt-no-relabel', 's.idx[oi].set(k, f.relabel[i] as i32);', 's.idx[oi].set(k, i as i32);'),
    ('lt-heap-min', 'if child + 1 < hn && h(s, child) < h(s, child + 1) {\n                            c2 = child + 1;',
     'if child + 1 < hn && h(s, child + 1) < h(s, child) {\n                            c2 = child + 1;'),
    ('lt-heap-never-seq', 'if hn != 1 && (top as f64) <= hn as f64 * 6.0 {', 'if false {'),
    ('lt-heap-always-seq', 'if hn != 1 && (top as f64) <= hn as f64 * 6.0 {', 'if true {'),
    ('lt-heap-strict', 'if big(zero, v) {\n                    emit(s, oi, ov, &mut n, top, v);', 'if zero < v.abs() {\n                    emit(s, oi, ov, &mut n, top, v);'),
    ('lt-level-as-heap', 'if f.levels < 0 {', 'if f.levels < 1000 {'),
    ('lt-path-b-never', 'if cnt_ge as f64 * d53 * d53 <= d55 {', 'if true {'),
    ('lt-path-b-always', 'if cnt_ge as f64 * d53 * d53 <= d55 {', 'if false {'),
    ('lt-dense-no-gather', 'let v = s.val[w].get(f.gather[k]);', 'let v = s.val[w].get(k);'),
    ('lt-dense-nonstrict', 'if f.zero < v.abs() {\n            let row = &f.lrow[i];\n            s.work', 'if v != 0.0 {\n            let row = &f.lrow[i];\n            s.work'),
    ('lt-dense-swap-out', '    s.output.idx = i35;\n    s.output.val = v44;\n    s.input.idx = i39;\n    s.input.val = v15;',
     '    s.output.idx = i39;\n    s.output.val = v15;\n    s.input.idx = i35;\n    s.input.val = v44;'),
    ('lt-heap-work-assoc', 'dv = (n as f64 * 7.0 * ((m as f64 + m as f64) + f.f1e0 as f64)) / (m + 1) as f64;',
     'dv = (n as f64 * (7.0 * ((m as f64 + m as f64) + f.f1e0 as f64))) / (m + 1) as f64;'),
    ('lt-level-work-factor', 'dv = (n as f64 * 5.0 * (f.f1e0 + m as i64 * 2) as f64) / (m + 1) as f64;',
     'dv = (n as f64 * 5.0 * (f.f1e0 + m as i64) as f64) / (m + 1) as f64;'),
    ('lt-dense-work-order', 's.work = (dv + m as f64) + m as f64;', 's.work = dv + (m as f64 + m as f64);'),
    ('lt-level-no-bucket-reset', 'for x in bucket.iter_mut().take(levels + 1) {\n                *x = 0;\n            }', 'let _ = levels;'),
]


def mutate_rust(path, want, header):
    scratch = ROOT / 'target/m3_btran_mutations'
    if scratch.exists():
        shutil.rmtree(scratch)
    scratch.mkdir(parents=True)
    for file in ('Cargo.toml', 'Cargo.lock'):
        shutil.copy2(ROOT / file, scratch / file)
    shutil.copytree(ROOT / 'crates', scratch / 'crates')
    (scratch / 'bench/oracle/m3_push_btran_rust').mkdir(parents=True)
    shutil.copy2(D / 'input.txt', scratch / 'bench/oracle/m3_push_btran_rust/input.txt')
    original = (ROOT / SOURCE).read_text(); results = []
    trace = scratch / 'target/m3_push_btran_rust/trace.txt'
    for name, old, new in MUTATIONS:
        equal(1, original.count(old), 'unique mutation ' + name)
        (scratch / SOURCE).write_text(original.replace(old, new, 1)); trace.unlink(missing_ok=True)
        proc = subprocess.run(['cargo', 'test', '-p', 'rsolver-lp', '--test', 'push_btran'], cwd=scratch,
                              text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        if proc.returncode or not trace.exists():
            results.append(dict(name=name, result='build-or-test-failed', log=proc.stdout[-2000:])); continue
        r = dict(name=name)
        try:
            compare(trace.read_text(), want, header)
            r['result'] = 'survived'
        except (Fail, KeyError, TypeError, ValueError) as exc:
            r['result'] = 'rejected'; r['first_difference'] = str(exc)
        results.append(r)
    path.write_text(json.dumps(results, ensure_ascii=False, indent=2) + '\n')
    return dict(Counter(r['result'] for r in results))


def manifest():
    files = [SOURCE, Path('crates/lp/src/lib.rs'), Path('crates/lp/tests/push_btran.rs'), Path('tools/push_btran_rust.py'),
             Path('tools/push_btran_check.py'), Path('bench/oracle/m3_push_btran/manifest.json')]
    files += [p.relative_to(ROOT) for p in sorted(D.rglob('*')) if p.is_file() and p.name != 'manifest.json']
    return dict(task='M3-04c-21d', baseline='4c99d26', binary_sha256=oracle.BINARY_SHA,
                oracle_manifest_sha256=sha(ROOT / oracle.D / 'manifest.json'), files={str(p): sha(ROOT / p) for p in files})


def main():
    p = argparse.ArgumentParser(description=__doc__)
    g = p.add_mutually_exclusive_group(required=True)
    for flag in ('write', 'compare', 'self-test'):
        g.add_argument('--' + flag, action='store_true')
    p.add_argument('--mutate-rust', type=Path)
    p.add_argument('--write-manifest', action='store_true')
    a = p.parse_args(); cs = cases(); source, want = project(cs); header = source.splitlines()[0]
    D.mkdir(parents=True, exist_ok=True)
    if a.write:
        (D / 'input.txt').write_text(source)
        result = dict(written=len(cs))
    else:
        equal(source, (D / 'input.txt').read_text(), 'fixed input-only projection')
        txt = TRACE.read_text(); result = compare(txt, want, header)
        result['models'] = dict(Counter(c['model'] for c in cs))
        if a.mutate_rust:
            result['source_mutations'] = mutate_rust(a.mutate_rust, want, header)
        elif a.self_test:
            rejected = self_test(txt, want, header)
            (D / 'self-test.json').write_text(json.dumps(rejected, ensure_ascii=False, indent=2) + '\n')
            result['rejected'] = len(rejected)
        else:
            (D / 'comparison.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
        if a.write_manifest:
            (D / 'trace.txt').write_text(txt)
            (D / 'manifest.json').write_text(json.dumps(manifest(), ensure_ascii=False, indent=2) + '\n')
    print(json.dumps(result, ensure_ascii=False))


if __name__ == '__main__':
    try:
        main()
    except (Fail, KeyError, TypeError, ValueError, IndexError) as exc:
        sys.exit('FAIL ' + str(exc))
