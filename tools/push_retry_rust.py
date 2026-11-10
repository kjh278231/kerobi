"""M3-04c-7: 056973 원본 합성 직접 호출과 자연 재시도 입력(retry:enter)을 Rust retry_ratio 입력으로 투영하고 trace를 비교한다.

입력 줄 형식은 primal_push_ratio_rust.py(M3-04c)와 같고 Rust 테스트 push_ratio::retry_ratio_trace가 읽는다. 자연 사례는
work를 retry:enter의 work(054eef 뒤 값)와 S+0x148 계수에서 시작해 retry:leave의 work까지 비교한다. 자연 단계의 게이트
연결(054eef → 056973)과 단계 판단은 primal_push_decision_rust.py가 한다.
"""
import argparse
import json
from pathlib import Path
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
import primal_push_ratio_rust as base  # noqa: E402  M3-04c 입력 줄 형식
import push_retry_ratio_oracle as synth  # noqa: E402  056973 합성 사례
import primal_push_decision_check as dc  # noqa: E402  자연 캡처 검사·로더

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / 'bench/oracle/m3_push_retry'
INPUT = DATA / 'ratio_input.txt'
TRACE = ROOT / 'target/m3_push_retry/trace.txt'
RUST = ROOT / 'crates/lp/src/crossover.rs'
# 작은 pivot 재진입 2차 패스는 Rust가 거부한다(계약 §1)
REFUSED = {f'{n}-{f}' for n in ('reentry-pick', 'reentry-keep', 'reentry-refactor') for f in ('sparse', 'dense')}
bits = base.bits


def cases():
    record = synth.check([DATA / f'ratio-{i}.json' for i in (1, 2, 3)])
    out = []
    for c, e in zip(synth.cases(), record['events'], strict=True):
        assert c['name'] == e['name']
        dense = c['dense']
        # 인자는 push_retry_ratio_oracle.capture()의 호출과 같다.
        out.append(dict(name=c['name'], dense=dense, m=len(c['x']),
                        scal=[bits(synth.H), bits(synth.PIV), bits(synth.INF), bits(c['cap']), bits(0.25), bits(0.0),
                              bits(synth.SENTINEL)],
                        idx=None if dense else c['order'],
                        val=[bits(v) for v in (c['alpha'] if dense else [c['alpha'][i] for i in c['order']])],
                        x=[bits(v) for v in c['x']], width=[bits(v) for v in c['width']],
                        marks=[ch.encode().hex() for ch in c['marks']],
                        expect=dict(pos=e['chosen'], row=e['row'], theta=e['theta'], pivot=e['pivot'],
                                    work=e['work'])))
    rm = json.loads((DATA / 'manifest.json').read_text())
    for case, c in rm['cases'].items():
        if c['role'] != 'target':
            continue
        rec = dc.validate_new([ROOT / x for x in c['captures']], rm, case)[0]
        ev = rec['events']
        for i, e in enumerate(ev):
            if e['event'] != 'retry:enter':
                continue
            out_ev, tol, a = ev[i + 1], e['selected'], e['alpha']
            assert out_ev['event'] == 'retry:leave' and a['count'] >= 0, (case, i, '재시도 사건 쌍·희소 열')
            tol = next(x for x in reversed(ev[:i]) if x['event'] == 'ratio:enter')['state']['tolerances']
            # xmm0..3 = S+0x380·0x398·0x370·0x370(호출점 0x1c25eb-0x1c2600), work는 054eef 뒤 값에서 이어진다
            out.append(dict(name=f"{case}-c{e['call']}s{e['step']}", dense=False, m=len(e['basic']),
                            scal=[tol['380'], tol['398'], tol['370'], tol['370'], e['work_coef'], e['work'], e['theta']],
                            idx=a['indices'], val=a['values'], x=e['basic'], width=e['width'], marks=e['marks'],
                            expect=dict(pos=out_ev['chosen'], row=out_ev['row'], theta=out_ev['theta'],
                                        pivot=out_ev['pivot'], work=out_ev['work'])))
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


# retry_ratio 분기별 Rust 소스 변조. 각 변조는 원본 대조에서 실패해야 한다(최종 코드에는 남기지 않는다).
MUTANTS = {
    'lane-sequential': [('let blocks = if pos.len() >= 8 { pos.len() & !7 } else { 0 };', 'let blocks = 0;')],
    'lane-combine': [('(acc[0][l] + acc[1][l]) + (acc[2][l] + acc[3][l])', '((acc[0][l] + acc[1][l]) + acc[2][l]) + acc[3][l]')],
    'sparse-cap': [("b'N' | b'U' => cand(v / a, idx.is_none()),", "b'N' | b'U' => cand(v / a, true),")],
    'posA-order': [("                    cand((v - w) / a, true);\n                    cand(v / a, true);",
                    "                    cand(v / a, true);\n                    cand((v - w) / a, true);")],
    'negB-inf': [("                    if inf > w {\n                        cand((v - w) / a, true);\n                    }",
                  "                    cand((v - w) / a, true);")],
    'negLN-inf': [("b'L' | b'N' if inf > w => cand((v - w) / a, true),", "b'L' | b'N' => cand((v - w) / a, true),")],
    'at-bound-sign': [("b'U' | b'X' => at_bound -= a,", "b'U' | b'X' => at_bound += a,")],
    'ab-sign': [("b'B' => ab += a,", "b'B' => ab -= a,")],
    'no-cand-cap': [('*theta = if cap < inf { cap } else { inf };', '*theta = inf;')],
    'hoare-mid': [('let p = rat[lo];', 'let p = rat[(lo + hi) / 2];')],
    'split-side': [('if 0.0 > slope { lo = j + 1 } else { hi = j }', 'if 0.0 >= slope { lo = j + 1 } else { hi = j }')],
    'near-no-x2': [('(num - t * a).abs() * 2.0 <= harris;', '(num - t * a).abs() <= harris;')],
    'at-x2': [('let at = |num: f64| (num - t * a).abs() <= harris;', 'let at = |num: f64| (num - t * a).abs() * 2.0 <= harris;')],
    'posA-second-off': [("                b'A' if near(v) => Some(v),\n", '')],
    'negB-second-off': [("                b'B' if inf > w && near(v - w) => Some(v - w),\n", '')],
    'best-le': [('if size > best {', 'if size >= best {')],
    'fallback-le': [('if low > q {', 'if low >= q {')],
    'fallback-cap': [('let mut low = inf;', 'let mut low = cap;')],
    't-zero-skip': [('let mut t = 0.0;', 'let mut t = 1e-3;')],
    'work-dense': [('c * 4.0 + tested as f64 + n }', 'c * 4.0 + n + n }')],
    'work-pass2': [('(if idx.is_none() { n * 3.0 } else { n * 4.0 })', '(n * 4.0)')],
    'cap-exit': [('if inf > cap && cap <= t {', 'if inf > cap && cap < t {')],
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
            try:  # 분할 루프를 끝내지 못하는 변조는 시간 초과로 거부한다
                run = subprocess.run(['cargo', 'test', '-q', '-p', 'rsolver-lp', '--test', 'push_ratio', 'retry_ratio_trace'],
                                     cwd=ROOT, capture_output=True, text=True, timeout=600)
            except subprocess.TimeoutExpired:
                result[name] = '거부: 시간 초과'
                continue
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
