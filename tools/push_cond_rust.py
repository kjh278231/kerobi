#!/usr/bin/env python3
"""M3-04c-1c: Rust 조건수(116fd7)를 원본 캡처와 비교한다. 1d 하네스(push_init_ext)에서 12f282 출력 주입을 없앤다.

  python3 tools/push_cond_rust.py --write           # 1c controls·acceptance 뒤 입력 작성(bench/oracle/m3_push_cond_rust/input.txt)
  cargo test -p rsolver-lp --test push_init_ext     # Rust 재생 → target/m3_push_cond_rust/trace.txt
  python3 tools/push_cond_rust.py --compare         # 1d 기대 줄 + 116fd7 내부 줄 비교(첫 차이, 실패 시 종료 1)
  python3 tools/push_cond_rust.py --self-test       # 기대 줄 변조가 모두 거부되는지
  python3 tools/push_cond_rust.py --mutate-rust OUT # Rust 변이(임시 복사본)마다 재생·비교

사례: M3-04c-1b 15사례(1c 재캡처 run 1, 1b 투영과 같음 — push_cond_check controls)와 Netlib 소형 45 자연.
1b 사례의 초기 재분해 입력은 1d와 같은 1a 입력이고 1c 캡처의 cond:lu와 같아야 한다. Netlib 사례는 cond:lu에서 만든다.
조건수 tape는 비우고 116fd7 진입 F+0xc·정렬(A 값·x·y 버퍼 mod 16)만 넣는다. 계약: docs/m3/primal_push_condition.md.
"""
import argparse, hashlib, json, os, shutil, subprocess, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import push_cond_check as pcc  # noqa: E402
import push_init_check as pic  # noqa: E402
import push_init_rust as pir  # noqa: E402
import push_init_survey as sv  # noqa: E402

ROOT = pir.ROOT
D = os.path.join(ROOT, 'bench/oracle/m3_push_cond_rust')
INPUT = os.path.join(D, 'input.txt')
TRACE = os.path.join(ROOT, 'target/m3_push_cond_rust/trace.txt')
SCHEMA = 'M3-04c-1c-v1'
STOP = dict(est='Estimate', limit='Limit', repeat='Repeat')
require = pir.require
j = pir.j


def lu_from_capture(evs, pe):
    """cond:lu와 push:enter로 1a 형식의 초기 재분해 입력(model·cfg·enter·lu 토큰)을 만든다."""
    cl = [e for e in evs if e['event'] == 'cond:lu']
    require(len(cl) >= 1 and len(cl[0]['block']) == 1, 'cond:lu')
    c = cl[0]
    st, ex = pe['state'], next(e for e in evs if e['event'] == 'init:enter')['extra']
    se = next((e for e in evs if e['event'] == 'supply:enter'), None)
    cnt = st['counters']
    b = c['block'][0]
    lu = ['lu', c['zero'], st['tolerances']['390'], str(c['x158']), str(c['pool_mod16'][0]), str(c['pool_mod16'][1]),
          str(c['x188']), c['x1f8'], str(b['x2c0']), str(b['x4e0']), str(b['x4e4']), str(c['pre']['x46c'])]
    enter = ['enter', *map(str, st['vstat']), *map(str, st['header']), *st['x'], *st['slack'], *pe['model']['rhs'],
             st['tolerances']['390'], cnt['iterations'], str(ex['request']), str(ex['numeric']), str(cnt['failure']),
             str(ex['status']), str(c['pre']['x2fc']), str(c['pre']['x2f8'])]
    # S+0x3a8은 루프(work 열 수)에서만 읽는다: supply 없이 retry로 끝나는 호출은 0
    cfg = ['cfg', '-', '-', '-', '-', se['supplier']['drop'] if se else pic.ZERO]
    return dict(model=pir.model_tokens(pe), cfg=cfg, enter=enter, lu=lu)


def cvec(h):
    return f"{h['count']} | {j(h['idx'] or [])} | {j(h['val'])}"


def cond_wants(evs):
    """116fd7 호출마다 기대 줄 목록(캡처 + C 재계산 est)과 진입 토큰."""
    out = []
    pe = next(e for e in evs if e['event'] == 'push:enter')
    starts = [i for i, e in enumerate(evs) if e['event'] == 'cond:enter']
    for i in starts:
        end = next(q for q in range(i, len(evs)) if evs[q]['event'] == 'cond:leave')
        blk = evs[i:end + 1]
        summ = []
        errs = pcc.check_cond('w', blk, pe['model'], pcc.RULES, summ)
        require(not errs, f'C 재계산 실패 {errs[:1]}')
        s = summ[0]
        en = blk[0]
        tok = [str(en['f0c']), str(en['a_val_mod16']), str(en['mod16']['X']), str(en['mod16']['Y'])]
        w = [('cnorm', dict(norm=s['normb'], hdr=j(en['header'])))]
        fl = None
        for e in blk[1:-1]:
            k = e['event']
            if k == 'cond:ftran:enter':
                fin = e['input']
            elif k == 'cond:ftran:leave':
                w.append(('cft', dict(k=str(e['k']), **{'in': cvec(fin)}, out=cvec(e['output']),
                                      x=e['input']['val_buf'], y=e['output']['val_buf'])))
                w.append(('cest', dict(k=str(e['k']), est=s['est'][e['k']])))
            elif k == 'cond:btran:enter':
                bin_ = e['input']
                require(bin_['count'] == -1, 'BTRAN 입력 dense')
            elif k == 'cond:btran:leave':
                w.append(('cbt', dict(k=str(e['k']), **{'in': j(bin_['val'])}, out=cvec(e['output']),
                                      x=e['output']['val_buf'], y=e['input']['val_buf'])))
                fl = e
        del fl
        w.append(('cres', dict(stop=STOP[s['reason']], iter=str(s['iters']), cond=blk[-1]['condition'])))
        out.append((tok, w))
    return out


def project(case, c, evs, a1):
    toks, want = pir.project_call(case, c, evs, a1)
    cw = cond_wants(evs)
    ext = toks[4]
    k = int(ext[7])
    require(k == len(cw), f'{case} call{c}: 12f282 {k} != 116fd7 {len(cw)}')
    toks[4] = ext[:7] + ['0']
    toks.append(['cond', str(len(cw))] + sum((t for t, _ in cw), []))
    # 각 cond 줄(12f282 직전 관측) 뒤에 116fd7 내부 줄
    out, r = [], 0
    for w in want:
        out.append(w)
        if w[0] == 'cond':
            out += cw[r][1]
            r += 1
    require(r == len(cw), f'{case} call{c}: cond 줄 {r} != 116fd7 {len(cw)}')
    return toks, out


def cases():
    out = []
    for case in pcc.cases():
        doc = sv.load(pcc.path(case, 1))
        for c, evs in sorted(pic.calls_of(doc).items()):
            out.append((case, c, evs))
    return out


def build():
    a1s = pir.load_1a()
    toks, wants = [], []
    for case, c, evs in cases():
        pe = evs[0]
        if case.startswith('netlib/'):
            a1 = lu_from_capture(evs, pe)
        else:
            a1 = a1s[pir.base_of(case)]
            if c == 1 and pe['first'] and pe['attempt'] == 0:
                # 1c 캡처의 초기 재분해 입력 = 1a 입력(같은 기저 모델)
                mine = lu_from_capture(evs, pe)
                require(mine['lu'] == a1['lu'], f'{case}: cond:lu != 1a lu')
                require(mine['enter'][-2:] == a1['enter'][-2:], f'{case}: S+0x2fc/0x2f8 != 1a')
                require(mine['cfg'][5] in (a1['cfg'][5], pic.ZERO), f'{case}: S+0x3a8 != 1a')
        t, w = project(case, c, evs, a1)
        toks.append(t)
        wants.append(((case, c), w))
    return toks, wants


def head():
    h = hashlib.sha256()
    for p in [pcc.MANIFEST, pic.MANIFEST, pir.INPUT_1A]:
        h.update(open(p, 'rb').read())
    return f'{SCHEMA} {h.hexdigest()}'


def write():
    docs = pcc.load_all()
    errs = pcc.controls(docs, json.load(open(pcc.MANIFEST))) + pcc.noninterference(docs)
    fails, _ = pcc.acceptance(docs)
    require(not errs and not fails, f'1c 검사 실패: {(errs + fails)[:3]}')
    toks, _ = build()
    os.makedirs(D, exist_ok=True)
    text = head() + '\n' + '\n'.join(' '.join(t) for blk in toks for t in blk) + '\n'
    if os.path.exists(INPUT) and open(INPUT).read() == text:
        print(f'입력 불변: {os.path.relpath(INPUT, ROOT)}')
    else:
        open(INPUT, 'w').write(text)
        print(f'작성: {os.path.relpath(INPUT, ROOT)} ({len(toks)} 호출)')


def use():
    """pir의 비교·self-test를 이 입력으로 돌리도록 바꾼다."""
    pir.build, pir.head, pir.TRACE = build, head, TRACE


def compare(text=None, verbose=True):
    use()
    errs, nl, nf = pir.compare(text, verbose=verbose)
    if verbose:
        got = pir.parse_trace(text if text is not None else open(TRACE).read())[1]
        kinds = {}
        for g in got.values():
            for k, _ in g:
                kinds[k] = kinds.get(k, 0) + 1
        print('Rust 줄 종류', dict(sorted(kinds.items())))
    return errs, nl, nf


C = 'crates/lp/src/crossover_condition.rs'
MUTATIONS = [
    # (이름, 파일, 원문, 변이) — 결과를 보기 전에 정한 C·asm 규칙
    ('sum-sequential', C, '    eta_dot(&a, mod16 == 8)', '    a.iter().fold(0.0, |s, &t| s + t)'),
    ('sum-no-peel', C, '    eta_dot(&a, mod16 == 8)', '    eta_dot(&a, false)'),
    ('a-align-ignored', C, '((i64::from(e.a_val_mod16) + 8 * b) % 16) as u8', '0'),
    ('slack-norm-zero', C, '        let s = if j as usize >= a.n {\n            1.0', '        let s = if j as usize >= a.n {\n            0.0'),
    ('norm-sum-not-max', C, '        norm = if s > norm { s } else { norm };', '        norm += s;'),
    ('start-ones', C, 'val: vec![1.0 / m as f64; m]', 'val: vec![1.0; m]'),
    ('stop-strict', C, '        if it > 0 && old >= est {', '        if it > 0 && old > est {'),
    ('stop-first-iter', C, '        if it > 0 && old >= est {', '        if old >= est {'),
    ('sign-le', C, '            for (i, v) in x.val.iter().enumerate() {\n                if *v < 0.0 {', '            for (i, v) in x.val.iter().enumerate() {\n                if *v <= 0.0 {'),
    ('sign-sparse-packed', C, '                    sign[x.idx[q] as usize] = -1.0;', '                    sign[q] = -1.0;'),
    ('limit-4', C, '        if it >= 5 {', '        if it >= 4 {'),
    ('argmax-last', C, '            if v.abs() > best {', '            if v.abs() >= best && v.abs() > 0.0 {'),
    ('argmax-position', C, '                j = if z.count < 0 { q as i32 } else { z.idx[q] };', '                j = q as i32;'),
    ('repeat-never', C, '        if j == prev {', '        if false {'),
    ('result-est-only', C, '    let c = norm * est;', '    let c = est;'),
    ('buffer-no-swap', C, '        (xb, yb) = (nx, ny);\n        ev(CondEvent::Ftran', '        let _ = (nx, ny);\n        ev(CondEvent::Ftran'),
    ('btran-cache', C, '        let s = lu.btran(-1, &[], &sign, x.count, false, 0.0);', '        let s = lu.btran(-1, &[], &sign, x.count, true, 0.0);'),
]


def mutate_rust(out_path):
    require(not os.path.exists(out_path), f'{out_path} 이미 있음')
    use()
    _, wants = build()
    work = os.path.join(ROOT, 'target/m3_push_cond_mutations')
    if os.path.exists(work):
        shutil.rmtree(work)
    src = os.path.join(work, 'src')
    shutil.copytree(ROOT, src, ignore=shutil.ignore_patterns('target', '.git', 'bench', 'python', 'docs'))
    os.symlink(os.path.join(ROOT, 'bench'), os.path.join(src, 'bench'))
    env = dict(os.environ, CARGO_TARGET_DIR=os.path.join(work, 'target'))
    results = []
    for name, f, a, b in MUTATIONS:
        p = os.path.join(src, f)
        orig = open(p).read()
        if orig.count(a) != 1:
            results.append(dict(name=name, result='원문 없음'))
            print(name, '원문 없음')
            continue
        open(p, 'w').write(orig.replace(a, b))
        tr = os.path.join(src, 'target/m3_push_cond_rust/trace.txt')
        try:
            r = subprocess.run(['cargo', 'test', '-q', '-p', 'rsolver-lp', '--test', 'push_init_ext', 'init_cond_trace'],
                               cwd=src, env=env, capture_output=True, text=True, timeout=900)
            if 'error[' in r.stderr and not os.path.exists(tr):
                res = dict(result='컴파일 실패', detail=r.stderr[-300:])
            elif not os.path.exists(tr):
                res = dict(result='trace 없음')
            else:
                errs, _, _ = pir.compare(open(tr).read(), wants, verbose=False)
                res = dict(result='거부' if errs else '생존', first=errs[0] if errs else None)
                os.remove(tr)
        except subprocess.TimeoutExpired:
            res = dict(result='시간 초과(거부)')
        finally:
            open(p, 'w').write(orig)
        res['name'] = name
        results.append(res)
        print(name, res['result'], (res.get('first') or '')[:160])
    json.dump(results, open(out_path, 'w'), ensure_ascii=False, indent=1)
    bad = [r for r in results if r['result'] in ('원문 없음', '컴파일 실패', 'trace 없음')]
    print(f'변이 {len(results)}: 거부 {sum(r["result"].startswith(("거부", "시간")) for r in results)}, '
          f'생존 {sum(r["result"] == "생존" for r in results)}, 무효 {len(bad)}')
    return 1 if bad else 0


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--write', action='store_true')
    p.add_argument('--compare', action='store_true')
    p.add_argument('--self-test', action='store_true')
    p.add_argument('--mutate-rust')
    a = p.parse_args()
    rc = 0
    if a.write:
        write()
    if a.compare:
        errs, _, _ = compare()
        rc |= 1 if errs else 0
    if a.self_test:
        use()
        rc |= pir.self_test()
    if a.mutate_rust:
        rc |= mutate_rust(a.mutate_rust)
    return rc


if __name__ == '__main__':
    sys.exit(main())
