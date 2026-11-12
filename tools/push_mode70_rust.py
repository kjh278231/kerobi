#!/usr/bin/env python3
"""M3-04c-21j: Rust `036ade` 풀 계수·풀 부족 재호출·local_70 모드를 원본 캡처(bench/oracle/m3_push_mode70)와 비교한다.

  --write           캡처 검사(controls·acceptance) 뒤 재분해 입력 input.txt를 쓴다
  --compare         target/m3_push_mode70_rust/trace.txt를 원본과 비교한다(불일치가 있으면 종료 1)
  --self-test       기대값 변조가 모두 거부되는지 확인한다
  --mutate-rust P   Rust 소스 변이마다 임시 Cargo 사본에서 재생·비교해 결과를 P(새 파일)에 쓴다

(A) 풀 계수: push 재분해 036ade 호출마다 Rust 풀 사건(start·mode70·short70·dense·end)의 값 = 원본 pool:* (정수 정확).
(B) 재분해 생성: 21h `--block` 입력 규칙(push_lu_rust.program, 생성·블록 모드)으로 5모델 push 재분해 전부를 Rust가 만들고
    036ade 반환 순열·factor 전체·블록 상태를 원본과 비교한다(21i의 범위 밖 예외 없음). 추가로 Rust 호출 수·풀 크기 =
    원본 호출, local_70 모드 피벗(조밀 평탄 번호) = mode70:pivot, 조밀 전환 단계·n·정렬·순서·입력 행렬 = dense:enter·dense:lu.
"""
import argparse, hashlib, json, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
D = ROOT / 'bench/oracle/m3_push_mode70_rust'
T = ROOT / 'target/m3_push_mode70_rust'
SCHEMA = 'M3-04c-21j-v1'
FIELDS = ('b8', 'c0', 'f8', 'x138', 'p2', 'p3', 'p4')


class Fail(Exception):
    pass


def require(c, msg):
    if not c:
        raise Fail(msg)


def records():
    import push_mode70_check as chk
    return chk.checked()


def program(caps):
    """21h 생성·블록 모드 연산열(M·K·G). 블록 상태 표는 이 캡처의 lu:block(재호출은 첫 진입·마지막 반환)."""
    import push_lu_rust as plr
    import push_dense_check as pdc
    plr.use_block()
    plr.block_rows = lambda name: pdc.block_table(caps[name], name)
    out = []
    for name in caps:
        out += [(name,) + x for x in plr.program(name, caps[name], True)]
    return out


def write():
    caps = records()
    D.mkdir(parents=True, exist_ok=True)
    head = f'{SCHEMA} ' + hashlib.sha256((ROOT / 'bench/oracle/m3_push_mode70/manifest.json').read_bytes()).hexdigest()
    cs = program(caps)
    (D / 'input.txt').write_text('\n'.join([head] + [' '.join(c[3]) for c in cs]) + '\n')
    from collections import Counter
    print(f'refactor input: {len(cs)} 연산', dict(Counter(c[1] for c in cs)))


def original(cap, name):
    """push 재분해 lu id → 036ade 호출별 (풀 사건, 모드 피벗) 목록과 조밀 (dense:enter, dense:lu)."""
    import push_mode70_check as chk
    out = {}
    for call in chk.calls(cap, name):
        if not call['push']:
            continue
        pool = [e for e in call['ev'] if e['event'].startswith('pool:')]
        piv = [e['idx'] for e in call['ev'] if e['event'] == 'mode70:pivot']
        n70 = next((e['n'] for e in pool if e['event'] == 'pool:mode70'), None)
        out.setdefault(call['lu'], []).append((pool, piv, n70))
    dense = {}
    ev = cap['events']
    for i, e in enumerate(ev):
        if e['event'] == 'dense:enter' and e['push']:
            dense[e['lu']] = (e, next(x for x in ev[i:] if x['event'] == 'dense:lu'))
    return out, dense


def first_list_diff(w, r):
    return next((p for p, (x, y) in enumerate(zip(w, r)) if x != y), min(len(w), len(r)))


def compare_chain(caps, trace=None, verbose=True):
    import push_lu_rust as plr
    cs = program(caps)
    tr = trace or (T / 'trace.txt')
    head = (D / 'input.txt').read_text().splitlines()[0]
    fields, fails, _, _ = plr.compare(tr, cs, head, verbose=False)
    got = plr.read_trace(tr, head)
    orig = {n: original(caps[n], n) for n in caps}
    stat = {'G': 0, 'calls': 0, 'pool_events': 0, 'retry': 0, 'mode70': 0, 'pivots': 0, 'dense': 0}
    for c, (name, ev, g) in zip(cs, got):
        cname, op, i, toks, want = c
        if op != 'G' or g.get('error') is not None:
            continue
        lu = next(x for x in caps[name]['events'][i:] if x['event'] == 'lu:refactor' and x['phase'] == 'leave')['id']
        calls, dense = orig[name][0].get(lu, []), orig[name][1].get(lu)
        tag = f'{name}#{ev} (lu {lu})'
        stat['G'] += 1
        rp, sizes = g.get('pool'), g.get('sizes')
        if rp is None or sizes is None:
            fails.append(f'{tag} G.pool: Rust 출력 없음')
            continue
        if len(rp) != len(calls) or len(sizes) != len(calls):
            fails.append(f'{tag} G.pool: 036ade 호출 원본 {len(calls)} != Rust {len(rp)}(크기 {sizes})')
            continue
        bad = None
        for k, ((pool, piv, n70), rcall) in enumerate(zip(calls, rp)):
            wk = [e['event'][5:] for e in pool]
            rk = [e['kind'] for e in rcall]
            if wk != rk:
                bad = f'호출 {k} 사건 원본 {wk} != Rust {rk}'
                break
            if pool and sizes[k] != pool[0]['p2']:
                bad = f'호출 {k} 크기 원본 {pool[0]["p2"]} != Rust {sizes[k]}'
                break
            for e, r in zip(pool, rcall):
                keys = FIELDS + (('nodes',) if e['event'] == 'pool:start' else ('n',) if e['event'] == 'pool:mode70' else ())
                d = [f'{x} 원본 {e[x]} != Rust {r[x]}' for x in keys if e[x] != r[x]]
                if d:
                    bad = f'호출 {k} {e["event"]}: ' + ', '.join(d)
                    break
                stat['pool_events'] += 1
            if bad:
                break
        if bad:
            fails.append(f'{tag} G.pool: {bad}')
            continue
        stat['calls'] += len(calls)
        stat['retry'] += len(calls) - 1
        m70 = [(piv, n70) for pool, piv, n70 in calls if piv]
        rm = g.get('mode70')
        if not m70:
            if rm is not None:
                fails.append(f'{tag} G.mode70: 원본 모드 없음, Rust 단계 {rm["step"]} n {rm["n"]}')
                continue
        else:
            require(len(m70) == 1, f'{tag}: 모드 피벗이 있는 호출 {len(m70)}')
            piv, n70 = m70[0]
            if rm is None:
                fails.append(f'{tag} G.mode70: 원본 모드(n {n70}) 없음')
                continue
            if rm['n'] != n70 or rm['pivots'] != piv:
                p = first_list_diff(piv, rm['pivots'])
                fails.append(f'{tag} G.mode70: n 원본 {n70} Rust {rm["n"]}, 피벗[{p}] 원본 {piv[p] if p < len(piv) else None} '
                             f'!= Rust {rm["pivots"][p] if p < len(rm["pivots"]) else None}')
                continue
            stat['mode70'] += 1
            stat['pivots'] += len(piv)
            fields += 1 + len(piv)
        rd = g.get('dense')
        if dense is None:
            if rd is not None:
                fails.append(f'{tag} G.dense: 원본은 조밀 전환 없음, Rust는 단계 {rd["steps"]} n {rd["n"]}')
            continue
        e, dl = dense
        if rd is None:
            fails.append(f'{tag} G.dense: 원본 조밀 전환(단계 {e["steps"]}, n {e["n"]}) 없음')
            continue
        for lab, w, r in (('steps', e['steps'], rd['steps']), ('n', e['n'], rd['n']), ('mat_mod16', dl['mat_mod16'], rd['mat_mod16']),
                          ('rows', dl['rows'], rd['rows']), ('cols', dl['cols'], rd['cols']), ('a', dl['a'], rd['a'])):
            if w != r:
                fails.append(f'{tag} G.dense.{lab}: 원본과 다름')
                break
        else:
            stat['dense'] += 1
            fields += 6 + 2 * e['n'] + e['n'] ** 2
    fields += stat['pool_events'] * 8
    if verbose:
        from collections import Counter
        failed = {(f.split('#')[0], int(f.split('#')[1].split(' ')[0])) for f in fails}
        passed = Counter(c[1] for c in cs if (c[0], c[2]) not in failed)
        print(f'(B) 연산 {dict(Counter(c[1] for c in cs))}: 통과 {dict(passed)}, 필드 {fields}')
        print(f'(A) 풀: {stat}')
        for f in fails[:30]:
            print('  FAIL', f)
    return fields, fails, stat


def self_test(caps):
    """trace 변조가 비교에서 모두 거부되는지 확인한다."""
    import copy
    ct = (T / 'trace.txt').read_text().splitlines()
    tmp = T / 'self-test'
    tmp.mkdir(parents=True, exist_ok=True)

    def chain_mut(fn):
        lines = copy.copy(ct); fn(lines); (tmp / 'c.txt').write_text('\n'.join(lines) + '\n')
        _, fails, _ = compare_chain(caps, tmp / 'c.txt', verbose=False)
        require(not fails, f'{len(fails)} 불일치: {fails[:1]}')

    def gl(lines, key):
        return [i for i, ln in enumerate(lines) if key in ln]

    def gedit(lines, i, fn):
        name, ev, js = lines[i].split(' ', 2)
        g = json.loads(js); fn(g); lines[i] = f'{name} {ev} ' + json.dumps(g, separators=(',', ':'))

    def bit(h):
        b = bytearray(bytes.fromhex(h)); b[0] ^= 1
        return b.hex()
    m70 = lambda L: [i for i in gl(L, '"mode70":{')]
    retry = lambda L: [i for i in gl(L, '"op":"G"') if json.loads(L[i].split(' ', 2)[2]).get('sizes', [0]).__len__() > 1]
    muts = [
        ('풀 start c0', lambda L: gedit(L, gl(L, '"op":"G"')[0], lambda g: g['pool'][0][0].update(c0=g['pool'][0][0]['c0'] + 1))),
        ('풀 start nodes', lambda L: gedit(L, gl(L, '"op":"G"')[3], lambda g: g['pool'][0][0].update(nodes=g['pool'][0][0]['nodes'] - 1))),
        ('풀 end b8', lambda L: gedit(L, gl(L, '"op":"G"')[5], lambda g: g['pool'][0][-1].update(b8=g['pool'][0][-1]['b8'] - 1))),
        ('풀 사건 누락', lambda L: gedit(L, gl(L, '"op":"G"')[2], lambda g: g['pool'][0].pop())),
        ('재호출 없음', lambda L: gedit(L, retry(L)[0], lambda g: (g['pool'].pop(0), g['sizes'].pop(0)))),
        ('재호출 크기', lambda L: gedit(L, retry(L)[0], lambda g: g['sizes'].__setitem__(1, g['sizes'][1] + 1))),
        ('모드 피벗 교환', lambda L: gedit(L, m70(L)[0], lambda g: (lambda p: p.__setitem__(slice(3, 5), p[4:2:-1]))(g['mode70']['pivots']))),
        ('모드 없음', lambda L: gedit(L, m70(L)[1], lambda g: g.update(mode70=None))),
        ('원본에 없는 모드', lambda L: gedit(L, gl(L, '"mode70":null')[0], lambda g: g.update(mode70={'step': 0, 'n': 1, 'pivots': [0]}))),
        ('모드 n', lambda L: gedit(L, m70(L)[2], lambda g: g['mode70'].update(n=g['mode70']['n'] + 1))),
        ('모드 재분해 대각', lambda L: gedit(L, m70(L)[0], lambda g: g['factor']['diag'].__setitem__(-1, bit(g['factor']['diag'][-1])))),
        ('모드 재분해 x20', lambda L: gedit(L, m70(L)[2], lambda g: g['perm']['x20'].__setitem__(-1, g['perm']['x20'][-2]))),
        ('조밀 단계', lambda L: gedit(L, gl(L, '"dense":{')[0], lambda g: g['dense'].update(steps=g['dense']['steps'] + 1))),
        ('조밀 없음', lambda L: gedit(L, gl(L, '"dense":{')[1], lambda g: g.pop('dense'))),
    ]
    ok = 0
    for name, fn in muts:
        try:
            chain_mut(fn)
        except (Fail, Exception) as e:  # noqa: B014 - 비교 중 예외도 거부로 센다
            ok += 1
            print(f'  거부: {name}: {str(e)[:120]}')
            continue
        raise Fail(f'변조 통과: {name}')
    print(f'trace 변조 {ok}종 모두 거부')


MUTATIONS = [
    ('lu.rs', '모드 끔(필수)', 'if self.dense_cap.is_some() {\n                    let nf = nact as f64;', 'if self.dense_cap.is_some() && false {\n                    let nf = nact as f64;'),
    ('lu.rs', '풀 부족 끔(필수)', '            if (avail as f64) < l40 as i32 as f64 + q ||', '            if false && (avail as f64) < l40 as i32 as f64 + q ||'),
    ('lu.rs', '판정 201→200', '((nact as i64 - 201) as u32) <= 0x2646', '((nact as i64 - 200) as u32) <= 0x2647'),
    ('lu.rs', '판정 0.1→0.2', 'nf * 0.1 < cidx as f64 * ridx as f64', 'nf * 0.2 < cidx as f64 * ridx as f64'),
    ('lu.rs', '판정 0.25→0.3', 'est < nf * nf * 0.25', 'est < nf * nf * 0.3'),
    ('lu.rs', '새 크기 1.4→1.5', 'let g = (pl.p2 + l30) as f64 * 1.4;', 'let g = (pl.p2 + l30) as f64 * 1.5;'),
    ('lu.rs', 'iVar67 식', 'let mut iv67 = (nn - 3 * n2 as i32) / (2 * n2 as i32);', 'let mut iv67 = (nn - 2 * n2 as i32) / (2 * n2 as i32);'),
    ('lu.rs', '표지값 대신 0', '                        a[r2 * n + c2] = sent;', '                        a[r2 * n + c2] = 0.0;'),
    ('lu.rs', '1e-100 → 0', '        const TINY: f64 = 1e-100;', '        const TINY: f64 = 0.0;'),
    ('lu.rs', '모드 비용 동률 무시', '|| (cost == bcost && ratio * mx < v)) {\n                                    best = (x + rr * n) as i64;\n                                    ratio = v / mx;\n                                    bcost = cost;\n                                }\n                            }\n                        } else {',
     ') {\n                                    best = (x + rr * n) as i64;\n                                    ratio = v / mx;\n                                    bcost = cost;\n                                }\n                            }\n                        } else {'),
    ('lu.rs', '모드 열 목록 동률 무시', '|| (cost == bcost && ratio * mx < v)) {\n                                    cov[4] += usize::from(cost == bcost);\n                                    best = (x + n * rr) as i64;', ') {\n                                    cov[4] += usize::from(cost == bcost);\n                                    best = (x + n * rr) as i64;'),
    ('lu.rs', '모드 행 목록 동률 무시', '|| (cost == bcost && mx * ratio < v)) {\n                                    cov[4] += usize::from(cost == bcost);\n                                    best = (c + n * y) as i64;', ') {\n                                    cov[4] += usize::from(cost == bcost);\n                                    best = (c + n * y) as i64;'),
    ('lu.rs', '모드 피벗 뒤 열 max 유지', '            for &(c2, _) in &urow {\n                cache[c2] = 0.0;', '            for &(c2, _) in &urow {\n                let _ = &cache;'),
    ('lu.rs', '모드 재계수 생략', '                est = cnt_c.iter().filter(|&&c| c >= 0).map(|&c| (c + 1) as f64).sum();\n                cov[10] += 1;', '                cov[10] += 1;'),
    ('lu.rs', '모드 추정 −2 제거', 'est += (cnt_r[r] - 1) as f64 * (cnt_c[c] - 1) as f64 - 2.0;', 'est += (cnt_r[r] - 1) as f64 * (cnt_c[c] - 1) as f64;'),
    ('lu.rs', '모드 탐색 한도 4→5', 'let limit = nact.min(4);', 'let limit = nact.min(5);'),
    ('lu.rs', '모드 bucket 0 제외', 'let cmin = (0..n).find(|&k| cb.head[k] >= 0).unwrap_or(n);', 'let cmin = (1..n).find(|&k| cb.head[k] >= 0).unwrap_or(n);'),
    ('lu.rs', '모드 head 회전 생략', '                            cb.head[k] = if (best % ni) as usize == x { x as i64 } else { nx as i64 };', '                            let _ = nx;'),
    ('lu.rs', '모드 행 우선 탐색 순서(열 prv→nxt)', '                        x = cb.prv[x];\n                        if x == h {', '                        x = cb.nxt[x];\n                        if x == h {'),
    ('lu.rs', '모드 새 비영 목록 추가 생략', '                                if clen[c2] < cap {\n                                    clist[c2 * cap + clen[c2]] = r2;', '                                if false {\n                                    clist[c2 * cap + clen[c2]] = r2;'),
    ('lu.rs', '모드 행 순서 역순', '            for &e in st.rl[i].iter().rev() {', '            for &e in st.rl[i].iter() {'),
    ('lu.rs', '모드 L 값 역수 곱', '                let l = v / pv;\n                lc.push((rowmap[r2], l));', '                let l = v * (1.0 / pv);\n                lc.push((rowmap[r2], l));'),
    ('lu.rs', 'chunk 4m → 4m+4', '                let slots = 16 * self.m;', '                let slots = 16 * self.m + 16;'),
    ('lu.rs', '정수 쪽 chunk 무시', '                if e != 0 {\n                    self.pend = e;\n                }\n                self.p3 += slots;', '                let _ = e;\n                self.p3 += slots;'),
    ('lu.rs', 'start L 계수 −1', '        let mut pl = Pool { m: m as i64, p2: cap, p4: cap - 1, f8, b8: cap - 1 - l,', '        let mut pl = Pool { m: m as i64, p2: cap, p4: cap - 1, f8, b8: cap - l,'),
    ('lu.rs', 'U 계수 생략(모드)', '                    urow.push((c2, v));\n                    self.pool_put_u();', '                    urow.push((c2, v));'),
    ('crossover_refactor.rs', '재호출 크기 ×1.5 고정', '            Some(crate::lu::Short::Grow(c)) => alloc.x158 = c,', '            Some(crate::lu::Short::Grow(_)) => alloc.x158 = (alloc.x158 as f64 * 1.5) as i64,'),
    ('crossover_refactor.rs', '재호출 블록 상태 되돌림', '        sizes.push(alloc.x158);\n', '        sizes.push(alloc.x158);\n        if sizes.len() > 1 {\n            block.x4e0 -= 1;\n        }\n'),
]


def mutate_rust(path, caps):
    import shutil, subprocess
    from collections import Counter
    require(not path.exists(), f'{path} 이미 있음')
    scratch = ROOT / 'target/m3_mode70_mutations'
    if scratch.exists():
        shutil.rmtree(scratch)
    scratch.mkdir(parents=True)
    for f in ('Cargo.toml', 'Cargo.lock'):
        shutil.copy2(ROOT / f, scratch / f)
    shutil.copytree(ROOT / 'crates', scratch / 'crates', ignore=shutil.ignore_patterns('target'))
    (scratch / 'bench/oracle/m3_push_mode70_rust').mkdir(parents=True)
    shutil.copy2(D / 'input.txt', scratch / 'bench/oracle/m3_push_mode70_rust/input.txt')
    results = []
    for src, name, old, new in MUTATIONS:
        rel = Path('crates/lp/src') / src
        original = (ROOT / rel).read_text()
        require(original.count(old) == 1, f'변이 원문이 하나가 아님: {name} ({original.count(old)})')
        (scratch / rel).write_text(original.replace(old, new, 1))
        tr = scratch / 'target/m3_push_mode70_rust'
        shutil.rmtree(tr, ignore_errors=True)
        first = None
        try:
            proc = subprocess.run(['cargo', 'test', '--release', '-p', 'rsolver-lp', '--test', 'push_lu', 'mode70_chain_trace'], cwd=scratch,
                                  text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=900)
            log = proc.stdout[-600:]
        except subprocess.TimeoutExpired:
            log = 'timeout'
        if not (tr / 'trace.txt').exists():
            result, first = 'replay-failed', log[-300:]
        else:
            try:
                _, fails, _ = compare_chain(caps, tr / 'trace.txt', verbose=False)
                require(not fails, f'{len(fails)}: {fails[0] if fails else ""}')
                result = 'survived'
            except Fail as e:
                result, first = 'rejected', str(e)
        (scratch / rel).write_text(original)
        results.append(dict(name=name, file=src, result=result, first=first))
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
        _, fails, _ = compare_chain(records())
        require(not fails, f'{len(fails)}개 불일치: 첫 {fails[0] if fails else None}')


if __name__ == '__main__':
    try:
        main()
    except Fail as exc:
        print('FAIL', exc, file=sys.stderr)
        sys.exit(1)
