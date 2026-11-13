#!/usr/bin/env python3
"""M3-04f-4a: Rust `crossover_dual`(251b4f + 21d BTRAN)·S+0x2d8 재계산·쌍대 검사를 원본 캡처와 비트 비교한다.

  python3 tools/xo_dual_rust.py --write            # 원본 run 1의 dv:enter·btran:enter 상태 → bench/oracle/m3_xo_dual_rust/input.txt.gz
  cargo test --release -p rsolver-lp --test xo_dual   # → target/m3_xo_dual_rust/trace.txt
  python3 tools/xo_dual_rust.py --compare          # 각 사례 3회 기대값과 비교
  python3 tools/xo_dual_rust.py --self-test        # trace 변조 거부
  python3 tools/xo_dual_rust.py --mutate-rust OUT  # Rust 소스 변이(임시 사본) 거부/생존
  python3 tools/xo_dual_rust.py --coverage         # 생존 변이 분기의 원본 자료 출현 수

입력 평문에는 진입 상태만 넣는다(원본 반환·y·d·S+0x2d8·r14는 이 비교기만 읽는다).
"""
import argparse, collections, copy, gzip, hashlib, json, os, shutil, subprocess, sys
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import xo_dual_check as oracle  # noqa: E402
from xo_dual_check import Fail, require  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'bench/oracle/m3_xo_dual_rust'
INPUT = OUT / 'input.txt.gz'
TRACE = ROOT / 'target/m3_xo_dual_rust/trace.txt'
SCHEMA = 'M3-04f-4a-rust-v1'
ZERO64 = '0000000000000000'


def ints(xs):
    return [str(len(xs))] + [str(int(x)) for x in xs]


def hexes(xs):
    for h in xs:
        require(isinstance(h, str) and len(h) == 16, f'16진 값 아님 {h!r}')
    return [str(len(xs))] + list(xs)


def opt(h):
    return ['0'] if h is None else ['1', h]


def factor_fields(f, name):
    """tools/push_btran_rust.py line()의 factor 순서(relabel부터)."""
    m, s = f['m'], f['scalars']
    out = []
    for key in ('x20', 'x30', 'x40', 'x48'):
        out += ints(f['perm'][key])
    out += hexes(f['diag'])
    U = f['U']
    for i in range(m):
        row = U['row'][i]
        require(U['row_end'][i] - U['row_start'][i] == len(row['cols']), f'{name}.Urow{i}')
        out += ints(row['cols']) + hexes(row['vals'])
    out += ints(f['prefix']['flag'])
    out += [str(s['x100']), str(s['x104']), s['x108'], s['x110'], str(s['xd8']), str(s['xe4'])]
    lb = f['L']['beg']
    out.append(str(lb[m] - lb[s['xe4']]))
    R = f['Lrow']; rb = R['beg']
    for i in range(m):
        lo, hi = rb[i] - rb[0], rb[i + 1] - rb[0]
        out += ints(R['cols'][lo:hi]) + hexes(R['vals'][lo:hi])
    require(s['x140'] == R['levels'], name + '.x140')
    out.append(str(R['levels']))
    if R['levels'] >= 0:
        out += ints(R['lev']) + ints(R['lst']) + ints(R['bucket'])
    out += [str(s['x1e0']), str(s['x184'])]
    require(s['x180'] == len(f['etas']), name + '.etas')
    out.append(str(len(f['etas'])))
    for t in f['etas']:
        out += [str(t['r'])] + ints(t['idx']) + hexes(t['val'])
    return out


def row_fields(A, m):
    start = [A['start'][i] - A['base'] if A['len'][i] > 0 else 0 for i in range(m)]
    for i in range(m):
        require(A['len'][i] == 0 or 0 <= start[i] and start[i] + A['len'][i] <= len(A['cols']), f'행 {i} 범위')
    return ints(start) + ints(A['len']) + ints(A['cols']) + hexes(A['vals'])


def line(name, k, A):
    e, be = k['enter'], k['bt_enter']
    n, m = e['n'], e['m']
    a = be['args']
    require(a['factor_s470'] and a['input_s220'] and a['output_s228'] and a['cache_null'], name + '.btran 인자')
    require(be['input']['count'] == -1, name + '.btran 입력 count')
    require(be['x200'] == ZERO64, name + '.F200 진입')
    f = be['factor']
    require(not f['scratch']['x1b0'] and not f['scratch']['x1b8'], name + '.scratch 진입이 0 아님')
    require(e['f1f8'] == e['s148'], name + '.F1f8 진입 = S+0x148')
    require(len(e['cost']) == n + m and len(e['vstat']) == n + m and len(e['header']) == m, name + '.길이')
    corr = []
    for kk, j, v in e['sub']:
        corr += [str(j), v]
    fields = [name, str(n), str(m)] + hexes(e['cost']) + ints(e['vstat']) + ints(e['header']) + [str(len(e['sub']))] + corr
    fields += [e['tol3a8']] + row_fields(A, m) + opt(e['work'])
    fields += [str(be['output_count']), str(f['x1d8']), f['zero']] + factor_fields(f, name)
    if k['site'] == 'post':
        require(k['sel']['vstat'] == e['vstat'], name + '.sel vstat')
        fields += opt(k['sel']['tol380'])
    else:
        fields += opt(None)
    if k['site'] == 'dualcheck':
        fields += opt(k['sel']['opttol_slot'])
    else:
        fields += opt(None)
    return ' '.join(fields)


def expected(k):
    lv, bl = k['leave'], k['bt_leave']
    want = dict(rhs=k['bt_enter']['input']['val'],
                btran=dict(output=bl['output'], input_after=bl['input_after'], alias=bl['alias'], x200=bl['x200'], scratch=bl['scratch']),
                y=lv['y'], d=lv['d'], work=lv['work'],
                s2d8=k['sel']['s2d8'] if k['site'] == 'post' else None,
                r14=k['decide']['r14'] if k['site'] == 'dualcheck' else None)
    return want


def build():
    m = oracle.load_manifest()
    oracle.check_files(m)
    data = oracle.records(m)
    oracle.controls(data)
    lines, wants, sites = [], {}, {}
    for case in sorted(data):
        rs = data[case]
        per_run = [oracle.calls(r) for r in rs]
        A = None
        for idx, k in enumerate(per_run[0]):
            e = k['enter']
            if e['rowA'] is not None:
                require(A is None, case + ': 행 행렬 두 번')
                A = e['rowA']
            else:
                require(e['a_same'] is True and A is not None, case + ': 행 행렬 변경')
            # 같은 run의 f1f8 복원 규칙(C: F1f8 = S+0x148)은 원본 자료로 확인한다.
            for calls in per_run:
                require(calls[idx]['leave']['f1f8'] == calls[idx]['enter']['s148'], case + ': F1f8 복원')
            name = f'{case}#{e["id"]}'
            lines.append(line(name, k, A))
            wants[name] = [expected(calls[idx]) for calls in per_run]
            sites[name] = k['site']
    body = '\n'.join(lines) + '\n'
    return SCHEMA + ' ' + hashlib.sha256(body.encode()).hexdigest() + '\n' + body, wants, sites


def first_diff(a, b, path=''):
    if type(a) is not type(b):
        return f'{path}: 형식 {a!r:.60} != {b!r:.60}'
    if isinstance(a, dict):
        for k in sorted(set(a) | set(b)):
            if k not in a or k not in b:
                return f'{path}.{k}: 키 누락'
            d = first_diff(a[k], b[k], f'{path}.{k}')
            if d:
                return d
        return None
    if isinstance(a, list):
        if len(a) != len(b):
            return f'{path}: 길이 {len(a)} != {len(b)}'
        for i, (x, y) in enumerate(zip(a, b)):
            d = first_diff(x, y, f'{path}[{i}]')
            if d:
                return d
        return None
    return None if a == b else f'{path}: 원본 {a!r} Rust {b!r}'


def parse_trace(text):
    ls = text.rstrip('\n').split('\n')
    got = {}
    for l in ls[1:]:
        name, body = l.split(';', 1)
        require(name not in got, '중복 trace ' + name)
        got[name] = json.loads(body)
    return ls[0], got


def count_fields(x):
    if isinstance(x, dict):
        return sum(count_fields(v) for v in x.values())
    if isinstance(x, list):
        return sum(count_fields(v) for v in x) if x else 1
    return 1


def compare(text, wants, sites, verbose=True):
    head, got = parse_trace(text)
    require(head == gzip.decompress(INPUT.read_bytes()).decode().split('\n', 1)[0], 'trace 머리줄이 입력과 다름')
    require(set(got) == set(wants), f'trace 사례 불일치 {sorted(set(got) ^ set(wants))[:3]}')
    errs, nfields, runs = [], 0, 0
    branches = collections.Counter()
    for name in sorted(wants):
        g = got[name]
        if g.get('error') is not None:
            errs.append(f'{name}: Rust 오류 {g["error"]}')
            continue
        br = g.pop('branches', None)
        if br:
            branches.update(f'{k}={v}' for k, v in br.items())
        for r, w in enumerate(wants[name], 1):
            runs += 1
            sub = {k: g[k] for k in w}
            d = first_diff(w, sub)
            if d:
                errs.append(f'{name} run {r} [{sites[name]}]{d}')
            nfields += count_fields(w)
    if verbose:
        for e in errs[:20]:
            print('차이', e)
        print(f'사례 호출 {len(wants)}, 비교 실행 {runs}, 필드 {nfields}, 차이 {len(errs)}')
        print('BTRAN 분기', dict(sorted(branches.items())))
        print('지점', dict(collections.Counter(sites.values())))
    return errs, len(wants), nfields


def self_test():
    _, wants, sites = build()
    text = TRACE.read_text()
    errs, _, _ = compare(text, wants, sites, verbose=False)
    require(not errs, '기준 비교가 먼저 일치해야 함')
    head, got = parse_trace(text)

    def emit(g):
        return head + '\n' + '\n'.join(f'{k};{json.dumps(v)}' for k, v in g.items()) + '\n'

    def flip(h):
        return format(int(h, 16) ^ 1, '016x')

    def pick(site, pred=lambda g: True):
        return next(k for k in sorted(got) if sites[k] == site and pred(got[k]))

    post, dual = pick('post'), pick('dualcheck', lambda g: g['r14'] == 1)
    corr = pick('dpush-a')
    muts = [
        ('rhs', corr, lambda g: g['rhs'].__setitem__(0, flip(g['rhs'][0]))),
        ('btran output', post, lambda g: g['btran']['output']['val'].__setitem__(0, flip(g['btran']['output']['val'][0]))),
        ('btran input_after', post, lambda g: g['btran']['input_after'].__setitem__('count', 7)),
        ('btran alias', post, lambda g: g['btran']['alias']['out'].__setitem__('val', 'in' if g['btran']['alias']['out']['val'] == 'out' else 'out')),
        ('btran x200', post, lambda g: g['btran'].__setitem__('x200', flip(g['btran']['x200']))),
        ('btran scratch', post, lambda g: g['btran']['scratch']['x1b0'].append([0, 1])),
        ('y', post, lambda g: g['y'].__setitem__(-1, flip(g['y'][-1]))),
        ('d 열', post, lambda g: g['d'].__setitem__(0, flip(g['d'][0]))),
        ('d 끝', dual, lambda g: g['d'].__setitem__(-1, flip(g['d'][-1]))),
        ('work', post, lambda g: g.__setitem__('work', flip(g['work'][1:-1] if False else g['work']))),
        ('s2d8', post, lambda g: g.__setitem__('s2d8', flip(g['s2d8']))),
        ('r14', dual, lambda g: g.__setitem__('r14', 0)),
        ('Rust 오류', post, lambda g: g.__setitem__('error', 'panic')),
    ]
    bad = 0
    for label, name, f in muts:
        g = copy.deepcopy(got)
        f(g[name])
        e2, _, _ = compare(emit(g), wants, sites, verbose=False)
        if e2:
            print('거부', label, '-', e2[0][:140])
        else:
            bad += 1
            print('생존', label)
    g = copy.deepcopy(got)
    g.pop(post)
    try:
        compare(emit(g), wants, sites, verbose=False)
        bad += 1
        print('생존 사례 삭제')
    except Fail as e:
        print('거부 사례 삭제 -', e)
    require(not bad, f'변조 생존 {bad}')
    print('self-test 통과', len(muts) + 1)


S = 'crates/lp/src/crossover_dual.rs'
MUTATIONS = [
    ('rhs-no-corr', S, '        cb[p] -= v;\n', '        let _ = (p, v);\n'),
    ('rhs-corr-add', S, '        cb[p] -= v;\n', '        cb[p] += v;\n'),
    ('rhs-identity-order', S, '(0..e.m).map(|i| e.cost[e.header[i] as usize])', '(0..e.m).map(|i| e.cost[e.header[e.m - 1 - i] as usize])'),
    ('drop-ge', S, '        if y[i].abs() > e.drop {', '        if y[i].abs() >= e.drop {'),
    ('drop-none', S, '        if y[i].abs() > e.drop {', '        if true {'),
    ('acc-fma', S, '                d[c] -= y[i] * e.a.vals[k];', '                d[c] = y[i].mul_add(-e.a.vals[k], d[c]);'),
    ('rows-reverse', S, '    for i in 0..m {\n        d[n + i] = -y[i];', '    for i in (0..m).rev() {\n        d[n + i] = -y[i];'),
    ('slack-sign', S, '        d[n + i] = -y[i];', '        d[n + i] = y[i];'),
    ('no-flip', S, '        if e.vstat[j] == -2 {\n            d[j] = -d[j];', '        if false {\n            d[j] = -d[j];'),
    ('flip-m1', S, '        if e.vstat[j] == -2 {\n            d[j] = -d[j];', '        if e.vstat[j] == -1 {\n            d[j] = -d[j];'),
    ('flip-slacks', S, '    for j in 0..n {\n        if e.vstat[j] == -2 {', '    for j in 0..n + m {\n        if e.vstat[j] == -2 {'),
    ('work-skip', S, '    f200 * 1.0 + work', '    work'),
    ('y-other-buffer', S, '        Buffer::In => 0,\n        Buffer::Out => 1,', '        Buffer::In => 1,\n        Buffer::Out => 0,'),
    ('inf-tol-ge', S, '        if tol > v {\n            continue;\n        }\n        nb += v;', '        if tol >= v {\n            continue;\n        }\n        nb += v;'),
    ('inf-no-tol', S, '        if tol > v {\n            continue;\n        }\n        nb += v;', '        nb += v;'),
    ('inf-no-basic', S, '            b += dj.abs();\n', '\n'),
    ('inf-m4-as-m1', S, '        let v = if s < -3 {\n            0.0', '        let v = if s < -4 {\n            0.0'),
    ('inf-m3-counted', S, '        if s == -3 {\n            continue;\n        }\n        let v', '        let v'),
    ('inf-sign', S, '            let t = -dj;', '            let t = dj;'),
]


def mutate_rust(out_path):
    out_path = Path(out_path)
    require(not out_path.exists(), f'{out_path} 이미 있음')
    _, wants, sites = build()
    work = ROOT / 'target/m3_xo_dual_mutations'
    if work.exists():
        shutil.rmtree(work)
    src = work / 'src'
    shutil.copytree(ROOT, src, ignore=shutil.ignore_patterns('target', '.git', 'bench', 'python', 'docs'))
    os.symlink(ROOT / 'bench', src / 'bench')
    env = dict(os.environ, CARGO_TARGET_DIR=str(work / 'target'))
    results = []
    for name, f, a, b in MUTATIONS:
        p = src / f
        orig = p.read_text()
        if orig.count(a) != 1:
            results.append(dict(name=name, result='원문 없음'))
            print(name, '원문 없음')
            continue
        p.write_text(orig.replace(a, b))
        tr = src / 'target/m3_xo_dual_rust/trace.txt'
        if tr.exists():
            tr.unlink()
        try:
            r = subprocess.run(['cargo', 'test', '-q', '--release', '-p', 'rsolver-lp', '--test', 'xo_dual'],
                               cwd=src, env=env, capture_output=True, text=True, timeout=1800)
            if not tr.exists():
                res = dict(result='컴파일·실행 실패', detail=r.stderr[-300:])
            else:
                errs, _, _ = compare(tr.read_text(), wants, sites, verbose=False)
                res = dict(result='거부' if errs else '생존', first=errs[0][:200] if errs else None, count=len(errs))
        finally:
            p.write_text(orig)
        results.append(dict(name=name, **res))
        print(name, res['result'], res.get('first') or '')
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(results, ensure_ascii=False, indent=1) + '\n')
    c = collections.Counter(r['result'] for r in results)
    print('변이', len(results), dict(c))


def f64(h):
    import struct
    return struct.unpack('<d', bytes.fromhex(h))[0]


def coverage():
    """생존 변이 분기의 원본 자료 출현 수(run 1, 원본 값으로 계산)."""
    m = oracle.load_manifest()
    oracle.check_files(m)
    data = oracle.records(m)
    c = collections.Counter()
    for case in sorted(data):
        for k in oracle.calls(data[case][0]):
            e, lv = k['enter'], k['leave']
            n, mm = e['n'], e['m']
            drop = f64(e['tol3a8'])
            ys = [f64(h) for h in lv['y']]
            c['|y| = drop'] += sum(abs(y) == drop for y in ys)
            c['|y| > drop'] += sum(abs(y) > drop for y in ys)
            c['slack vstat -2'] += sum(v == -2 for v in e['vstat'][n:])
            c['column vstat -2'] += sum(v == -2 for v in e['vstat'][:n])
            if k['site'] == 'post':
                tol = f64(k['sel']['tol380'])
                for dh, v in zip(lv['d'], e['vstat']):
                    if v == -3:
                        c['post vstat -3'] += 1
                        t = -f64(dh)
                        c['post vstat -3 with v >= tol'] += (0.0 if 0.0 > t else t) >= tol
                    elif v in (-1, -2):
                        t = -f64(dh)
                        vv = 0.0 if 0.0 > t else t
                        c['post v = tol'] += vv == tol
                        c['post v > tol'] += vv > tol
                    elif v == -4:
                        c['post vstat -4'] += 1
    for k2, v in sorted(c.items()):
        print(f'{k2}: {v}')


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--write', action='store_true')
    ap.add_argument('--compare', action='store_true')
    ap.add_argument('--self-test', action='store_true')
    ap.add_argument('--mutate-rust')
    ap.add_argument('--coverage', action='store_true')
    a = ap.parse_args()
    if a.coverage:
        coverage()
    if a.write:
        text, wants, _ = build()
        OUT.mkdir(parents=True, exist_ok=True)
        INPUT.write_bytes(gzip.compress(text.encode(), mtime=0))
        print('입력', len(wants), '호출', len(text) // 1024, 'KiB')
    if a.compare:
        _, wants, sites = build()
        require(TRACE.exists(), f'{TRACE} 없음')
        errs, _, _ = compare(TRACE.read_text(), wants, sites)
        require(not errs, f'차이 {len(errs)}')
    if a.self_test:
        self_test()
    if a.mutate_rust:
        mutate_rust(a.mutate_rust)
    return 0


if __name__ == '__main__':
    try:
        sys.exit(main())
    except Fail as e:
        print('실패:', e)
        sys.exit(1)
