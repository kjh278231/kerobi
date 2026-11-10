"""M3-04c-2: 054eef 원본 합성 직접 호출 캡처를 Rust infeasible_ratio 입력으로 투영하고 trace를 원본과 비교한다.

입력 줄 형식은 primal_push_ratio_rust.py(M3-04c)와 같고 Rust 테스트 push_ratio::infeasible_ratio_trace가 읽는다.
자연 위반 basis 단계의 ratio 연결 검사는 primal_push_decision_rust.py가 한다.
"""
import argparse
import json
from pathlib import Path
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
import primal_push_ratio_rust as base  # noqa: E402  M3-04c 입력 줄 형식
import push_infeasible_ratio_oracle as synth  # noqa: E402  054eef 합성 사례

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / 'bench/oracle/m3_push_violated'
INPUT = DATA / 'ratio_input.txt'
TRACE = ROOT / 'target/m3_push_violated/trace.txt'
RUST = ROOT / 'crates/lp/src/crossover.rs'
REFUSED = {'reentry-sparse', 'reentry-dense'}  # 작은 pivot 재진입 2차 패스는 Rust가 거부한다(계약 §1)
bits = base.bits


def cases():
    record = synth.check([DATA / f'ratio-{i}.json' for i in (1, 2, 3)])
    out = []
    for c, e in zip(synth.cases(), record['events'], strict=True):
        assert c['name'] == e['name']
        dense = c['dense']
        # 인자는 push_infeasible_ratio_oracle.capture()의 호출과 같다.
        out.append(dict(name=c['name'], dense=dense, m=len(c['x']),
                        scal=[bits(synth.H), bits(synth.PIV), bits(synth.INF), bits(c['cap']), bits(0.25), bits(0.0),
                              bits(-123.0)],
                        idx=None if dense else c['order'],
                        val=[bits(v) for v in (c['alpha'] if dense else [c['alpha'][i] for i in c['order']])],
                        x=[bits(v) for v in c['x']], width=[bits(v) for v in c['width']],
                        marks=[ch.encode().hex() for ch in c['marks']],
                        expect=dict(pos=e['chosen'], row=e['row'], theta=e['theta'], pivot=e['pivot'],
                                    work=e['work'])))
    return out


def compare(cs, text, trace):
    assert text == base.render(cs), '입력 텍스트가 원본 캡처 투영과 다름(--write 필요)'
    rows = trace.splitlines()
    assert rows and rows[0] == text.splitlines()[0], 'trace의 입력 해시 불일치(오래된 trace)'
    assert len(rows) == len(cs) + 1, ('사례 수', len(cs), len(rows) - 1)
    refused = []
    for c, row in zip(cs, rows[1:], strict=True):
        t = row.split()
        assert t[0] == c['name'], ('사례 순서', c['name'], t[0])
        if c['name'] in REFUSED:
            assert t[1:] == ['reentry'], (c['name'], '재진입 사례는 Rust가 거부해야 함', t[1:])
            refused.append(c['name'])
            continue
        assert t[1:] != ['reentry'], (c['name'], 'Rust 재진입 거부', '원본은 첫 패스에서 반환함')
        assert len(t) == 5, (c['name'], '필드 수', t)
        pos = int(t[1])
        got = dict(pos=pos, row=pos if pos < 0 or c['dense'] else c['idx'][pos], theta=t[2], pivot=t[3], work=t[4])
        for f in base.FIELDS:
            assert got[f] == c['expect'][f], (c['name'], f, '기준', c['expect'][f], '실제', got[f], '정확·비트 비교')
    assert sorted(refused) == sorted(REFUSED), ('거부 사례 누락', refused)
    return dict(cases=len(cs), compared=len(cs) - len(refused), refused=refused, fields=5)


def self_test(cs, text, trace):
    compare(cs, text, trace)
    rows = trace.splitlines()

    def edit(i, k, v):
        t = rows[i].split()
        t[k] = v
        return '\n'.join(rows[:i] + [' '.join(t)] + rows[i + 1:]) + '\n'
    ok = next(i for i, c in enumerate(cs, 1) if c['name'] not in REFUSED and c['expect']['pos'] >= 0)
    ref = next(i for i, c in enumerate(cs, 1) if c['name'] in REFUSED)
    bad = {'theta': edit(ok, 2, bits(3.0)), 'pivot': edit(ok, 3, bits(0.0)), 'pos': edit(ok, 1, '-1'),
           'work': edit(ok, 4, bits(1.0)), 'refused-accepted': edit(ref, 1, '1 x x x'),
           'missing-case': '\n'.join(rows[:-1]) + '\n', 'stale-hash': '\n'.join(['input ' + '0' * 64] + rows[1:]) + '\n'}
    rejected = []
    for name, tr in bad.items():
        try:
            compare(cs, text, tr)
        except AssertionError:
            rejected.append(name)
            continue
        raise AssertionError(('변조 허용', name))
    return rejected


# infeasible_ratio 분기별 Rust 소스 변조. 각 변조는 원본 대조에서 실패해야 한다(최종 코드에는 남기지 않는다).
MUTANTS = {
    'pos-A-num': [("(true, b'A') | (false, b'U' | b'X') => v - w,", "(false, b'U' | b'X') => v - w,\n            (true, b'A') => v,")],
    'pos-B-taken': [("(true, b'N' | b'U' | b'L' | b'X') | (false, b'B') => v,", "(true, b'N' | b'U' | b'L' | b'X' | b'B') | (false, b'B') => v,")],
    'neg-B-num': [("| (false, b'B') => v,", "=> v,\n            (false, b'B') => v - w,")],
    'neg-U-num': [("(true, b'A') | (false, b'U' | b'X') => v - w,", "(true, b'A') | (false, b'X') => v - w,\n            (false, b'U') => v,")],
    'neg-LN-inf': [("(false, b'L' | b'N') if inf > w => v - w,", "(false, b'L' | b'N') => v - w,")],
    'neg-A-taken': [("(false, b'L' | b'N') if inf > w => v - w,", "(false, b'L' | b'N' | b'A') if inf > w => v - w,")],
    'dense-strict-off': [('let strict = idx.is_none() && !up', 'let strict = false && !up')],
    'strict-everywhere': [('let strict = idx.is_none() && !up', 'let strict = !up')],
    'harris-sign': [('(if up { num + harris } else { num - harris }) / a', '(num + harris) / a')],
    'tested-skip': [('        tested += 1;\n        let (v, w)', '        let (v, w)')],
    'work-dense': [('c + c + t + t + n', 'c + c + t + n')],
    'work-sparse': [('c + c + n * 4.0', 'c + n * 4.0')],
    'tol-ge': [('down) = (a > piv_tol, a < -piv_tol);', 'down) = (a > piv_tol, a <= -piv_tol);')],
    # 공유 꼬리(ratio_tail)를 위반 합성 사례로도 판별하는지: 04c MUTANTS와 같은 대상
    'tail-second-tie': [('best < val[k].abs()', 'best <= val[k].abs()')],
    'tail-second-bound': [('if ratio <= bound && best', 'if ratio < bound && best')],
    'tail-cap-return': [('cap < inf && cap <= bound', 'cap < inf && cap < bound')],
    'tail-no-candidate': [('if bound == inf || cands.is_empty()', 'if cands.is_empty()')],
    'tail-pivot-threshold': [('if *pivot >= 0.1 {', 'if *pivot >= 1e-12 {')],
    'tail-small-scale': [('b if b < 10.0 => 0.1,', 'b if b < 10.0 => 1e-12,')],
}


def mutate_rust(cs, text):
    original, result = RUST.read_text(), {}
    try:
        for name, edits in MUTANTS.items():
            src = original
            for old, new in edits:
                assert src.count(old) == 1, ('변조 대상 없음', name, old)
                src = src.replace(old, new)
            RUST.write_text(src)
            TRACE.unlink(missing_ok=True)
            run = subprocess.run(['cargo', 'test', '-q', '-p', 'rsolver-lp', '--test', 'push_ratio'], cwd=ROOT,
                                 capture_output=True, text=True)
            try:
                assert run.returncode == 0, 'Rust 테스트 실패'
                compare(cs, text, TRACE.read_text())
                result[name] = '생존'
            except AssertionError as error:
                result[name] = '거부: ' + str(error)[:200]
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
        cs = cases()
        if args.write:
            INPUT.write_text(base.render(cs))
            print(f'{len(cs)}사례 저장: {INPUT.relative_to(ROOT)}')
            return
        text = INPUT.read_text()
        if args.mutate_rust:
            assert not args.mutate_rust.exists(), '검증 기록 덮어쓰기 거부'
            result = mutate_rust(cs, text)
            args.mutate_rust.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
            print(json.dumps(result, ensure_ascii=False, indent=1))
        elif args.compare:
            print(json.dumps(compare(cs, text, TRACE.read_text()), ensure_ascii=False))
        else:
            print(json.dumps({'rejected': self_test(cs, text, TRACE.read_text())}, ensure_ascii=False))
    except (AssertionError, KeyError, ValueError, IndexError) as error:
        print('불합격: ' + str(error), file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
