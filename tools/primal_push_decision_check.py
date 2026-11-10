"""M3-04b-2: primal push 한 단계의 방향·거리 flip·거부·승인·이탈 상태 판단을 원본 캡처와 대조한다.

판단 모델은 134a87 asm(0x1c2164-0x1c2d7f)을 따른다. feasible basis 경로만 다룬다.
04b-1 저장 캡처(frame 없음)와 04b-2 새 캡처(frame 있음)를 모두 읽는다.
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
BINARY_SHA = step.BINARY_SHA
TOOLS = ['tools/primal_push_decision_capture.js', 'tools/primal_push_decision_agent.js']
FRAME = ['r', 'dL', 'dU', 'k100', 'km100', 'k2', 'e']


def f64(bits):
    return struct.unpack('<d', bytes.fromhex(bits))[0]


def bits(value):
    return struct.pack('<d', value).hex()


DEFAULT = dict(ten=10.0, e_scale=0.01, dir_strict=True, neg_rdl=True, r_pos_strict=True, r_reverse=False,
               flip_eps=1e-10, flip_le=True, k_reject=100.0, reject_arm='formula', reject_swap=False,
               acc_pivot=1e-6, acc_prod=-1e-5, k_leave=2.0, fixed_le=True, l_nonneg=-1, neg_default=-1,
               x_state=-4, u_nonneg_lt=True)


class Mismatch(AssertionError):
    pass


def need(cond, where, field, expected, actual):
    if not cond:
        raise Mismatch(f'{where} {field}: 기대 {expected} 실제 {actual}')


def direction(p, r, dL, dU, e):
    first = (p['ten'] * dL < dU if p['dir_strict'] else p['ten'] * dL <= dU) and ((-(r * dL) if p['neg_rdl'] else r * dL) < e)
    pos = 0 < r if p['r_pos_strict'] else 0 <= r
    return 1 if first or ((dL <= p['ten'] * dU or e <= dU * r) and pos) else 0


def leave_state(p, a, t, mark, b, w, k, width):
    if (width <= p['fixed']) if p['fixed_le'] else (width < p['fixed']):
        return -4, 'fixed'
    if 0 <= a:
        if mark in 'LN':
            return (p['l_nonneg'] if mark == 'L' else -1), 'nonneg-' + mark
        if mark == 'X':
            return p['x_state'], 'nonneg-X'
        if mark == 'A':
            return (-2 if abs((b - w) / a - t) < (t + 1.0) * k else -1), 'nonneg-A'
        if mark == 'U':
            hit = abs(a) * t < k if p['u_nonneg_lt'] else abs(a) * t <= k
            return (-2 if hit else -1), 'nonneg-U'
        return -1, 'nonneg-' + mark
    if mark in 'NU':
        return -2, 'neg-' + mark
    if mark == 'B':
        return (-2 if (t + 1.0) * k <= abs(b / a - t) else -1), 'neg-B'
    if mark == 'X':
        return p['x_state'], 'neg-X'
    if mark == 'L':
        return (-2 if k <= abs(a) * t else -1), 'neg-L'
    return p['neg_default'], 'neg-' + mark


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
        ratios = []
        while k < len(events) and events[k]['event'] not in ('supply:enter', 'push:leave', 'step:enter'):
            if events[k]['event'] == 'ratio:enter':
                ratios.append((events[k], events[k + 1]))
            k += 1
        need(len(ratios) == 1, where, 'ratio 호출 수(재호출 제외 범위)', 1, len(ratios))
        rin, rout = ratios[0]
        need(rin['name'] == 'feasible', where, 'ratio 경로(제외 범위)', 'feasible', rin['name'])
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
        d = direction(p, r, dL, dU, e_tol)
        need(rin['selected']['direction'] == d, where, '방향', d, rin['selected']['direction'])
        post = rin['alpha']
        need(post['indices'] == pre['indices'], where, 'alpha 인덱스', pre['indices'], post['indices'])
        for a0, a1 in zip(pre['values'], post['values'], strict=True):
            want = bits(-f64(a0)) if d else a0
            need(a1 == want, where, 'alpha 부호 반전', want, a1)
        chosen, theta, piv = rout['chosen'], f64(rout['theta']), f64(rout['pivot'])
        need(chosen >= 0, where, 'ratio 반환(-1은 재호출 경로, 제외)', '>=0', chosen)
        kreject = p['k_reject'] * h
        if d == 0:
            dist = (dU - p['flip_eps'] <= theta) if p['flip_le'] else (dU - p['flip_eps'] < theta)
        else:
            dist = not (theta < dL - p['flip_eps'])
        arm = d if p['reject_arm'] == 'formula' else rin['selected']['direction']
        if p['reject_swap']:
            arm = 1 - arm
        if arm == 1:
            reject = r > kreject and dL < kreject
        else:
            reject = r < -kreject and dU < kreject
        if dist:
            kind, cause = 'flip', 'distance'
        elif reject:
            kind, cause = 'flip', 'reject'
        elif p['acc_pivot'] <= piv and p['acc_prod'] <= theta * piv:
            kind, cause = 'pivot', 'accept'
        else:
            kind, cause = 'defer', 'accept-fail'
        nxt = events[k] if k < len(events) else None
        got = nxt['event'] == 'step:enter' and nxt['kind'] or 'defer'
        need(got == kind, where, '단계 종류', f'{kind}({cause})', got)
        info = dict(call=e['call'], step=e['step'], enter=j, direction=d, kind=kind, cause=cause, r=bits(r))
        if kind != 'defer':
            se = nxt
            need(se['selected']['direction'] == rin['selected']['direction'], where, '방향 보존', rin['selected']['direction'], se['selected']['direction'])
            want_theta = bits(inf) if cause == 'reject' else rout['theta']
            need(se['theta'] == want_theta, where, 'step theta', want_theta, se['theta'])
            if 'frame' in se:
                fr = se['frame']
                want = dict(r=bits(r), dL=bits(dL), dU=bits(dU), k100=bits(100.0 * h), km100=bits(-100.0 * h), k2=bits(h + h), e=bits(h * 0.01))
                if p is not DEFAULT:
                    want.update(r=bits(r), e=bits(e_tol), k100=bits(kreject), km100=bits(-kreject), k2=bits(p['k_leave'] * h))
                for name in FRAME:
                    need(fr[name] == want[name], where, 'frame ' + name, want[name], fr[name])
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
}


def mutate(datasets):
    """모델 변조별로 저장 자료가 거부하는지 기록한다. 첫 거부 위치를 남긴다."""
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
    for r in recs:
        assert r['schema'] == 'M3-04b-2-v1' and r['origin'] == 'natural', '출처'
        assert r['binary_sha256'] == BINARY_SHA, 'binary_sha256'
        assert r['tool_sha256'] == manifest['tools'][TOOLS[0]] == sha(ROOT / TOOLS[0]), 'tool_sha256'
        assert r['agent_sha256'] == manifest['tools'][TOOLS[1]] == sha(ROOT / TOOLS[1]), 'agent_sha256'
        assert r['model_sha256'] == c['model_sha256'] == sha(ROOT / c['model']), 'model_sha256'
        assert r['kind'] == c['kind'] and r['params'] == c['params'], '설정'
        assert r['errors'] == [] and r['reason'] == 'process-terminated', '실행 오류'
        assert r['events'], '목표 0회'
        for e in r['events']:
            assert 'call' in e, '알 수 없는 사건(x87 등): ' + e.get('event', '?')
            if 'state' in e:
                step.state(e['state'], e['event'] != 'push:enter')
            if e['event'] == 'supply:enter':
                assert e['batch'] == 0, 'batch는 범위 밖'
            if e['event'] == 'step:enter':
                assert set(e['frame']) == set(FRAME), 'frame 누락'
                for v in e['frame'].values():
                    step.number(v)
    fields = ['kind', 'model_sha256', 'binary_sha256', 'tool_sha256', 'agent_sha256', 'params', 'environment', 'result', 'reason', 'errors', 'events']
    for r in recs[1:]:
        for f in fields:
            assert r[f] == recs[0][f], '독립 실행 불일치: ' + f
    assert step.digest(recs[0]['events']) == c['events_sha256'], '기준 사건·상태 변조'
    return recs


def summary(steps):
    keys = ['direction', 'kind', 'cause', 'arm']
    out = {}
    for s in steps:
        for kk in keys:
            if kk in s:
                tag = f'{kk}={s[kk]}'
                out[tag] = out.get(tag, 0) + 1
    out['r_nonzero'] = sum(1 for s in steps if f64(s['r']) != 0.0)
    return dict(sorted(out.items()))


def load_all(manifest):
    """04b-1 저장 캡처(검사 통과 후)와 04b-2 사례들을 (이름, 레코드) 목록으로 돌려준다."""
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
    return data


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
            data = load_all(manifest)
        report = {}
        for label, rec in data:
            st = decide(rec, label=label)
            report[label] = {'steps': len(st), 'summary': summary(st)}
        if a.mutation:
            assert not a.mutation.exists(), '기록 덮어쓰기 거부'
            res = mutate(data)
            a.mutation.write_text(json.dumps(res, ensure_ascii=False, indent=2) + '\n')
            report['mutants'] = {'rejected': sum(v != 'survived' for v in res.values()), 'survived': [k for k, v in res.items() if v == 'survived']}
        if a.self_test:
            report['self_test'] = self_test(data)
        print(json.dumps(report, ensure_ascii=False, indent=1))
    except (AssertionError, KeyError, TypeError, ValueError, IndexError) as err:
        print('불합격: ' + str(err), file=sys.stderr)
        sys.exit(1)


def capture_tamper():
    """첫 대상 사례 캡처 하나를 변조해 validate_new가 거부하는지 확인한다."""
    import tempfile
    manifest = json.loads((BASE / 'manifest.json').read_text())
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
    label, rec = data[-1]
    edits = {}

    def first(rec, ev, pred=lambda e: True):
        return next(e for e in rec['events'] if e['event'] == ev and pred(e))
    edits['direction'] = lambda x: first(x, 'ratio:enter')['selected'].__setitem__('direction', 1 - first(x, 'ratio:enter')['selected']['direction'])
    edits['alpha-sign'] = lambda x: first(x, 'ratio:enter')['alpha']['values'].__setitem__(0, bits(-f64(first(x, 'ratio:enter')['alpha']['values'][0])))
    edits['kind'] = lambda x: first(x, 'step:enter').__setitem__('kind', 'pivot' if first(x, 'step:enter')['kind'] == 'flip' else 'flip')
    edits['theta'] = lambda x: first(x, 'step:enter').__setitem__('theta', bits(f64(first(x, 'step:enter')['theta']) * 2 + 1))
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
    for name, edit in edits.items():
        x = copy.deepcopy(rec)
        edit(x)
        try:
            decide(x, label=label)
            raise AssertionError('변조 허용: ' + name)
        except (Mismatch, StopIteration) as err:
            out[name] = str(err)
    return out


if __name__ == '__main__':
    main()
