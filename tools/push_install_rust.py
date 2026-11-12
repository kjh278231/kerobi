#!/usr/bin/env python3
"""M3-04f-1: Rust 해 설치(crossover_install)를 원본 캡처와 비교한다.

  python3 tools/push_install_rust.py --write          # 검사기 통과 뒤 입력 작성(bench/oracle/m3_push_install_rust/)
  cargo test -p rsolver-lp --test push_install --test push_init_ext   # Rust 재생 → target/m3_push_install_{unit,rust}/
  python3 tools/push_install_rust.py --compare        # (A) 설치 경계 단위 + (B) 진입→초기화→루프→설치 연결
  python3 tools/push_install_rust.py --self-test      # 기대 줄 변조가 모두 거부되는지
  python3 tools/push_install_rust.py --mutate-rust OUT  # Rust 변이(임시 복사본)마다 재생·비교

(A) unit.txt: 모든 사례(1b 15 + 합성 5 + Netlib 45 + Crossover=-1 45)의 install:enter 상태(설치 편집 반영)와 설치 중
    12f282 출력 → push:leave의 retry·remaining·S+0x120·condition·vstat·x·slack.
(B) input.txt: 1c 사례(1b 15 + Netlib 45)의 1c 입력 그대로(1c 투영이 같음은 검사기 비간섭) + inst 기록(2ba0·S+0x46c·
    설치 중 116fd7 진입). Rust factor가 있는 호출(lu new/carry)만 루프·설치까지 이어 install:enter·icond·push:leave와 비교.
계약: docs/m3/primal_push_install.md.
"""
import argparse, hashlib, json, os, shutil, subprocess, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import push_cond_check as pcc  # noqa: E402
import push_cond_rust as prc  # noqa: E402
import push_init_check as pic  # noqa: E402
import push_init_rust as pir  # noqa: E402
import push_init_survey as sv  # noqa: E402
import push_install_check as pik  # noqa: E402

ROOT = pir.ROOT
D = os.path.join(ROOT, 'bench/oracle/m3_push_install_rust')
INPUT, UNIT = os.path.join(D, 'input.txt'), os.path.join(D, 'unit.txt')
TRACE = os.path.join(ROOT, 'target/m3_push_install_rust/trace.txt')
UTRACE = os.path.join(ROOT, 'target/m3_push_install_unit/trace.txt')
SCHEMA = 'M3-04f-1-v1'
require, j = pir.require, pir.j


def head():
    h = hashlib.sha256()
    for p in [pik.MANIFEST, pcc.MANIFEST, pic.MANIFEST, pir.INPUT_1A]:
        h.update(open(p, 'rb').read())
    return f'{SCHEMA} {h.hexdigest()}'


def leave_fields(pl):
    ls = pl['leave_state']
    return dict(retry=str(pl['retry']), rem=str(pl['remaining']), pinf=ls['pinf'], cond=pl['condition'],
                vstat=j(ls['vstat']), xs=j(ls['x'] + ls['slack']))


# ---------------------------------------------------------------- (A) 설치 경계
def unit_cases():
    out = []
    for case in pik.cases():
        doc = sv.load(pik.path(case, 1))
        for c, evs in sorted(pic.calls_of(doc).items()):
            ie = [e for e in evs if e['event'] == 'install:enter']
            if not ie:
                continue
            inp = pik.install_input(evs, ie[0])
            st = ie[0]['state']
            pl = next(e for e in evs if e['event'] == 'push:leave')
            toks = ['case', case, str(c), str(st['n']), str(st['m']), *map(pcc.hx, inp['lb']), *map(pcc.hx, inp['ub']),
                    ie[0]['tau'], ie[0]['inf'], str(ie[0]['first']), str(ie[0]['attempt']), str(ie[0]['model_status']),
                    str(ie[0]['x46c']), str(ie[0]['env_2ba0']), str(ie[0]['count']), *map(str, ie[0]['candidates']),
                    *map(str, st['header']), *st['basic'], *st['width'], *map(str, st['vstat']), *st['x'], *st['slack'],
                    pcc.hx(inp['cond'])]
            out.append(((case, c), toks, [('leave', leave_fields(pl))]))
    return out


# ---------------------------------------------------------------- (B) 연결
def chain_cases():
    """[(사례, 호출, 1c 투영 사건열, 전체 사건열)] — 새 캡처 run 1."""
    out = []
    for case in pcc.cases():
        doc = sv.load(pik.path(case, 1))
        full = pic.calls_of(doc)
        proj = pic.calls_of(dict(events=pik.project_1c(doc)))
        for c in sorted(full):
            out.append((case, c, proj[c], full[c]))
    return out


def install_wants(case, c, full, toks):
    """inst 기록과 기대 줄(Rust factor가 있는 설치 호출만)."""
    ies = [e for e in full if e['event'] == 'install:enter']
    if not ies or toks[5][1] not in ('new', 'carry'):
        return None, []
    ie = ies[0]
    st = ie['state']
    i0 = next(i for i, e in enumerate(full) if e['event'] == 'icond:enter')
    i1 = next(i for i in range(i0, len(full)) if full[i]['event'] == 'icond:leave')
    blk = [full[0]] + [dict(e, event='cond:' + e['event'][6:]) for e in full[i0:i1 + 1]]
    cw = prc.cond_wants(blk)
    require(len(cw) == 1, f'{case} call{c}: 설치 116fd7')
    tok, lines = cw[0]
    rec = ['inst', str(ie['env_2ba0']), str(ie['x46c']), *tok]
    w = [('inst', dict(cnt=str(ie['count']), cand=j(ie['candidates']), hdr=j(st['header']), basic=j(st['basic']),
                       width=j(st['width']), vstat=j(st['vstat']), xs=j(st['x'] + st['slack']), pinf=st['pinf'],
                       iter=st['counters']['iterations']))]
    w += [('i' + k, v) for k, v in lines]
    pl = next(e for e in full if e['event'] == 'push:leave')
    w.append(('leave', leave_fields(pl)))
    return rec, w


def build():
    a1s = pir.load_1a()
    toks, wants = [], []
    for case, c, evs, full in chain_cases():
        pe = evs[0]
        if case.startswith('netlib/'):
            a1 = prc.lu_from_capture(evs, pe)
        else:
            a1 = a1s[pir.base_of(case)]
        t, w = prc.project(case, c, evs, a1)
        rec, iw = install_wants(case, c, full, t)
        if rec is not None:
            t.append(rec)
            w = w + iw
        toks.append(t)
        wants.append(((case, c), w))
    return toks, wants


def use():
    pir.build, pir.head, pir.TRACE = build, head, TRACE


def write():
    docs = pik.load_all()
    errs = pik.controls(docs, json.load(open(pik.MANIFEST))) + pik.noninterference(docs)
    fails, summ = pik.acceptance(docs)
    fails += pik.required_missing(summ)
    require(not errs and not fails, f'설치 검사 실패: {(errs + fails)[:3]}')
    toks, wants = build()
    os.makedirs(D, exist_ok=True)
    for p, text, n in ((INPUT, head() + '\n' + '\n'.join(' '.join(t) for blk in toks for t in blk) + '\n', len(toks)),
                       (UNIT, head() + '\n' + '\n'.join(' '.join(t) for _, t, _ in unit_cases()) + '\n', None)):
        if os.path.exists(p) and open(p).read() == text:
            print(f'입력 불변: {os.path.relpath(p, ROOT)}')
        else:
            open(p, 'w').write(text)
            print(f'작성: {os.path.relpath(p, ROOT)}' + (f' ({n} 호출)' if n else ''))
    print(f'연결 설치 호출 {sum(any(k == "leave" for k, _ in w) for _, w in wants)}')


def compare_unit(text=None, wants=None, verbose=True):
    if wants is None:
        wants = [(k, w) for k, _, w in unit_cases()]
    h, got = pir.parse_trace(text if text is not None else open(UTRACE).read())
    errs = [] if h == head() else ['unit trace 머리(입력 해시) 불일치: --write 뒤 cargo test를 다시 실행']
    e2, nl, nf = pir.compare_lines(wants, got, verbose=False)
    errs += e2
    extra = set(got) - {k for k, _ in wants}
    if extra:
        errs.append(f'예상하지 않은 호출 {sorted(extra)[:3]}')
    if verbose:
        for e in errs[:10]:
            print('불일치(A)', e)
        print(f'(A) 설치 경계 {len(wants)}호출·줄 {nl}·필드 {nf} 비교, 불일치 {len(errs)}')
    return errs, nl, nf


# 결과 확인 뒤 분류한 연결 실패(사전 명시 아님, 통과 수에 넣지 않는다). 04f-1의 grow7(마지막 round 재분해의 036ade
# 조밀 전환 미이식, 첫 차이 "줄15 inst.hdr[89]: 원본 234 Rust 182")은 M3-04c-21i에서 조밀 전환을 이식해 통과로 바뀌어
# 비웠다(docs/m3/primal_push_dense_lu.md §6). 비어 있으면 모든 차이가 실패다.
KNOWN = {}


def compare(text=None, wants=None, verbose=True):
    use()
    errs, nl, nf = pir.compare(text, wants, verbose=False)
    known = [e for e in errs if e in KNOWN.values()]
    errs = [e for e in errs if e not in KNOWN.values()]
    errs += [f'{k}: 분류된 실패가 다른 모습(첫 차이 바뀜 또는 통과)' for k, v in KNOWN.items() if v not in known]
    if verbose:
        for e in known:
            print('분류된 실패(B, 통과 아님)', e)
        for e in errs[:10]:
            print('불일치(B)', e)
        _, wl = build() if wants is None else (None, wants)
        ninst = sum(any(k == 'leave' for k, _ in w) for _, w in wl)
        nin = sum(sum(1 for k, _ in w if k.startswith(('inst', 'ic', 'leave'))) for _, w in wl)
        print(f'(B) 연결 {len(wl)}호출(설치까지 {ninst}, 통과 {ninst - len(known)}·분류된 실패 {len(known)})·줄 {nl}(설치 쪽 {nin})·'
              f'필드 {nf} 비교, 그 밖의 불일치 {len(errs)}')
    return errs, nl, nf


def self_test():
    """(B)는 1d self-test와 같은 변조(줄 종류별 1비트·순서·누락)를 분류된 실패를 뺀 호출에, (A)는 leave 필드별 1비트."""
    use()
    _, allw = build()
    btext = open(TRACE).read()
    got = pir.parse_trace(btext)[1]
    bw = [(k, w) for k, w in allw if k not in KNOWN]
    require(not pir.compare_lines(bw, got, verbose=False)[0], '(B) 기준 비교가 먼저 통과해야 함')
    real_build = pir.build
    pir.build = lambda: (None, bw)
    real_compare = pir.compare
    pir.compare = lambda text, wants, verbose=True: (pir.compare_lines(wants, pir.parse_trace(text)[1], False)[0], 0, 0)
    try:
        rc = pir.self_test()
    finally:
        pir.build, pir.compare = real_build, real_compare
    wants = [(k, w) for k, _, w in unit_cases()]
    text = open(UTRACE).read()
    require(not compare_unit(text, wants, verbose=False)[0], '(A) 기준 비교가 먼저 통과해야 함')
    tried, survived = 0, []
    import copy
    for wi, (key, want) in enumerate(wants):
        for f in want[0][1]:
            mut = copy.deepcopy(wants)
            v = mut[wi][1][0][1][f].split()
            t = v[-1] if v else ''
            v[-1:] = [format(int(t, 16) ^ 1, '016x') if len(t) == 16 else str(int(t) + 1)] if v else ['0']
            mut[wi][1][0][1][f] = ' '.join(v)
            tried += 1
            if not compare_unit(text, mut, verbose=False)[0]:
                survived.append(f'{key} {f}')
    print(f'(A) 변조 {tried}종, 생존 {len(survived)}')
    for s in survived[:10]:
        print('생존', s)
    return rc | (1 if survived else 0)


S = 'crates/lp/src/crossover_install.rs'
MUTATIONS = [
    # (이름, 파일, 원문, 변이) — 결과를 보기 전에 정한 C·asm 규칙
    ('no-write-back', S, '    if e.first {\n        write_back', '    if false {\n        write_back'),
    ('write-back-always', S, '    if e.first {\n        write_back', '    if true {\n        write_back'),
    ('write-back-no-lb', S, '{ lp.lb[v] + b } else { b }', '{ b } else { b }'),
    ('write-back-lb-ge', S, 'v < lp.n && below_or_unordered(neg_inf, lp.lb[v])', 'v < lp.n && neg_inf <= lp.lb[v]'),
    ('pinf-free-ub-only', S, '        if free(lp.lb[v], lp.ub[v], inf) {', '        if lp.ub[v] >= inf {'),
    ('pinf-low-ge', S, '        if neg_tau > b {', '        if neg_tau >= b {'),
    ('pinf-tau-ge', S, 'if tau > b || w - tau', 'if tau >= b || w - tau'),
    ('pinf-upper-ge', S, '|| tau + w > b {', '|| tau + w >= b {'),
    ('pinf-upper-sign', S, '        p += b - w;', '        p += w - b;'),
    ('pp-always-pinf', S, 'let p = if e.model_status == 2 { pinf } else { 1.0 };', 'let p = pinf;'),
    ('retry-1e14-ge', S, 'cond > 1e14 ||', 'cond >= 1e15 ||'),
    ('no-1e30', S, ') || cond > 1e30;', ');'),
    ('retry-rem-zero', S, 'remaining: e.candidates.len() as i32 };', 'remaining: 0 };'),
    ('free-ignored', S, '    if e.candidates.iter().any(', '    if false && e.candidates.iter().any('),
    ('env-any', S, '    if e.env_2ba0 != -1 {', '    if e.env_2ba0 == -1 {'),
    ('threshold-1', S, '(e.interval / 10).max(5)', '(e.interval / 10).max(1)'),
    ('vstat-ge', S, 'lp.lb[v] + lp.ub[v] > st.xs[v] * 2.0', 'lp.lb[v] + lp.ub[v] >= st.xs[v] * 2.0'),
    ('vstat-swap', S, '{ -1 } else { -2 };', '{ -2 } else { -1 };'),
    ('no-snap', S, '                -1 => st.xs[j] = lp.lb[j],\n                -2 => st.xs[j] = lp.ub[j],', '                -1 | -2 => {}'),
    ('snap-swap', S, '                -1 => st.xs[j] = lp.lb[j],\n                -2 => st.xs[j] = lp.ub[j],', '                -1 => st.xs[j] = lp.ub[j],\n                -2 => st.xs[j] = lp.lb[j],'),
    ('cond-before-pinf', S, '    let (pinf, violations) = infeasibility(lp, par, st);\n    let cond = condition();', '    let cond = condition();\n    let (pinf, violations) = infeasibility(lp, par, st);'),
]


def mutate_rust(out_path):
    require(not os.path.exists(out_path), f'{out_path} 이미 있음')
    use()
    _, wants = build()
    uw = [(k, w) for k, _, w in unit_cases()]
    work = os.path.join(ROOT, 'target/m3_push_install_mutations')
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
        tr = os.path.join(src, 'target/m3_push_install_rust/trace.txt')
        ut = os.path.join(src, 'target/m3_push_install_unit/trace.txt')
        try:
            r = subprocess.run(['cargo', 'test', '-q', '-p', 'rsolver-lp', '--test', 'push_init_ext', '--test', 'push_install',
                                '--', 'init_install_trace', 'install_unit_trace'],
                               cwd=src, env=env, capture_output=True, text=True, timeout=900)
            if 'error[' in r.stderr and not (os.path.exists(tr) or os.path.exists(ut)):
                res = dict(result='컴파일 실패', detail=r.stderr[-300:])
            elif not (os.path.exists(tr) and os.path.exists(ut)):
                res = dict(result='trace 없음')
            else:
                # 분류된 실패(KNOWN)는 변이 거부로 세지 않는다(첫 실행에서 grow7 기존 실패가 6종을 거짓 거부로 셌다).
                eb, _, _ = compare(open(tr).read(), wants, verbose=False)
                ea, _, _ = compare_unit(open(ut).read(), uw, verbose=False)
                res = dict(result='거부' if (ea or eb) else '생존', unit=ea[0] if ea else None, chain=eb[0] if eb else None)
                os.remove(tr)
                os.remove(ut)
        except subprocess.TimeoutExpired:
            res = dict(result='시간 초과(거부)')
        finally:
            open(p, 'w').write(orig)
        res['name'] = name
        results.append(res)
        print(name, res['result'], ((res.get('unit') or res.get('chain') or ''))[:160])
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
        ea, _, _ = compare_unit()
        eb, _, _ = compare()
        rc |= 1 if (ea or eb) else 0
    if a.self_test:
        rc |= self_test()
    if a.mutate_rust:
        rc |= mutate_rust(a.mutate_rust)
    return rc


if __name__ == '__main__':
    sys.exit(main())
