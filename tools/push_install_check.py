#!/usr/bin/env python3
"""M3-04f-1: primal push 해 설치(134a87 C 2419–2582) 원본 캡처 검사기.

  python3 tools/push_install_check.py --controls --acceptance   # 재현성·1c 비간섭·C 규칙 재계산
  python3 tools/push_install_check.py --self-test                # 변조·규칙 변이 거부
  python3 tools/push_install_check.py --write-manifest           # 정식 캡처 뒤 1회

사례: 1b 15사례 + 설치 합성 5(synthetic.json), Netlib 소형 45(Crossover=3), 같은 45(Crossover=-1, auto/).
acceptance는 install:enter 상태(+설치 중 편집)와 설치 중 조건수만으로 push:leave의 x·slack·vstat·S+0x120·retry·remaining을
재계산하고, 설치 중 116fd7(icond:*)은 1c check_cond로 비트 재계산한다. 계약: docs/m3/primal_push_install.md.
"""
import argparse, copy, json, os, struct, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import push_cond_check as pcc  # noqa: E402
import push_init_check as pic  # noqa: E402
import push_init_runs as pir  # noqa: E402
import push_init_survey as sv  # noqa: E402

ROOT = pic.ROOT
DATA = os.path.join(ROOT, 'bench/oracle/m3_push_install')
MANIFEST = os.path.join(DATA, 'manifest.json')
SYNTH = json.load(open(os.path.join(DATA, 'synthetic.json')))
TOOLS = ['tools/push_install_agent.js', 'tools/push_install_capture.js', 'tools/push_install_runs.py',
         'bench/oracle/m3_push_install/synthetic.json']
SCHEMA = 'M3-04f-1-v1'
# C·asm 규칙(§2). 변이는 하나씩 바꾼다.
RULES = dict(wb_first=True, wb_lb='gt', pinf_free='and', pinf_neg='gt', pinf_lo='gt', pinf_w='gt', vstat_cmp='gt',
             free='first-any', env=-1, thr_min=5, retry_p='status2', snap=True, retry_rem='count', cond_1e30=True)
f, hx = pcc.f, pcc.hx


def synthetic():
    return [k for k in SYNTH if not k.startswith('_')]


def cases():
    return pic.cases() + synthetic() + [f'netlib/{x}' for x in pir.SURVEY45] + [f'auto/{x}' for x in pir.SURVEY45]


def path(case, r):
    p = os.path.join(DATA, f'{case}-{r}.json')
    return p if os.path.exists(p) else p + '.gz'


def load_all():
    return {c: [sv.load(path(c, r)) for r in (1, 2, 3)] for c in cases()}


# ---------------------------------------------------------------- 통제
def controls(docs, manifest):
    errs = []
    if manifest.get('binary_sha256') != sv.BINARY_SHA:
        errs.append('manifest binary_sha256')
    for t in TOOLS:
        if manifest.get('tools', {}).get(t) != sv.sha(os.path.join(ROOT, t)):
            errs.append(f'manifest tool {t}')
    for c, runs in docs.items():
        for r, d in enumerate(runs, 1):
            if manifest.get('captures', {}).get(os.path.relpath(path(c, r), ROOT)) != sv.sha(path(c, r)):
                errs.append(f'{c}-{r} manifest 해시')
            if d.get('binary_sha256') != sv.BINARY_SHA:
                errs.append(f'{c}-{r} binary_sha256')
            if d.get('agent_sha256') != manifest.get('tools', {}).get('tools/push_install_agent.js'):
                errs.append(f'{c}-{r} agent 해시')
            if d.get('errors') or d.get('reason') != 'process-terminated' or not isinstance((d.get('result') or {}).get('status'), int):
                errs.append(f'{c}-{r} 실행 실패')
            if not d.get('events'):
                errs.append(f'{c}-{r} 사건 없음')
            if d.get('schema') != SCHEMA:
                errs.append(f'{c}-{r} schema')
            if any(e['event'].startswith('unsupported') for e in d.get('events', [])):
                errs.append(f'{c}-{r} x87 경로')
        base = pic.strip_doc(runs[0])
        for r, d in enumerate(runs[1:], 2):
            s = pic.strip_doc(d)
            if s != base:
                k = next(k for k in sorted(set(base) | set(s)) if base.get(k) != s.get(k))
                i = ''
                if k == 'events':
                    i = next((i for i, (a, b) in enumerate(zip(base['events'], s['events'])) if a != b),
                             min(len(base['events']), len(s['events'])))
                errs.append(f'{c} run{r}≠run1: {k} {i}')
    return errs


def project_1c(d):
    """install:*·icond:*을 빼고 push:leave를 1c 필드로 줄인 사건열."""
    out = []
    for e in d['events']:
        if e['event'].startswith(('install:', 'icond:')):
            continue
        if e['event'] == 'push:leave':
            e = {k: v for k, v in e.items() if k != 'leave_state'}
        out.append(e)
    return out


def noninterference(docs):
    """1c 사례(1b 15 + Netlib 45)의 run 1 투영이 1c run 1과 같다."""
    errs = []
    for c in pcc.cases():
        old = sv.load(pcc.path(c, 1))
        mine = docs[c][0]
        evs = project_1c(mine)
        if evs != old['events']:
            i = next((i for i, (a, b) in enumerate(zip(old['events'], evs)) if a != b), min(len(old['events']), len(evs)))
            errs.append(f'{c}: 1c 투영 사건 {i} 다름')
        for k in ('config', 'params', 'result', 'model_sha256', 'kind', 'origin'):
            if mine.get(k) != old.get(k):
                errs.append(f'{c}: 1c {k} 다름')
    return errs


# ---------------------------------------------------------------- C 규칙 재계산
def gt(a, b, rule):
    return a > b if rule == 'gt' else a >= b


def install(inp, rules):
    """§2 규칙 1–6. inp: n·m·first·attempt·status·cands·x46c·env·tau·inf·header·basic·width·vstat·xs·lb·ub·cond(편집 뒤)."""
    n, m, tau, inf = inp['n'], inp['m'], inp['tau'], inp['inf']
    lb, ub, xs, vstat = inp['lb'], inp['ub'], list(inp['xs']), list(inp['vstat'])
    if inp['first'] and rules['wb_first']:
        for i, v in enumerate(inp['header']):
            b = inp['basic'][i]
            xs[v] = lb[v] + b if v < n and gt(lb[v], -inf, rules['wb_lb']) else b
    p, viol = 0.0, 0
    for i, v in enumerate(inp['header']):
        free = (not (-inf < lb[v])) and ub[v] >= inf if rules['pinf_free'] == 'and' else ub[v] >= inf
        if free:
            continue
        b, w = inp['basic'][i], inp['width'][i]
        if gt(-tau, b, rules['pinf_neg']):
            viol += 1
            p -= b
            continue
        if gt(tau, b, rules['pinf_lo']) or w - tau > b or w >= inf or gt(tau + w, b, rules['pinf_w']):
            continue
        viol += 1
        p += b - w
    cond = inp['cond']
    pp = p if (inp['status'] == 2 or rules['retry_p'] == 'always') else 1.0
    c = len(inp['cands'])
    retry = (inp['attempt'] == 0 and (cond > 1e14 or pp > 1e10 or pp * cond > 1e16)) or (rules['cond_1e30'] and cond > 1e30)
    if retry:
        return dict(xs=xs, vstat=vstat, pinf=p, viol=viol, retry=1, remaining=c if rules['retry_rem'] == 'count' else 0)
    rem = c
    if c:
        if rules['free'] == 'first-any' and any((not (-inf < lb[v])) and ub[v] >= inf for v in inp['cands']):
            rem = -c
        elif inp['env'] == rules['env']:
            t = max(rules['thr_min'], int(inp['x46c'] / 10))
            rem = c if c >= t else -c
    if rem > 0:
        for v in inp['cands'][:rem]:
            vstat[v] = -1 if gt(lb[v] + ub[v], xs[v] * 2.0, rules['vstat_cmp']) else -2
    elif rem < 0 and rules['snap']:
        for j, s in enumerate(vstat):
            if s == -1:
                xs[j] = lb[j]
            elif s == -2:
                xs[j] = ub[j]
    return dict(xs=xs, vstat=vstat, pinf=p, viol=viol, retry=0, remaining=rem)


def install_input(evs, ie):
    """push:enter 모델 bound(설치 편집 반영)와 install:enter 상태."""
    pe = evs[0]
    st = ie['state']
    lb = [f(h) for h in pe['model']['lb']]
    ub = [f(h) for h in pe['model']['ub']]
    for e in evs:
        if e['event'] == 'edit' and e['at'] == 'install' and e['field'] == 'free_first':
            lb[e['index']], ub[e['index']] = f(e['after'][0]), f(e['after'][1])
    cl = [e for e in evs if e['event'] == 'install:condition:leave']
    return dict(n=st['n'], m=st['m'], first=ie['first'], attempt=ie['attempt'], status=ie['model_status'],
                cands=ie['candidates'], x46c=ie['x46c'], env=ie['env_2ba0'], tau=f(ie['tau']), inf=f(ie['inf']),
                header=st['header'], basic=[f(h) for h in st['basic']], width=[f(h) for h in st['width']],
                vstat=st['vstat'], xs=[f(h) for h in st['x'] + st['slack']], lb=lb, ub=ub,
                cond=f(cl[0]['condition']) if cl else None)


def check_call(tag, evs, rules, summ):
    errs = []
    pe = evs[0]
    pl = [e for e in evs if e['event'] == 'push:leave']
    if pe['event'] != 'push:enter' or len(pl) != 1:
        return [f'{tag}: push:enter/leave 구조']
    pl = pl[0]
    ies = [e for e in evs if e['event'] == 'install:enter']
    if not ies:
        if pl['leave_state']['installed']:
            errs.append(f'{tag}: 설치 진입 없이 installed')
        summ.append(dict(tag=tag, install=False, retry=pl['retry'], remaining=pl['remaining']))
        return errs
    if len(ies) != 1:
        return [f'{tag}: install:enter {len(ies)}회']
    ie = ies[0]
    st, ls = ie['state'], pl['leave_state']
    n, m = st['n'], st['m']
    if not all(ie['ptrs'].values()):
        errs.append(f'{tag}: 설치 slot 별칭 {[k for k, v in ie["ptrs"].items() if not v]}')
    tol = st['tolerances']
    if (ie['m_slot'], ie['n_slot'], ie['nm_slot']) != (m, n, n + m) or ie['tau'] != tol['380'] or ie['inf'] != tol['370'] \
            or ie['neg_inf'] != hx(-f(tol['370'])) or ie['neg_tau'] != hx(-f(tol['380'])):
        errs.append(f'{tag}: 설치 slot 값(m·n·τ·I)')
    if ie['first'] != pe['first'] or ie['attempt'] != pe['attempt'] or ie['model_status'] != pe['model_status']:
        errs.append(f'{tag}: first·attempt·param_3 != push:enter')
    if ie['count'] != len(ie['candidates']):
        errs.append(f'{tag}: 후보 수')
    # 설치 중 116fd7: 1c check_cond(사건 이름만 cond:로)
    ic = [i for i, e in enumerate(evs) if e['event'] == 'icond:enter']
    cle = [e for e in evs if e['event'] == 'install:condition:leave']
    if len(ic) != 1 or len(cle) != 1:
        return errs + [f'{tag}: 설치 조건수 호출 {len(ic)}/{len(cle)}']
    end = next((q for q in range(ic[0], len(evs)) if evs[q]['event'] == 'icond:leave'), None)
    if end is None:
        return errs + [f'{tag}: icond:leave 없음']
    blk = [dict(e, event='cond:' + e['event'][6:]) for e in evs[ic[0]:end + 1]]
    cs = []
    errs += pcc.check_cond(tag + ' icond', blk, pe['model'], pcc.RULES, cs)
    edited = [e for e in evs if e['event'] == 'edit' and e['at'] == 'install_cond']
    want_cl = edited[0]['after'] if edited else blk[-1]['condition']
    if edited and edited[0]['before'] != blk[-1]['condition']:
        errs.append(f'{tag}: 조건수 편집 전 값 != icond:leave')
    if cle[0]['condition'] != want_cl or pl['condition'] != want_cl:
        errs.append(f'{tag}: 설치 condition 경로(icond → install:condition:leave → push:leave)')
    inp = install_input(evs, ie)
    want = install(inp, rules)
    got = dict(xs=[f(h) for h in ls['x'] + ls['slack']], vstat=ls['vstat'], pinf=f(ls['pinf']), retry=pl['retry'],
               remaining=pl['remaining'])
    for k in ('retry', 'remaining', 'vstat'):
        if want[k] != got[k]:
            i = next((i for i, (a, b) in enumerate(zip(want[k], got[k])) if a != b), None) if k == 'vstat' else None
            errs.append(f'{tag}: {k}{"" if i is None else f"[{i}]"} 재계산 {want[k] if i is None else want[k][i]} 원본 {got[k] if i is None else got[k][i]}')
    if hx(want['pinf']) != ls['pinf'] or pl['extra']['s120'] != ls['pinf']:
        errs.append(f'{tag}: S+0x120 재계산 {hx(want["pinf"])} 원본 {ls["pinf"]}')
    wx = [hx(v) for v in want['xs']]
    gx = ls['x'] + ls['slack']
    if wx != gx:
        i = next(i for i, (a, b) in enumerate(zip(wx, gx)) if a != b)
        errs.append(f'{tag}: xs[{i}] 재계산 {wx[i]} 원본 {gx[i]}')
    for k in ('header', 'basic', 'width'):
        if ls[k] != st[k]:
            errs.append(f'{tag}: {k} 설치 중 변경')
    if not ls['installed'] or pl['ret'] != 0:
        errs.append(f'{tag}: 반환 {pl["ret"]}·installed')
    wb = sum(1 for i, v in enumerate(st['header']) if inp['first'] and hx(inp['xs'][v]) != wx[v])
    summ.append(dict(tag=tag, install=True, first=ie['first'], from_init=ie['from_init'], count=ie['count'],
                     attempt=ie['attempt'], status=ie['model_status'], env=ie['env_2ba0'], retry=pl['retry'],
                     remaining=pl['remaining'], pinf=want['pinf'], viol=want['viol'], cond=f(pl['condition']),
                     cond_edit=bool(edited), changed_x=wb, cond_iters=cs[0]['iters'] if cs else None,
                     snap=sum(1 for a, b in zip(inp['xs'], want['xs']) if hx(a) != hx(b)) if want['remaining'] < 0 else 0,
                     free=any((not (-inp['inf'] < inp['lb'][v])) and inp['ub'][v] >= inp['inf'] for v in ie['candidates'])))
    return errs


def acceptance(docs, rules=RULES):
    errs, summ = [], []
    for c in cases():
        for call, evs in sorted(pic.calls_of(docs[c][0]).items()):
            try:
                errs += check_call(f'{c} call{call}', evs, rules, summ)
            except Exception as e:  # noqa: BLE001 — 형식 오류도 거부로 센다
                errs.append(f'{c} call{call}: 예외 {type(e).__name__}: {e}')
    if not any(s['install'] for s in summ):
        errs.append('설치 0회')
    return errs, summ


BRANCHES = {
    'first-writeback': lambda s: s['install'] and s['first'] and s['changed_x'] > 0,
    'first0': lambda s: s['install'] and not s['first'],
    'from-init': lambda s: s['install'] and s['from_init'],
    'pinf>0': lambda s: s['install'] and s['pinf'] > 0,
    'rem0': lambda s: s['install'] and s['count'] == 0,
    'rem>0': lambda s: s['install'] and s['remaining'] > 0 and not s['retry'],
    'threshold-neg': lambda s: s['install'] and s['remaining'] < 0 and not s['free'],
    'free': lambda s: s['install'] and s['remaining'] < 0 and s['free'],
    'snap': lambda s: s['install'] and s['snap'] > 0,
    'retry': lambda s: s['install'] and s['retry'] and s['attempt'] == 0,
    'att1-no-retry': lambda s: s['install'] and not s['retry'] and s['attempt'] == 1 and s['cond'] > 1e14,
    'retry-1e30': lambda s: s['install'] and s['retry'] and s['attempt'] == 1,
    'no-install': lambda s: not s['install'],
}


def branch_counts(summ):
    return {k: sum(1 for s in summ if fn(s)) for k, fn in BRANCHES.items()}


def required_missing(summ):
    """synthetic.json의 require가 그 사례에서 실제로 지났는지."""
    out = []
    for name in synthetic():
        mine = [s for s in summ if s['tag'].startswith(name + ' ')]
        for r in SYNTH[name]['require']:
            if not any(BRANCHES[r](s) for s in mine):
                out.append(f'{name}: 필수 분기 {r} 미경유')
    return out


# ---------------------------------------------------------------- self-test
def _ev(doc, kind, k=0):
    return [e for e in doc['events'] if e['event'] == kind][k]


def _flip(h):
    b = bytearray(bytes.fromhex(h)); b[0] ^= 1; return b.hex()


def _leave(D, c, k=0):
    return _ev(D[c][0], 'push:leave', k)


TAMPERS = [
    ('바이너리 해시 0', 'controls', lambda D: D['arms'][0].__setitem__('binary_sha256', '0' * 64)),
    ('사건 비우기', 'controls', lambda D: D['interior'][1].__setitem__('events', [])),
    ('run3 leave x 1비트', 'controls', lambda D: (lambda s: s['x'].__setitem__(0, _flip(s['x'][0])))(_ev(D['capri'][2], 'push:leave')['leave_state'])),
    ('1c 비간섭: 116fd7 출력 1비트', 'nonint', lambda D: (lambda e: e.__setitem__('condition', _flip(e['condition'])))(_ev(D['netlib/scsd1'][0], 'cond:leave'))),
    ('leave remaining 부호', 'accept', lambda D: _leave(D, 'inst_env_neg').__setitem__('remaining', 1)),
    ('leave retry 0', 'accept', lambda D: _leave(D, 'inst_retry').__setitem__('retry', 0)),
    ('leave vstat -1↔-2', 'accept', lambda D: (lambda s, v: s['vstat'].__setitem__(v, -3 - s['vstat'][v]))(_leave(D, 'arms')['leave_state'], _ev(D['arms'][0], 'install:enter')['candidates'][0])),
    ('leave x 1비트(기록된 basic 열)', 'accept', lambda D: (lambda s, v: s['x'].__setitem__(v, _flip(s['x'][v])))(_leave(D, 'netlib/capri')['leave_state'], next(v for v in _ev(D['netlib/capri'][0], 'install:enter')['state']['header'] if v < _ev(D['netlib/capri'][0], 'install:enter')['state']['n']))),
    ('leave slack 1비트', 'accept', lambda D: (lambda s: s['slack'].__setitem__(0, _flip(s['slack'][0])))(_leave(D, 'netlib/afiro')['leave_state'])),
    ('leave S+0x120 1비트', 'accept', lambda D: (lambda s: s.__setitem__('pinf', _flip(s['pinf'])))(_leave(D, 'retry')['leave_state'])),
    ('install basic 1비트', 'accept', lambda D: (lambda s: s['basic'].__setitem__(0, _flip(s['basic'][0])))(_ev(D['netlib/capri'][0], 'install:enter')['state'])),
    ('install 후보 목록 비우기', 'accept', lambda D: _ev(D['arms'][0], 'install:enter').__setitem__('candidates', [])),
    ('install env -1 → 3', 'accept', lambda D: _ev(D['inst_env_neg'][0], 'install:enter').__setitem__('env_2ba0', 3)),
    ('install first 0', 'accept', lambda D: _ev(D['netlib/capri'][0], 'install:enter').__setitem__('first', 0)),
    # 반복 0의 출력은 다음 반복이 이어지면 결과에 들어가지 않는다(부호 불변이면 est 비교만): 1c처럼 결과를 정하는 k=1을 바꾼다.
    ('icond 마지막 FTRAN 출력 상위 가수 1비트', 'accept', lambda D: (lambda e: e['output']['val'].__setitem__(0, pcc._flip6(e['output']['val'][0])))(_ev(D['netlib/capri'][0], 'icond:ftran:leave', 1))),
    ('install:condition:leave 1비트', 'accept', lambda D: (lambda e: e.__setitem__('condition', _flip(e['condition'])))(_ev(D['adlittle'][0], 'install:condition:leave'))),
    ('icond 사건 제거', 'accept', lambda D: D['scsd1'][0].__setitem__('events', [e for e in D['scsd1'][0]['events'] if not e['event'].startswith('icond:')])),
    ('설치 slot 별칭 거짓', 'accept', lambda D: _ev(D['interior'][0], 'install:enter')['ptrs'].__setitem__('list', False)),
    ('header 설치 중 변경', 'accept', lambda D: (lambda s: s['header'].reverse())(_leave(D, 'netlib/capri')['leave_state'])),
]
MUTATIONS = [
    # (이름, 규칙, 필수) — 필수는 자료가 구별해야 하는 것(결과를 보기 전에 정함)
    ('first여도 x 기록 안 함', dict(wb_first=False), True),
    ('x 기록 lb ≥ -I', dict(wb_lb='ge'), False),
    ('PInf 자유 판정 ub만', dict(pinf_free='ub'), False),
    ('PInf 하한 위반 ≥', dict(pinf_neg='ge'), False),
    ('PInf 하한 τ ≥', dict(pinf_lo='ge'), False),
    ('PInf 상한 τ+w ≥', dict(pinf_w='ge'), False),
    ('vstat lb+ub ≥ 2x', dict(vstat_cmp='ge'), False),
    ('자유 후보 무시', dict(free='none'), True),
    ('임계값 환경값 3', dict(env=3), True),
    ('임계값 최소 1', dict(thr_min=1), True),
    ('P = PInf(상태 무관)', dict(retry_p='always'), False),
    ('음수 remaining 스냅 없음', dict(snap=False), True),
    ('retry remaining 0', dict(retry_rem='zero'), True),
    ('cond > 1e30 항 없음', dict(cond_1e30=False), True),
]


def self_test(docs, manifest):
    bad = 0
    base_c = controls(docs, manifest) + noninterference(docs)
    base_a, summ = acceptance(docs)
    base_a += required_missing(summ)
    if base_c or base_a:
        print('기준 자료가 먼저 실패:', (base_c + base_a)[:5])
        return 1
    for name, layer, fn in TAMPERS:
        D = copy.deepcopy(docs)
        try:
            fn(D)
            errs = controls(D, manifest) if layer == 'controls' else noninterference(D) if layer == 'nonint' else acceptance(D)[0]
        except Exception as e:  # noqa: BLE001
            errs = [f'예외 {type(e).__name__}: {e}']
        print(('REJECTED     ' if errs else 'NOT REJECTED ') + name + (f'  ← {errs[0]}' if errs else ''))
        bad += not errs
    for name, mut, required in MUTATIONS:
        errs, _ = acceptance(docs, dict(RULES, **mut))
        tag = 'REJECTED     ' if errs else ('NOT REJECTED ' if required else 'SURVIVED     ')
        print(tag + '변이 ' + name + (f'  ← {errs[0]}' if errs else ''))
        bad += required and not errs
    print('필수 변조/변이 통과', bad)
    return 1 if bad else 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--controls', action='store_true')
    ap.add_argument('--acceptance', action='store_true')
    ap.add_argument('--self-test', action='store_true')
    ap.add_argument('--write-manifest', action='store_true')
    a = ap.parse_args()
    if a.write_manifest:
        if os.path.exists(MANIFEST):
            sys.exit('기존 manifest 덮어쓰기 거부')
        man = dict(task='M3-04f-1', binary_sha256=sv.BINARY_SHA, schema=SCHEMA,
                   tools={t: sv.sha(os.path.join(ROOT, t)) for t in TOOLS},
                   captures={os.path.relpath(path(c, r), ROOT): sv.sha(path(c, r)) for c in cases() for r in (1, 2, 3)},
                   baseline='bench/oracle/m3_push_condition (1c manifest)', baseline_sha256=sv.sha(pcc.MANIFEST))
        open(MANIFEST, 'w').write(json.dumps(man, indent=1, ensure_ascii=False) + '\n')
        print('manifest 작성', len(man['captures']))
        return 0
    docs = load_all()
    manifest = json.load(open(MANIFEST))
    if manifest.get('baseline_sha256') != sv.sha(pcc.MANIFEST):
        print('1c manifest 해시 불일치')
        return 1
    if a.self_test:
        return self_test(docs, manifest)
    errs = []
    if a.controls:
        e = controls(docs, manifest) + noninterference(docs)
        print('controls', '통과' if not e else f'실패 {len(e)}', f'(사례 {len(docs)} × 3회, 1c 비간섭 {len(pcc.cases())}사례)')
        errs += e
    if a.acceptance:
        e, summ = acceptance(docs)
        e += required_missing(summ)
        ins = [s for s in summ if s['install']]
        print('acceptance', '통과' if not e else f'실패 {len(e)}',
              f'(push 호출 {len(summ)}, 설치 {len(ins)}, 설치 116fd7 {sum(s["cond_iters"] is not None for s in ins)})')
        print('  분기', branch_counts(summ))
        for s in summ:
            if not s['install'] or s['count'] or s['retry'] or s['pinf'] or s['cond_edit']:
                print('  ', {k: v for k, v in s.items()})
        errs += e
    for x in errs[:30]:
        print('  ', x)
    return 1 if errs else 0


if __name__ == '__main__':
    sys.exit(main())
