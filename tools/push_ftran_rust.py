"""M3-04c-21b: 한 RHS FTRAN Rust adapter를 21a 원본 캡처의 커널 경계·raw·입력 후상태·spike·scratch와 비트 비교한다.

입력 평문에는 `ftran:enter`의 factor·입력·spike·인자만 넣는다. 원본 커널/반환 사건은 이 비교기만 읽는다.
"""
import argparse
from collections import Counter
import copy
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

import push_ftran_check as oracle
from push_supplier_check import Fail, equal, require

ROOT = Path(__file__).resolve().parents[1]
D = ROOT / 'bench/oracle/m3_push_ftran_rust'
TRACE = ROOT / 'target/m3_push_ftran_rust/trace.txt'
SOURCE = Path('crates/lp/src/crossover_ftran.rs')
SCHEMA = 'M3-04c-21b-v1'
MODELS = ('arms', 'retry', 'interior')


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def grouped(r):
    """id → (enter, [kernel 사건], leave) in capture order."""
    out = {}
    for e in r['events']:
        if not oracle.is_new(e):
            continue
        k = e['id']
        if e['event'] == 'ftran:enter':
            require(k not in out, f'중복 enter {k}')
            out[k] = (e, [], None)
        elif e['event'] == 'ftran:kernel':
            out[k][1].append(e)
        else:
            out[k] = (out[k][0], out[k][1], e)
    for k, (en, ks, lv) in out.items():
        require(lv is not None, f'leave 누락 {k}')
    return out


def cases():
    data, m = oracle.records(), oracle.load(oracle.D / 'manifest.json')
    oracle.check_files(m)
    oracle.controls(data, m)
    out = []
    for name in MODELS:
        for k, (en, ks, lv) in grouped(data[name][0]).items():
            out.append(dict(name=f'{name}/{k}', model=name, enter=en, kernels=ks, leave=lv))
    return out


def ints(xs):
    return [str(len(xs))] + [str(int(x)) for x in xs]


def hexes(xs):
    for h in xs:
        oracle.f64(h)
    return [str(len(xs))] + list(xs)


def line(c):
    e = c['enter']; f = e['factor']; a = e['args']; s = f['scalars']; m = f['m']
    equal(0, a['k5'], c['name'] + '.k5'); require(a['p6_null'], c['name'] + '.p6')
    require(a['vals_null'] and a['mark_null'], c['name'] + '.scratch args')
    require(a['save'] == 0 or a['spike_f1c8'] or a['spike_null'], c['name'] + '.spike F1c8')
    equal(oracle.ZERO_LE, f['zero'], c['name'] + '.zero')
    sp = f['spike']
    fields = [c['name'], str(m), f['zero'], str(a['save'])]
    if sp is None:
        fields.append('0')
    else:
        fields += ['1', str(sp['count'])] + ints(sp['indices']) + hexes(sp['values'])
    inp = e['input']
    fields += [str(inp['count'])] + ints(inp['idx'] or []) + hexes(inp['val'])
    p = f['perm']
    for key in ('x20', 'x30', 'x40', 'x48'):
        fields += ints(p[key])
    fields += hexes(f['diag'])
    beg = f['L']['beg']
    fields += ints([x - beg[0] for x in beg]) + ints(f['L']['rows']) + hexes(f['L']['vals'])
    U = f['U']
    for j in range(m):
        col = U['col'][j]
        equal(U['col_end'][j] - U['col_start'][j], len(col['rows']), f'{c["name"]}.U{j}')
        fields += [str(U['col_bound'][j] - U['col_start'][j])] + ints(col['rows']) + hexes(col['vals'])
    fields += [str(s['xd8']), str(s['xe4']), str(s['x100']), str(s['x104']), s['x108'], s['x110'], str(s['x128'])]
    equal(s['xec'], len(f['prefix']['list']), c['name'] + '.ec')
    fields += ints(f['prefix']['list']) + ints(f['prefix']['flag']) + ints(f['eta_oldest'])
    equal(s['x180'], len(f['etas']), c['name'] + '.etas')
    fields.append(str(len(f['etas'])))
    for t in f['etas']:
        require(t['val_mod16'] in (0, 8), c['name'] + '.eta align')
        fields += [str(t['r']), str(int(t['val_mod16'] == 8))] + ints(t['idx']) + hexes(t['val'])
    return ' '.join(fields)


def expected(c):
    """원본 커널 경계·반환(비교 대상만)."""
    ks = [{key: k[key] for key in ('kernel', 'phase', 'input', 'output', 'alias', 'spike') if key in k}
          for k in c['kernels']]
    for k in ks:
        equal(k['phase'] == 'leave', 'output' in k, c['name'] + '.kernel output 유무')
    lv = c['leave']
    return dict(kernels=ks, leave=dict(output=lv['output'], input_after=lv['input_after'], alias=lv['alias'],
                                       spike=lv['factor']['spike'], scratch=lv['factor']['scratch']))


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


def count_leaves(x):
    if isinstance(x, dict):
        return sum(count_leaves(v) for v in x.values())
    if isinstance(x, list):
        return sum(count_leaves(v) for v in x) or 1
    return 1


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
    names = list(want)
    # 사례마다: 출력 값 1비트, 출력 count, 입력 후상태, 별칭, spike, scratch, 커널 사건 누락/순서/이름/단계
    for i, name in enumerate(names, 1):
        g0 = json.loads(lines[i].split(';', 1)[1])
        if g0['leave']['output']['val']:
            reject(name + '/leave.output.val', edit(i, lambda g: g['leave']['output']['val'].__setitem__(
                0, flip(g['leave']['output']['val'][0]))))
        reject(name + '/leave.output.count', edit(i, lambda g: g['leave']['output'].__setitem__(
            'count', g['leave']['output']['count'] + 1)))
        if g0['leave']['input_after']['val']:
            reject(name + '/leave.input_after.val', edit(i, lambda g: g['leave']['input_after']['val'].__setitem__(
                -1, flip(g['leave']['input_after']['val'][-1]))))
        if g0['leave']['input_after']['idx']:
            reject(name + '/leave.input_after.idx', edit(i, lambda g: g['leave']['input_after']['idx'].__setitem__(
                -1, g['leave']['input_after']['idx'][-1] + 1)))
        reject(name + '/leave.alias', edit(i, lambda g: g['leave']['alias']['out'].__setitem__(
            'val', 'in' if g['leave']['alias']['out']['val'] == 'out' else 'out')))
        if g0['leave']['spike'] is not None:
            reject(name + '/leave.spike.count', edit(i, lambda g: g['leave']['spike'].__setitem__(
                'count', g['leave']['spike']['count'] + 1)))
            if g0['leave']['spike']['values']:
                reject(name + '/leave.spike.value', edit(i, lambda g: g['leave']['spike']['values'].__setitem__(
                    0, flip(g['leave']['spike']['values'][0]))))
        reject(name + '/scratch', edit(i, lambda g: g['leave']['scratch']['x1b0'].append([0, 1])))
        reject(name + '/unknown-cell', edit(i, lambda g: g['leave']['output']['val'].__setitem__(0, None)
                                         if g['leave']['output']['val'] else g['leave']['output'].__setitem__('count', None)))
        reject(name + '/error', edit(i, lambda g: g.__setitem__('error', 'injected')))
        if g0['kernels']:
            reject(name + '/kernel-missing', edit(i, lambda g: g['kernels'].pop()))
            reject(name + '/kernel-name', edit(i, lambda g: g['kernels'][0].__setitem__('kernel', 'xxxxxx')))
            reject(name + '/kernel-phase', edit(i, lambda g: g['kernels'][0].__setitem__('phase', 'leave')))
            reject(name + '/kernel-input-count', edit(i, lambda g: g['kernels'][0]['input'].__setitem__(
                'count', (g['kernels'][0]['input']['count'] or 0) + 1)))
            last = g0['kernels'][-1]
            if last.get('output') and last['output']['val']:
                reject(name + '/kernel-output-val', edit(i, lambda g: g['kernels'][-1]['output']['val'].__setitem__(
                    0, flip(g['kernels'][-1]['output']['val'][0]))))
            if len(g0['kernels']) > 2:
                reject(name + '/kernel-order', edit(i, lambda g: g['kernels'].insert(0, g['kernels'].pop(2))))
    return results


# (이름, 원래 문자열, 변이) — 원본과 다른 동작이 저장 자료 차이로 거부되어야 한다.
MUTATIONS = [
    ('relabel-skip', 's.idx[ii].set(k, f.relabel[i as usize] as i32);', 's.idx[ii].set(k, i);'),
    ('gather-identity', 'let v = s.val[iv].get(f.gather[i]);', 'let v = s.val[iv].get(i);'),
    ('dense-l-always', 'if (m as i64) < c as i64 * f.mult || m as f64 * 0.1 <= f.levels as f64 {', 'if true {'),
    ('sparse-l-boundary', 'if (m as i64) < c as i64 * f.mult ||', 'if (m as i64) <= c as i64 * f.mult ||'),
    ('l-dense-skip-d8', 'for k in (0..f.d8).chain(f.e4..m) {', 'for k in f.e4..m {'),
    ('l-dense-nonstrict', 'if f.zero < xk.abs() {', 'if f.zero <= xk.abs() && xk != 0.0 {'),
    ('l-dense-plus', 's.val[ov].set(i, x - f.lval[e] * xk);', 's.val[ov].set(i, x + f.lval[e] * xk);'),
    ('kahn-lifo', 'let j = s.idx[oi].get(p) as usize;\n        p += 1;\n        let v = std::mem::take(&mut s.vals[j]);',
     'let j = s.idx[oi].get(q - 1 - p) as usize;\n        p += 1;\n        let v = std::mem::take(&mut s.vals[j]);'),
    ('kahn-keep-small', 'let big = f.zero <= v || v <= -f.zero;', 'let big = v != 0.0;'),
    ('r-no-swap', '        let oc = s.input.count;\n        s.swap_headers();\n        s.output.count = oc;',
     '        let oc = s.input.count;\n        s.output.count = oc;'),
    ('eta-sequential', 'if terms.len() < 8 || terms.len() < p + 8 {', 'if true {'),
    ('eta-no-drop', 's.val[iv].set(e.row, if v.abs() <= f.zero { 0.0 } else { v });', 's.val[iv].set(e.row, v);'),
    ('eta-reverse', 'for e in &f.etas[from..] {', 'for e in f.etas[from..].iter().rev() {'),
    ('spike-nonstrict', 'if f.zero < v.abs() {\n                sp.idx.set(k, i as i32);', 'if v != 0.0 {\n                sp.idx.set(k, i as i32);'),
    ('spike-unscaled', 'sp.val.set(i as usize, v * sv);', 'sp.val.set(i as usize, v);'),
    ('spike-dense-unscaled', 'sp.val.set(i, v * sv);', 'sp.val.set(i, -v);'),
    ('u-dense-always', 'if n >= 0 && (te < m as i64 ||', 'if false && (te < m as i64 ||'),
    ('u-sparse-always', 'if n >= 0 && (te < m as i64 ||', 'if true || (te < m as i64 ||'),
    ('u-dense-no-drop', 'if x.abs() <= f.zero || (x / d).abs() <= f.zero {', 'if x == 0.0 {'),
    ('u-dense-scatter-prefix', '        if p >= f.t {\n            let col = &f.ucol[i];', '        if p + 1 >= f.t {\n            let col = &f.ucol[i];'),
    ('u-dense-multiply', 'let y = x / d;\n        s.val[ov].set(i, y);', 'let y = x * d;\n        s.val[ov].set(i, y);'),
    ('u-sparse-ascending', 'for p in (f.t..m).rev() {\n                let i = f.uord[p];\n                if s.mark[i]',
     'for p in f.t..m {\n                let i = f.uord[p];\n                if s.mark[i]'),
    ('u-sparse-direct-nodiv', 'let y = v / f.diag[i];\n            if zero < y.abs() {\n                emit(s, i, y, &mut out);\n            }\n        } else if',
     'let y = v;\n            if zero < y.abs() {\n                emit(s, i, y, &mut out);\n            }\n        } else if'),
    ('u-sparse-no-input-compaction', 's.idx[ii].set(ntail, i as i32);', 'let _ = ntail;'),
    ('u-sparse-h-always', 'if d2 * ((npre + ntail) as f64) <= (ec + m - f.t) as f64 {', 'if false {'),
    ('input-swap-dense', '        s.output.count = Some(c);\n        std::mem::swap(&mut s.input.idx, &mut s.output.idx);',
     '        s.output.count = Some(c);'),
]


def mutate_rust(path, want, header):
    scratch = ROOT / 'target/m3_ftran_mutations'
    if scratch.exists():
        shutil.rmtree(scratch)
    scratch.mkdir(parents=True)
    for file in ('Cargo.toml', 'Cargo.lock'):
        shutil.copy2(ROOT / file, scratch / file)
    shutil.copytree(ROOT / 'crates', scratch / 'crates')
    (scratch / 'bench/oracle/m3_push_ftran_rust').mkdir(parents=True)
    shutil.copy2(D / 'input.txt', scratch / 'bench/oracle/m3_push_ftran_rust/input.txt')
    original = (ROOT / SOURCE).read_text(); results = []
    trace = scratch / 'target/m3_push_ftran_rust/trace.txt'
    for name, old, new in MUTATIONS:
        equal(1, original.count(old), 'unique mutation ' + name)
        (scratch / SOURCE).write_text(original.replace(old, new, 1)); trace.unlink(missing_ok=True)
        proc = subprocess.run(['cargo', 'test', '-p', 'rsolver-lp', '--test', 'push_ftran'], cwd=scratch,
                              text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        if proc.returncode or not trace.exists():
            results.append(dict(name=name, result='build-or-test-failed', log=proc.stdout[-2000:])); continue
        try:
            compare(trace.read_text(), want, header)
        except (Fail, KeyError, TypeError, ValueError) as exc:
            results.append(dict(name=name, result='rejected', first_difference=str(exc)))
        else:
            results.append(dict(name=name, result='survived'))
    path.write_text(json.dumps(results, ensure_ascii=False, indent=2) + '\n')
    return dict(Counter(r['result'] for r in results))


def manifest():
    files = [SOURCE, Path('crates/lp/src/lib.rs'), Path('crates/lp/tests/push_ftran.rs'), Path('tools/push_ftran_rust.py'),
             Path('tools/push_ftran_check.py'), Path('bench/oracle/m3_push_ftran/manifest.json')]
    files += [p.relative_to(ROOT) for p in sorted(D.rglob('*')) if p.is_file() and p.name != 'manifest.json']
    return dict(task='M3-04c-21b', baseline='40e1280', binary_sha256=oracle.BINARY_SHA,
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
        (D / 'input.txt').write_text(source); result = dict(written=len(cs))
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
