#!/usr/bin/env python3
"""M3-04c-21i: Rust `036ade` 조밀 전환을 원본 캡처(bench/oracle/m3_push_dense)와 비교한다.

  --write           캡처 검사(controls·acceptance) 뒤 (A) unit.txt와 (B) refactor.txt 입력을 쓴다
  --compare         target/m3_push_dense_rust/의 trace를 원본과 비교한다(첫 차이에서 종료 1)
  --self-test       기대값 변조가 모두 거부되는지 확인한다
  --mutate-rust P   Rust 소스 변이마다 임시 Cargo 사본에서 재생·비교해 결과를 P(새 파일)에 쓴다

(A) 조밀 LU 경계: dense:lu(행렬·목록·ZERO·정렬·첫 시도) → Rust dense_lu → dense:done(행렬 비트·목록 정확).
(B) 재분해 생성: 21h `--block` 입력 규칙(push_lu_rust.program, 생성·블록 모드)으로 5모델 push 재분해 전부를 Rust가 만들고
    036ade 반환 순열·factor 전체·블록 상태를 원본과 비교한다. 조밀 재분해는 Rust의 전환 단계·n·정렬·조밀 순서·입력 행렬이
    dense:enter·dense:lu와 같아야 하고, 원본이 조밀로 가지 않은 재분해에서 Rust도 가지 않아야 한다.
"""
import argparse, gzip, hashlib, json, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
D = ROOT / 'bench/oracle/m3_push_dense_rust'
T = ROOT / 'target/m3_push_dense_rust'
SCHEMA = 'M3-04c-21i-v1'


class Fail(Exception):
    pass


def require(c, msg):
    if not c:
        raise Fail(msg)


def load(p):
    b = p.read_bytes()
    return json.loads(gzip.decompress(b) if p.suffix == '.gz' else b)


def records():
    import push_dense_check as chk
    return chk.checked()


def unit_cases(caps):
    """(A) 사례: 각 모델 run 1의 dense:lu와 바로 다음 dense:done 짝(push 안·밖 모두)."""
    out = []
    for name, cap in caps.items():
        ev = cap['events']
        k = 0
        for i, e in enumerate(ev):
            if e['event'] != 'dense:lu':
                continue
            j = i + 1
            while ev[j]['event'] != 'dense:done':
                require(ev[j]['event'] not in ('dense:lu',), f'{name}#{i}: dense:done 없이 dense:lu')
                j += 1
            out.append((name, k, e, ev[j]))
            k += 1
    return out


def unit_line(name, k, e):
    n = e['n']
    return ' '.join([name, str(k), str(n), e['zero'], str(e['mat_mod16']), str(e['scratch_mod16']), str(e['first'])]
                    + [str(x) for x in e['cols']] + [str(x) for x in e['rows']] + list(e['a']))


def program(caps):
    """(B) 입력: 21h 생성·블록 모드 연산열(M·K·G). 블록 상태 표는 이 캡처의 lu:block에서 읽는다."""
    import push_lu_rust as plr
    import push_block_check as pbc
    plr.use_block()

    import push_dense_check as chk

    def brows(name):
        # 풀 부족 재호출이 있는 재분해는 첫 진입·마지막 반환으로 두고, 그 재분해부터는 비교기가 범위 밖으로 센다
        return chk.block_table(caps[name], name)
    plr.block_rows = brows
    out = []
    for name in caps:
        out += [(name,) + x for x in plr.program(name, caps[name], True)]
    return out


def dense_by_lu(cap):
    """push 재분해 lu id → (dense:enter, dense:lu)."""
    out = {}
    ev = cap['events']
    for i, e in enumerate(ev):
        if e['event'] == 'dense:enter' and e['push']:
            lu = next(x for x in ev[i:] if x['event'] == 'dense:lu')
            out[e['lu']] = (e, lu)
    return out


def write():
    caps = records()
    us = unit_cases(caps)
    D.mkdir(parents=True, exist_ok=True)
    head = f'{SCHEMA} ' + hashlib.sha256((ROOT / 'bench/oracle/m3_push_dense/manifest.json').read_bytes()).hexdigest()
    (D / 'unit.txt').write_text('\n'.join([head] + [unit_line(n, k, e) for n, k, e, _ in us]) + '\n')
    print(f'unit: {len(us)} 조밀 LU', {n: sum(1 for x in us if x[0] == n) for n in caps})
    cs = program(caps)
    (D / 'input.txt').write_text('\n'.join([head] + [' '.join(c[3]) for c in cs]) + '\n')
    from collections import Counter
    print(f'refactor input: {len(cs)} 연산', dict(Counter(c[1] for c in cs)))


def compare_chain(caps, trace=None, verbose=True):
    import push_lu_rust as plr
    cs = program(caps)
    head = (D / 'input.txt').read_text().splitlines()[0]
    fields, fails, _, _ = plr.compare(trace or (T / 'trace.txt'), cs, head, verbose=False)
    got = plr.read_trace(trace or (T / 'trace.txt'), head)
    import push_dense_check as chk
    first_bad = {n: min(chk.out_of_scope(cap, n), default=None) for n, cap in caps.items()}
    lu_of = {}
    for c in cs:
        name, op, i = c[0], c[1], c[2]
        if op == 'G':
            lu_of[(name, i)] = next(x for x in caps[name]['events'][i:] if x['event'] == 'lu:refactor' and x['phase'] == 'leave')['id']

    def outside(name, ev):
        """계약 §1(i): 범위 밖 재분해(풀 부족 재호출·local_70)와 그 뒤의 같은 push 연산."""
        fb = first_bad[name]
        return fb is not None and lu_of.get((name, ev), 0) >= fb
    dense_ok = 0
    for c, (name, ev, g) in zip(cs, got):
        cname, op, i, toks, want = c
        if op != 'G' or g.get('error') is not None:
            continue
        lu = lu_of[(name, i)]
        orig = dense_by_lu(caps[name]).get(lu)
        rd = g.get('dense')
        if orig is None:
            if rd is not None:
                fails.append(f'{name}#{ev} G.dense: 원본은 조밀 전환 없음, Rust는 단계 {rd["steps"]} n {rd["n"]}')
            continue
        e, dl = orig
        if rd is None:
            fails.append(f'{name}#{ev} G.dense: 원본 조밀 전환(단계 {e["steps"]}, n {e["n"]}) 없음')
            continue
        for lab, w, r in (('steps', e['steps'], rd['steps']), ('n', e['n'], rd['n']), ('mat_mod16', dl['mat_mod16'], rd['mat_mod16']),
                          ('rows', dl['rows'], rd['rows']), ('cols', dl['cols'], rd['cols']), ('a', dl['a'], rd['a'])):
            if w != r:
                pos = next((p for p, (x, y) in enumerate(zip(w, r)) if x != y), None) if isinstance(w, list) else None
                fails.append(f'{name}#{ev} G.dense.{lab}' + (f'[{pos}]: 원본 {w[pos]} != Rust {r[pos]}' if pos is not None else f': 원본 {w if not isinstance(w, list) else len(w)} != Rust {r if not isinstance(r, list) else len(r)}'))
                break
        else:
            dense_ok += 1
            fields += 6 + 2 * e['n'] + e['n'] ** 2
    key = lambda f: (f.split('#')[0], int(f.split('#')[1].split(' ')[0]))
    expected = [f for f in fails if outside(*key(f))]
    bad = [f for f in fails if not outside(*key(f))]
    failed = {key(f) for f in fails}
    passed = [(c[0], c[2], c[1]) for c in cs if (c[0], c[2]) not in failed and not outside(c[0], c[2])]
    if verbose:
        from collections import Counter
        ops = Counter(c[1] for c in cs)
        print(f'(B) 연산 {dict(ops)}: 범위 안 통과 {dict(Counter(p[2] for p in passed))}, 조밀 재분해 일치 {dense_ok}, 필드 {fields}')
        print(f'    범위 밖(계약 §1(i), 첫 lu {first_bad}): 연산 {sum(1 for c in cs if outside(c[0], c[2]))}, 그중 실패 {len(expected)}')
        for f in expected[:3]:
            print('    범위 밖 첫 차이:', f)
        for f in bad[:30]:
            print('  FAIL', f)
    return fields, bad, dense_ok


def compare_unit(caps, trace=None, verbose=True):
    us = unit_cases(caps)
    lines = (trace or (T / 'unit-trace.txt')).read_text().splitlines()
    head = (D / 'unit.txt').read_text().splitlines()[0]
    require(lines[0] == head, f'unit trace 머리 {lines[0]!r} != {head!r}')
    require(len(lines) - 1 == len(us), f'unit trace 줄 {len(lines) - 1} != {len(us)}')
    fields = 0
    for (name, k, e, done), ln in zip(us, lines[1:]):
        t = ln.split(' ')
        n = e['n']
        require(t[0] == name and t[1] == str(k), f'unit 순서 {t[:2]} != {name} {k}')
        require(t[2] == '0', f'{name}#{k}: Rust 반환 {t[2]} (원본 결함 0, 0xde68c ecx {done["ecx"]})')
        cols, rows, a = t[3:3 + n], t[3 + n:3 + 2 * n], t[3 + 2 * n:]
        want_cols = [str(x) for x in done['cols']]
        want_rows = [str(x) for x in done['rows']]
        for lab, w, g in (('cols', want_cols, cols), ('rows', want_rows, rows), ('a', done['a'], a)):
            require(len(w) == len(g), f'{name}#{k}.{lab}: 길이 {len(w)} != {len(g)}')
            for p, (x, y) in enumerate(zip(w, g)):
                if x != y:
                    raise Fail(f'{name}#{k} (n={n}, mat16={e["mat_mod16"]}).{lab}[{p}] (행 {p % n}, 열 {p // n}): 원본 {x} != Rust {y}')
            fields += len(w)
    if verbose:
        print(f'(A) 조밀 LU {len(us)}회 일치, 필드 {fields}')
    return fields


def self_test(caps):
    """trace 변조가 (A)·(B) 비교에서 모두 거부되는지 확인한다."""
    import copy
    ut = (T / 'unit-trace.txt').read_text().splitlines()
    ct = (T / 'trace.txt').read_text().splitlines()
    tmp = T / 'self-test'
    tmp.mkdir(parents=True, exist_ok=True)

    def unit_mut(fn):
        lines = copy.copy(ut); fn(lines); (tmp / 'u.txt').write_text('\n'.join(lines) + '\n')
        compare_unit(caps, tmp / 'u.txt', verbose=False)

    def chain_mut(fn):
        lines = copy.copy(ct); fn(lines); (tmp / 'c.txt').write_text('\n'.join(lines) + '\n')
        _, fails, _ = compare_chain(caps, tmp / 'c.txt', verbose=False)
        require(not fails, f'{len(fails)} 불일치: {fails[:1]}')

    def tok(lines, i, j, fn):
        t = lines[i].split(' '); t[j] = fn(t[j]); lines[i] = ' '.join(t)

    def bit(h):
        b = bytearray(bytes.fromhex(h)); b[0] ^= 1
        return b.hex()

    def dense_lines(lines):
        return [i for i, ln in enumerate(lines) if '"dense":' in ln]

    def gedit(lines, i, fn):
        name, ev, js = lines[i].split(' ', 2)
        g = json.loads(js); fn(g); lines[i] = f'{name} {ev} ' + json.dumps(g, separators=(',', ':'))

    n0 = int(ut[1].split(' ')[2])
    muts = [
        ('A 행렬 1비트', unit_mut, lambda L: tok(L, 1, 3 + 2 * n0 + 7, bit)),
        ('A 열 목록 교환', unit_mut, lambda L: (lambda t: (t.__setitem__(slice(3, 5), [t[4], t[3]]), L.__setitem__(3, ' '.join(t))))(L[3].split(' '))),
        ('A 결함 수', unit_mut, lambda L: tok(L, 2, 2, lambda x: '1')),
        ('A 줄 누락', unit_mut, lambda L: L.pop()),
        ('A 머리', unit_mut, lambda L: L.__setitem__(0, L[0] + 'x')),
        ('B 조밀 단계', chain_mut, lambda L: gedit(L, dense_lines(L)[0], lambda g: g['dense'].update(steps=g['dense']['steps'] + 1))),
        ('B 조밀 정렬', chain_mut, lambda L: gedit(L, dense_lines(L)[1], lambda g: g['dense'].update(mat_mod16=8 - g['dense']['mat_mod16']))),
        ('B 조밀 행 순서', chain_mut, lambda L: gedit(L, dense_lines(L)[2], lambda g: g['dense']['rows'].reverse())),
        ('B 조밀 입력 1비트', chain_mut, lambda L: gedit(L, dense_lines(L)[3], lambda g: g['dense']['a'].__setitem__(0, bit(g['dense']['a'][0])))),
        ('B 조밀 없음', chain_mut, lambda L: gedit(L, dense_lines(L)[0], lambda g: g.pop('dense'))),
        ('B 원본에 없는 조밀', chain_mut, lambda L: gedit(L, next(i for i, ln in enumerate(L) if '"op":"G"' in ln and '"dense"' not in ln),
                                                       lambda g: g.update(dense={'steps': 0, 'n': 0, 'mat_mod16': 0, 'rows': [], 'cols': [], 'a': []}))),
        ('B 조밀 재분해 factor 대각', chain_mut, lambda L: gedit(L, dense_lines(L)[0], lambda g: g['factor']['diag'].__setitem__(-1, bit(g['factor']['diag'][-1])))),
        ('B 조밀 재분해 x20', chain_mut, lambda L: gedit(L, dense_lines(L)[4], lambda g: g['perm']['x20'].__setitem__(-1, g['perm']['x20'][-2]))),
    ]
    ok = 0
    for name, run, fn in muts:
        try:
            run(fn)
        except (Fail, Exception) as e:  # noqa: B014 - 비교 중 예외도 거부로 센다
            ok += 1
            print(f'  거부: {name}: {str(e)[:120]}')
            continue
        raise Fail(f'변조 통과: {name}')
    print(f'trace 변조 {ok}종 모두 거부')


SOURCE = Path('crates/lp/src/lu.rs')
MUTATIONS = [
    ('dense-off(필수)', 'if let Some(cap) = self.dense_cap {', 'if let Some(cap) = self.dense_cap.filter(|_| false) {'),
    ('gate 0.36→0.4', 'n2 * 0.36 < est', 'n2 * 0.4 < est'),
    ('gate 0.325→0.36', 'n2 * 0.325 < est', 'n2 * 0.36 < est'),
    ('gate n>50→n>=50', 'nact > 50 &&', 'nact >= 50 &&'),
    ('gate 4m 제거', 'mf * 4.0 < n2 && ', ''),
    ('추정 −2 제거', '(ccs[c] as f64 - 1.0) - 2.0;', '(ccs[c] as f64 - 1.0);'),
    ('재계수 생략', 'est = ccs.iter().filter(|&&c| c >= 0).map(|&c| (c + 1) as f64).sum();', 'let _ = &ccs;'),
    ('행 bucket nxt', 'x = cb.prv[x];', 'x = cb.nxt[x];'),
    ('행 bucket 역순', 'for k in 0..=m {\n            if rows.len() == n {', 'for k in (0..=m).rev() {\n            if rows.len() == n {'),
    ('열 역순', 'let cols: Vec<usize> = (0..m).filter(|&i| rcs[i] >= 0).collect();', 'let cols: Vec<usize> = (0..m).rev().filter(|&i| rcs[i] >= 0).collect();'),
    ('peel 정렬 무시', 'usize::from(!(mat16 + 8 * c * n).is_multiple_of(16))', 'usize::from(!(8 * c * n).is_multiple_of(16))'),
    ('정렬 짝홀 반전', '% 2 != 0 { 8 } else { 0 };', '% 2 != 0 { 0 } else { 8 };'),
    ('블록 1단계 단일 누적', 'let end = k - ((k - peel) & 1);', 'let end = peel;'),
    ('블록 2단계 단일 누적', 'let end = j - ((j - peel) & 1);', 'let end = peel;'),
    ('블록 피벗 동률 마지막', '                    if best < x {\n                        best = x;\n                        bi = t;', '                    if best <= x {\n                        best = x;\n                        bi = t;'),
    ('블록 열 목록 교환 생략', '                    cols.swap(bi, p);\n', ''),
    ('꼬리 8-lane ((A+B)+C)+D', 'let x0 = (l[0][0] + l[1][0]) + (l[2][0] + l[3][0]);', 'let x0 = ((l[0][0] + l[1][0]) + l[2][0]) + l[3][0];'),
    ('꼬리 피벗 동률 마지막', '            if best < x {\n                best = x;\n                bi = t;', '            if best <= x {\n                best = x;\n                bi = t;'),
    ('꼬리 시작 0', '        c0 = j;', '        c0 = 0;'),
    ('scratch peel 반전', 'usize::from(!scr16.is_multiple_of(16))', 'usize::from(scr16.is_multiple_of(16))'),
    ('변환 ZERO 버림 제거', '            if v.abs() < ZERO {\n                *v = 0.0;', '            if v.abs() < 0.0 {\n                *v = 0.0;'),
    ('L 열 역순', 'self.lcol[s0 + k] = (k + 1..n)', 'self.lcol[s0 + k] = (k + 1..n).rev()'),
    ('U 행 역순', 'self.kernel_u[s0 + k] = (k + 1..n)', 'self.kernel_u[s0 + k] = (k + 1..n).rev()'),
]


def mutate_rust(path, caps):
    import shutil, subprocess
    from collections import Counter
    require(not path.exists(), f'{path} 이미 있음')
    scratch = ROOT / 'target/m3_dense_mutations'
    if scratch.exists():
        shutil.rmtree(scratch)
    scratch.mkdir(parents=True)
    for f in ('Cargo.toml', 'Cargo.lock'):
        shutil.copy2(ROOT / f, scratch / f)
    shutil.copytree(ROOT / 'crates', scratch / 'crates')
    (scratch / 'bench/oracle/m3_push_dense_rust').mkdir(parents=True)
    for f in ('unit.txt', 'input.txt'):
        shutil.copy2(D / f, scratch / 'bench/oracle/m3_push_dense_rust' / f)
    original = (ROOT / SOURCE).read_text()
    results = []
    for name, old, new in MUTATIONS:
        require(original.count(old) == 1, f'변이 원문이 하나가 아님: {name} ({original.count(old)})')
        (scratch / SOURCE).write_text(original.replace(old, new, 1))
        tr = scratch / 'target/m3_push_dense_rust'
        shutil.rmtree(tr, ignore_errors=True)
        log = ''
        ok = True
        for test, fn in (('push_dense', 'dense_unit_trace'), ('push_lu', 'dense_chain_trace')):
            try:
                proc = subprocess.run(['cargo', 'test', '--release', '-p', 'rsolver-lp', '--test', test, fn], cwd=scratch, text=True,
                                      stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=900)
            except subprocess.TimeoutExpired:
                ok = False; log = 'timeout'; break
            log += proc.stdout[-800:]
        first = None
        if not ok or not (tr / 'unit-trace.txt').exists() or not (tr / 'trace.txt').exists():
            result = 'replay-failed'
            first = log[-300:]
        else:
            try:
                compare_unit(caps, tr / 'unit-trace.txt', verbose=False)
                _, fails, _ = compare_chain(caps, tr / 'trace.txt', verbose=False)
                require(not fails, f'(B) {len(fails)}: {fails[0] if fails else ""}')
                result = 'survived'
            except Fail as e:
                result, first = 'rejected', str(e)
        results.append(dict(name=name, result=result, first=first))
        print(f'  {name}: {result} {(first or "")[:140]}', flush=True)
    path.write_text(json.dumps(results, ensure_ascii=False, indent=2) + '\n')
    print(dict(Counter(r['result'] for r in results)))


def main():
    sys.path.insert(0, str(ROOT / 'tools'))
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument('--write', action='store_true')
    g.add_argument('--compare', action='store_true')
    g.add_argument('--self-test', action='store_true')
    g.add_argument('--mutate-rust', type=Path)
    a = ap.parse_args()
    if a.write:
        write()
    elif a.self_test:
        self_test(records())
    elif a.mutate_rust:
        mutate_rust(a.mutate_rust, records())
    else:
        caps = records()
        compare_unit(caps)
        _, fails, _ = compare_chain(caps)
        require(not fails, f'(B) {len(fails)}개 불일치: 첫 {fails[0] if fails else None}')


if __name__ == '__main__':
    try:
        main()
    except Fail as exc:
        print('FAIL', exc, file=sys.stderr)
        sys.exit(1)
