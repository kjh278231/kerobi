"""M3-04c-22: 초기화된 직렬 push 루프(Rust crossover_loop::push_loop)를 21e 지연 LU 연쇄 캡처에 연결 재생해 비교한다.

Rust 입력은 첫 패스 준비 직전 상태와 수치 backend 출력 tape(공급 FTRAN raw, 지원 BTRAN raw, 05b257 반환·수치 카운터·
factor 통계·재분해 순열, 재계산 FTRAN raw)뿐이다. 원본 후상태는 이 비교기만 읽는다(재주입 0회).

  --write             21e 캡처 검사(controls·acceptance) 뒤 bench/oracle/m3_push_loop_rust/input.txt로 투영
  --compare           target/m3_push_loop_rust/trace.txt를 원본 canonical 사건열과 비교(첫 차이에서 종료 1)
  --self-test         trace 변조(필드·줄 누락·추가·순서)가 모두 거부되는지 확인
  --mutate-rust PATH  Rust 소스 변이마다 target 사본에서 재생·비교해 결과를 PATH(새 파일)에 기록
  --lu                M3-04c-24: tape 대신 실제 Rust backend(crossover_backend)로 재생하는 모드. 입력에는 수치 출력이
                      없고(초기화 factor 입력만, 21h 첫 재분해와 같음), canonical 줄에 init 줄과 fcall 통계를 더한다.
  --init              M3-04c-1a: --lu에 더해 루프 시작 상태(state·round 줄)도 입력에서 빼고 push:enter 상태만 넣는다.
                      Rust crossover_init::push_init이 초기화를 수행하며, canonical 줄에 루프 시작 상태 start 줄을 더한다.
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

sys.path.insert(0, str(Path(__file__).resolve().parent))
import push_lu_check as oracle  # noqa: E402  21e 캡처 검사(독립 3회·비간섭·변조 거부)
from push_supplier_check import Fail, require  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
D = ROOT / 'bench/oracle/m3_push_loop_rust'
INPUT = D / 'input.txt'
TRACE = ROOT / 'target/m3_push_loop_rust/trace.txt'
SCHEMA = 'M3-04c-22-v1'
TEST = 'push_loop'
LU = False
INIT = False


def use_init():
    global D, INPUT, TRACE, SCHEMA, TEST, INIT
    use_lu()
    D = ROOT / 'bench/oracle/m3_push_init_rust'
    INPUT = D / 'input.txt'
    TRACE = ROOT / 'target/m3_push_init_rust/trace.txt'
    SCHEMA = 'M3-04c-1a-v1'
    TEST = 'push_init_loop'
    INIT = True


def use_lu():
    global D, INPUT, TRACE, SCHEMA, TEST, LU
    D = ROOT / 'bench/oracle/m3_push_loop_lu'
    INPUT = D / 'input.txt'
    TRACE = ROOT / 'target/m3_push_loop_lu/trace.txt'
    SCHEMA = 'M3-04c-24-v1'
    TEST = 'push_loop_lu'
    LU = True


def stats_text(s):
    return ' '.join(str(s[k]) for k in ('k', 'limit', 'q', 'C', 'E', 'L', 'U'))


def init_line(name, rec, k0):
    """초기화 factor 입력(21h 첫 G·K와 같은 토큰)과 원본 기대 init 줄."""
    import push_lu_rust as plr
    plr.use_block()
    prog = plr.program(name, rec, True)
    ks = [x for x in prog if x[0] == 'K']
    gs = [x for x in prog if x[0] == 'G']
    require(ks and gs and ks[0][1] == gs[0][1] < k0, f'{name}: 초기화 재분해 입력')
    g, kk = gs[0][2], ks[0][2]
    require(g[6] == '1', f'{name}: 초기화 할당')
    ev = rec['events']
    i = gs[0][1]
    lv = next(e for e in ev[i:] if e['event'] == 'lu:leave')
    fl = [e for e in ev[:k0] if e['event'] == 'factor:leave']
    require(len(fl) == 1, f'{name}: 초기화 factor:leave')
    fc = fl[0]['factor_ctl']['state']
    m = int(g[3])
    # 21h G 토큰: ... 할당5, [m, header…], leave, [n+m, vstat…] (목록은 길이 접두)
    rest = g[12:]
    require(int(rest[0]) == m and int(rest[m + 2]) == len(lv['vstat']) == len(rest) - m - 3,
            f'{name}: 초기화 header/vstat 길이')
    toks = ['lu', g[4], g[5], *g[7:12], *kk[4:7], str(fc['q']), *rest[1:m + 1], rest[m + 1], *rest[m + 3:]]
    want = ('init', dict(hdr=j(lv['header']), leave=str(lv['leave']), vstat=j(lv['vstat']), stats=stats_text(fc)))
    return toks, want
def init_tokens(name, rec, k0, toks):
    """M3-04c-1a 입력(case·model·cfg·enter·lu)과 기대 start 줄. 루프 시작 상태는 입력에 넣지 않는다."""
    ev = rec['events']
    push = next(e for e in ev if e['event'] == 'push:enter')
    fe = next(e for e in ev if e['event'] == 'factor:enter')
    le = next(e for e in ev if e['event'] == 'lu:enter')
    pa = ev[k0]['state']
    ready = next(e for e in ev[k0:] if e['event'] == 'roundctl:pass:ready')
    se = next(e for e in ev[k0:] if e['event'] == 'supply:enter')
    ps, ss = push['state'], se['state']
    n, m = ps['n'], ps['m']
    lu = toks[5]
    require(lu[0] == 'lu' and len(lu) == 12 + m + 1 + n + m, f'{name}: lu 줄 길이')
    # 초기화 factor 입력은 push:enter 상태와 같아야 한다(그 사이 쓰기 없음): header·S+0x2f8·vstat·Markowitz
    require(lu[12:12 + m] == list(map(str, ps['header'])) and lu[13 + m:] == list(map(str, ps['vstat'])),
            f'{name}: 초기 factor header/vstat != push:enter')
    require(lu[12 + m] == str(le['leave']), f'{name}: 초기 factor S+0x2f8')
    require(lu[2] == ps['tolerances']['390'], f'{name}: 초기 factor Markowitz != S+0x390')
    require(fe['header'] == ps['header'] and fe['request'] == 10, f'{name}: factor:enter')
    require(push['first'] == 1 and ps['precision'] == 0 and push['alias_contiguous'], f'{name}: 범위(first·double·별칭)')
    require(len(ps['slack']) == m and len(ps['x']) == n, f'{name}: x·slack 길이')
    tol, cnt = ps['tolerances'], ps['counters']
    fc = fe['factor_ctl']['state']
    require(fc['numeric'] == cnt['numeric'] and fc['failure'] == cnt['failure'], f'{name}: factor:enter 카운터')
    model, cfg = toks[1], toks[2]
    cfg = ['cfg', tol['380'], tol['398'], tol['370'], tol['3b0'], se['supplier']['drop'], str(push['model_status']),
           str(push['attempt']), str(push['first'])]
    enter = ['enter', *map(str, ps['vstat']), *map(str, ps['header']), *ps['x'], *ps['slack'], *push['model']['rhs'],
             tol['390'], cnt['iterations'], str(cnt['request']), str(cnt['numeric']), str(cnt['failure']),
             str(fc['status']), str(le['enter']), str(le['leave'])]
    toks = [toks[0], model, cfg, enter, lu[:12]]
    adj = any(a in (-1, -2) and b == -3 for a, b in zip(ps['vstat'], ss['vstat']))
    sc = ss['counters']
    start = ('start', dict(
        cand=j(pa['candidates']), total=str(pa['total']), sup=str(pa['support']), vstat=j(ss['vstat']),
        header=j(ss['header']), leave=str(ready['selected']['leave']), rhs=j(ss['rhs']), basic=j(ss['basic']),
        width=j(ss['width']), marks=j(ss['marks']), pinf=ss['pinf'], iter=sc['iterations'], req=str(sc['request']),
        numeric=str(sc['numeric']), failure=str(pa['failure']), status=str(pa['status']), retry=str(pa['retry']),
        nlimit=str(pa['numeric_limit']), mk=ss['tolerances']['390'], limit=str(ss['factor']['limit']),
        adj=str(int(adj)), cond='unknown', pos=str(pa['position']), dfr=str(pa['deferred']), fail=j(pa['failures']),
        smarks=opt(pa['marks']), csum=pa['count_sum'], succ=str(pa['successes'])))
    require(pa['request'] == sc['request'] and pa['numeric'] == sc['numeric'], f'{name}: pass/supply 카운터')
    return toks, start


MODELS = ('arms', 'retry', 'interior', 'adlittle', 'scsd1', 'capri')
SITE = {'0x1c3538': 'pivot', '0x1c3d31': 'round'}
# 계약 §4에서 캡처를 세어 고정한 집계
EXPECT = dict(pass_=14, supply_ok=107, supply_end=12, flip=32, pivot=69, defer=6, btran=32, dense=2, select=67,
              fcall_pivot=37, fcall_round=12, recompute_pivot=1, recompute_round=12, end=6)


def j(xs):
    return ' '.join(map(str, xs))


def opt(xs):
    return 'none' if xs is None else j(xs)


def vec_fields(v):
    """FTRAN/BTRAN raw. dense(count<0)는 m개 값."""
    if v['count'] < 0:
        return dict(count=str(v['count']), val=j(v['values']))
    return dict(count=str(v['count']), idx=j(v['indices']), val=j(v['values']))


def state_fields(st, prefix=''):
    return {prefix + 'vstat': j(st['vstat']), prefix + 'header': j(st['header']), prefix + 'basic': j(st['basic']),
            prefix + 'width': j(st['width']), prefix + 'marks': j(st['marks']),
            prefix + 'iter': st['counters']['iterations']}


def round_fields(st, marks=True):
    w = dict(cand=j(st['candidates']), pos=str(st['position']), dfr=str(st['deferred']), sup=str(st['support']))
    if marks:
        w['marks'] = opt(st['marks'])
    w['req'] = str(st['request'])
    return w


def i64(h):
    return int.from_bytes(bytes.fromhex(h), 'little', signed=True)


def tape_vec(v, m):
    if v['count'] < 0:
        require(len(v['values']) == m, 'dense 길이')
        return [str(v['count']), *v['values']]
    require(len(v['indices']) == v['count'] == len(v['values']), 'sparse 길이')
    return [str(v['count']), *map(str, v['indices']), *v['values']]


def project(name, rec):
    """(입력 토큰 줄 목록, 기대 canonical 줄 목록, 집계)."""
    ev = rec['events']
    k0 = next(i for i, e in enumerate(ev) if e['event'] == 'roundctl:pass:enter')
    push = next(e for e in ev if e['event'] == 'push:enter')
    init = [e for e in ev[:k0] if e['event'] == 'factor:leave']
    require(len(init) == 1 and init[0]['request'] == 10 and init[0]['ret'] == 0, f'{name}: 초기화 factor 1회')
    require(sum(e['event'] == 'push:enter' for e in ev) == 1, f'{name}: push 1회')
    pe = ev[k0]['state']
    ready = next(e for e in ev[k0:] if e['event'] == 'roundctl:pass:ready')
    se = next(e for e in ev[k0:] if e['event'] == 'supply:enter')
    st = se['state']
    n, m = st['n'], st['m']
    require(se['batch'] == 0 and st['precision'] == 0 and pe['batch'] == [0, 0, 0], f'{name}: batch·x87 범위 밖')
    periods = {e['frame']['period'] for e in ev if e['event'] == 'factorctl:select'}
    first_period = next((e['frame']['period'] for e in ev if e['event'].startswith('factorctl:')), 50)
    require(first_period == 50, f'{name}: 품질 주기 초기값')
    lp, tol = push['model'], st['tolerances']
    toks = [['case', name]]
    toks.append(['model', str(n), str(m), str(len(lp['rows'])), *(str(i64(s)) for s in lp['starts']),
                 *map(str, lp['lengths']), *map(str, lp['rows']), *lp['values'], *lp['cost'], *lp['lb'], *lp['ub']])
    toks.append(['cfg', tol['380'], tol['398'], tol['370'], tol['3b0'], se['supplier']['drop'],
                 str(push['model_status']), str(push['attempt']), str(pe['numeric_limit']), '50'])
    require(pe['attempt'] == push['attempt'], 'attempt')
    sel = ready['selected']
    toks.append(['state', *map(str, st['vstat']), *map(str, st['header']), *st['x'], *st['slack'], *st['basic'],
                 *st['width'], *st['marks'], *st['rhs'], st['counters']['iterations'], str(st['counters']['request']),
                 st['pinf'], str(sel['enter']), str(sel['leave']), str(sel['direction'])])
    require(pe['request'] == st['counters']['request'], '요청')
    toks.append(['round', str(pe['total']), *map(str, pe['candidates']), str(pe['position']), str(pe['deferred']),
                 *map(str, pe['failures']), str(pe['support']),
                 *(['0'] if pe['marks'] is None else ['1', *map(str, pe['marks'])]),
                 pe['count_sum'], str(pe['successes']), str(pe['status']), str(pe['numeric']), str(pe['failure']),
                 str(pe['retry'])])
    require(len(pe['candidates']) == pe['total'], '후보 길이')
    fc = init[0]['factor_ctl']['state']
    if LU:
        lt, lw = init_line(name, rec, k0)
        toks.append(lt)
    else:
        toks.append(['stats', str(fc['k']), str(fc['limit']), str(fc['q']), fc['C'], str(fc['E']), fc['L'], fc['U']])

    # backend tape
    body = ev[k0:]
    ft = [e['raw'] for e in body if e['event'] == 'supplier:ftran:leave']
    bt = [e['raw'] for e in body if e['event'] == 'roundctl:btran:leave']
    rc = [e['raw'] for e in body if e['event'] == 'recompute:ftran:leave']
    require(not [e for e in body if e['event'].startswith('quality')], f'{name}: 품질 helper 자연 호출(범위 밖)')
    if LU:
        ft, bt, rc = [], [], []
    toks.append(['ftran', str(len(ft))] + sum((tape_vec(v, m) for v in ft), []))
    toks.append(['btran', str(len(bt))] + sum((tape_vec(v, m) for v in bt), []))
    fac, refac = [], {}
    for e in body:
        if e['event'] == 'lu:refactor' and e['phase'] == 'leave':
            require(e['ret'] == 0 and e['out'] == [0, 0], f'{name}: 036ade 실패(범위 밖)')
            refac[e['id']] = e['state']
    lus = [e for e in body if e['event'] == 'lu:leave']
    fls = [e for e in body if e['event'] == 'factor:leave']
    fes = [e for e in body if e['event'] == 'factor:enter']
    require(len(lus) == len(fls) == len(fes), f'{name}: factor/lu 쌍')
    for fe, fl, lu in zip(fes, fls, lus):
        s = fl['factor_ctl']['state']
        require(fl['request'] == fe['request'] == s['request'], f'{name}: 05b257가 요청을 바꿈(범위 밖)')
        ren = refac.get(lu['id']) if fe['request'] != 0 else None
        require((ren is not None) == (fe['request'] != 0), f'{name}: 재분해 순열')
        fac += [str(fl['ret']), str(s['numeric']), str(s['failure']), str(s['status']), str(s['k']),
                str(s['limit']), str(s['q']), s['C'], str(s['E']), s['L'], s['U']]
        fac += ['0'] if ren is None else ['1', *map(str, ren['x28']), *map(str, ren['x38'])]
    toks.append(['factor', str(len(fes))] + fac)
    toks.append(['recompute', str(len(rc))] + sum((list(r) for r in rc), []))
    for r in rc:
        require(len(r) == m, 'recompute 길이')
    if LU:
        toks = toks[:6]  # case·model·cfg·state·round·lu: 수치 tape 없음
    if INIT:
        toks, start = init_tokens(name, rec, k0, toks)

    # 기대 canonical 사건열
    want, cnt = ([lw] if LU else []), Counter()
    if INIT:
        want.append(start)
    step = None
    lu_of = {}
    for i, e in enumerate(body):
        k = e['event']
        if k == 'roundctl:pass:ready':
            want.append(('pass', round_fields(e['state'])))
            cnt['pass_'] += 1
        elif k == 'step:leave':
            s = e['state']
            w = state_fields(s)
            w['xs'] = j(s['x'] + s['slack'])
            w['rhs'] = j(s['rhs'])
            w['pinf'] = s['pinf']
            want.append(('leave', w))
        elif k == 'supplier:ftran:enter':
            w = vec_fields(e['input'])
            w['spike'] = '0' if e['aliases']['spike_null'] else '1'
            want.append(('ftran', w))
        elif k == 'supply:leave':
            c = e['supplier']['context']
            require(e['position'] == c['position'], 'position')
            w = dict(ret=str(e['candidate']), pos=str(c['position']), dfr=str(c['deferred']), cand=j(c['candidates']),
                     fail=j(c['failures']), sup=str(c['support']), smarks=opt(c['support_marks']),
                     csum=c['count_sum'], succ=str(c['successes']))
            if e['candidate'] >= 0:
                a = e['alpha']
                w.update(acount=str(a['count']), aidx=j(a['indices']), aval=j(a['values']))
                cnt['supply_ok'] += 1
            else:
                cnt['supply_end'] += 1
            want.append(('supply', w))
            step = dict(calls=[])
        elif k == 'ratio:enter':
            step['calls'].append([e, body[i + 1]])
            require(body[i + 1]['event'] == 'ratio:leave', 'ratio 쌍')
        elif k == 'retry:leave':
            step['calls'][-1][1] = e
        elif k in ('step:enter', 'step:defer'):
            step['enter'] = e
        elif k == 'roundctl:accepted':
            step['approved'] = e['state']['approved']
        elif k in ('step:applied', 'roundctl:defer:leave'):
            calls, se_ = step['calls'], step['enter']
            require(len(calls) in (1, 2), 'ratio 호출 수')
            kind = 'defer' if se_['event'] == 'step:defer' else se_['kind']
            w = {'path': 'V' if calls[0][0]['name'] == 'infeasible' else 'F'}
            for c, (rin, rout) in enumerate(calls):
                s = rin['selected']
                w[f'seen{c}'] = f"{s['leave']} {s['direction']} {rin['state']['counters']['request']}"
                w[f'alpha{c + 1}'] = j(rin['alpha']['values'])
                w[f'ratio{c + 1}'] = f"{rout['chosen']} {rout['theta']} {rout['pivot']}"
            fr = se_['frame']
            w['frame'] = f"{fr['r']} {fr['dL']} {fr['dU']}"
            w['step'] = f"{kind} {se_['theta']}"
            if kind == 'defer':
                sel_ = se_['selected']
                w.update(state_fields(se_['state'], 'a.'))
                rs = e['state']
                w['dfr'] = str(rs['deferred'])
                w['cand'] = j(rs['candidates'])
                w['failj'] = str(rs['failures'][sel_['enter']])
            else:
                sel_ = e['selected']
                w.update(state_fields(e['state'], 'a.'))
                w['approved'] = str(step['approved'])
            w['sel'] = f"{sel_['enter']} {sel_['leave']} {sel_['direction']}"
            want.append(('step', w))
            cnt[kind] += 1
        elif k == 'roundctl:btran:leave':
            step['braw'] = e['raw']
        elif k == 'roundctl:btran:enter':
            require(e['input']['count'] == 1 and e['input']['values'] == ['000000000000f03f'], 'BTRAN 단위 입력')
            step['bin'] = e['input']['indices'][0]
        elif k in ('roundctl:btran:applied', 'roundctl:dense'):
            w = dict(leave=str(step['bin']), **{'r' + a: b for a, b in vec_fields(step['braw']).items()})
            if k == 'roundctl:btran:applied':
                w.update(round_fields(e['state']))
                del w['req']
            else:
                cnt['dense'] += 1
            want.append(('btran', w))
            cnt['btran'] += 1
        elif k == 'factorctl:select':
            nxt = next(x for x in body[i + 1:] if x['event'] in ('factorctl:call', 'factorctl:clear'))
            call = nxt['event'] == 'factorctl:call'
            req = nxt['state']['request'] if call else 0
            want.append(('select', dict(req=str(req), call=str(int(call)))))
            cnt['select'] += 1
        elif k == 'factor:enter':
            lu_of = dict(fe=e)
        elif k == 'lu:enter':
            lu_of['le'] = e
        elif k == 'factor:leave':
            lu_of['fl'] = e
        elif k == 'lu:leave':
            fe, le, fl = lu_of['fe'], lu_of['le'], lu_of['fl']
            require(fe['header'] == le['header'], 'factor/lu header')
            site = SITE[fe['factor_ctl']['caller']]
            w = dict(site=site, req=str(fe['request']), leave=str(le['leave']), hdr=j(le['header']),
                     ret=str(fl['ret']), hdr2=j(e['header']), vstat=j(e['vstat']), leave2=str(e['leave']))
            if LU:
                w['stats'] = stats_text(fl['factor_ctl']['state'])
            want.append(('fcall', w))
            cnt['fcall_' + site] += 1
        elif k == 'recompute:ftran:enter':
            require(e['input']['count'] == -1, 'rhs FTRAN count')
            want.append(('rin', dict(rhs=j(e['input']['values']))))
        elif k == 'recompute:leave':
            ent = next(x for x in reversed(body[:i]) if x['event'] == 'recompute:enter')
            want.append(('recompute', dict(site=e['site'], hdr=j(ent['input']['header']), basic=j(e['basic']),
                                           width=j(e['width']), marks=j(e['marks']), pinf=e['pinf'],
                                           viol=str(e['violations']), retry=str(e['retry']), exit=e['exit'])))
            cnt['recompute_' + e['site']] += 1
        elif k == 'factorctl:clear':
            want.append(('clear', dict(period=str(e['frame']['period']))))
        elif k == 'roundctl:exhausted':
            nxt = next(x for x in body[i + 1:] if x['event'] in ('roundctl:restart', 'roundctl:end'))
            s = e['state']
            want.append(('exhausted', dict(pos=str(s['position']), dfr=str(s['deferred']), total=str(s['total']),
                                           sup=str(s['support']), appr=str(e['approved_live']),
                                           end=str(int(nxt['event'] == 'roundctl:end')))))
        elif k == 'roundctl:factor':
            want.append(('round', round_fields(e['state'])))
        elif k == 'roundctl:end':
            want.append(('end', round_fields(e['state'])))
            cnt['end'] += 1
    return [' '.join(t) for t in toks], want, cnt, periods


def cases():
    m = oracle.load(oracle.D / 'manifest.json')
    oracle.check_files(m)
    data = oracle.records()
    oracle.controls(data, m)
    oracle.acceptance(data)
    out, total = [], Counter()
    for name in MODELS:
        lines, want, cnt, _ = project(name, data[name][0])
        out.append((name, lines, want))
        total += cnt
    got = {k: total.get(k, 0) for k in EXPECT}
    require(got == EXPECT, f'캡처 집계가 계약 §4와 다름: {got}')
    return out, m


def write():
    cs, _ = cases()
    head = f'{SCHEMA} {hashlib.sha256((ROOT / oracle.D / "manifest.json").read_bytes()).hexdigest()}'
    body = '\n'.join(sum((lines for _, lines, _ in cs), [])) + '\n'
    D.mkdir(parents=True, exist_ok=True)
    INPUT.write_text(head + '\n' + body)
    print(f'input {INPUT.relative_to(ROOT)} 사례 {len(cs)} 줄 {body.count(chr(10))} sha256 '
          f'{hashlib.sha256(INPUT.read_bytes()).hexdigest()}')


def parse_trace(text):
    lines = text.splitlines()
    head, rest = lines[0], lines[1:]
    out, cur = {}, None
    for ln in rest:
        tag, *kv = ln.split(';')
        if tag == 'case':
            cur = kv[0]
            out[cur] = []
            continue
        d = {}
        for item in kv:
            key, _, val = item.partition(' ')
            require(key not in d, f'중복 키 {key}')
            d[key] = val
        out[cur].append((tag, d))
    return head, out


def compare_cases(cs, head, got, verbose=True):
    want_head = INPUT.read_text().splitlines()[0]
    require(head == want_head, f'trace 머리 {head!r} != 입력 {want_head!r}')
    require(list(got) == [n for n, _, _ in cs], f'사례 목록 {list(got)}')
    fields = lines = 0
    for name, _, want in cs:
        g = got[name]
        for idx, (w, r) in enumerate(zip(want, g)):
            require(w[0] == r[0], f'{name} 줄{idx}: 태그 원본 {w[0]} != Rust {r[0]}')
            wk, rk = set(w[1]), set(r[1])
            require(wk == rk, f'{name} 줄{idx} {w[0]}: 키 원본 {sorted(wk - rk)} Rust {sorted(rk - wk)}')
            for key in sorted(wk):
                if w[1][key] != r[1][key]:
                    a, b = w[1][key].split(' '), r[1][key].split(' ')
                    pos = next((p for p, (x, y) in enumerate(zip(a, b)) if x != y), min(len(a), len(b)))
                    raise Fail(f'{name} 줄{idx} {w[0]}.{key}[{pos}]: 원본 {a[pos:pos + 3]} != Rust {b[pos:pos + 3]}'
                               f' (길이 {len(a)}/{len(b)})')
                fields += 1
            lines += 1
        require(len(want) == len(g), f'{name}: 줄 수 원본 {len(want)} != Rust {len(g)}')
    if verbose:
        tags = Counter(t for _, _, want in cs for t, _ in want)
        print(f'일치: 사례 {len(cs)} 줄 {lines} 필드 {fields} 태그 {dict(sorted(tags.items()))}')
    return lines, fields


def compare(trace=None, verbose=True):
    cs, _ = cases()
    head, got = parse_trace((trace or TRACE).read_text())
    return compare_cases(cs, head, got, verbose)


def self_test():
    cs, _ = cases()
    head, got = parse_trace(TRACE.read_text())
    compare_cases(cs, head, got, verbose=False)
    muts = []

    def flip(v):
        parts = v.split(' ')
        p = parts[-1]
        if p == 'none':
            parts[-1] = '0'
        elif len(p) == 16 and all(c in '0123456789abcdef' for c in p):
            parts[-1] = p[:15] + ('0' if p[15] != '0' else '1')
        elif p.lstrip('-').isdigit():
            parts[-1] = str(int(p) + 1)
        else:
            parts[-1] = p + 'x'
        return ' '.join(parts)

    seen = set()
    for name, lines in got.items():
        for idx, (tag, d) in enumerate(lines):
            for key in d:
                if (tag, key) in seen:
                    continue
                seen.add((tag, key))
                muts.append((f'{tag}.{key}', name, idx, key))
    rejected, survived = 0, []
    for label, name, idx, key in muts:
        g = copy.deepcopy(got)
        g[name][idx][1][key] = flip(g[name][idx][1][key])
        try:
            compare_cases(cs, head, g, verbose=False)
            survived.append(label)
        except Fail:
            rejected += 1
    structural = []
    n0 = MODELS[0]
    g = copy.deepcopy(got); g[n0].pop(5); structural.append(('줄 누락', g))
    g = copy.deepcopy(got); g[n0].insert(5, g[n0][5]); structural.append(('줄 중복', g))
    g = copy.deepcopy(got); g[n0][5], g[n0][6] = g[n0][6], g[n0][5]; structural.append(('순서 바꿈', g))
    g = copy.deepcopy(got); g[n0][3][1]['extra'] = '1'; structural.append(('여분 키', g))
    g = copy.deepcopy(got); g[n0].pop(); structural.append(('끝 줄 누락', g))
    g = copy.deepcopy(got); del g[MODELS[-1]]; structural.append(('사례 누락', g))
    for label, g in structural:
        try:
            compare_cases(cs, head, g, verbose=False)
            survived.append(label)
        except Fail:
            rejected += 1
    try:
        compare_cases(cs, 'X' + head, got, verbose=False)
        survived.append('머리')
    except Fail:
        rejected += 1
    total = len(muts) + len(structural) + 1
    print(f'self-test: 변조 {total}종 중 거부 {rejected}, 생존 {survived}')
    require(not survived, f'생존 변조 {survived}')
    return dict(total=total, rejected=rejected)


# 각 변이는 (이름, 파일, 원문, 대체). 원문은 파일에 정확히 1회 있어야 한다.
MUTATIONS = [
    ('pass-skip', 'crossover_loop.rs', '        round_pass(&mut run.s.round);\n', ''),
    ('begin-skip', 'crossover_loop.rs', '            round_begin(&mut run.s.round);\n', ''),
    ('accept-skip', 'crossover_loop.rs', '            round_accept(&mut s.round);\n', ''),
    ('defer-skip', 'crossover_loop.rs', '                round_defer(&mut s.round, j);\n', ''),
    ('flip-goes-on', 'crossover_loop.rs', '            if s.st.leave < 0 {\n                return Next::Supply;',
     '            if s.st.leave < -1 {\n                return Next::Supply;'),
    ('btran-always', 'crossover_loop.rs', '            if s.round.support != 0 {\n                let leave',
     '            if s.round.support >= 0 {\n                let leave'),
    ('no-clear-request', 'crossover_loop.rs', '        if !decision.call {\n            s.round.request = 0;',
     '        if !decision.call {\n            s.round.request = s.round.request;'),
    ('select-support0', 'crossover_loop.rs', 'u_nnz: stats.u_nnz, support: s.round.support }',
     'u_nnz: stats.u_nnz, support: 0 }'),
    ('pivot-no-recompute', 'crossover_loop.rs',
     '                    if self.recompute(FactorSite::Pivot) == RecomputeExit::Retry {',
     '                    if false && self.recompute(FactorSite::Pivot) == RecomputeExit::Retry {'),
    ('round-no-recompute', 'crossover_loop.rs',
     '        if self.recompute(FactorSite::Round) == RecomputeExit::Retry {',
     '        if false && self.recompute(FactorSite::Round) == RecomputeExit::Retry {'),
    ('round-no-clear', 'crossover_loop.rs', '        round_clear(&mut s.round);\n', ''),
    ('no-renumber', 'crossover_loop.rs', '    if let Some(r) = renumbering {\n        renumber(st, &r);',
     '    if let Some(_r) = renumbering {\n'),
    ('renumber-no-vstat', 'crossover_loop.rs', '        st.vstat[j as usize] = p as i32;', '        let _ = (p, j);'),
    ('renumber-no-leave', 'crossover_loop.rs', '        st.leave = r.new_of_old[st.leave as usize];',
     '        st.leave = st.leave;'),
    ('renumber-inverse', 'crossover_loop.rs', '        *h = old[r.old_of_new[p] as usize];',
     '        *h = old[r.new_of_old[p] as usize];'),
    ('exhausted-skip', 'crossover_loop.rs', '            let exit = round_exhausted(&mut run.s.round);',
     '            let exit = RoundExit::Continue;'),
    ('restart-skip', 'crossover_loop.rs', '            round_restart(&mut run.s.round);\n', '            run.s.round.request = 10;\n'),
    ('end-on-empty-skip', 'crossover_loop.rs', '        if s.round.candidates.is_empty() {\n            (self.trace)(LoopEvent::End(&s.round));',
     '        if s.round.candidates.is_empty() && false {\n            (self.trace)(LoopEvent::End(&s.round));'),
    ('pinf-not-saved', 'crossover_loop.rs', '        s.pinf = rs.pinf;\n', ''),
    ('violations-not-saved', 'crossover_loop.rs', '        st.violations = rs.violations;\n', ''),
    ('request-not-synced', 'crossover_loop.rs', '            s.st.request = s.round.request;\n', ''),
    ('period-not-saved', 'crossover_loop.rs', '            s.period = frame.period;\n', ''),
    ('updates-stale', 'crossover_loop.rs', '        state.updates = self.backend.stats().updates;\n        ret',
     '        ret'),
    ('resume-restarts', 'crossover_factor.rs', '        Tail::Again => factor_run(state, frame, vectors, backend),\n        Tail::Exit(exit) => exit,',
     '        Tail::Again => factor_run(state, frame, vectors, backend),\n        Tail::Exit(_) => FactorExit::Continue,'),
    ('resume-no-period', 'crossover_factor.rs', '    } else if state.updates != 0 && frame.period > 50 {\n        frame.period = 50;',
     '    } else if state.updates != 0 && frame.period > 50 {\n        frame.period = 51;'),
]


# M3-04c-24: 실제 backend 변이(자연 경로를 지나면 거부돼야 함). 루프 제어 변이는 22 목록과 같은 효과라 그대로 함께 돌린다.
LU_MUTATIONS = [
    ('column-work-skip', 'crossover_backend.rs', '            self.add_column_work(workspace.expect("update without a work column"));\n', ''),
    ('column-work-no-plus1', 'crossover_backend.rs', '            i64::from(w.count) + 1\n', '            i64::from(w.count)\n'),
    ('column-work-no-reset', 'crossover_backend.rs', '        self.column_work = 0;\n        let alloc', '        let alloc'),
    ('u-stat-no-m', 'crossover_backend.rs', '        lu.re[m - 1] + m as i64', '        lu.re[m - 1]'),
    ('u-stat-stale', 'crossover_backend.rs', '        self.u_nnz = u_stat(&lu);\n        self.lu = lu;', '        self.lu = lu;'),
    ('no-wrapper-clear', 'crossover_backend.rs', '            self.lu.wrapper_clear();\n', ''),
    ('supplier-save0', 'crossover_backend.rs', '.ftran(input.count, &input.indices, &input.values, with_spike as i32, with_spike)', '.ftran(input.count, &input.indices, &input.values, 0, with_spike)'),
    ('supplier-no-spike', 'crossover_backend.rs', '.ftran(input.count, &input.indices, &input.values, with_spike as i32, with_spike)', '.ftran(input.count, &input.indices, &input.values, with_spike as i32, false)'),
    ('ft-header-stale', 'crossover_backend.rs', '            self.lu.header = st.header.clone();\n', ''),
    ('ft-leave-zero', 'crossover_backend.rs', '            self.lu.ft_update(st.leave as usize)', '            self.lu.ft_update(0)'),
    ('refactor-no-lu-renumber', 'crossover_backend.rs', '        lu.renumber(&st.header, st.leave, &st.vstat, &rf.o, &rf.n);\n', ''),
    ('refactor-prev-none', 'crossover_backend.rs', 'alloc, Some(&self.lu), &mut self.block)', 'alloc, None, &mut self.block)'),
    ('refactor-block-off', 'crossover_backend.rs', 'alloc, Some(&self.lu), &mut self.block)', 'alloc, Some(&self.lu), &mut BlockState::OFF.clone())'),
    ('refactor-block-no-raise', 'crossover_backend.rs', '        self.block.x2c0 += 1;\n', ''),
    ('initial-block-no-raise', 'crossover_backend.rs', '        block.x2c0 += 1;\n        let mut lu = rf.state;\n        let r =', '        let mut lu = rf.state;\n        let r ='),
    ('btran-no-cache', 'crossover_backend.rs', 'input.values, 0, true, 0.0)', 'input.values, 0, false, 0.0)'),
    ('rhs-ftran-save1', 'crossover_backend.rs', 'self.lu.ftran(-1, &[], rhs, 0, false)', 'self.lu.ftran(-1, &[], rhs, 1, false)'),
    ('stats-interval-zero', 'crossover_backend.rs', '            interval: self.interval,\n', '            interval: 0,\n'),
    ('loop-no-workspace', 'crossover_loop.rs', 'FactorSite::Pivot, &mut ctl, Some(vectors.workspace), self.trace)', 'FactorSite::Pivot, &mut ctl, None, self.trace)')] + MUTATIONS


def mutate_rust(out):
    require(not out.exists(), f'{out} 이미 있음(덮어쓰기 거부)')
    cs, _ = cases()
    work = ROOT / ('target/m3_push_loop_lu_mutations' if LU else 'target/m3_push_loop_mutations')
    if work.exists():
        shutil.rmtree(work)
    work.mkdir(parents=True)
    for p in ('Cargo.toml', 'Cargo.lock', 'crates', str(INPUT.relative_to(ROOT))):
        src = ROOT / p
        dst = work / p
        dst.parent.mkdir(parents=True, exist_ok=True)
        if src.is_dir():
            shutil.copytree(src, dst, ignore=shutil.ignore_patterns('target'))
        else:
            shutil.copy2(src, dst)
    results = []
    for name, fname, old, new in (LU_MUTATIONS if LU else MUTATIONS):
        f = work / 'crates/lp/src' / fname
        orig = (ROOT / 'crates/lp/src' / fname).read_text()
        require(orig.count(old) == 1, f'{name}: 원문 {orig.count(old)}회')
        f.write_text(orig.replace(old, new))
        # 빌드는 먼저 끝내고 재생만 시간 제한: 실제 backend에서는 tape 고갈이 없어 루프가 끝나지 않는 변이가 있다
        subprocess.run(['cargo', 'test', '-q', '--no-run', '-p', 'rsolver-lp', '--test', TEST], cwd=work,
                       capture_output=True, text=True, timeout=900)
        trace = work / TRACE.relative_to(ROOT)
        try:
            r = subprocess.run(['cargo', 'test', '-q', '-p', 'rsolver-lp', '--test', TEST], cwd=work,
                               capture_output=True, text=True, timeout=120)
        except subprocess.TimeoutExpired:
            r = None
        if r is None:
            verdict, why = 'rejected-timeout', ['재생 120초 초과(루프 미종료)']
        elif r.returncode != 0:
            text = (r.stdout + r.stderr).splitlines()
            why = [ln.strip() for ln in text if 'panicked' in ln or ln.strip().startswith(('left', 'right', 'assertion'))][:3]
            verdict, why = 'rejected-run', why or text[-1:]
        else:
            try:
                head, got = parse_trace(trace.read_text())
                compare_cases(cs, head, got, verbose=False)
                verdict, why = 'survived', []
            except Fail as e:
                verdict, why = 'rejected', [str(e)[:300]]
        f.write_text(orig)
        if trace.exists():
            trace.unlink()
        results.append(dict(name=name, file=fname, verdict=verdict, why=why))
        print(name, verdict, *why)
    out.write_text(json.dumps(dict(task='M3-04c-24' if LU else 'M3-04c-22', results=results,
                                   rejected=sum(r['verdict'] != 'survived' for r in results),
                                   total=len(results)), ensure_ascii=False, indent=2) + '\n')
    surv = [r['name'] for r in results if r['verdict'] == 'survived']
    print(f'변이 {len(results)}종 중 거부 {len(results) - len(surv)}, 생존 {surv}')


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    g = p.add_mutually_exclusive_group(required=True)
    g.add_argument('--write', action='store_true')
    g.add_argument('--compare', action='store_true')
    g.add_argument('--self-test', action='store_true')
    g.add_argument('--mutate-rust', type=Path)
    p.add_argument('--lu', action='store_true')
    p.add_argument('--init', action='store_true')
    a = p.parse_args()
    if a.init:
        use_init()
    elif a.lu:
        use_lu()
    if a.write:
        write()
    elif a.compare:
        compare()
    elif a.self_test:
        self_test()
    else:
        mutate_rust(a.mutate_rust)


if __name__ == '__main__':
    try:
        main()
    except Fail as exc:
        print('FAIL', exc, file=sys.stderr)
        sys.exit(1)
