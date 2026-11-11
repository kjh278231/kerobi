"""M3-04c: 원본 feasible ratio 캡처를 Rust 입력으로 투영하고, Rust trace를 원본 출력과 비교한다."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import struct
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
import primal_push_ratio_oracle as synth  # noqa: E402  M3-04b 합성 29사례
import primal_push_step_check as step  # noqa: E402  M3-04b-1 자연 캡처

ROOT = Path(__file__).resolve().parents[1]
STEP = ROOT / 'bench/oracle/m3_primal_push_step'
SYN = ROOT / 'bench/oracle/m3_primal_push'
INPUT = ROOT / 'bench/oracle/m3_push_ratio_rust/input.txt'
TRACE = ROOT / 'target/m3_push_ratio_rust/trace.txt'
NAN = struct.pack('<d', float('nan')).hex()
FIELDS = ('pos', 'row', 'theta', 'pivot', 'work')


def bits(v):
    return struct.pack('<d', v).hex()


def natural():
    cases = []
    for kind in ('primal', 'dual'):
        paths = [STEP / f'{kind}-{i}.json' for i in (1, 2, 3)]
        step.check(paths)  # 해시·출처·독립 3회 일치
        events = json.loads(paths[0].read_text())['events']
        enter = [e for e in events if e['event'] == 'push:enter']
        assert len(enter) == 1, 'push 1회 전제'
        tol = enter[0]['state']['tolerances']
        for i, e in enumerate(events):
            if e['event'] != 'ratio:enter':
                continue
            out = events[i + 1]
            assert out['event'] == 'ratio:leave' and (out['call'], out['step']) == (e['call'], e['step'])
            assert e['name'] == out['name'] == 'feasible'
            # xmm0..3은 진입 때 저장한 S+0x380·0x398·0x370 슬롯이다(계약 §2). 값이 바뀌었으면 재구성 불가.
            assert e['state']['tolerances'] == tol, 'ratio 중 허용오차 변경'
            a, m = e['alpha'], e['state']['m']
            cases.append(dict(name=f"{kind}-c{e['call']}s{e['step']}", dense=a['count'] < 0, m=m,
                              scal=[tol['380'], tol['398'], tol['370'], tol['370'], NAN, NAN, NAN],
                              idx=a['indices'], val=a['values'], x=e['basic'], width=e['width'], marks=e['marks'],
                              expect=dict(pos=out['chosen'], row=out['row'], theta=out['theta'],
                                          pivot=out['pivot'], work=None)))
    return cases


def synthetic():
    record = synth.check([SYN / f'ratio-{i}.json' for i in (1, 2, 3)])
    cases = []
    for c, e in zip(synth.cases(), record['events'], strict=True):
        assert c['name'] == e['name']
        dense = c['dense']
        # 인자는 primal_push_ratio_oracle.capture()의 호출과 같다.
        cases.append(dict(name='synthetic-' + c['name'], dense=dense, m=len(c['x']),
                          scal=[bits(1e-6), bits(1e-9), bits(1e30), bits(c['cap']), bits(0.25), bits(0.0),
                                bits(-123.0)],
                          idx=None if dense else c['order'],
                          val=[bits(v) for v in (c['alpha'] if dense else [c['alpha'][i] for i in c['order']])],
                          x=[bits(v) for v in c['x']], width=[bits(v) for v in c['width']],
                          marks=[ch.encode().hex() for ch in c['marks']],
                          expect=dict(pos=e['chosen'], row=e['row'], theta=e['theta'], pivot=e['pivot'],
                                      work=e['work'])))
    return cases


def line(c):
    parts = [c['name'], '1' if c['dense'] else '0', str(c['m']), *c['scal']]
    if not c['dense']:
        parts += [str(len(c['idx'])), *map(str, c['idx'])]
    return ' '.join(parts + c['val'] + c['x'] + c['width'] + c['marks'])


def render(cases):
    body = ''.join(line(c) + '\n' for c in cases)
    return f"input {hashlib.sha256(body.encode()).hexdigest()}\n" + body


def compare(cases, text, trace):
    assert text == render(cases), '입력 텍스트가 원본 캡처 투영과 다름(--write 필요)'
    rows = trace.splitlines()
    assert rows and rows[0] == text.splitlines()[0], 'trace의 입력 해시 불일치(오래된 trace)'
    assert len(rows) == len(cases) + 1, ('사례 수', len(cases), len(rows) - 1)
    for c, row in zip(cases, rows[1:], strict=True):
        t = row.split()
        assert t[0] == c['name'], ('사례 순서', c['name'], t[0])
        # 원본은 모든 사례에서 반환했으므로 Rust의 재진입 거부는 곧 불일치다.
        assert t[1:] != ['reentry'], (c['name'], 'Rust 재진입 거부', '원본은 반환함')
        assert len(t) == 5, (c['name'], '필드 수', t)
        pos = int(t[1])
        got = dict(pos=pos, row=pos if pos < 0 or c['dense'] else c['idx'][pos], theta=t[2], pivot=t[3], work=t[4])
        for f in FIELDS:
            want = c['expect'][f]
            if want is not None:
                assert got[f] == want, (c['name'], f, '기준', want, '실제', got[f], '정확·비트 비교')
    nat = [c for c in cases if not c['name'].startswith('synthetic-')]
    unique = len({line(dict(c, name='')) for c in nat})
    return dict(natural=len(nat), natural_unique_inputs=unique, synthetic=len(cases) - len(nat),
                fields_natural=4, fields_synthetic=5)


def self_test(cases, text, trace):
    compare(cases, text, trace)
    rows = trace.splitlines()
    syn = next(i for i, c in enumerate(cases, 1) if c['name'].startswith('synthetic-'))

    def edit(i, k, v):
        t = rows[i].split()
        t[k] = v
        return '\n'.join(rows[:i] + [' '.join(t)] + rows[i + 1:]) + '\n'
    bad = {
        'theta': edit(1, 2, bits(3.0)), 'pivot': edit(1, 3, bits(0.0)), 'pos': edit(1, 1, '0'),
        'synthetic-work': edit(syn, 4, bits(1.0)), 'synthetic-theta-sign': edit(syn, 2, bits(-2.0)),
        'missing-case': '\n'.join(rows[:-1]) + '\n', 'reentry': '\n'.join(rows[:2] + [rows[2].split()[0] + ' reentry']
                                                                          + rows[3:]) + '\n',
        'stale-hash': '\n'.join(['input ' + '0' * 64] + rows[1:]) + '\n',
    }
    rejected = []
    for name, t in bad.items():
        try:
            compare(cases, text, t)
        except AssertionError:
            rejected.append(name)
            continue
        raise AssertionError(('변조 허용', name))
    changed = copy.deepcopy(cases)
    changed[0]['x'][0] = bits(1.0)
    try:
        compare(changed, text, trace)
    except AssertionError:
        rejected.append('input-drift')
    else:
        raise AssertionError(('변조 허용', 'input-drift'))
    return rejected


# Rust 분기별 오류 주입. 각 변조는 원본 대조에서 실패해야 한다(최종 코드에는 남기지 않는다).
RUST = ROOT / 'crates/lp/src/crossover.rs'
MUTANTS = {  # M3-04c-5에서 feasible_ratio 재진입 구현에 맞춰 대상 문자열만 바꿨다(의미는 04c와 같음)
    'accept-strict': ('if ratio <= bound {', 'if ratio < bound {'),
    'harris-sign': ('{ (num + harris) / a }', '{ (num - harris) / a }'),
    'second-tie': ('.zip(&cands) {\n            if ratio <= bound && best < val[k].abs()',
                   '.zip(&cands) {\n            if ratio <= bound && best <= val[k].abs()'),
    'second-bound': ('.zip(&cands) {\n            if ratio <= bound && best', '.zip(&cands) {\n            if ratio < bound && best'),
    'width-inf': ('-tol > a && inf > width[r]', '-tol > a && inf >= width[r]'),
    'free-mark': ("let up = a > tol && marks[r] != b'F';", 'let up = a > tol;'),
    'lower-num': ('x[r] - width[r]', 'width[r] - x[r]'),
    'cap-return': ('if cap < inf && cap <= bound {\n            *pivot = 0.0;\n            *theta = cap;\n            return -1;',
                   'if cap < inf && cap < bound {\n            *pivot = 0.0;\n            *theta = cap;\n            return -1;'),
    'no-candidate': ('if bound == inf || cands.is_empty() {\n            if first', 'if cands.is_empty() {\n            if first'),
    'pivot-threshold': ('let small = first && 0.1 > *pivot;', 'let small = first && 1e-12 > *pivot;'),
    'small-scale': ('b if 10.0 > b => 0.1,', 'b if 10.0 > b => 1e-12,'),
    'small-check-strict': ('let reenter = tol * big * scale > *pivot;', 'let reenter = tol * big * scale * 1e-12 > *pivot;'),
    'work-dense': ('tested as f64 + tested as f64 + count', 'tested as f64 + count'),
    'work-sparse': ('count as f64 * 4.0', 'count as f64 * 3.0'),
}


def mutate_rust(cases, text):
    import subprocess
    original, result = RUST.read_text(), {}
    try:
        for name, (old, new) in MUTANTS.items():
            assert original.count(old) == 1, ('변조 대상 없음', name)
            RUST.write_text(original.replace(old, new))
            TRACE.unlink(missing_ok=True)
            run = subprocess.run(['cargo', 'test', '-q', '-p', 'rsolver-lp', '--test', 'push_ratio'], cwd=ROOT,
                                 capture_output=True, text=True)
            try:
                assert run.returncode == 0, 'Rust 테스트 실패'
                compare(cases, text, TRACE.read_text())
                result[name] = '생존'
            except AssertionError as error:
                result[name] = '거부: ' + str(error)[:160]
    finally:
        RUST.write_text(original)
        TRACE.unlink(missing_ok=True)
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
        cases = natural() + synthetic()
        if args.write:
            INPUT.parent.mkdir(parents=True, exist_ok=True)
            INPUT.write_text(render(cases))
            print(f'{len(cases)}사례 저장: {INPUT.relative_to(ROOT)}')
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
