#!/usr/bin/env python3
"""M3-04c-1b: push 초기화 중간 경계 오라클 검사기.

  python3 tools/push_init_check.py --controls --acceptance   # manifest·3회·21h 비간섭·재계산 규칙·필수 증거
  python3 tools/push_init_check.py --self-test               # 변조·규칙 변이 판별력(필수 변조가 통과하면 종료 1)
  python3 tools/push_init_check.py --write-manifest          # 최초 작성(기존 파일 덮어쓰기 거부)

재계산 규칙은 134a87 C 401–1142(조사 primal_push_init.md §2.1)와 089d8f·061bfe C를 따른다. 비교는 f64 비트·정확.
"""
import argparse, copy, gzip, hashlib, json, math, os, struct, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import push_init_survey as sv  # noqa: E402  (aabbe3 이식·분류·폭·21h 로더)

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = os.path.join(ROOT, 'bench/oracle/m3_push_init')
MANIFEST = os.path.join(DATA, 'manifest.json')
BINARY_SHA = sv.BINARY_SHA
NATURAL = ['arms', 'retry', 'interior', 'adlittle', 'scsd1', 'capri']
DUAL = ['adlittle_x1']
TOOLS = ['tools/push_init_agent.js', 'tools/push_init_capture.js', 'tools/push_init_runs.py']
ZERO = '0' * 16
f64, hx = sv.f64, sv.hx

RULES = dict(slack_m2='stale', m5='ub', rebuild_skip_m45=True, tau_mult=True, promote_lb='le', mk_hi=1e-8,
             mk_lo=4e-9, numeric=1e-6, limit_rule='061bfe', gate_p='status', count_strict=True, shift=True,
             residual_order='header')


def synthetic():
    return {k: v for k, v in json.load(open(os.path.join(DATA, 'synthetic.json'))).items() if not k.startswith('_')}


def cases():
    return NATURAL + DUAL + list(synthetic())


def path(case, r):
    p = os.path.join(DATA, f'{case}-{r}.json')
    return p if os.path.exists(p) else p + '.gz'


def load_all():
    return {c: [sv.load(path(c, r)) for r in (1, 2, 3)] for c in cases()}


# ---------------------------------------------------------------- 통제
def strip_doc(d):
    return {k: v for k, v in d.items() if k not in ('pid', 'run_id', 'output')}


def controls(docs, manifest):
    errs = []
    if manifest.get('binary_sha256') != BINARY_SHA:
        errs.append('manifest binary_sha256')
    for t in TOOLS:
        if manifest.get('tools', {}).get(t) != sv.sha(os.path.join(ROOT, t)):
            errs.append(f'manifest tool {t}')
    for c, runs in docs.items():
        for r, d in enumerate(runs, 1):
            rel = os.path.relpath(path(c, r), ROOT)
            if manifest.get('captures', {}).get(rel) != sv.sha(path(c, r)):
                errs.append(f'{c}-{r} manifest 해시')
            if d.get('binary_sha256') != BINARY_SHA:
                errs.append(f'{c}-{r} binary_sha256')
            if d.get('agent_sha256') != manifest.get('tools', {}).get('tools/push_init_agent.js'):
                errs.append(f'{c}-{r} agent 해시')
            if d.get('errors') or d.get('reason') != 'process-terminated' or not isinstance((d.get('result') or {}).get('status'), int):
                errs.append(f'{c}-{r} 실행 실패')
            if not d.get('events'):
                errs.append(f'{c}-{r} 사건 없음')
            if d.get('schema') != 'M3-04c-1b-v1':
                errs.append(f'{c}-{r} schema')
        base = strip_doc(runs[0])
        for r, d in enumerate(runs[1:], 2):
            s = strip_doc(d)
            if s != base:
                k = next(k for k in sorted(set(base) | set(s)) if base.get(k) != s.get(k))
                detail = ''
                if k == 'events':
                    i = next((i for i, (a, b) in enumerate(zip(base['events'], s['events'])) if a != b), min(len(base['events']), len(s['events'])))
                    detail = f' 사건 {i}'
                errs.append(f'{c} run{r}≠run1: {k}{detail}')
    return errs


def noninterference(docs):
    """자연 6사례: 21h 캡처의 push:enter·첫 supply:enter와 JSON 전체 일치(step·supplier.id 제외 없음, 21h의 step만 제거)."""
    errs = []
    for c in NATURAL:
        old = sv.load(sv.case_paths(c)[0])['events']
        mine = docs[c][0]['events']
        for kind in ('push:enter', 'supply:enter'):
            a = next(e for e in old if e['event'] == kind)
            a = {k: v for k, v in a.items() if k != 'step'}
            b = next(e for e in mine if e['event'] == kind)
            if a != b:
                k = next(k for k in sorted(set(a) | set(b)) if a.get(k) != b.get(k))
                errs.append(f'{c} 21h 비간섭 {kind}.{k}')
    return errs


# ---------------------------------------------------------------- 재계산 모델
def decode_enter(pe):
    md, st = pe['model'], pe['state']
    return dict(n=st['n'], m=st['m'], first=pe['first'], attempt=pe['attempt'], model_status=pe['model_status'],
                starts=[sv.i64(h) for h in md['starts']], lengths=md['lengths'], rows=md['rows'],
                values=[f64(h) for h in md['values']], rhs=[f64(h) for h in md['rhs']],
                lb=[f64(h) for h in md['lb']], ub=[f64(h) for h in md['ub']], vstat=list(st['vstat']),
                x=[f64(h) for h in st['x']], slack=[f64(h) for h in st['slack']],
                inf=f64(st['tolerances']['370']), tau=f64(st['tolerances']['380']),
                header=list(st['header']), iterations=f64(st['counters']['iterations']),
                numeric=st['counters']['numeric'], limit=st['factor']['limit'])


def keyed(inp, pairs):
    n = inp['n']
    idx = [j for j, _ in pairs]
    key = [-(float(inp['lengths'][j]) + sv.minsd(d, sv.CAP)) if j < n else -1.0 - sv.minsd(d, sv.CAP) for j, d in pairs]
    sv.sort_desc(key, idx)
    return idx


def sub_col(inp, r, j, v):
    if v != 0.0 or v != v:
        for k in range(inp['starts'][j], inp['starts'][j] + inp['lengths'][j]):
            r[inp['rows'][k]] = r[inp['rows'][k]] - v * inp['values'][k]


def first_scan(inp, tau_in, rules):
    """C 401–596: rhs·후보·첫 스캔 셈(0.001·0.01·0.1)."""
    n, m = inp['n'], inp['m']
    lb, ub, x, sl, vs = inp['lb'], inp['ub'], inp['x'], inp['slack'], inp['vstat']
    r = list(inp['rhs'])
    cols, slacks = [], []
    cnt = [0, 0, 0]
    gt = (lambda a, b: a < b) if rules['count_strict'] else (lambda a, b: a <= b)

    def count(d):
        for t, c in enumerate((0.001, 0.01, 0.1)):
            if gt(tau_in * c, d):
                cnt[t] += 1
    xmm0 = 0.0
    for j in range(n):
        s = vs[j]
        if s >= 0:
            continue
        if s == -1:
            v = lb[j]; count(x[j] - v)
        elif s == -2:
            v = ub[j]; count(v - x[j])
        elif s == -3:
            v = x[j]
            d = (x[j] - lb[j]) if (ub[j] + lb[j] > x[j] + x[j]) else (ub[j] - x[j])
            cols.append((j, d))
        elif s == -4:
            v = lb[j]
        elif s == -5:
            v = {'ub': ub[j], 'lb': lb[j], 'zero': 0.0}[rules['m5']]
        else:
            v = 0.0
        xmm0 = v
        sub_col(inp, r, j, v)
    for i in range(m):
        s = vs[n + i]
        if s >= 0 or s < -3:
            continue
        if s == -1:
            v = 0.0; count(sl[i])
        elif s == -3:
            v = sl[i]; slacks.append((n + i, sl[i]))
        else:
            v = xmm0 if rules['slack_m2'] == 'stale' else 0.0
        xmm0 = v
        r[i] = r[i] - v
    return dict(rhs=r, cand=keyed(inp, cols + slacks), counts=cnt)


def rebuild(inp, tau2, rules):
    """C 945–972·1063–1141: rhs 원본 복사 뒤 -1/-2/-3만 다시 빼고 승격. vstat를 제자리 갱신한다."""
    n, m = inp['n'], inp['m']
    lb, ub, x, sl, vs = inp['lb'], inp['ub'], inp['x'], inp['slack'], inp['vstat']
    r = list(inp['rhs'])
    pairs = []
    for j in range(n):
        s = vs[j]
        if not s < 0:
            continue
        if s < -3:
            if not rules['rebuild_skip_m45']:
                sub_col(inp, r, j, lb[j] if s == -4 else ub[j] if s == -5 else 0.0)
            continue
        xv = x[j]
        promote = False
        if s == -1:
            v = lb[j]
            ok = (tau2 + v <= xv) if rules['promote_lb'] == 'le' else (tau2 + v < xv)
            promote = ok
        elif s == -2:
            v = ub[j]
            promote = not (v - tau2 < xv)
        else:
            promote = True
        if promote:
            u, l = ub[j], lb[j]
            vs[j] = -3
            pairs.append((j, (u - xv) if (u + l <= xv + xv) else (xv - l)))
            v = xv
        sub_col(inp, r, j, v)
    for i in range(m):
        s = vs[n + i]
        if not (s < 0 and -4 < s):
            continue
        if s != -1 or tau2 <= sl[i]:
            vs[n + i] = -3
            pairs.append((n + i, sl[i]))
            v = sl[i]
        else:
            v = 0.0
        r[i] = r[i] - v
    return dict(rhs=r, cand=keyed(inp, pairs))


def amax(vals):
    acc = 0.0
    for v in vals:
        a = abs(v)
        if acc <= a:
            acc = a
    return acc


def residual089(inp, header, x, pre, rules):
    """089d8f: header 위치 순서로 out[row] = a·x + out[row](mulsd·addsd), slack은 out[i] += x."""
    n = inp['n']
    acc = list(pre)
    order = range(len(header)) if rules['residual_order'] == 'header' else reversed(range(len(header)))
    for p in order:
        h, xv = header[p], x[p]
        if h < n:
            for k in range(inp['starts'][h], inp['starts'][h] + inp['lengths'][h]):
                acc[inp['rows'][k]] = inp['values'][k] * xv + acc[inp['rows'][k]]
        else:
            acc[h - n] = acc[h - n] + xv
    return acc


def limit061(m, s3c8, env, rules):
    if env is None or env >= 1:
        return None
    d3 = math.pow(m, 0.5) if rules['limit_rule'] == '061bfe' else float(m)
    d4 = s3c8 * 0.9
    d5 = float(int(d3))
    if d4 <= float(int(d3)):
        d5 = d4
    i2 = 50
    if 50 < int(d5):
        i2 = int(d5)
    return i2 if i2 < s3c8 else None


def calls_of(doc):
    out = {}
    for e in doc['events']:
        if 'call' in e:
            out.setdefault(e['call'], []).append(e)
    return out


def apply_edit(inp, e):
    v = e['after']
    if e['field'] == 'vstat':
        inp['vstat'][e['index']] = v
    elif e['field'] == 'x':
        inp['x'][e['index']] = f64(v)
    elif e['field'] == 'slack':
        inp['slack'][e['index']] = f64(v)


def check_call(case, c, evs, rules, spec, ev_out):
    """push 호출 하나의 초기화 사건열을 재계산과 비교. 실패 문자열 목록과 관측 분기 집합."""
    fail = []
    seen = set()
    F = lambda msg: fail.append(f'{case} call{c}: {msg}')
    it = iter(evs)
    seq = list(evs)
    kinds = [e['event'] for e in seq]
    pe = seq[0]
    if pe['event'] != 'push:enter':
        return [f'{case} call{c}: 첫 사건이 push:enter 아님'], seen
    inp = decode_enter(pe)
    ie = next((e for e in seq if e['event'] == 'init:enter'), None)
    if ie is None:
        return [f'{case} call{c}: init:enter 없음'], seen
    ex0 = ie['extra']
    if any(v != ZERO for v in ie['residual_scratch']):
        F('진입 S+0x1f8 scratch가 0이 아님')
    # 편집: 명세와 정확히 같아야 한다(첫 호출만).
    edits = [e for e in seq if e['event'] == 'edit']
    want = spec['edits'] if (spec and c == 1) else []
    if len(edits) != len(want):
        F(f'편집 수 {len(edits)} ≠ 명세 {len(want)}')
    for e, w in zip(edits, want):
        if (e['at'], e['field'], e['index']) != (w['at'], w['field'], w.get('index')) or e['after'] != w.get('hex', w.get('int')):
            F(f'편집 불일치 {e} vs {w}')
    for e in edits:
        if e['at'] == 'enter':
            apply_edit(inp, e)
    tau_ev = next((e for e in seq if e['event'] == 'init:tau'), None)
    if tau_ev is None:
        return fail + [f'{case} call{c}: init:tau 없음'], seen
    tau_in = f64(tau_ev['tau_in'])
    if inp['first'] == 0:
        seen.add('first0')
    # 초기 재분해
    lus = [e for e in seq if e['event'] == 'init:lu']
    if inp['first']:
        if not lus or lus[0]['request_in'] != 10:
            F('first≠0인데 요청 10 초기 재분해 없음')
        if any(e['ret'] != 0 for e in lus):
            seen.add('lu_ret')
        header = lus[-1]['header'] if lus else inp['header']
    else:
        if lus:
            F('first=0인데 초기 재분해')
        header = inp['header']
    scan = first_scan(inp, tau_in, rules)
    rhs_exp, cand = scan['rhs'], scan['cand']
    s390 = f64(ex0['s390'])
    s3c8 = ex0['s3c8']
    numeric = ex0['numeric']
    s120 = f64(ex0['s120'])
    tau_cur = tau_in
    adjusted = 0
    rnd = 0
    pinf = viol = None
    basic = None
    I = inp['inf']
    pos = 0
    final_rhs = None
    while True:
        rnd += 1
        # FTRAN
        try:
            i_fe = kinds.index('init:ftran:enter', pos)
        except ValueError:
            F(f'round{rnd} FTRAN 없음'); break
        fe = seq[i_fe]
        if fe['n'] != rnd:
            F(f'FTRAN 번호 {fe["n"]} ≠ {rnd}')
        if not all(fe['args'].values()) or fe['args']['flag'] != 0 or fe['count'] != -1:
            if not (fe['args']['factor'] and fe['args']['input'] and fe['args']['output'] and fe['args']['flag'] == 0 and fe['count'] == -1):
                F(f'round{rnd} FTRAN 인자 {fe["args"]} count {fe["count"]}')
        for e in edits:
            if e['at'] == 'ftran' and rnd == 1:
                if e['field'] == 's390':
                    s390 = f64(e['after'])
                if e['field'] == 's3c8':
                    s3c8 = e['after']
        if f64(fe['extra']['s390']) != s390 and hx(f64(fe['extra']['s390'])) != hx(s390):
            F(f'round{rnd} FTRAN 직전 S+0x390 {fe["extra"]["s390"]} ≠ {hx(s390)}')
        got = fe['values']
        exp = [hx(v) for v in rhs_exp]
        if got != exp:
            i = next((i for i, (a, b) in enumerate(zip(exp, got)) if a != b), len(exp))
            F(f'round{rnd} FTRAN 입력 행{i} 기대 {exp[i] if i < len(exp) else None} 실제 {got[i] if i < len(got) else None}')
        final_rhs = got
        fl = seq[i_fe + 1]
        if fl['event'] != 'init:ftran:leave' or fl['n'] != rnd or fl['count'] != -1:
            F(f'round{rnd} FTRAN leave 누락'); break
        if len(got) != inp['m'] or len(fl['values']) != inp['m']:
            F(f'round{rnd} FTRAN 입력/출력 길이 {len(got)}/{len(fl["values"])} ≠ m {inp["m"]}'); break
        xb = [f64(h) for h in fl['values']]
        # 잔차
        re_ = seq[i_fe + 2] if i_fe + 2 < len(seq) else None
        rl = seq[i_fe + 3] if i_fe + 3 < len(seq) else None
        if not re_ or re_['event'] != 'init:residual:enter' or not rl or rl['event'] != 'init:residual:leave':
            F(f'round{rnd} 잔차 사건 누락'); break
        if not all(re_['args'].values()):
            F(f'round{rnd} 잔차 인자 {re_["args"]}')
        if re_['x'] != fl['values']:
            F(f'round{rnd} 잔차 입력 ≠ FTRAN 출력')
        if any(v != ZERO for v in re_['out']):
            F(f'round{rnd} 잔차 scratch 진입이 +0이 아님')
        bx = residual089(inp, header, xb, [f64(h) for h in re_['out']], rules)
        if [hx(v) for v in bx] != rl['out']:
            i = next(i for i, (a, b) in enumerate(zip([hx(v) for v in bx], rl['out'])) if a != b)
            F(f'round{rnd} 잔차 출력 행{i} 기대 {hx(bx[i])} 실제 {rl["out"][i]}')
        bx = [f64(h) for h in rl['out']]
        rhs_v = [f64(h) for h in got]
        mr, mx = amax(rhs_v), amax(xb)
        res = amax([a - b for a, b in zip(rhs_v, bx)])
        cand_mk = 1e-4
        if mr * 1e10 < mx or rules['mk_hi'] < res:
            cand_mk = 0.5
        elif mr * 1e9 < mx or rules['mk_lo'] < res:
            cand_mk = 0.25
        if cand_mk > 1e-4:
            seen.add(f'mk_cand_{cand_mk}')
        strengthen = s390 <= cand_mk and cand_mk != s390
        pos = i_fe + 4
        nxt = seq[pos] if pos < len(seq) else None
        if strengthen:
            seen.add('strengthen')
            if not nxt or nxt['event'] != 'init:strengthen:enter':
                F(f'round{rnd} 강화 기대인데 사건 없음')
            else:
                if nxt['markowitz_new'] != hx(cand_mk):
                    F(f'round{rnd} 강화 새 값 {nxt["markowitz_new"]} ≠ {hx(cand_mk)}')
                if nxt['residual'] != hx(res):
                    F(f'round{rnd} 강화 잔차 {nxt["residual"]} ≠ {hx(res)}')
                lv = seq[pos + 1]
                if lv['event'] != 'init:strengthen:leave':
                    F('강화 leave 없음')
                else:
                    lim = limit061(inp['m'], s3c8, nxt['extra']['env_2c54'], rules)
                    want_s, want_f = (lim, lim) if lim is not None else (s3c8, nxt['extra']['f188'])
                    if (lv['extra']['s3c8'], lv['extra']['f188']) != (want_s, want_f):
                        F(f'round{rnd} 061bfe 한도 {lv["extra"]["s3c8"]},{lv["extra"]["f188"]} ≠ {want_s},{want_f}')
                    if lim is not None:
                        seen.add('limit')
                    s3c8 = lv['extra']['s3c8']
                pos += 2
            s390 = cand_mk
        elif nxt and nxt['event'] == 'init:strengthen:enter':
            F(f'round{rnd} 강화 사건이 있으나 판단은 강화 없음(후보 {cand_mk}, S+0x390 {s390})')
        if rules['numeric'] < res:
            numeric += 1
            seen.add('numeric_residual')
        # basic·분류
        basic = []
        for p, h in enumerate(header):
            v = xb[p]
            if rules['shift'] and inp['lb'][h] > -I:
                v = v - inp['lb'][h]
            basic.append(v)
        width = sv.widths(inp, header, basic)
        marks, viol, pinf = sv.classify(inp, header, basic, width)
        nxt = seq[pos] if pos < len(seq) else None
        if inp['first']:
            if not nxt or nxt['event'] != 'init:condition:enter':
                F(f'round{rnd} 조건수 호출 사건 없음'); break
            if nxt['pinf'] != hx(pinf) or nxt['violations'] != viol:
                F(f'round{rnd} gate PInf/위반 {nxt["pinf"]},{nxt["violations"]} ≠ {hx(pinf)},{viol}')
            if not nxt['cond_is_param']:
                F('조건수 출력 포인터가 param_8 아님')
            if nxt['extra']['numeric'] != numeric:
                F(f'round{rnd} gate 직전 S+0x178 {nxt["extra"]["numeric"]} ≠ {numeric}')
            if hx(f64(nxt['extra']['s390'])) != hx(s390):
                F(f'round{rnd} gate 직전 S+0x390 {nxt["extra"]["s390"]} ≠ {hx(s390)}')
            cl = seq[pos + 1]
            cond = f64(cl['condition'])
            pos += 2
        else:
            cond = f64(ie['condition_in'])
        P = pinf if (inp['model_status'] == 2 or rules['gate_p'] != 'status') else 1.0
        retry = None
        if inp['attempt'] == 0 and (1e14 < cond or 1e10 < P or 1e16 < P * cond):
            retry = 'retry_p'
        elif 1e30 < cond:
            retry = 'retry_cond'
        elif 1e15 < pinf:
            retry = 'retry_pinf'; numeric += 1
        nxt = seq[pos] if pos < len(seq) else None
        if retry:
            seen.add(retry)
            if not nxt or nxt['event'] != 'push:leave' or nxt['retry'] != 1 or not nxt['init']:
                F(f'round{rnd} {retry} 기대인데 다음 사건 {nxt and nxt["event"]} retry {nxt and nxt.get("retry")}')
            elif nxt['extra']['numeric'] != numeric:
                F(f'retry 반환 S+0x178 {nxt["extra"]["numeric"]} ≠ {numeric}')
            return fail, seen
        if not nxt or nxt['event'] != 'init:adjust-gate':
            F(f'round{rnd} 조정 판정 사건 없음 ({nxt and nxt["event"]})'); break
        g = nxt
        if g['adjusted'] != adjusted or g['candidates'] != len(cand) or g['tau_in'] != hx(tau_cur):
            F(f'round{rnd} 조정 gate 상태 flag {g["adjusted"]} 후보 {g["candidates"]} τ {g["tau_in"]} ≠ {adjusted},{len(cand)},{hx(tau_cur)}')
        if [g['count0001'], g['count001'], g['count01']] != scan['counts']:
            F(f'round{rnd} 첫 스캔 셈 {[g["count0001"], g["count001"], g["count01"]]} ≠ {scan["counts"]}')
        if g['h'] != hx(inp['tau']) or g['extra']['s120'] != ex0['s120']:
            F(f'round{rnd} 조정 gate τ/S+0x120')
        if inp['first'] and g['pinf_slot'] != hx(pinf):
            F(f'round{rnd} 조정 gate PInf 슬롯')
        pos += 1
        h = inp['tau']
        do_adj = (not adjusted) and 1.0 < pinf and s120 < h and scan['counts'][2] < len(cand)
        if do_adj:
            seen.add('adjust')
            adjusted = 1
            n_c = len(cand)
            if not rules['tau_mult']:
                tau_cur = tau_cur * 0.1
            elif scan['counts'][0] < n_c:
                tau_cur = tau_cur * 0.001
            elif scan['counts'][1] < n_c:
                tau_cur = tau_cur * 0.01
            else:
                tau_cur = tau_cur * 0.1
            before = list(inp['vstat'])
            rb = rebuild(inp, tau_cur, rules)
            if inp['vstat'] != before:
                seen.add('promote')
            rhs_exp, cand = rb['rhs'], rb['cand']
            continue
        break
    if fail:
        return fail, seen
    # 첫 supply:enter
    se = next((e for e in seq[pos:] if e['event'] == 'supply:enter'), None)
    if se is None:
        F('첫 supply:enter 없음'); return fail, seen
    st = se['state']
    ctx = se['supplier']['context']
    if se['candidates'] != cand or ctx['total'] != len(cand):
        F(f'후보 순서 {se["candidates"][:6]} ≠ {cand[:6]}')
    sup = 1 if math.sqrt(float(inp['m'])) <= float(len(cand)) else 0
    if se['support'] != sup:
        F(f'지원 flag {se["support"]} ≠ {sup}')
    nb_exp = [v for v in inp['vstat'] if v < 0]
    nb_got = [v for v in st['vstat'] if v < 0]
    if [(k, v) for k, v in enumerate(inp['vstat']) if v < 0] != [(k, v) for k, v in enumerate(st['vstat']) if v < 0]:
        F('비기저 vstat(승격 포함) 불일치')
    if st['header'] != header:
        F('header 불일치')
    if st['rhs'] != final_rhs:
        F('S+0x200 ≠ 마지막 FTRAN 입력')
    if st['basic'] != [hx(v) for v in basic]:
        F('basic ≠ FTRAN 출력 − lb')
    if st['width'] != [hx(v) for v in sv.widths(inp, header, basic)]:
        F('폭')
    if st['marks'] != marks:
        F('표시')
    if st['pinf'] != hx(pinf):
        F(f'S+0x120 {st["pinf"]} ≠ {hx(pinf)}')
    if f64(st['counters']['iterations']) != inp['iterations'] + 1.0:
        F('반복')
    if st['counters']['numeric'] != numeric:
        F(f'S+0x178 {st["counters"]["numeric"]} ≠ {numeric}')
    if hx(f64(st['tolerances']['390'])) != hx(s390):
        F('S+0x390')
    ev_out.append(dict(case=case, call=c, rounds=rnd, cand=len(cand), pinf=hx(pinf), viol=viol, counts=scan['counts'],
                       tau=hx(tau_cur), mk=hx(s390), numeric=numeric))
    del nb_exp, nb_got, it
    return fail, seen


REQUIRE = {'adjust': {'adjust', 'promote'}, 'strengthen': {'strengthen'}, 'limit': {'limit'},
           'retry_p': {'retry_p'}, 'retry_pinf': {'retry_pinf'}}


def discriminate(inp_doc, kind, rules):
    """slack -2·-5 사례가 규칙을 실제로 가르는지(대안 규칙의 FTRAN 입력이 달라지는지)."""
    evs = calls_of(inp_doc)[1]
    inp = decode_enter(evs[0])
    for e in evs:
        if e['event'] == 'edit' and e['at'] == 'enter':
            apply_edit(inp, e)
    tau = f64(next(e for e in evs if e['event'] == 'init:tau')['tau_in'])
    got = next(e for e in evs if e['event'] == 'init:ftran:enter')['values']
    alt = {'slack_m2_stale': [dict(slack_m2='zero')], 'col_m5': [dict(m5='zero')],
           'col_m5_ub': [dict(m5='zero'), dict(m5='lb')]}[kind]
    errs = []
    if [hx(v) for v in first_scan(inp, tau, rules)['rhs']] != got:
        errs.append(f'{kind}: 채택 규칙 FTRAN 입력 불일치')
    for a in alt:
        r2 = dict(rules, **a)
        if [hx(v) for v in first_scan(inp, tau, r2)['rhs']] == got:
            errs.append(f'{kind}: 대안 규칙 {a}도 같은 입력(판별 불가)')
    return errs


def acceptance(docs, rules=RULES, summary=None):
    errs = []
    syn = synthetic()
    observed = {}
    for c in cases():
        doc = docs[c][0]
        seen_all = set()
        for call, evs in sorted(calls_of(doc).items()):
            f, seen = check_call(c, call, evs, rules, syn.get(c), summary if summary is not None else [])
            errs += f
            seen_all |= seen
        observed[c] = sorted(seen_all)
        if c in syn:
            for req in syn[c]['require']:
                if req in REQUIRE:
                    if not REQUIRE[req] <= seen_all:
                        errs.append(f'{c}: 필수 분기 {req} 미관측 ({sorted(seen_all)})')
                else:
                    errs += [f'{c}: {e}' for e in discriminate(doc, req, rules)]
        if c in NATURAL and any(s in seen_all for s in ('adjust', 'strengthen', 'retry_p', 'retry_pinf', 'retry_cond')):
            observed[c].append('!자연 분기')
        if c in DUAL and 'first0' not in seen_all:
            errs.append(f'{c}: first=0 미관측')
    return errs, observed


# ---------------------------------------------------------------- self-test
def _ev(doc, kind, k=0, call=1):
    return [e for e in doc['events'] if e['event'] == kind and e.get('call') == call][k]


def _flip(h):
    b = bytearray(bytes.fromhex(h)); b[0] ^= 1; return b.hex()


def _all(docs, case, fn):
    for d in docs[case]:
        fn(d)


TAMPERS = [
    ('바이너리 해시 0', 'controls', lambda D: D['arms'][0].__setitem__('binary_sha256', '0' * 64)),
    ('사건 비우기', 'controls', lambda D: D['interior'][1].__setitem__('events', [])),
    ('run2 FTRAN 출력 1비트', 'controls', lambda D: _ev(D['arms'][1], 'init:ftran:leave')['values'].__setitem__(0, _flip(_ev(D['arms'][1], 'init:ftran:leave')['values'][0]))),
    ('21h 비간섭: push:enter x 1비트(3회 모두)', 'nonint', lambda D: _all(D, 'retry', lambda d: _ev(d, 'push:enter')['state']['x'].__setitem__(2, _flip(_ev(d, 'push:enter')['state']['x'][2])))),
    ('FTRAN 입력 1비트', 'accept', lambda D: _ev(D['capri'][0], 'init:ftran:enter')['values'].__setitem__(3, _flip(_ev(D['capri'][0], 'init:ftran:enter')['values'][3]))),
    ('FTRAN 출력 1비트', 'accept', lambda D: _ev(D['adlittle'][0], 'init:ftran:leave')['values'].__setitem__(1, _flip(_ev(D['adlittle'][0], 'init:ftran:leave')['values'][1]))),
    ('잔차 출력 1비트', 'accept', lambda D: _ev(D['scsd1'][0], 'init:residual:leave')['out'].__setitem__(0, _flip(_ev(D['scsd1'][0], 'init:residual:leave')['out'][0]))),
    ('잔차 scratch 비영', 'accept', lambda D: _ev(D['interior'][0], 'init:residual:enter')['out'].__setitem__(0, hx(1e-300))),
    ('τ_in 1비트', 'accept', lambda D: _ev(D['adlittle'][0], 'init:tau').__setitem__('tau_in', _flip(_ev(D['adlittle'][0], 'init:tau')['tau_in']))),
    ('0.1셈 +1', 'accept', lambda D: _ev(D['adlittle'][0], 'init:adjust-gate').__setitem__('count01', _ev(D['adlittle'][0], 'init:adjust-gate')['count01'] + 1)),
    ('강화 사건 제거', 'accept', lambda D: D['mk_low'][0].__setitem__('events', [e for e in D['mk_low'][0]['events'] if not e['event'].startswith('init:strengthen')])),
    ('강화 새 값 1비트', 'accept', lambda D: _ev(D['quality'][0], 'init:strengthen:enter').__setitem__('markowitz_new', _flip(_ev(D['quality'][0], 'init:strengthen:enter')['markowitz_new']))),
    ('061bfe 한도 변경', 'accept', lambda D: _ev(D['mk_limit'][0], 'init:strengthen:leave')['extra'].__setitem__('s3c8', 51)),
    ('gate PInf 1비트', 'accept', lambda D: _ev(D['retry'][0], 'init:condition:enter').__setitem__('pinf', _flip(_ev(D['retry'][0], 'init:condition:enter')['pinf']))),
    ('조정 2회째 FTRAN 제거', 'accept', lambda D: D['adjust'][0].__setitem__('events', [e for e in D['adjust'][0]['events'] if not (e['event'].startswith('init:ftran') and e.get('n') == 2)])),
    ('조정 flag 반전', 'accept', lambda D: _ev(D['adjust'][0], 'init:adjust-gate', 1).__setitem__('adjusted', 0)),
    ('조정 τ\' 1비트', 'accept', lambda D: _ev(D['col_m5_ub'][0], 'init:adjust-gate', 1).__setitem__('tau_in', _flip(_ev(D['col_m5_ub'][0], 'init:adjust-gate', 1)['tau_in']))),
    ('품질 retry 0', 'accept', lambda D: _ev(D['quality'][0], 'push:leave').__setitem__('retry', 0)),
    ('supply 후보 교환', 'accept', lambda D: (lambda c: c.__setitem__(slice(0, 2), c[1::-1]))(_ev(D['arms'][0], 'supply:enter')['candidates'])),
    ('supply 승격 되돌림', 'accept', lambda D: _ev(D['adjust'][0], 'supply:enter')['state']['vstat'].__setitem__(1, -1)),
    ('supply basic 1비트', 'accept', lambda D: _ev(D['interior'][0], 'supply:enter')['state']['basic'].__setitem__(0, _flip(_ev(D['interior'][0], 'supply:enter')['state']['basic'][0]))),
    ('편집 사건 제거', 'accept', lambda D: D['slack_m2'][0].__setitem__('events', [e for e in D['slack_m2'][0]['events'] if e['event'] != 'edit'])),
    ('편집 값 변경', 'accept', lambda D: [e for e in D['col_m5_ub'][0]['events'] if e['event'] == 'edit'][0].__setitem__('after', -4)),
    ('null-null: FTRAN 입력·출력 비우기', 'accept', lambda D: [_ev(D['interior'][0], k).__setitem__('values', []) for k in ('init:ftran:enter', 'init:ftran:leave')]),
    ('first=0 사례 first=1로', 'accept', lambda D: _ev(D['adlittle_x1'][0], 'push:enter').__setitem__('first', 1)),
]
MUTATIONS = [
    ('slack -2 = 0', dict(slack_m2='zero'), True),
    ('열 -5 = 0', dict(m5='zero'), True),
    ('열 -5 = lb', dict(m5='lb'), True),
    ('재구성에서 -4/-5도 뺌', dict(rebuild_skip_m45=False), True),
    ('τ\' 항상 0.1배', dict(tau_mult=False), True),
    ('-1 승격 strict', dict(promote_lb='lt'), False),
    ('Markowitz 0.5 임계 1e-3', dict(mk_hi=1e-3), True),
    ('Markowitz 0.25 임계 1e-3', dict(mk_lo=1e-3), False),
    ('수치 카운터 임계 1e-3', dict(numeric=1e-3), True),
    ('061bfe √m 대신 m', dict(limit_rule='m'), False),
    ('gate P 항상 PInf', dict(gate_p='always'), False),
    ('셈 ≥', dict(count_strict=False), False),
    ('basic lower shift 생략', dict(shift=False), False),
    ('잔차 역순', dict(residual_order='reverse'), False),
]


def self_test(docs, manifest):
    bad = 0
    base_c = controls(docs, manifest) + noninterference(docs)
    base_a, _ = acceptance(docs)
    if base_c or base_a:
        print('기준 자료가 먼저 실패:', (base_c + base_a)[:5])
        return 1
    for name, layer, fn in TAMPERS:
        D = copy.deepcopy(docs)
        try:
            fn(D)
            if layer == 'controls':
                errs = controls(D, manifest)
            elif layer == 'nonint':
                errs = noninterference(D)
            else:
                errs, _ = acceptance(D)
        except Exception as e:  # 구조 파괴로 검사기가 예외를 내도 거부로 친다
            errs = [f'예외 {type(e).__name__}: {e}']
        print(('REJECTED     ' if errs else 'NOT REJECTED ') + name + (f'  ← {errs[0]}' if errs else ''))
        bad += not errs
    for name, mut, required in MUTATIONS:
        try:
            errs, _ = acceptance(docs, dict(RULES, **mut))
        except Exception as e:
            errs = [f'예외 {type(e).__name__}: {e}']
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
    ap.add_argument('--summary')
    a = ap.parse_args()
    if a.write_manifest:
        if os.path.exists(MANIFEST):
            sys.exit('기존 manifest 덮어쓰기 거부')
        man = dict(task='M3-04c-1b', binary_sha256=BINARY_SHA, schema='M3-04c-1b-v1',
                   tools={t: sv.sha(os.path.join(ROOT, t)) for t in TOOLS},
                   captures={os.path.relpath(path(c, r), ROOT): sv.sha(path(c, r)) for c in cases() for r in (1, 2, 3)},
                   synthetic_sha256=sv.sha(os.path.join(DATA, 'synthetic.json')),
                   baseline='bench/oracle/m3_push_block (21h manifest)')
        open(MANIFEST, 'w').write(json.dumps(man, indent=1, ensure_ascii=False) + '\n')
        print('manifest 작성', len(man['captures']))
        return 0
    docs = load_all()
    manifest = json.load(open(MANIFEST))
    if manifest.get('synthetic_sha256') != sv.sha(os.path.join(DATA, 'synthetic.json')):
        print('synthetic.json 해시 불일치'); return 1
    if a.self_test:
        return self_test(docs, manifest)
    errs = []
    if a.controls:
        e = controls(docs, manifest) + noninterference(docs)
        print('controls', '통과' if not e else f'실패 {len(e)}', f'(사례 {len(docs)} × 3회, 21h 비간섭 {len(NATURAL)}사례)')
        errs += e
    if a.acceptance:
        summ = []
        e, observed = acceptance(docs, summary=summ)
        print('acceptance', '통과' if not e else f'실패 {len(e)}', f'(push 호출 {len(summ)}개 초기화 완료 + retry 반환)')
        for c in cases():
            print(f'  {c:13s} {" ".join(observed[c]) or "-"}')
        errs += e
        if a.summary:
            open(a.summary, 'w').write(json.dumps(dict(observed=observed, calls=summ), indent=1, ensure_ascii=False, sort_keys=True) + '\n')
    for x in errs[:30]:
        print('  ', x)
    return 1 if errs else 0


if __name__ == '__main__':
    sys.exit(main())
