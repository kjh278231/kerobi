#!/usr/bin/env python3
"""M3-04c-21i: `036ade` 조밀 전환 경계 캡처(bench/oracle/m3_push_dense) 검사기.

controls   : manifest 해시, 5모델 × 3회 사건열 정확 일치(실행 식별자 제외), 대조군 7사례(controls/*-1)의 사건열에서
             dense 사건을 뺀 것 = 21h 정식 `*-1`에 같은 거르기(push 재분해 경계만)를 한 것, 결과·출력·파라미터 같음.
acceptance : 조밀 사건 순서(036ade 한 호출 안 dense:enter → dense:fill → dense:lu → dense:done), push 재분해 표,
             계약 §4의 조밀 수·n, 범위 전제(자체 LU, 첫 시도, scratch 정렬 0, ZERO = 1e-13, 결함 0, n < 1001).
--self-test: 변조(바이너리 해시·빈 사건·회차 차이·비간섭 투영·행렬 1비트·목록 교환·n·순서)가 모두 거부되는지 확인.
--write-manifest: 도구·캡처 해시를 manifest.json에 쓴다.
"""
import copy, gzip, hashlib, json, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
D = ROOT / 'bench/oracle/m3_push_dense'
BASE = ROOT / 'bench/oracle/m3_push_block'
MODELS = ('grow7', 'grow22', 'perold', 'pilot4', 'stair')
CONTROLS = ('arms', 'retry', 'interior', 'control', 'adlittle', 'scsd1', 'capri')
BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
SCHEMA = 'M3-04c-21i-v1'
TOOLS = ('tools/push_dense_agent.js', 'tools/push_dense_capture.js', 'tools/push_dense_runs.py', 'tools/push_lu_runs.py',
         'tools/push_block_agent.js', 'tools/frida_resolve.cjs', 'tools/gurun.sh', 'tools/push_dense_check.py',
         'tools/push_dense_survey.py', 'tools/push_dense_survey_agent.js', 'tools/push_dense_survey_capture.js')
# 계약 §4: push 재분해 조밀(n) — 조사 1회 값. 정식 캡처가 다르면 실패로 드러난다.
EXPECT_PUSH = {'grow7': [51], 'grow22': [54, 51], 'perold': [54, 63], 'pilot4': [58, 56, 66], 'stair': [73, 84]}
KEEP = {'push:enter', 'push:leave', 'factor:enter', 'lu:enter', 'lu:leave', 'lu:refactor', 'lu:block'}
VOLATILE = ('pid', 'run_id')


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
    base = {c: load(path_of(BASE, c, 1)) for c in CONTROLS}
    return caps, ctl, base


def keep(events):
    """push_dense_agent.js의 keep(): push 재분해 경계 사건만, 재분해 lu:enter(request≠0)와 그 lu:leave만."""
    out, ref = [], None
    for e in events:
        k = e['event']
        if k not in KEEP:
            continue
        if k == 'lu:enter':
            if e['request'] == 0:
                continue
            ref = e['id']
        if k == 'lu:leave':
            if e['id'] != ref:
                continue
            ref = None
        out.append(e)
    return out


def first_diff(a, b):
    return next((j for j, (x, y) in enumerate(zip(a, b)) if x != y), min(len(a), len(b)))


def head_ok(cap, tag):
    require(cap['binary_sha256'] == BINARY and cap['schema'] == SCHEMA and not cap['errors']
            and cap['reason'] == 'process-terminated' and cap['result'] is not None, f'{tag}: 바이너리·스키마·오류·종료 사유')


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
            require(any(e['event'] == 'dense:lu' for e in cap['events']), f'{c}-{i}: 조밀 사건 없음')
            body = {k: v for k, v in cap.items() if k not in VOLATILE}
            if ref is None:
                ref = body
            elif body != ref:
                if body['events'] != ref['events']:
                    k = first_diff(body['events'], ref['events'])
                    raise Fail(f'{c}-{i}: 회차 차이 사건 #{k}')
                raise Fail(f'{c}-{i}: 회차 차이 ' + str(sorted(k for k in body if body[k] != ref[k])))
    for c in CONTROLS:
        cap, b = ctl[c], base[c]
        head_ok(cap, f'controls/{c}')
        proj = [e for e in cap['events'] if not e['event'].startswith('dense:')]
        require(len(proj) == len(cap['events']), f'controls/{c}: 대조군에 조밀 사건')
        want = keep(b['events'])
        if proj != want:
            raise Fail(f'controls/{c}: 21h 투영 차이 사건 #{first_diff(proj, want)}')
        for f in ('result', 'output', 'params', 'starts', 'model_sha256', 'kind'):
            require(cap[f] == b[f], f'controls/{c}: 21h {f} 차이')


def dense_table(cap, name):
    """036ade 호출별 조밀 사건 묶음(push 여부·lu id·n·enter/fill/lu/done)과 push 재분해 lu id 목록."""
    ev = cap['events']
    groups, cur = [], None
    for i, e in enumerate(ev):
        k = e['event']
        if k == 'dense:enter':
            require(cur is None, f'{name}#{i}: 앞 조밀 묶음이 끝나지 않음')
            cur = {'enter': e, 'seq': [k]}
        elif k.startswith('dense:'):
            require(cur is not None, f'{name}#{i}: dense:enter 없이 {k}')
            require(all(e[f] == cur['enter'][f] for f in ('call', 'step', 'lu', 'push')), f'{name}#{i}: 조밀 문맥 차이')
            cur['seq'].append(k)
            cur[k.split(':')[1]] = e
            if k == 'dense:done':
                groups.append(cur)
                cur = None
    require(cur is None, f'{name}: 끝나지 않은 조밀 묶음')
    return groups


def acceptance(caps, verbose=True):
    total = {'push': 0, 'other': 0}
    for c in MODELS:
        cap = caps[(c, 1)]
        groups = dense_table(cap, c)
        push_n = []
        for g in groups:
            e, f, lu, dn = g['enter'], g.get('fill'), g.get('lu'), g.get('done')
            require(g['seq'] == ['dense:enter', 'dense:fill', 'dense:lu', 'dense:done'], f'{c}: 조밀 순서 {g["seq"]}')
            n = e['n']
            require(f['n'] == n and lu['n'] == n and dn['n'] == n, f'{c}: 조밀 n 차이')
            require(f['mode70'] == 0, f'{c}: local_70 모드 조밀(범위 밖)')
            require(lu['first'] == 1 and n < 1001, f'{c}: 첫 시도·자체 LU 전제')
            require(lu['scratch_mod16'] == 0 and lu['mat_mod16'] in (0, 8), f'{c}: 정렬 {lu["mat_mod16"]}/{lu["scratch_mod16"]}')
            require(lu['zero'] == '82764968c2253c3d', f'{c}: ZERO {lu["zero"]}')
            require(dn['ecx'] == 0, f'{c}: 결함 {dn["ecx"]}(범위 밖)')
            require(len(lu['a']) == n * n and len(dn['a']) == n * n, f'{c}: 행렬 크기')
            require(sorted(lu['cols']) == lu['cols'], f'{c}: 열 목록이 오름차순이 아님')
            require(sorted(dn['cols']) == lu['cols'] and dn['rows'] == lu['rows'], f'{c}: 목록 보존')
            require(e['steps'] + n == cap['events'][0]['state']['m'], f'{c}: 단계 수 + n != m')
            if e['push']:
                push_n.append(n)
                total['push'] += 1
            else:
                total['other'] += 1
        require(push_n == EXPECT_PUSH[c], f'{c}: push 조밀 n {push_n} != 계약 {EXPECT_PUSH[c]}')
    if verbose:
        print('조밀 LU', total, {c: [g['enter']['n'] for g in dense_table(caps[(c, 1)], c)] for c in MODELS})
    return total


def checked():
    """controls·acceptance를 통과한 모델별 run 1 캡처."""
    caps, ctl, base = load_all()
    controls(caps, ctl, base)
    acceptance(caps, verbose=False)
    return {c: caps[(c, 1)] for c in MODELS}


def self_test(caps, ctl, base):
    def bit(h):
        b = bytearray(bytes.fromhex(h)); b[0] ^= 1
        return b.hex()

    def first_of(cap, kind):
        return next(e for e in cap['events'] if e['event'] == kind)

    muts = []
    def mut(name, fn, which='caps'):
        muts.append((name, fn, which))
    mut('바이너리 해시', lambda c, t, b: c[('grow7', 2)].update(binary_sha256='0' * 64))
    mut('빈 사건', lambda c, t, b: c[('perold', 1)].update(events=[]))
    mut('회차 행렬 1비트', lambda c, t, b: first_of(c[('stair', 3)], 'dense:lu')['a'].__setitem__(5, bit(first_of(c[('stair', 3)], 'dense:lu')['a'][5])))
    mut('회차 done 1비트', lambda c, t, b: first_of(c[('pilot4', 2)], 'dense:done')['a'].__setitem__(0, bit(first_of(c[('pilot4', 2)], 'dense:done')['a'][0])))
    mut('회차 열 목록 교환', lambda c, t, b: (lambda l: l.__setitem__(slice(0, 2), l[1::-1]))(first_of(c[('grow22', 2)], 'dense:done')['cols']))
    mut('회차 n', lambda c, t, b: first_of(c[('grow7', 3)], 'dense:enter').update(n=52))
    mut('회차 lu:leave factor', lambda c, t, b: first_of(c[('perold', 2)], 'lu:leave')['factor']['diag'].__setitem__(0, bit(first_of(c[('perold', 2)], 'lu:leave')['factor']['diag'][0])))
    mut('대조군 사건 누락', lambda c, t, b: t['capri']['events'].pop(5))
    mut('대조군 factor 1비트', lambda c, t, b: first_of(t['adlittle'], 'lu:leave')['factor']['diag'].__setitem__(1, bit(first_of(t['adlittle'], 'lu:leave')['factor']['diag'][1])))
    mut('대조군 조밀 사건', lambda c, t, b: t['arms']['events'].append({'event': 'dense:enter'}))
    mut('대조군 결과', lambda c, t, b: t['scsd1']['result'].update(status=3))
    mut('계약 n(전 회차)', lambda c, t, b: [first_of(c[('stair', i)], 'dense:enter').update(n=74) for i in (1, 2, 3)])
    mut('조밀 순서', lambda c, t, b: [cc['events'].remove(first_of(cc, 'dense:fill')) for cc in (c[('grow7', i)] for i in (1, 2, 3))])
    mut('결함 ecx', lambda c, t, b: [first_of(c[('perold', i)], 'dense:done').update(ecx=1) for i in (1, 2, 3)])
    mut('열 목록 정렬 깨짐', lambda c, t, b: [(lambda l: l.__setitem__(slice(0, 2), l[1::-1]))(first_of(c[('pilot4', i)], 'dense:lu')['cols']) for i in (1, 2, 3)])
    mut('scratch 정렬', lambda c, t, b: [first_of(c[('grow22', i)], 'dense:lu').update(scratch_mod16=8) for i in (1, 2, 3)])
    ok = 0
    for name, fn, _ in muts:
        c2, t2, b2 = copy.deepcopy(caps), copy.deepcopy(ctl), base
        fn(c2, t2, b2)
        try:
            controls(c2, t2, b2, manifest=False)
            acceptance(c2, verbose=False)
        except Fail as e:
            ok += 1
            print(f'  거부: {name}: {e}')
            continue
        raise Fail(f'변조 통과: {name}')
    print(f'변조 {ok}종 모두 거부')


def write_manifest():
    caps = sorted(str(p.relative_to(ROOT)) for p in D.rglob('*.json.gz'))
    m = {'task': 'M3-04c-21i', 'schema': SCHEMA, 'binary_sha256': BINARY,
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
        print('controls 통과: 5모델 × 3회 동일, 대조군 7사례 21h 투영 일치')
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
