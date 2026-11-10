"""M3-04c-6: push 단계 판단 원본 캡처를 Rust 입력으로 투영하고, Rust trace를 원본 캡처 필드와 비교한다.

Rust에는 원시 캡처 필드만 넘긴다. 판단 모델(primal_push_decision_check.decide)의 계산값은 쓰지 않는다.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
import primal_push_decision_check as dc  # noqa: E402  M3-04b-2 캡처 검사·로더

ROOT = Path(__file__).resolve().parents[1]
INPUT = ROOT / 'bench/oracle/m3_push_decision_rust/input.txt'
TRACE = ROOT / 'target/m3_push_decision_rust/trace.txt'
RUST = ROOT / 'crates/lp/src/crossover.rs'
FRAME = ['r', 'dL', 'dU', 'e', 'k100', 'km100', 'k2']  # trace 열 2..8 순서
KIND = {'Flip': 'flip', 'Reject': 'flip', 'Pivot': 'pivot', 'Defer': 'defer'}
# 대상 40단계(04b-1 9+9, face5 11+11)와 보조 9단계(비간섭 단일 실행)의 기대 집계
EXPECT = {'target': dict(steps=40, dir1=6, pivot=36, flip=4, defer=0, frame=22),
          'support': dict(steps=9, dir1=0, pivot=7, flip=2, defer=0, frame=9)}


def project(label, rec, role):
    """한 캡처의 단계별 Rust 입력(원시 필드)과 원본 기대 출력."""
    events, model, cases = rec['events'], None, []
    i = 0
    while i < len(events):
        e = events[i]
        if e['event'] == 'push:enter':
            model = e['model']
        if e['event'] != 'supply:leave' or e['candidate'] < 0:
            i += 1
            continue
        k, ratios = i + 1, []
        while k < len(events) and events[k]['event'] not in ('supply:enter', 'push:leave', 'step:enter'):
            if events[k]['event'] == 'ratio:enter':
                ratios.append((events[k], events[k + 1]))
            k += 1
        name = f"{label}/c{e['call']}s{e['step']}"
        assert len(ratios) == 1, (name, 'ratio 호출 수(재호출은 범위 밖)', len(ratios))
        rin, rout = ratios[0]
        assert rin['name'] == 'feasible' and rout['event'] == 'ratio:leave', (name, '경로')
        pre, post, st = e['alpha'], rin['alpha'], rin['state']
        assert pre['count'] >= 0 and pre['indices'] == post['indices'], (name, '희소 열·인덱스 보존')
        tol = st['tolerances']
        assert rout['chosen'] < 0 or rout['row'] == pre['indices'][rout['chosen']], (name, 'ratio 행')
        nxt = events[k] if k < len(events) else None
        se = nxt if nxt is not None and nxt['event'] == 'step:enter' else None
        want = dict(dir=rin['selected']['direction'], alpha=post['values'], kind=se['kind'] if se else 'defer',
                    theta=se and se['theta'], frame=se and se.get('frame'), state=None,
                    link=[str(rout['chosen']), rout['theta'], rout['pivot']])
        if se and se['kind'] == 'pivot':
            applied = events[k + 1]
            assert applied['event'] == 'step:applied', (name, '적용 사건')
            want['state'] = applied['state']['vstat'][st['header'][rout['row']]]
        fields = [name, str(st['n']), str(st['m']), str(e['candidate']), tol['380'], tol['398'], tol['370'],
                  tol['3b0'], str(rout['chosen']), rout['theta'], rout['pivot'], *model['cost'], *model['lb'],
                  *model['ub'], *st['x'], *st['slack'], *map(str, st['header']), str(pre['count']),
                  *map(str, pre['indices']), *pre['values'], *rin['basic'], *rin['width'], *rin['marks']]
        cases.append(dict(name=name, role=role, line=' '.join(fields), want=want))
        i = k
    return cases


def collect():
    manifest = json.loads((dc.BASE / 'manifest.json').read_text())
    cases = []
    for label, rec in dc.load_all(manifest):  # 04b-1·face5: 해시·출처·순서별 독립 3회 검사 후 첫 실행
        cases += project(label, rec, 'target')
    # 보조: 04b-2 비간섭 단일 실행. frame을 빼면 04b-1 primal-1과 같아야 한다.
    c = manifest['cases']['interior-noninterference']
    new = json.loads((ROOT / c['captures'][0]).read_text())
    old = json.loads((dc.OLD / 'primal-1.json').read_text())
    assert new['agent_sha256'] == manifest['tools'][dc.TOOLS[1]] == dc.sha(ROOT / dc.TOOLS[1]), '보조 agent'
    assert new['binary_sha256'] == dc.BINARY_SHA and new['model_sha256'] == old['model_sha256'], '보조 출처'
    assert dc.strip(new['events']) == old['events'] and new['params'] == old['params'], '보조 사건이 04b-1과 다름'
    return cases + project('interior-frame', new, 'support')


def render(cases):
    body = ''.join(c['line'] + '\n' for c in cases)
    return f"input {hashlib.sha256(body.encode()).hexdigest()}\n" + body


def compare(cases, text, trace):
    assert text == render(cases), '입력 텍스트가 원본 캡처 투영과 다름(--write 필요)'
    rows = trace.splitlines()
    assert rows and rows[0] == text.splitlines()[0], 'trace의 입력 해시 불일치(오래된 trace)'
    assert len(rows) == len(cases) + 1, ('사례 수', len(cases), len(rows) - 1)
    tally = {role: dict(steps=0, dir1=0, pivot=0, flip=0, defer=0, frame=0) for role in EXPECT}
    link_bad = []
    for c, row in zip(cases, rows[1:], strict=True):
        t, w, name = row.split(), c['want'], c['name']
        assert t[0] == name, ('사례 순서', name, t[0])

        def need(field, want, got):
            assert want == got, (name, field, '기준', want, '실제', got, '정확·비트 비교')
        need('방향', w['dir'], int(t[1]))
        assert t[9] != 'recall', (name, 'Rust 재호출 거부', '원본은 판단함')
        kind = KIND[t[9]]
        need('단계 종류', w['kind'], kind)
        if kind != 'defer':
            need('theta', w['theta'], t[10])
        if w['frame'] is not None:
            for f, v in zip(FRAME, t[2:9]):
                need('frame ' + f, w['frame'][f], v)
        if w['state'] is not None:
            need('이탈 상태', w['state'], int(t[11]))
        else:
            need('이탈 상태 없음', 'x', t[11])
        need('반전 뒤 alpha', w['alpha'], t[15:])
        if t[12:15] != w['link']:
            link_bad.append((name, '기준', w['link'], '실제', t[12:15]))
        s = tally[c['role']]
        s['steps'] += 1
        s['dir1'] += w['dir']
        s[kind] += 1
        s['frame'] += w['frame'] is not None
    for role, want in EXPECT.items():
        assert tally[role] == want, ('집계', role, '기대', want, '실제', tally[role])
    unique = len({c['line'].split(' ', 1)[1] for c in cases if c['role'] == 'target'})
    return dict(tally=tally, target_unique_inputs=unique, link_cases=len(cases), link_mismatch=link_bad)


def self_test(cases, text, trace):
    compare(cases, text, trace)
    rows = trace.splitlines()
    piv = next(i for i, c in enumerate(cases, 1) if c['want']['state'] is not None and c['want']['frame'] is not None)
    d1 = next(i for i, c in enumerate(cases, 1) if c['want']['dir'] == 1)

    def edit(i, k, v):
        t = rows[i].split()
        t[k] = v
        return '\n'.join(rows[:i] + [' '.join(t)] + rows[i + 1:]) + '\n'
    bad = {
        'direction': edit(d1, 1, '0'), 'r-bits': edit(piv, 2, dc.bits(dc.f64(rows[piv].split()[2]) + 1.0)),
        'du-bits': edit(piv, 4, dc.bits(1.0)), 'k2-bits': edit(piv, 8, dc.bits(1.0)),
        'kind': edit(piv, 9, 'Flip'), 'theta': edit(piv, 10, dc.bits(3.0)), 'leave-state': edit(piv, 11, '-2'),
        'alpha-sign': edit(d1, 15, dc.bits(-dc.f64(rows[d1].split()[15]))), 'recall': edit(1, 9, 'recall'),
        'missing-case': '\n'.join(rows[:-1]) + '\n', 'stale-hash': '\n'.join(['input ' + '0' * 64] + rows[1:]) + '\n',
    }
    rejected = []
    for name, tr in bad.items():
        try:
            compare(cases, text, tr)
        except AssertionError:
            rejected.append(name)
            continue
        raise AssertionError(('변조 허용', name))
    changed = copy.deepcopy(cases)
    changed[0]['line'] += ' 0'
    try:
        compare(changed, text, trace)
    except AssertionError:
        rejected.append('input-drift')
    else:
        raise AssertionError(('변조 허용', 'input-drift'))
    link = compare(cases, text, edit(1, 13, dc.bits(5.0)))['link_mismatch']
    assert len(link) == 1, ('연결 변조 미검출', link)
    rejected.append('link-theta(연결 불일치로 보고)')
    return rejected


# Rust 소스 변조: M3-04b-2 모델 변조 20종의 대응과 Rust 전용 몇 종. 최종 코드에는 남기지 않는다.
MUTANTS = {
    'dir-ten-9': [('10.0 * dl < du', '9.0 * dl < du'), ('dl > 10.0 * du', 'dl > 9.0 * du')],
    'dir-le': [('10.0 * dl < du', '10.0 * dl <= du')],
    'dir-e-0.1h': [('e: h * 0.01', 'e: h * 0.1')],
    'dir-no-neg': [('-(r * dl) < e', 'r * dl < e')],
    'dir-r-ge0': [('&& 0.0 < r)', '&& 0.0 <= r)')],
    'r-reverse-order': [('idx.iter().zip(val.iter())', 'idx.iter().zip(val.iter()).rev()')],
    'flip-no-eps': [('gap - 1e-10 <= *theta', 'gap - 0.0 <= *theta')],
    'flip-lt': [('gap - 1e-10 <= *theta', 'gap - 1e-10 < *theta')],
    'reject-10h': [('reject: 100.0 * h', 'reject: 10.0 * h'), ('reject_neg: -100.0 * h', 'reject_neg: -10.0 * h')],
    'reject-1e4h': [('reject: 100.0 * h', 'reject: 1e4 * h'), ('reject_neg: -100.0 * h', 'reject_neg: -1e4 * h')],
    'reject-arm-swap': [('let reject = if entry.decrease {', 'let reject = if !entry.decrease {')],
    'accept-pivot-1e-3': [('1e-6 > pivot', '1e-3 > pivot')],
    'accept-prod-0': [('-1e-5 > *theta * pivot', '0.0 > *theta * pivot')],
    'leave-k-h': [('leave: h + h', 'leave: h')],
    'leave-fixed-lt': [('if range <= fixed', 'if range < fixed')],
    'leave-L-nonneg-2': [("b'L' | b'N' => -1,", "b'N' => -1,\n            b'L' => -2,")],
    'leave-neg-default-2': [("{ -1 } else { -2 },\n            _ => -1,", "{ -1 } else { -2 },\n            _ => -2,")],
    'leave-X-1': [("b'X' => -4,\n            b'L'", "b'X' => -1,\n            b'L'"),
                  ("b'X' => -4,\n            b'A'", "b'X' => -1,\n            b'A'")],
    'leave-U-le': [("b'U' if k > a.abs() * t", "b'U' if k >= a.abs() * t")],
    'alpha-no-flip': [('val.iter_mut().for_each(|v| *v = -*v);', '')],
    'slack-gap-swap': [('(slack[j - n], inf)', '(inf, slack[j - n])')],
    'sign-zero-neg': [('if 0.0 > a {', 'if 0.0 >= a {')],
    'reject-keep-theta': [('*theta = inf;\n        return Ok(PushStep::Reject);', 'return Ok(PushStep::Reject);')],
    'recall-no-theta': [(' || *theta == inf', '')],
}
NOT_PORTED = {'reject-arm-current': 'Rust 방향은 방향식 결과 하나뿐이다. 현재 S+0x328과 다른 경우는 재호출 경로(Recall 거부)에만 있다.'}


def mutate_rust(cases, text):
    original, result = RUST.read_text(), {}
    try:
        for name, edits in MUTANTS.items():
            src = original
            for old, new in edits:
                assert src.count(old) == 1, ('변조 대상 없음', name, old)
                src = src.replace(old, new)
            RUST.write_text(src)
            TRACE.unlink(missing_ok=True)
            run = subprocess.run(['cargo', 'test', '-q', '-p', 'rsolver-lp', '--test', 'push_decision'], cwd=ROOT,
                                 capture_output=True, text=True)
            try:
                assert run.returncode == 0, 'Rust 테스트 실패'
                link = compare(cases, text, TRACE.read_text())['link_mismatch']
                result[name] = '생존' + (f' (연결 불일치 {len(link)})' if link else '')
            except AssertionError as error:
                result[name] = '거부: ' + str(error)[:200]
    finally:
        RUST.write_text(original)
        TRACE.unlink(missing_ok=True)
    result.update({k: '미적용: ' + v for k, v in NOT_PORTED.items()})
    return result


def main():
    p = argparse.ArgumentParser(description=__doc__)
    g = p.add_mutually_exclusive_group(required=True)
    g.add_argument('--write', action='store_true')
    g.add_argument('--compare', action='store_true')
    g.add_argument('--self-test', action='store_true')
    g.add_argument('--mutate-rust', type=Path, help='변조 결과 JSON 경로(덮어쓰기 거부)')
    args = p.parse_args()
    try:
        cases = collect()
        if args.write:
            INPUT.parent.mkdir(parents=True, exist_ok=True)
            INPUT.write_text(render(cases))
            print(f'{len(cases)}단계 저장: {INPUT.relative_to(ROOT)}')
            return
        text = INPUT.read_text()
        if args.mutate_rust:
            assert not args.mutate_rust.exists(), '검증 기록 덮어쓰기 거부'
            result = mutate_rust(cases, text)
            args.mutate_rust.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
            print(json.dumps(result, ensure_ascii=False, indent=1))
        elif args.compare:
            print(json.dumps(compare(cases, text, TRACE.read_text()), ensure_ascii=False))
        else:
            print(json.dumps({'rejected': self_test(cases, text, TRACE.read_text())}, ensure_ascii=False))
    except (AssertionError, KeyError, ValueError, IndexError) as error:
        print('불합격: ' + str(error), file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
