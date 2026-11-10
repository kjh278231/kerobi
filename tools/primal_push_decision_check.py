"""M3-04b-2: primal push 한 단계의 방향·거리 flip·거부·승인·이탈 상태 판단을 원본 캡처와 대조한다.

판단 모델은 134a87 asm(0x1c2164-0x1c2d7f)을 따른다. M3-04c-2부터 위반 basis(local_1b0>0)의 방향식
(0x1c227c-0x1c23d1)과 054eef 단계도 다룬다. M3-04c-7부터 056973 재시도 게이트(054eef 위치<0 또는 |pivot|<0.001)와
재시도 결과로 가는 단계도 다루며, theta=I 반대 방향 재호출은 범위 밖으로 거부한다.
04b-1 저장 캡처(frame 없음), 04b-2 새 캡처(frame 있음), M3-04b-3 arm LP 캡처(시작 벡터·보류 사건 step:defer),
M3-04c-2 위반 basis 캡처(재시도 사건 retry:*)를 읽는다.
"""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import struct
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
import primal_push_step_check as step  # noqa: E402  M3-04b-1 자연 캡처 검사

BASE = ROOT / 'bench/oracle/m3_push_decision'
OLD = ROOT / 'bench/oracle/m3_primal_push_step'
ARMS = ROOT / 'bench/oracle/m3_push_arms'
VIOL = ROOT / 'bench/oracle/m3_push_violated'
BINARY_SHA = step.BINARY_SHA
TOOLS = ['tools/primal_push_decision_capture.js', 'tools/primal_push_decision_agent.js']
VTOOLS = ['tools/primal_push_violated_capture.js', 'tools/primal_push_violated_agent.js']
ATOOLS = ['tools/primal_push_arm_capture.js', 'tools/primal_push_arm_agent.js']
# M3-04c-2 계약 §4: 대상 두 순서 합집합에서 반드시 나와야 하는 위반 경로 arm
REQUIRED_VIOLATED = {
    'clause': {'pre-reject1', 'pre-reject0', 'bound-sigma', 'bound-sigma0', 'small-bound', 'small-bound0', 'small-r',
               'small-r0', 'sigma', 'sigma0'},
    'cause': {'distance', 'reject', 'accept'}, 'arm': {'neg-B', 'nonneg-A'}}
FRAME = ['r', 'dL', 'dU', 'k100', 'km100', 'k2', 'e']


def f64(bits):
    return struct.unpack('<d', bytes.fromhex(bits))[0]


def bits(value):
    return struct.pack('<d', value).hex()


DEFAULT = dict(ten=10.0, e_scale=0.01, dir_strict=True, neg_rdl=True, r_pos_strict=True, r_reverse=False,
               flip_eps=1e-10, flip_le=True, k_reject=100.0, reject_arm='formula', reject_swap=False,
               acc_pivot=1e-6, acc_prod=-1e-5, k_leave=2.0, fixed_le=True, l_nonneg=-1, neg_default=-1,
               x_state=-4, u_nonneg_lt=True,
               # M3-04b-3 arm별 변조용. 기본값은 원본 식 그대로다.
               reject_on=(True, True), defer_on=True, flip_dir1=True, first_on=True, fixed_state=-4,
               neg_u=-2, neg_l='formula', nonneg_u='formula',
               # M3-04c-2 위반 basis 방향식 변조용. 기본값은 원본 식 그대로다.
               v_pre=(True, True), v_sigma_sign=1.0, v_clause=(True, True), v_small=True, v_small_clause=(True, True),
               v_r_strict=True, v_sigma_strict=True, v_ten=10.0,
               # M3-04c-7 재시도 게이트 변조용(asm 0x1c258e-0x1c25ae)
               gate_pivot=0.001, gate_neg=True, retry_use=True)


class Mismatch(AssertionError):
    pass


def need(cond, where, field, expected, actual):
    if not cond:
        raise Mismatch(f'{where} {field}: 기대 {expected} 실제 {actual}')


def direction(p, r, dL, dU, e):
    first = p['first_on'] and (p['ten'] * dL < dU if p['dir_strict'] else p['ten'] * dL <= dU) and ((-(r * dL) if p['neg_rdl'] else r * dL) < e)
    pos = 0 < r if p['r_pos_strict'] else 0 <= r
    second = (dL <= p['ten'] * dU or e <= dU * r) and pos
    # 방향 1을 만든 절: bound(첫 절)·cost(r>0 절)·both
    return (1 if first or second else 0), ('both' if first and second else 'bound' if first else 'cost' if second else 'none')


def direction_violated(p, r, dL, dU, s, e, k):
    """위반 basis 방향(asm 0x1c227c-0x1c23d1). 각 비교는 원본의 ja라 비순서 비교는 받지 않는다. s=ΣB alpha-ΣA alpha."""
    s = s * p['v_sigma_sign']
    ten = p['v_ten']
    if p['v_pre'][0] and r > k and k > dL:
        return 1, 'pre-reject1'
    if p['v_pre'][1] and -k > r and k > dU:
        return 0, 'pre-reject0'
    if p['v_clause'][0] and dU > ten * dL and e > -(dL * s) and e > -(r * dL):
        return 1, 'bound-sigma'
    if p['v_clause'][1] and dL > ten * dU and e > dU * s and e > dU * r:
        return 0, 'bound-sigma0'
    if p['v_small'] and e > abs(s):
        if p['v_small_clause'][0] and dU > ten * dL and e > -(r * dL):
            return 1, 'small-bound'
        if p['v_small_clause'][1] and dL > ten * dU and e > dU * r:
            return 0, 'small-bound0'
        return (1, 'small-r') if (r > 0 if p['v_r_strict'] else r >= 0) else (0, 'small-r0')
    return (1, 'sigma') if (s > 0 if p['v_sigma_strict'] else s >= 0) else (0, 'sigma0')


def leave_state(p, a, t, mark, b, w, k, width):
    if (width <= p['fixed']) if p['fixed_le'] else (width < p['fixed']):
        return p['fixed_state'], 'fixed'
    if 0 <= a:
        if mark in 'LN':
            return (p['l_nonneg'] if mark == 'L' else -1), 'nonneg-' + mark
        if mark == 'X':
            return p['x_state'], 'nonneg-X'
        if mark == 'A':
            return (-2 if abs((b - w) / a - t) < (t + 1.0) * k else -1), 'nonneg-A'
        if mark == 'U':
            hit = abs(a) * t < k if p['u_nonneg_lt'] else abs(a) * t <= k
            return (-2 if hit else -1) if p['nonneg_u'] == 'formula' else p['nonneg_u'], 'nonneg-U'
        return -1, 'nonneg-' + mark
    if mark in 'NU':
        return (p['neg_u'] if mark == 'U' else -2), 'neg-' + mark
    if mark == 'B':
        return (-2 if (t + 1.0) * k <= abs(b / a - t) else -1), 'neg-B'
    if mark == 'X':
        return p['x_state'], 'neg-X'
    if mark == 'L':
        return (-2 if k <= abs(a) * t else -1) if p['neg_l'] == 'formula' else p['neg_l'], 'neg-L'
    return p['neg_default'], 'neg-' + mark


def check_frame(se, p, where, r, dL, dU, h, e_tol, kreject):
    if 'frame' not in se:
        return
    fr = se['frame']
    want = dict(r=bits(r), dL=bits(dL), dU=bits(dU), k100=bits(100.0 * h), km100=bits(-100.0 * h), k2=bits(h + h), e=bits(h * 0.01))
    if p is not DEFAULT:
        want.update(r=bits(r), e=bits(e_tol), k100=bits(kreject), km100=bits(-kreject), k2=bits(p['k_leave'] * h))
    for name in FRAME:
        need(fr[name] == want[name], where, 'frame ' + name, want[name], fr[name])


def decide(record, p=DEFAULT, label=''):
    """record의 모든 단계를 판단 모델로 재구성해 대조한다. 단계별 분기 기록을 반환한다."""
    out = []
    events = record['events']
    i = 0
    model = None
    while i < len(events):
        e = events[i]
        if e['event'] == 'push:enter':
            model = e['model']
        if e['event'] != 'supply:leave' or e['candidate'] < 0:
            i += 1
            continue
        j = e['candidate']
        pre = e['alpha']
        where = f"{label} call{e['call']} step{e['step']}"
        k = i + 1
        ratios, retries = [], []
        while k < len(events) and events[k]['event'] not in ('supply:enter', 'push:leave', 'step:enter', 'step:defer'):
            if events[k]['event'] == 'ratio:enter':
                ratios.append((events[k], events[k + 1]))
            if events[k]['event'] == 'retry:enter':
                retries.append((events[k], events[k + 1] if k + 1 < len(events) else {}))
            k += 1
        need(len(ratios) == 1, where, 'ratio 호출 수(재호출 제외 범위)', 1, len(ratios))
        rin, rout = ratios[0]
        violated = rin['name'] == 'infeasible'
        need(rin['name'] in ('feasible', 'infeasible'), where, 'ratio 경로', 'feasible|infeasible', rin['name'])
        # feasible 경로는 위반 수 local_1b0==0일 때만 온다. A·B 표시는 그 수를 올리므로 있으면 안 된다(C 839·856·1688-1717·1940-1978·2199-2222·2342-2405).
        # 같은 이유로 위반 경로(054eef)에는 A·B 표시가 적어도 하나 있어야 한다(C 1225).
        marks_ab = [i for i, mk in enumerate(rin['marks']) if mk in ('41', '42')]
        need(bool(marks_ab) == violated, where, f"{rin['name']} ratio의 A·B 표시", '있음' if violated else '없음', marks_ab)
        st = rin['state']
        n, m = st['n'], st['m']
        tol = {kk: f64(v) for kk, v in st['tolerances'].items()}
        h, inf, fixed_tol = tol['380'], tol['370'], tol['3b0']
        cost = [f64(v) for v in model['cost']]
        lb = [f64(v) for v in model['lb']]
        ub = [f64(v) for v in model['ub']]
        if j < n:
            xj = f64(st['x'][j])
            dU, dL = ub[j] - xj, xj - lb[j]
        else:
            dU, dL = inf, f64(st['slack'][j - n])
        need(pre['count'] >= 0, where, '희소 alpha', '>=0', pre['count'])
        terms = list(zip(pre['indices'], pre['values']))
        if p['r_reverse']:
            terms.reverse()
        r = cost[j]
        for idx, v in terms:
            r = r - f64(v) * cost[st['header'][idx]]
        e_tol = h * p['e_scale']
        if violated:
            sigma = 0.0
            for idx, v in zip(pre['indices'], pre['values']):  # C 1329-1349: B는 더하고 A는 뺀다(반전 전 alpha)
                mk = rin['marks'][idx]
                if mk == '42':
                    sigma = sigma + f64(v)
                elif mk == '41':
                    sigma = sigma - f64(v)
            d, clause = direction_violated(p, r, dL, dU, sigma, e_tol, p['k_reject'] * h)
        else:
            d, clause = direction(p, r, dL, dU, e_tol)
        need(rin['selected']['direction'] == d, where, '방향', d, rin['selected']['direction'])
        # 재시도 게이트(C 1447-1452, asm 0x1c258e-0x1c25ae): 위반 경로에서 054eef가 위치<0 또는 |pivot|<0.001이면 056973
        gate = violated and ((p['gate_neg'] and rout['chosen'] < 0) or abs(f64(rout['pivot'])) < p['gate_pivot'])
        need(len(retries) == int(gate), where, '056973 재시도 횟수', int(gate), len(retries))
        first = rout
        if retries:
            qin, qout = retries[0]
            need(qout.get('event') == 'retry:leave', where, '재시도 반환 사건', 'retry:leave', qout.get('event'))
            # 재시도 입력은 054eef와 같은 배열이고 theta·pivot 슬롯에는 054eef 결과가 들어 있다(04c-7 agent만 배열을 기록)
            for f in ('alpha', 'basic', 'width', 'marks'):
                if f in qin:
                    need(qin[f] == rin[f], where, '재시도 입력 ' + f, '054eef 입력과 같음', '다름')
            need((qin['theta'], qin['pivot']) == (rout['theta'], rout['pivot']), where, '재시도 진입 theta·pivot', (rout['theta'], rout['pivot']), (qin['theta'], qin['pivot']))
            if p['retry_use']:
                rout = qout
        post = rin['alpha']
        need(post['indices'] == pre['indices'], where, 'alpha 인덱스', pre['indices'], post['indices'])
        for a0, a1 in zip(pre['values'], post['values'], strict=True):
            want = bits(-f64(a0)) if d else a0
            need(a1 == want, where, 'alpha 부호 반전', want, a1)
        chosen, theta, piv = rout['chosen'], f64(rout['theta']), f64(rout['pivot'])
        if rout['theta'] == bits(inf):
            # C 1458-1586: theta=I이면 자유 열 또는 방향 0·ub>=I일 때 반대 방향 재호출(범위 밖). 그 밖은 아래 거리 flip이다.
            recall = (lb[j] <= -inf and inf <= ub[j]) or (d == 0 and inf <= ub[j])
            need(not recall, where, 'theta=I 반대 방향 재호출(범위 밖)', '재호출 조건 거짓', '참')
        else:
            # 위치<0·유한 theta는 056973 재진입의 요청 2 반환뿐이며 관측하지 않았다
            need(chosen >= 0, where, 'ratio 반환(위치<0·유한 theta, 미관측)', '>=0', chosen)
        kreject = p['k_reject'] * h
        if d == 0:
            dist = (dU - p['flip_eps'] <= theta) if p['flip_le'] else (dU - p['flip_eps'] < theta)
        else:
            dist = p['flip_dir1'] and not (theta < dL - p['flip_eps'])
        arm = d if p['reject_arm'] == 'formula' else rin['selected']['direction']
        if p['reject_swap']:
            arm = 1 - arm
        if arm == 1:
            reject = r > kreject and dL < kreject
        else:
            reject = r < -kreject and dU < kreject
        reject = reject and p['reject_on'][arm]
        if dist:
            kind, cause = 'flip', 'distance'
        elif reject:
            kind, cause = 'flip', 'reject'
        elif not p['defer_on'] or (p['acc_pivot'] <= piv and p['acc_prod'] <= theta * piv):
            kind, cause = 'pivot', 'accept'
        else:
            kind, cause = 'defer', 'accept-fail'
        nxt = events[k] if k < len(events) else None
        # 보류는 양성 사건(step:defer, M3-04b-3 agent)으로만 인정한다. 단계 사건이 없으면 실패다.
        got = {'step:enter': nxt and nxt.get('kind'), 'step:defer': 'defer'}.get(nxt and nxt['event'], '사건 없음')
        need(got == kind, where, '단계 종류', f'{kind}({cause})', got)
        info = dict(call=e['call'], step=e['step'], enter=j, direction=d, clause=clause, path=rin['name'], r_sign='+' if r > 0 else '-' if r < 0 else '0', kind=kind, cause=cause, r=bits(r),
                    retry=('none' if not retries else 'theta-I' if rout['theta'] == bits(inf) else
                           'changed' if (rout['chosen'], rout['theta']) != (first['chosen'], first['theta']) else 'same'))
        if kind == 'defer':
            need(nxt['theta'] == rout['theta'] and nxt['pivot'] == rout['pivot'], where, '보류 theta·pivot', (rout['theta'], rout['pivot']), (nxt['theta'], nxt['pivot']))
            check_frame(nxt, p, where, r, dL, dU, h, e_tol, kreject)
        else:
            se = nxt
            need(se['selected']['direction'] == rin['selected']['direction'], where, '방향 보존', rin['selected']['direction'], se['selected']['direction'])
            want_theta = bits(inf) if cause == 'reject' else rout['theta']
            need(se['theta'] == want_theta, where, 'step theta', want_theta, se['theta'])
            check_frame(se, p, where, r, dL, dU, h, e_tol, kreject)
            applied = events[k + 1]
            need(applied['event'] == 'step:applied', where, '적용 사건', 'step:applied', applied['event'])
            if kind == 'pivot':
                row = rout['row']
                lv = st['header'][row]
                a = f64(post['values'][chosen])
                mark = bytes.fromhex(rin['marks'][row]).decode('latin1')
                pp = dict(p, fixed=fixed_tol)
                want_state, arm_name = leave_state(pp, a, theta, mark, f64(rin['basic'][row]), f64(rin['width'][row]),
                                                   p['k_leave'] * h, ub[lv] - lb[lv])
                got_state = applied['state']['vstat'][lv]
                need(got_state == want_state, where, f'이탈 상태 leave={lv}', f'{want_state}({arm_name})', got_state)
                info.update(leave=lv, leave_state=want_state, arm=arm_name, mark=mark, alpha_sign='neg' if a < 0 else 'nonneg')
            else:
                need(se['selected']['leave'] == -1, where, 'flip leave', -1, se['selected']['leave'])
        out.append(info)
        i = k
    return out


MUTANTS = {
    'dir-ten-9': dict(ten=9.0), 'dir-le': dict(dir_strict=False), 'dir-e-0.1h': dict(e_scale=0.1),
    'dir-no-neg': dict(neg_rdl=False), 'dir-r-ge0': dict(r_pos_strict=False), 'r-reverse-order': dict(r_reverse=True),
    'flip-no-eps': dict(flip_eps=0.0), 'flip-lt': dict(flip_le=False), 'reject-10h': dict(k_reject=10.0),
    'reject-1e4h': dict(k_reject=1e4), 'reject-arm-current': dict(reject_arm='current'), 'reject-arm-swap': dict(reject_swap=True),
    'accept-pivot-1e-3': dict(acc_pivot=1e-3), 'accept-prod-0': dict(acc_prod=0.0), 'leave-k-h': dict(k_leave=1.0),
    'leave-fixed-lt': dict(fixed_le=False), 'leave-L-nonneg-2': dict(l_nonneg=-2), 'leave-neg-default-2': dict(neg_default=-2),
    'leave-X-1': dict(x_state=-1), 'leave-U-le': dict(u_nonneg_lt=False),
    # M3-04b-3: 새로 관측한 arm마다 그 arm만 바꾸는 변조
    'reject-off-0': dict(reject_on=(False, True)), 'reject-off-1': dict(reject_on=(True, False)), 'defer-off': dict(defer_on=False),
    'flip-dir1-off': dict(flip_dir1=False), 'dir-first-off': dict(first_on=False), 'leave-fixed-1': dict(fixed_state=-1),
    'leave-fixed-lt+X-1': dict(fixed_le=False, x_state=-1), 'leave-negU-1': dict(neg_u=-1),
    'leave-negL-always-2': dict(neg_l=-2), 'leave-negL-always-1': dict(neg_l=-1),
    'leave-nonnegU-always-1': dict(nonneg_u=-1), 'leave-nonnegU-always-2': dict(nonneg_u=-2),
    # M3-04c-2: 위반 basis 방향식의 절마다 그 절만 바꾸는 변조
    'v-pre1-off': dict(v_pre=(False, True)), 'v-pre0-off': dict(v_pre=(True, False)), 'v-sigma-neg': dict(v_sigma_sign=-1.0),
    'v-clause1-off': dict(v_clause=(False, True)), 'v-clause0-off': dict(v_clause=(True, False)), 'v-small-off': dict(v_small=False),
    'v-small1-off': dict(v_small_clause=(False, True)), 'v-small0-off': dict(v_small_clause=(True, False)),
    'v-r-ge0': dict(v_r_strict=False), 'v-sigma-ge0': dict(v_sigma_strict=False), 'v-ten-9': dict(v_ten=9.0),
}


def mutate(datasets, decision_only=False):
    """모델 변조별로 저장 자료가 거부하는지 기록한다. 첫 거부 위치를 남긴다.
    decision_only면 frame을 지운 사본으로 돌려 상수 값 대조가 아닌 판단으로만 거부되는지 본다."""
    if decision_only:
        datasets = [(label, dict(rec, events=strip(rec['events']))) for label, rec in datasets]
    res = {}
    for name, change in MUTANTS.items():
        p = dict(DEFAULT, **change)
        res[name] = 'survived'
        for label, rec in datasets:
            try:
                decide(rec, p, label)
            except Mismatch as err:
                res[name] = 'rejected: ' + str(err)
                break
    return res


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def strip(events):
    ev = copy.deepcopy(events)
    for e in ev:
        e.pop('frame', None)
    return ev


def validate_new(paths, manifest, case):
    """04b-2 캡처 3회의 출처·해시·독립성·사건 일치와 상태 형식을 검사한다."""
    assert len(paths) >= 3 and len({p.resolve() for p in paths}) == len(paths), '서로 다른 독립 캡처 3회 필요'
    recs = [json.loads(p.read_text()) for p in paths]
    assert len({r['run_id'] for r in recs}) == len(recs) and len({r['pid'] for r in recs}) == len(recs), '독립 실행 중복'
    c = manifest['cases'][case]
    tool, agent = c.get('tool', TOOLS[0]), c.get('agent', TOOLS[1])  # M3-04b-3 사례는 시작 벡터 도구·보류 훅 agent
    for r in recs:
        assert r['schema'] == c.get('schema', 'M3-04b-2-v1') and r['origin'] == 'natural', '출처'
        assert r['binary_sha256'] == BINARY_SHA, 'binary_sha256'
        assert r['tool_sha256'] == manifest['tools'][tool] == sha(ROOT / tool), 'tool_sha256'
        assert r['agent_sha256'] == manifest['tools'][agent] == sha(ROOT / agent), 'agent_sha256'
        assert r['model_sha256'] == c['model_sha256'] == sha(ROOT / c['model']), 'model_sha256'
        assert r['kind'] == c['kind'] and r['params'] == c['params'], '설정'
        assert r.get('starts') == c.get('starts'), '시작 벡터'
        assert r['errors'] == [] and r['reason'] == 'process-terminated', '실행 오류'
        if c.get('role') == 'control':
            assert r['events'] == [], '대조군에 push 사건'
            continue
        assert r['events'], '목표 0회'
        for e in r['events']:
            assert 'call' in e, '알 수 없는 사건(x87 등): ' + e.get('event', '?')
            if 'state' in e:
                step.state(e['state'], e['event'] != 'push:enter')
            if e['event'] == 'supply:enter':
                assert e['batch'] == 0, 'batch는 범위 밖'
            if e['event'] in ('step:enter', 'step:defer'):
                assert set(e['frame']) == set(FRAME), 'frame 누락'
                for v in e['frame'].values():
                    step.number(v)
    fields = ['kind', 'model_sha256', 'binary_sha256', 'tool_sha256', 'agent_sha256', 'params', 'starts', 'environment', 'output', 'result', 'reason', 'errors', 'events']
    for r in recs[1:]:
        for f in fields:
            assert r[f] == recs[0][f], '독립 실행 불일치: ' + f
    assert step.digest(recs[0]['events']) == c['events_sha256'], '기준 사건·상태 변조'
    return recs


def summary(steps):
    steps_tags = {}
    keys = ['direction', 'kind', 'cause', 'arm', 'retry']
    for s in steps:
        tag = f"dir{s['direction']}-{s['clause']}-r{s['r_sign']}" + (f"-{s['cause']}" if s['kind'] == 'flip' else '')
        out_tag = 'step:' + tag
        steps_tags[out_tag] = steps_tags.get(out_tag, 0) + 1
    out = steps_tags
    for s in steps:
        for kk in keys:
            if kk in s:
                tag = f'{kk}={s[kk]}'
                out[tag] = out.get(tag, 0) + 1
    for s in steps:
        if 'leave_state' in s:
            tag = f"leave={s['arm']}{s['leave_state']}"
            out[tag] = out.get(tag, 0) + 1
    out['r_nonzero'] = sum(1 for s in steps if f64(s['r']) != 0.0)
    return dict(sorted(out.items()))


def load_all(manifest, arms=False, violated=False):
    """04b-1 저장 캡처(검사 통과 후)와 04b-2 사례들을 (이름, 레코드) 목록으로 돌려준다.
    arms면 M3-04b-3 대상 사례도 더한다(대조군·비간섭도 함께 검사). violated면 M3-04c-2 위반 basis 사례도 더하고
    필수 arm(REQUIRED_VIOLATED)을 확인한다."""
    data = []
    for kind in ('primal', 'dual'):
        paths = [OLD / f'{kind}-{i}.json' for i in (1, 2, 3)]
        step.check(paths)
        data.append((f'04b-1/{kind}', json.loads(paths[0].read_text())))
    for case, c in manifest['cases'].items():
        if c.get('role') != 'target':
            continue
        recs = validate_new([ROOT / x for x in c['captures']], manifest, case)
        data.append((case, recs[0]))
    if arms:
        am = json.loads((ARMS / 'manifest.json').read_text())
        for case, c in am['cases'].items():
            if c['role'] == 'noninterference':
                noninterference_arm(am, c)
                continue
            if c['role'] == 'noninterference-old-agent':
                noninterference_old_agent(manifest, am, c)
                continue
            recs = validate_new([ROOT / x for x in c['captures']], am, case)
            if c['role'] == 'target':
                data.append((case, recs[0]))
    if violated:
        vm = json.loads((VIOL / 'manifest.json').read_text())
        steps = {}
        for case, c in vm['cases'].items():
            if c['role'] == 'noninterference':
                noninterference_retry(vm, c)
                continue
            recs = validate_new([ROOT / x for x in c['captures']], vm, case)
            if c['role'] == 'target':
                data.append((case, recs[0]))
                steps[case] = decide(recs[0], label=case)
        required_violated(steps)
    return data


def required_violated(steps):
    """위반 대상 두 순서의 단계가 모두 위반 경로이고 필수 arm(REQUIRED_VIOLATED)을 합집합으로 다 덮는지 확인한다."""
    assert len(steps) == 2, '위반 대상 두 순서 필요'
    seen = {k: set() for k in REQUIRED_VIOLATED}
    for case, st in steps.items():
        for s in st:
            assert s['path'] == 'infeasible', f"{case} call{s['call']} step{s['step']}: 위반 LP의 feasible 단계"
            for k in seen:
                seen[k].add(s.get(k))
    for k, want in REQUIRED_VIOLATED.items():
        assert want <= seen[k], f'위반 경로 필수 {k} 누락: {sorted(want - seen[k])}'


def noninterference_retry(vm, c):
    """재시도 훅(retry:*)을 뺀 새 agent 사건이 04b-3 arm agent 캡처와 같아야 한다. firing이면 훅이 실제로 발동해야 한다."""
    new, old = (json.loads((ROOT / c[k]).read_text()) for k in ('new', 'old'))
    am = json.loads((ARMS / 'manifest.json').read_text())
    assert new['agent_sha256'] == vm['tools'][VTOOLS[1]] == sha(ROOT / VTOOLS[1]), '새 agent'
    assert new['tool_sha256'] == vm['tools'][VTOOLS[0]] == sha(ROOT / VTOOLS[0]), '새 도구'
    for i, what in enumerate(('tool', 'agent')):
        assert old[what + '_sha256'] == am['tools'][ATOOLS[i]] == vm['tools'][ATOOLS[i]] == sha(ROOT / ATOOLS[i]), '이전 ' + what
    assert new['binary_sha256'] == old['binary_sha256'] == BINARY_SHA, '비간섭 바이너리'
    assert new['model_sha256'] == old['model_sha256'] == c['model_sha256'], '비간섭 모델'
    assert new['params'] == old['params'] and new['starts'] == old['starts'] and new['kind'] == old['kind'] == c['kind'], '비간섭 설정'
    assert new['errors'] == old['errors'] == [] and new['reason'] == old['reason'] == 'process-terminated', '비간섭 실행 오류'
    fired = [e for e in new['events'] if e['event'].startswith('retry:')]
    assert bool(fired) == c['firing'], ('재시도 훅 발동 여부', c['firing'], len(fired))
    assert [e for e in new['events'] if not e['event'].startswith('retry:')] == old['events'], '재시도 사건을 뺀 사건이 이전 agent와 다름'
    assert new['result'] == old['result'] and new['output'] == old['output'], '비간섭 결과'
    return len(fired)


def noninterference_old_agent(manifest, am, c):
    """보류 훅이 실제로 발동하는 arm LP에서: 새 agent 대상 캡처에서 step:defer를 빼면 04b-2 agent 탐색 캡처와 같아야 한다."""
    old = json.loads((ROOT / c['captures'][0]).read_text())
    new = json.loads((ROOT / c['compare']).read_text())
    assert old['agent_sha256'] == manifest['tools'][TOOLS[1]] == sha(ROOT / TOOLS[1]), '탐색 캡처 agent가 04b-2 agent가 아님'
    assert new['agent_sha256'] == am['tools'][c['agent']], '대상 agent'
    assert old['binary_sha256'] == new['binary_sha256'] == BINARY_SHA and old['model_sha256'] == new['model_sha256'] == c['model_sha256'], '비간섭 출처'
    assert old['params'] == new['params'] and old['starts'] == new['starts'] and old['kind'] == new['kind'], '비간섭 설정'
    assert any(e['event'] == 'step:defer' for e in new['events']), '보류 훅이 발동하지 않음'
    assert [e for e in new['events'] if e['event'] != 'step:defer'] == old['events'], '보류 훅을 뺀 사건이 이전 agent와 다름'
    assert new['result'] == old['result'] and new['output'] == old['output'], '비간섭 결과'


def noninterference_arm(am, c):
    """보류 훅 agent의 face5 primal 캡처에서 step:defer를 빼면 04b-2 저장 캡처와 사건이 같아야 한다."""
    new = json.loads((ROOT / c['captures'][0]).read_text())
    old = json.loads((ROOT / c['compare']).read_text())
    assert new['agent_sha256'] == am['tools'][c['agent']] == sha(ROOT / c['agent']), '비간섭 agent'
    assert new['tool_sha256'] == am['tools'][c['tool']] == sha(ROOT / c['tool']), '비간섭 도구'
    assert new['binary_sha256'] == BINARY_SHA and new['model_sha256'] == old['model_sha256'] == c['model_sha256'], '비간섭 출처'
    assert new['params'] == old['params'] and new['starts'] is None, '비간섭 설정'
    assert [e for e in new['events'] if e['event'] != 'step:defer'] == old['events'], '보류 훅을 뺀 사건이 04b-2와 다름'
    assert new['result'] == old['result'] and new['output'] == old['output'], '비간섭 결과'
    return len(new['events'])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--old', action='store_true', help='04b-1 저장 캡처만 재생(새 manifest 불필요)')
    ap.add_argument('--probe', type=Path, nargs='+', help='탐색 캡처 한 개씩 판단 재생(합격 근거 아님)')
    ap.add_argument('--noninterference', nargs=2, type=Path, metavar=('NEW', 'OLD'))
    ap.add_argument('--mutation', type=Path, help='모델 변조 결과 기록(새 파일)')
    ap.add_argument('--self-test', action='store_true', help='캡처 변조 거부 확인')
    a = ap.parse_args()
    try:
        if a.probe:
            for path in a.probe:
                rec = json.loads(path.read_text())
                try:
                    st = decide(rec, label=path.name)
                    print(path.name, 'ok', json.dumps(summary(st), ensure_ascii=False))
                except (Mismatch, KeyError, IndexError) as err:
                    print(path.name, 'out:', err)
            return
        if a.noninterference:
            new, old = (json.loads(x.read_text()) for x in a.noninterference)
            assert new['agent_sha256'] == sha(ROOT / TOOLS[1]), 'agent_sha256'
            assert all('frame' in e for e in new['events'] if e['event'] == 'step:enter'), 'frame 누락'
            assert strip(new['events']) == old['events'], '새 필드를 뺀 사건이 04b-1과 다름'
            assert new['params'] == old['params'] and new['model_sha256'] == old['model_sha256']
            st = decide(new, label='noninterference')
            print(json.dumps({'events': len(new['events']), 'steps': len(st), 'summary': summary(st)}, ensure_ascii=False))
            return
        if a.old:
            data = []
            for kind in ('primal', 'dual'):
                paths = [OLD / f'{kind}-{i}.json' for i in (1, 2, 3)]
                step.check(paths)
                data.append((f'04b-1/{kind}', json.loads(paths[0].read_text())))
        else:
            manifest = json.loads((BASE / 'manifest.json').read_text())
            data = load_all(manifest, arms=True, violated=True)
        report = {}
        for label, rec in data:
            st = decide(rec, label=label)
            report[label] = {'steps': len(st), 'summary': summary(st)}
        if a.mutation:
            assert not a.mutation.exists(), '기록 덮어쓰기 거부'
            res = {'full': mutate(data), 'decision_only': mutate(data, decision_only=True)}
            a.mutation.write_text(json.dumps(res, ensure_ascii=False, indent=2) + '\n')
            report['mutants'] = {k: {'rejected': sum(v != 'survived' for v in r.values()), 'survived': [m for m, v in r.items() if v == 'survived']}
                                 for k, r in res.items()}
        if a.self_test:
            report['self_test'] = self_test(data)
        print(json.dumps(report, ensure_ascii=False, indent=1))
    except (AssertionError, KeyError, TypeError, ValueError, IndexError) as err:
        print('불합격: ' + str(err), file=sys.stderr)
        sys.exit(1)


def capture_tamper(base=BASE, extra=None):
    """첫 대상 사례 캡처 하나를 변조해 validate_new가 거부하는지 확인한다."""
    import tempfile
    manifest = json.loads((base / 'manifest.json').read_text())
    case, c = next((k, v) for k, v in manifest['cases'].items() if v.get('role') == 'target')
    paths = [ROOT / x for x in c['captures']]
    rec = json.loads(paths[0].read_text())
    edits = {
        'binary': lambda x: x.__setitem__('binary_sha256', '0' * 64),
        'agent': lambda x: x.__setitem__('agent_sha256', '0' * 64),
        'model': lambda x: x.__setitem__('model_sha256', '0' * 64),
        'synthetic': lambda x: x.__setitem__('origin', 'original-synthetic'),
        'params': lambda x: x['params'].__setitem__('Crossover', 0),
        'events-empty': lambda x: x.__setitem__('events', []),
        'x87': lambda x: x['events'].append({'event': 'unsupported:x87'}),
        'batch': lambda x: next(e for e in x['events'] if e['event'] == 'supply:enter').__setitem__('batch', 1),
        'frame-missing': lambda x: next(e for e in x['events'] if e['event'] == 'step:enter').pop('frame'),
        'event-value': lambda x: next(e for e in x['events'] if e['event'] == 'ratio:leave').__setitem__('theta', '0' * 16),
        'run-duplicate': lambda x: x.__setitem__('run_id', json.loads(paths[1].read_text())['run_id']),
        **(extra or {}),
    }
    out = {}
    with tempfile.TemporaryDirectory() as tmp:
        for name, edit in edits.items():
            x = copy.deepcopy(rec)
            edit(x)
            p = Path(tmp) / f'{name}.json'
            p.write_text(json.dumps(x))
            try:
                validate_new([p, *paths[1:]], manifest, case)
                raise Mismatch('캡처 변조 허용: ' + name)
            except Mismatch:
                raise
            except (AssertionError, KeyError, StopIteration) as err:
                out['capture-' + name] = str(err) or type(err).__name__
    return out


def self_test(data):
    """실제 캡처의 판단 필드를 하나씩 바꿔 대조가 실패하는지 확인한다."""
    label, rec = next((lb, r) for lb, r in reversed(data) if lb.startswith('arms-'))
    edits = {}

    def first(rec, ev, pred=lambda e: True):
        return next(e for e in rec['events'] if e['event'] == ev and pred(e))
    edits['direction'] = lambda x: first(x, 'ratio:enter')['selected'].__setitem__('direction', 1 - first(x, 'ratio:enter')['selected']['direction'])
    edits['alpha-sign'] = lambda x: first(x, 'ratio:enter')['alpha']['values'].__setitem__(0, bits(-f64(first(x, 'ratio:enter')['alpha']['values'][0])))
    edits['kind'] = lambda x: first(x, 'step:enter').__setitem__('kind', 'pivot' if first(x, 'step:enter')['kind'] == 'flip' else 'flip')
    edits['theta'] = lambda x: first(x, 'step:enter').__setitem__('theta', bits(f64(first(x, 'step:enter')['theta']) * 2 + 1))
    edits['marks-B'] = lambda x: first(x, 'ratio:enter')['marks'].__setitem__(0, '42')
    edits['cost'] = lambda x: first(x, 'push:enter')['model']['cost'].__setitem__(first(x, 'supply:leave', lambda e: e['candidate'] >= 0)['candidate'], bits(7.25))
    def leave_state_edit(x):
        ev = x['events']
        k = next(i for i, e in enumerate(ev) if e['event'] == 'step:enter' and e['kind'] == 'pivot')
        row = next(e for e in reversed(ev[:k]) if e['event'] == 'ratio:leave')['row']
        ev[k + 1]['state']['vstat'][ev[k]['state']['header'][row]] = -7
    edits['leave-state'] = leave_state_edit
    if any('frame' in e for e in rec['events']):
        for name in FRAME:
            edits['frame-' + name] = lambda x, nm=name: first(x, 'step:enter')['frame'].__setitem__(nm, bits(f64(first(x, 'step:enter')['frame'][nm]) + 1.0))
    out = capture_tamper()
    def pstart(x):
        x['starts']['PStart'][0] = x['starts']['PStart'][0] + 1.0
    arm_edits = {'starts': pstart, 'defer-frame-missing': lambda x: next(e for e in x['events'] if e['event'] == 'step:defer').pop('frame')}
    out.update({'arms-' + k: v for k, v in capture_tamper(ARMS, arm_edits).items()})
    am = json.loads((ARMS / 'manifest.json').read_text())
    case, c = next((k, v) for k, v in am['cases'].items() if v['role'] == 'control')
    import tempfile
    with tempfile.TemporaryDirectory() as tmp:
        x = json.loads((ROOT / c['captures'][0]).read_text())
        x['events'] = [{'event': 'push:enter', 'call': 1}]
        q = Path(tmp) / 'control.json'
        q.write_text(json.dumps(x))
        try:
            validate_new([q, *[ROOT / y for y in c['captures'][1:]]], am, case)
        except AssertionError as err:
            out['control-events'] = str(err)
        else:
            raise Mismatch('캡처 변조 허용: control-events')
    manifest = json.loads((BASE / 'manifest.json').read_text())
    case, c = next((k, v) for k, v in am['cases'].items() if v['role'] == 'noninterference-old-agent')
    with tempfile.TemporaryDirectory() as tmp:
        x = json.loads((ROOT / c['compare']).read_text())
        k = next(i for i, e in enumerate(x['events']) if e['event'] == 'step:enter')
        x['events'][k]['theta'] = '0' * 16
        q = Path(tmp) / 'new.json'
        q.write_text(json.dumps(x))
        try:
            noninterference_old_agent(manifest, am, dict(c, compare=str(q)))
        except AssertionError as err:
            out['old-agent-noninterference'] = str(err)
        else:
            raise Mismatch('캡처 변조 허용: old-agent-noninterference')
    if any(e['event'] == 'step:defer' for e in rec['events']):
        def drop_defer(x):
            x['events'] = [e for e in x['events'] if e['event'] != 'step:defer']
        def defer_pivot(x):
            d = first(x, 'step:defer')
            d['pivot'] = bits(f64(d['pivot']) * 2)
        edits['defer-drop'] = drop_defer
        edits['defer-pivot'] = defer_pivot
        edits['defer-frame-r'] = lambda x: first(x, 'step:defer')['frame'].__setitem__('r', bits(f64(first(x, 'step:defer')['frame']['r']) + 1.0))
    for name, edit in edits.items():
        x = copy.deepcopy(rec)
        edit(x)
        try:
            decide(x, label=label)
            raise AssertionError('변조 허용: ' + name)
        except (Mismatch, StopIteration) as err:
            out[name] = str(err)
    out.update(self_test_violated(data))
    return out


def self_test_violated(data):
    """M3-04c-2: 위반 경로 캡처·판단·비간섭·필수 arm 변조가 거부되는지 확인한다."""
    import tempfile
    label, rec = next((lb, r) for lb, r in reversed(data) if lb.startswith('violated-'))
    out = {}
    steps = decide(rec, label=label)

    def window(x, clause):
        s = next(s for s in steps if s['clause'] == clause)
        ev = x['events']
        return ev, next(i for i, e in enumerate(ev) if e['event'] == 'ratio:leave' and (e['call'], e['step']) == (s['call'], s['step']))

    def retry_inject(x):
        ev, k = window(x, 'sigma')
        ev.insert(k + 1, dict(event='retry:enter', call=ev[k]['call'], step=ev[k]['step']))

    def path_feasible(x):
        ev, k = window(x, 'sigma')
        ev[k - 1]['name'] = 'feasible'

    def sigma_mark(x):  # σ=+1 행의 B 표시를 N으로: σ=0이 되어 방향이 바뀌어야 한다
        ev, k = window(x, 'sigma')
        rin = ev[k - 1]
        row = next(e for e in reversed(ev[:k]) if e['event'] == 'supply:leave')['alpha']['indices'][0]
        rin['marks'][row] = '4e'

    def tiny_sigma(x):  # |σ|<e bound 절(S6)의 σ=5e-9를 5e-6으로: |σ|>=e의 σ 부호 절로 가서 방향이 1이 되어야 한다
        s = next(s for s in steps if s['clause'] == 'small-bound0')
        sl = next(e for e in x['events'] if e['event'] == 'supply:leave' and (e['call'], e['step']) == (s['call'], s['step']))
        vals = sl['alpha']['values']
        i = min(range(len(vals)), key=lambda q: abs(f64(vals[q])))
        vals[i] = bits(f64(vals[i]) * 1e3)

    for name, edit in {'v-retry': retry_inject, 'v-path-feasible': path_feasible, 'v-sigma-mark': sigma_mark,
                       'v-tiny-sigma': tiny_sigma}.items():
        x = copy.deepcopy(rec)
        edit(x)
        try:
            decide(x, label=label)
            raise AssertionError('변조 허용: ' + name)
        except (Mismatch, StopIteration) as err:
            out[name] = str(err)
    vm = json.loads((VIOL / 'manifest.json').read_text())
    out.update({'violated-' + k: v for k, v in capture_tamper(VIOL, {'retry-event': lambda x: x['events'].insert(
        3, dict(event='retry:enter', call=1, step=1))}).items()})
    # 비간섭: 발동 사례에서 재시도 사건을 지우면 firing 조건, 비발동 사례 사건 값 변조는 사건 비교로 거부돼야 한다
    for case, edit in (('retry-primal', lambda x: x.__setitem__('events', [e for e in x['events'] if not e['event'].startswith('retry:')])),
                       ('violated-primal-armagent', lambda x: next(e for e in x['events'] if e['event'] == 'step:enter').__setitem__('theta', '0' * 16))):
        c = vm['cases'][case]
        with tempfile.TemporaryDirectory() as tmp:
            x = json.loads((ROOT / c['new']).read_text())
            edit(x)
            q = Path(tmp) / 'new.json'
            q.write_text(json.dumps(x))
            try:
                noninterference_retry(vm, dict(c, new=str(q)))
            except AssertionError as err:
                out['noninterference-' + case] = str(err)
            else:
                raise Mismatch('비간섭 변조 허용: ' + case)
    # 필수 arm: 실제 단계에서 종단·원인·이탈 arm 하나씩을 지우면 required_violated가 거부해야 한다
    steps = {lb: decide(r, label=lb) for lb, r in data if lb.startswith('violated-')}
    required_violated(steps)
    for key, value in (('clause', 'small-bound'), ('cause', 'reject'), ('arm', 'nonneg-A'), ('path', 'feasible')):
        if key == 'path':
            changed = {lb: [dict(s, path=value) if i == 0 else s for i, s in enumerate(st)] for lb, st in steps.items()}
        else:
            changed = {lb: [s for s in st if s.get(key) != value] for lb, st in steps.items()}
        try:
            required_violated(changed)
        except AssertionError as err:
            out[f'required-{key}-{value}'] = str(err)
        else:
            raise Mismatch(f'필수 arm 누락 허용: {key}={value}')
    return out


if __name__ == '__main__':
    main()
