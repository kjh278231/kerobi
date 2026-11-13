#!/usr/bin/env python3
"""M3-04c-21j: `036ade` 풀 계수·local_70 모드 경계 캡처(bench/oracle/m3_push_mode70) 검사기.

controls   : manifest 해시, 5모델 × 3회 사건열 정확 일치(실행 식별자 제외), 5모델 run 1과 21i 대조군 7사례에서 새 사건
             (pool:*, mode70:*)을 뺀 사건열 = 21i 정식 같은 사례(JSON 정확), 결과·출력·파라미터 같음.
acceptance : 036ade 호출마다 풀 사건 순서(start → [mode70 → short70 | mode70 → pivot… → end | dense | end]),
             C 규칙 재계산(start: p3 = f8 + 4·nodes, x138 = max(p3, f8), p4 = p2 − 1; mode70: 풀 검사 C:2572–2620와 새 크기,
             재호출의 p2; 모드 피벗 수 = n, 번호 < n'²), 계약 §1·§4 범위(local_70·풀 부족은 grow22 push lu 392·431·455와
             push 밖 1회뿐, 빈 행·열 0).
--self-test: 변조(바이너리 해시·빈 사건·회차 차이·비간섭 투영·풀 값 1·피벗 교환·short 삭제·새 크기)가 모두 거부되는지 확인.
--write-manifest: 도구·캡처 해시를 manifest.json에 쓴다.
"""
import copy, gzip, hashlib, json, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
D = ROOT / 'bench/oracle/m3_push_mode70'
BASE = ROOT / 'bench/oracle/m3_push_dense'
MODELS = ('grow7', 'grow22', 'perold', 'pilot4', 'stair')
CONTROLS = ('arms', 'retry', 'interior', 'control', 'adlittle', 'scsd1', 'capri')
BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
SCHEMA = 'M3-04c-21j-v1'
TOOLS = ('tools/push_mode70_agent.js', 'tools/push_mode70_capture.js', 'tools/push_mode70_runs.py', 'tools/push_lu_runs.py',
         'tools/push_dense_agent.js', 'tools/frida_resolve.cjs', 'tools/gurun.sh', 'tools/push_mode70_check.py',
         'tools/push_mode70_survey.py', 'tools/push_mode70_survey_agent.js', 'tools/push_mode70_survey_capture.js')
# 계약 §1: local_70 모드 push 재분해(lu id → n')와 풀 부족 재호출 lu
EXPECT_MODE70 = {'grow22': {392: 211, 431: 250, 455: 272}}
EXPECT_SHORT = {'grow22': [431]}
VOLATILE = ('pid', 'run_id')
NEW = ('pool:', 'mode70:')


class Fail(Exception):
    pass


def require(c, msg):
    if not c:
        raise Fail(msg)


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def path_of(d, name, i):
    p = d / f'{name}-{i}.json'
    return p if p.exists() else d / f'{name}-{i}.json.gz'


def load(p):
    b = p.read_bytes()
    return json.loads(gzip.decompress(b) if p.suffix == '.gz' else b)


def load_all():
    caps = {(c, i): load(path_of(D, c, i)) for c in MODELS for i in (1, 2, 3)}
    ctl = {c: load(path_of(D / 'controls', c, 1)) for c in CONTROLS}
    base = {c: load(path_of(BASE, c, 1)) for c in MODELS}
    base.update({('ctl', c): load(path_of(BASE / 'controls', c, 1)) for c in CONTROLS})
    return caps, ctl, base


def first_diff(a, b):
    return next((j for j, (x, y) in enumerate(zip(a, b)) if x != y), min(len(a), len(b)))


def head_ok(cap, tag):
    require(cap['binary_sha256'] == BINARY and cap['schema'] == SCHEMA and not cap['errors']
            and cap['reason'] == 'process-terminated' and cap['result'] is not None, f'{tag}: 바이너리·스키마·오류·종료 사유')


def project(cap):
    return [e for e in cap['events'] if not e['event'].startswith(NEW)]


def same_as_21i(cap, b, tag):
    proj = project(cap)
    if proj != b['events']:
        raise Fail(f'{tag}: 21i 투영 차이 사건 #{first_diff(proj, b["events"])}')
    for f in ('result', 'output', 'params', 'starts', 'model_sha256', 'kind'):
        require(cap[f] == b[f], f'{tag}: 21i {f} 차이')


def controls(caps, ctl, base, manifest=True):
    if manifest:
        m = json.loads((D / 'manifest.json').read_text())
        for k, v in {**m['tools'], **m['captures']}.items():
            require(sha(ROOT / k) == v, f'manifest 해시 불일치 {k}')
    for c in MODELS:
        ref = None
        for i in (1, 2, 3):
            cap = caps[(c, i)]
            head_ok(cap, f'{c}-{i}')
            require(any(e['event'] == 'pool:start' for e in cap['events']), f'{c}-{i}: 풀 사건 없음')
            body = {k: v for k, v in cap.items() if k not in VOLATILE}
            if ref is None:
                ref = body
            elif body != ref:
                if body['events'] != ref['events']:
                    raise Fail(f'{c}-{i}: 회차 차이 사건 #{first_diff(body["events"], ref["events"])}')
                raise Fail(f'{c}-{i}: 회차 차이 ' + str(sorted(k for k in body if body[k] != ref[k])))
        same_as_21i(caps[(c, 1)], base[c], f'{c}-1')
    for c in CONTROLS:
        head_ok(ctl[c], f'controls/{c}')
        require(not any(e['event'].startswith('mode70:') or e['event'] == 'pool:short70' for e in ctl[c]['events']),
                f'controls/{c}: 대조군에 local_70·풀 부족')
        same_as_21i(ctl[c], base[('ctl', c)], f'controls/{c}')


def calls(cap, name):
    """036ade 호출(lu:block enter/leave로 나눔)마다 (push, lu, 풀·모드 사건 목록, 반환)."""
    out, cur = [], None
    for i, e in enumerate(cap['events']):
        k = e['event']
        if k == 'lu:block' and e['phase'] == 'enter':
            require(cur is None, f'{name}#{i}: 036ade 중첩')
            cur = {'push': e['push'], 'lu': e['lu'], 'ev': [], 'at': i}
        elif k == 'lu:block':
            require(cur is not None, f'{name}#{i}: 짝 없는 반환')
            cur['ret'] = e['ret']
            out.append(cur)
            cur = None
        elif k.startswith(NEW):
            require(cur is not None, f'{name}#{i}: 036ade 밖의 {k}')
            require(e['push'] == cur['push'] and e['lu'] == cur['lu'], f'{name}#{i}: 호출 문맥 차이')
            cur['ev'].append(e)
    require(cur is None, f'{name}: 끝나지 않은 036ade 호출')
    return out


def first_fill_free(e):
    return e['p3'] == e['f8'] + 4 * e['nodes'] and e['x138'] == max(e['p3'], e['f8'])


def pool_check(e, m):
    """C:2572–2620의 풀 검사와 새 크기(asm 0xd85ee–0xd88a6)."""
    n2 = e['n'] - min(e['e60'], e['ef0'])
    nn = n2 * n2
    iv67 = (nn - 3 * n2) // (2 * n2)
    iv65 = max(3, (n2 + 2) // 3)
    l40 = nn if iv67 >= iv65 else 3 * n2 + 2 * n2 * iv65
    q = float(nn) * 0.3
    avail = e['b8'] - e['p3']
    short = avail < l40 + q or (e['f8'] - e['c0']) < q or avail < m
    d1 = (l40 + q) - avail
    q2 = q - (e['f8'] - e['c0'])
    x = float(int(max(d1, 0.0)))
    x = q2 if x <= q2 else x
    grow = int((e['p2'] + max(m - avail, int(x))) * 1.4)
    return short, grow, l40


def acceptance(caps, verbose=True):
    tot = {'calls': 0, 'push': 0, 'start': 0, 'end': 0, 'dense': 0, 'mode70': 0, 'short': 0, 'pivots': 0}
    for c in MODELS:
        cap = caps[(c, 1)]
        m = cap['events'][0]['state']['m']
        mode, short = {}, []
        cs = calls(cap, c)
        for j, call in enumerate(cs):
            ev = call['ev']
            kinds = [e['event'] for e in ev]
            tot['calls'] += 1
            tot['push'] += bool(call['push'])
            if not ev:
                continue
            require(kinds[0] == 'pool:start', f'{c} 호출 {j}: 첫 풀 사건 {kinds[0]}')
            s = ev[0]
            require(first_fill_free(s) and s['p4'] == s['p2'] - 1, f'{c} 호출 {j}: start 규칙 {s}')
            tot['start'] += 1
            tail = kinds[1:]
            if tail == ['pool:end'] or tail == ['pool:dense']:
                tot[tail[0][5:]] += 1
                require(call['ret'] == 0, f'{c} 호출 {j}: 반환 {call["ret"]}')
                continue
            require(tail[0] == 'pool:mode70', f'{c} 호출 {j}: 풀 사건 순서 {tail[:3]}')
            e = ev[1]
            require(e['e60'] == 0 and e['ef0'] == 0 and e['m'] == m, f'{c} 호출 {j}: 빈 행·열(범위 밖) {e}')
            need_short, grow, l40 = pool_check(e, m)
            tot['mode70'] += 1
            if call['push']:
                mode.setdefault(call['lu'], e['n'])
            if need_short:
                require(tail == ['pool:mode70', 'pool:short70'] and call['ret'] == -1, f'{c} 호출 {j}: 풀 부족 순서·반환')
                require(all(ev[2][k] == e[k] for k in ('b8', 'c0', 'f8', 'x138', 'p2', 'p3', 'p4')), f'{c} 호출 {j}: short 값')
                nxt = cs[j + 1]
                require(nxt['lu'] == call['lu'] and nxt['ev'] and nxt['ev'][0]['p2'] == grow and nxt['ev'][0]['p4'] == grow - 1,
                        f'{c} 호출 {j}: 재호출 크기 {nxt["ev"][0]["p2"] if nxt["ev"] else None} != C {grow}')
                tot['short'] += 1
                if call['push']:
                    short.append(call['lu'])
                continue
            piv = [x for x in ev[2:] if x['event'] == 'mode70:pivot']
            require(kinds[2:] == ['mode70:pivot'] * len(piv) + ['pool:end'] and call['ret'] == 0, f'{c} 호출 {j}: 모드 순서·반환')
            n2 = e['n']
            require(len(piv) == n2 and all(x['n'] == n2 and 0 <= x['idx'] < n2 * n2 for x in piv), f'{c} 호출 {j}: 모드 피벗 수·번호')
            end = ev[-1]
            require(end['p3'] == e['p3'] + l40 and end['x138'] == e['x138'] + l40, f'{c} 호출 {j}: 모드 배열 칸 {l40}')
            tot['pivots'] += len(piv)
        require(mode == EXPECT_MODE70.get(c, {}), f'{c}: local_70 push 재분해 {mode} != 계약 {EXPECT_MODE70.get(c, {})}')
        require(short == EXPECT_SHORT.get(c, []), f'{c}: 풀 부족 push 재분해 {short} != 계약 {EXPECT_SHORT.get(c, [])}')
    if verbose:
        print('풀·모드 사건', tot)
    return tot


def checked():
    """controls·acceptance를 통과한 모델별 run 1 캡처."""
    caps, ctl, base = load_all()
    controls(caps, ctl, base)
    acceptance(caps, verbose=False)
    return {c: caps[(c, 1)] for c in MODELS}


def self_test(caps, ctl, base):
    def first_of(cap, kind, k=0):
        return [e for e in cap['events'] if e['event'] == kind][k]

    muts = []

    def mut(name, fn):
        muts.append((name, fn))
    allr = lambda c, m: [c[(m, i)] for i in (1, 2, 3)]
    mut('바이너리 해시', lambda c, t: c[('grow7', 2)].update(binary_sha256='0' * 64))
    mut('빈 사건', lambda c, t: c[('perold', 1)].update(events=[]))
    mut('회차 풀 값', lambda c, t: first_of(c[('stair', 3)], 'pool:start').update(c0=first_of(c[('stair', 3)], 'pool:start')['c0'] + 1))
    mut('회차 모드 피벗', lambda c, t: first_of(c[('grow22', 2)], 'mode70:pivot').update(idx=0))
    mut('비간섭 lu:leave', lambda c, t: [first_of(x, 'lu:leave')['factor']['diag'].__setitem__(0, '00' * 8) for x in allr(c, 'pilot4')])
    mut('비간섭 대조군 사건 누락', lambda c, t: t['capri']['events'].remove(next(e for e in t['capri']['events'] if not e['event'].startswith(NEW))))
    mut('대조군 결과', lambda c, t: t['scsd1']['result'].update(status=3))
    mut('대조군 모드 사건', lambda c, t: t['arms']['events'].append({'event': 'mode70:pivot'}))
    mut('start 노드 수(전 회차)', lambda c, t: [first_of(x, 'pool:start').update(nodes=first_of(x, 'pool:start')['nodes'] + 1) for x in allr(c, 'grow7')])
    mut('short 삭제(전 회차)', lambda c, t: [x['events'].remove(first_of(x, 'pool:short70')) for x in allr(c, 'grow22')])
    mut('재호출 크기(전 회차)', lambda c, t: [e.update(p2=e['p2'] + 1) for x in allr(c, 'grow22') for e in x['events'] if e['event'] == 'pool:start' and e['p2'] > 200000][:1])
    mut('모드 피벗 누락(전 회차)', lambda c, t: [x['events'].remove(first_of(x, 'mode70:pivot', 7)) for x in allr(c, 'grow22')])
    mut('모드 여유 값(전 회차)', lambda c, t: [first_of(x, 'pool:mode70', 1).update(b8=first_of(x, 'pool:mode70', 1)['b8'] + 20000) for x in allr(c, 'grow22')])
    mut('빈 열(전 회차)', lambda c, t: [first_of(x, 'pool:mode70').update(ef0=1) for x in allr(c, 'grow22')])
    ok = 0
    for name, fn in muts:
        c2, t2 = copy.deepcopy(caps), copy.deepcopy(ctl)
        fn(c2, t2)
        try:
            controls(c2, t2, base, manifest=False)
            acceptance(c2, verbose=False)
        except (Fail, IndexError, KeyError) as e:
            ok += 1
            print(f'  거부: {name}: {str(e)[:120]}')
            continue
        raise Fail(f'변조 통과: {name}')
    print(f'변조 {ok}종 모두 거부')


def write_manifest():
    caps = sorted(str(p.relative_to(ROOT)) for p in D.glob('**/*.json.gz') if 'history' not in p.parts)
    m = {'task': 'M3-04c-21j', 'schema': SCHEMA, 'binary_sha256': BINARY,
         'settings': {'Threads': 1, 'Seed': 0, 'Presolve': 0, 'Method': 2, 'Crossover': 3, 'LPWarmStart': 2},
         'models': {c: load(path_of(D, c, 1))['model_sha256'] for c in MODELS},
         'tools': {t: sha(ROOT / t) for t in TOOLS}, 'captures': {p: sha(ROOT / p) for p in caps}}
    (D / 'manifest.json').write_text(json.dumps(m, indent=1, ensure_ascii=False) + '\n')
    print('manifest', len(caps), '캡처')


def main():
    a = set(sys.argv[1:])
    if '--write-manifest' in a:
        write_manifest()
    caps, ctl, base = load_all()
    if '--controls' in a:
        controls(caps, ctl, base)
        print('controls 통과: 5모델 × 3회 동일, 5모델·대조군 7사례 21i 투영 일치')
    if '--acceptance' in a:
        acceptance(caps)
    if '--self-test' in a:
        self_test(caps, ctl, base)


if __name__ == '__main__':
    try:
        main()
    except Fail as exc:
        print('FAIL', exc, file=sys.stderr)
        sys.exit(1)
