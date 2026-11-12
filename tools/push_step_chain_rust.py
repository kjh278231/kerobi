"""M3-04c-10: push 한 단계 선택·적용(Rust push_pick·push_apply)을 원본 자연 push 캡처에 연쇄 재생해 비교한다.

구간은 push 시작 또는 재분해(factor 요청 10) 뒤 첫 supply:enter 상태에서 시작한다. Rust는 그 뒤 단계마다 자기 상태를
이어 가고, 단계마다 공급자 출력(후보 j, 반전 전 희소 열)과 12dd49가 돌려줄 factor 갱신 수만 입력으로 받는다.
원본 판단 모델이나 원본 ratio 결과는 Rust에 넘기지 않는다.

  --write      캡처를 bench/oracle/m3_push_step_chain/input.txt로 투영
  --compare    target/m3_push_step_chain/trace.txt를 원본 사건과 비교(첫 차이에서 종료 1)
  --self-test  trace 변조가 모두 거부되는지 확인
  --mutate-rust PATH  Rust 소스 변이마다 재생·비교해 결과를 PATH(새 파일)에 기록
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
import primal_push_decision_check as dc  # noqa: E402  캡처 검사·로더(독립 3회·대조군·비간섭 재검사)

ROOT = Path(__file__).resolve().parents[1]
INPUT = ROOT / 'bench/oracle/m3_push_step_chain/input.txt'
TRACE = ROOT / 'target/m3_push_step_chain/trace.txt'
RUST = ROOT / 'crates/lp/src/crossover.rs'
KIND = {'flip': ('Flip', 'Reject'), 'pivot': ('Pivot',), 'defer': ('Defer',)}
# 캡처에서 세어 계약 §4에 고정한 기대 집계
EXPECT = dict(flip=45, pivot=109, defer=12, refactor_skip=6)


def i64(h):
    return int.from_bytes(bytes.fromhex(h), 'little', signed=True)


def state_line(st):
    return [*map(str, st['vstat']), *map(str, st['header']), *st['x'], *st['slack'], *st['basic'], *st['width'],
            *st['marks'], *st['rhs']]


def want_state(st, prefix, full):
    w = {prefix + 'vstat': ' '.join(map(str, st['vstat'])), prefix + 'header': ' '.join(map(str, st['header'])),
         prefix + 'basic': ' '.join(st['basic']), prefix + 'width': ' '.join(st['width']),
         prefix + 'marks': ' '.join(st['marks']), prefix + 'iter': st['counters']['iterations']}
    if full:
        w[prefix + 'xs'] = ' '.join(st['x'] + st['slack'])
        w[prefix + 'rhs'] = ' '.join(st['rhs'])
    return w


def project(label, rec):
    """구간(seg)·단계(step) 줄과 단계별 기대값."""
    ev, lines, steps = rec['events'], [], []
    model, seg_open, stats = None, False, dict(segments=0, refactor_skip=0, first_leave_input=0)
    pending_seg = None
    i = 0
    while i < len(ev):
        e = ev[i]
        if e['event'] == 'push:enter':
            model, pending_seg = e['model'], 'start'
        elif e['event'] == 'factor:leave' and e['request'] == 10:
            pending_seg = 'refactor'
        elif e['event'] == 'supply:enter' and pending_seg:
            assert e['batch'] == 0 and e['state']['precision'] == 0, (label, 'batch·x87 범위 밖')
            seg_state, seg_open, pending_seg, seg_first = e['state'], True, None, True
        elif e['event'] == 'supply:leave' and e['candidate'] >= 0:
            assert seg_open, (label, '구간 시작 없음')
            name = f"{label}/c{e['call']}s{e['step']}"
            k, calls = i + 1, []
            while ev[k]['event'] not in ('step:enter', 'step:defer'):
                if ev[k]['event'] == 'ratio:enter':
                    calls.append([ev[k], ev[k + 1]])
                elif ev[k]['event'] == 'retry:leave':
                    calls[-1][1] = ev[k]
                k += 1
            assert len(calls) in (1, 2), (name, 'ratio 호출 수')
            se = ev[k]
            if seg_first:
                st = seg_state
                tol = st['tolerances']
                lp = model
                nnz = len(lp['rows'])
                lines.append(' '.join(['seg', label, str(st['n']), str(st['m']), str(nnz), tol['380'], tol['398'],
                                       tol['370'], tol['3b0'], st['counters']['iterations'],
                                       str(calls[0][0]['selected']['leave']), str(st['counters']['request']),
                                       *(str(i64(s)) for s in lp['starts']), *map(str, lp['lengths']),
                                       *map(str, lp['rows']), *lp['values'], *lp['cost'], *lp['lb'], *lp['ub'],
                                       *state_line(st)]))
                stats['segments'] += 1
            a = e['alpha']
            assert a['count'] >= 0, (name, 'dense 열은 범위 밖')
            lines.append(' '.join(['step', name, str(e['candidate']), str(calls[0][0]['state']['factor']['updates']),
                                   str(a['count']), *map(str, a['indices']), *a['values']]))
            kind = 'defer' if se['event'] == 'step:defer' else se['kind']
            w = {'path': 'V' if calls[0][0]['name'] == 'infeasible' else 'F'}
            for c, (rin, rout) in enumerate(calls):
                s = rin['selected']
                w[f'seen{c}'] = f"{s['leave']} {s['direction']} {rin['state']['counters']['request']}"
                w[f'alpha{c + 1}'] = ' '.join(rin['alpha']['values'])
                w[f'ratio{c + 1}'] = f"{rout['chosen']} {rout['theta']} {rout['pivot']}"
                assert rin['selected']['enter'] == e['candidate'], (name, 'ratio 진입 열')
            if seg_first:  # 첫 S+0x2f8은 구간 입력이다
                w['seen0'] = None
                stats['first_leave_input'] += 1
            fr = se.get('frame')
            if fr:
                w['frame'] = f"{fr['r']} {fr['dL']} {fr['dU']}"
            w['step'] = (kind, se['theta'])
            sel = se['selected'] if kind == 'defer' else ev[k + 1]['selected']
            if kind != 'defer':
                assert ev[k + 1]['event'] == 'step:applied', (name, '적용 사건')
                w.update(want_state(ev[k + 1]['state'], 'a.', False))
            else:
                w.update(want_state(se['state'], 'a.', False))
            w['sel'] = f"{sel['enter']} {sel['leave']} {sel['direction']}"
            # 단계 뒤: 다음 supply:enter 또는 push:leave의 상태. 재분해 요청 10이 끼면 비교하지 않는다.
            q, refactor = k + 1, False
            while ev[q]['event'] not in ('supply:enter', 'push:leave'):
                if ev[q]['event'] == 'factor:enter' and ev[q]['request'] == 10:
                    refactor = True
                q += 1
            if refactor:
                stats['refactor_skip'] += 1
            else:
                w.update(want_state(ev[q]['state'], 'z.', True))
            steps.append(dict(name=name, kind=kind, want=w, refactor=refactor))
            seg_first = False
            i = k
        i += 1
    return lines, steps, stats


def collect():
    manifest = json.loads((dc.BASE / 'manifest.json').read_text())
    lines, steps, stats = [], [], dict(segments=0, refactor_skip=0, first_leave_input=0)
    for label, rec in dc.load_all(manifest, arms=True, violated=True, retry=True, recall=True):
        ln, st, s = project(label, rec)
        lines += ln
        steps += st
        for key in stats:
            stats[key] += s[key]
    return lines, steps, stats


def render(lines):
    body = ''.join(x + '\n' for x in lines)
    return f"input {hashlib.sha256(body.encode()).hexdigest()}\n" + body


def parse_row(row):
    parts = row.split(';')
    d = {'name': parts[0]}
    for p in parts[1:]:
        k, _, v = p.partition(' ')
        d[k] = v
    return d


def compare(lines, steps, stats, text, trace):
    assert text == render(lines), '입력 텍스트가 원본 캡처 투영과 다름(--write 필요)'
    rows = trace.splitlines()
    assert rows and rows[0] == text.splitlines()[0], 'trace의 입력 해시 불일치(오래된 trace)'
    assert len(rows) == len(steps) + 1, ('단계 수', len(steps), len(rows) - 1)
    tally, fields = dict(flip=0, pivot=0, defer=0, recall=0, violated=0), 0
    for s, row in zip(steps, rows[1:], strict=True):
        got, w, name = parse_row(row), s['want'], s['name']
        assert got['name'] == name, ('단계 순서', name, got['name'])
        for key, ref in w.items():
            if ref is None:
                continue
            if key == 'step':
                kind, theta = ref
                rk, rt = got['step'].split()
                assert rk in KIND[kind], (name, '단계 종류', '기준', kind, '실제', rk)
                assert rt == theta, (name, 'theta', '기준', theta, '실제', rt, '비트 비교')
                fields += 2
                continue
            assert key in got, (name, key, '필드 누락')
            if got[key] != ref:
                g, r = got[key].split(), ref.split()
                first = next((ix for ix, (a, b) in enumerate(zip(g, r)) if a != b), min(len(g), len(r)))
                raise AssertionError((name, key, '인덱스', first, '기준', r[first:first + 1], '실제', g[first:first + 1],
                                      '정확·비트 비교'))
            fields += 1
        extra = set(got) - set(w) - {'name'} - ({'frame'} if 'frame' not in w else set())  # 04b-1은 frame 미캡처
        if s['refactor']:  # 재분해 뒤 상태는 비교하지 않는다(계약 §3)
            extra -= {k for k in extra if k.startswith('z.')}
        assert not extra, (name, '기준에 없는 필드', sorted(extra))
        tally[s['kind']] += 1
        tally['recall'] += 'ratio2' in w
        tally['violated'] += w['path'] == 'V'
    got = dict(flip=tally['flip'], pivot=tally['pivot'], defer=tally['defer'], refactor_skip=stats['refactor_skip'])
    assert got == EXPECT, ('집계', '기대', EXPECT, '실제', got)
    return dict(steps=len(steps), fields=fields, tally=tally, **stats)


def tamper(trace, i, key, fn):
    rows = trace.splitlines()
    d = parse_row(rows[i])
    assert key in d, ('변조 대상 없음', i, key)
    d[key] = fn(d[key])
    rows[i] = ';'.join([d.pop('name')] + [f'{k} {v}' for k, v in d.items()])
    return '\n'.join(rows) + '\n'


def flip_bit(tokens, k=0):
    t = tokens.split()
    t[k] = format(int(t[k], 16) ^ 1, '016x')
    return ' '.join(t)


def self_test(lines, steps, stats, text, trace):
    idx = {s['name']: n + 1 for n, s in enumerate(steps)}
    first = lambda pred: next(s['name'] for s in steps if pred(s))  # noqa: E731
    piv = first(lambda s: s['kind'] == 'pivot' and 'z.rhs' in s['want'])
    flp = first(lambda s: s['kind'] == 'flip' and 'z.rhs' in s['want'])
    dfr = first(lambda s: s['kind'] == 'defer')
    rec = first(lambda s: 'ratio2' in s['want'])
    vio = first(lambda s: s['want']['path'] == 'V')
    later = first(lambda s: s['want'].get('seen0') is not None)
    frm = first(lambda s: 'frame' in s['want'])
    cases = {
        'path': (vio, 'path', lambda v: 'F'),
        'seen0-leave': (later, 'seen0', lambda v: ' '.join([str(int(v.split()[0]) + 1)] + v.split()[1:])),
        'alpha1-bit': (piv, 'alpha1', flip_bit),
        'ratio1-pos': (piv, 'ratio1', lambda v: ' '.join([str(int(v.split()[0]) + 1)] + v.split()[1:])),
        'ratio1-theta': (piv, 'ratio1', lambda v: ' '.join(v.split()[:1] + [flip_bit(v.split()[1])] + v.split()[2:])),
        'recall-drop': (rec, 'ratio2', lambda v: v.split()[0] + ' x x'),
        'kind': (flp, 'step', lambda v: 'Pivot ' + v.split()[1]),
        'theta': (piv, 'step', lambda v: v.split()[0] + ' ' + flip_bit(v.split()[1])),
        'defer-kind': (dfr, 'step', lambda v: 'Pivot ' + v.split()[1]),
        'sel-leave': (piv, 'sel', lambda v: ' '.join([v.split()[0], str(int(v.split()[1]) + 1), v.split()[2]])),
        'applied-vstat': (piv, 'a.vstat', lambda v: ' '.join(['-4'] + v.split()[1:])),
        'applied-header': (piv, 'a.header', lambda v: ' '.join(reversed(v.split()))),
        'applied-basic': (piv, 'a.basic', flip_bit),
        'applied-marks': (piv, 'a.marks', lambda v: '42 ' + ' '.join(v.split()[1:])),
        'applied-iter': (flp, 'a.iter', flip_bit),
        'after-rhs': (piv, 'z.rhs', flip_bit),
        'after-basic': (flp, 'z.basic', flip_bit),
        'after-xs': (flp, 'z.xs', flip_bit),
        'after-width': (piv, 'z.width', flip_bit),
        'defer-state': (dfr, 'a.basic', flip_bit),
        'frame': (frm, 'frame', flip_bit),
    }
    rejected = []
    for label, (name, key, fn) in cases.items():
        try:
            compare(lines, steps, stats, text, tamper(trace, idx[name], key, fn))
        except AssertionError:
            rejected.append(label)
            continue
        raise AssertionError(('변조 미검출', label, name, key))
    # 단계 누락·순서 바꿈·여분 필드
    rows = trace.splitlines()
    for label, bad in (('drop-step', rows[:-1]), ('swap-steps', rows[:1] + [rows[2], rows[1]] + rows[3:]),
                       ('extra-field', rows[:1] + [rows[1] + ';junk 1'] + rows[2:])):
        try:
            compare(lines, steps, stats, text, '\n'.join(bad) + '\n')
        except AssertionError:
            rejected.append(label)
            continue
        raise AssertionError(('변조 미검출', label))
    return rejected


# Rust 소스 변이: 각 항목은 (원문, 대체) 목록. 결과를 기록만 하고 합격 조건은 사람이 판정한다.
MUTANTS = {
    'pick-feasible-writes-leave': [("if violated || pos != -1 {", "if true {")],
    'pick-no-inf-theta': [("            st.theta = inf; // feasible -1 leaves S+0x2f8 alone", "")],
    'pick-violated-from-marks0': [("let violated = st.violations != 0;", "let violated = false;")],
    'pick-recall-no-reflip': [("            val.iter_mut().for_each(|v| *v = -*v); // 0x1c49e2\n", "")],
    'pick-recall-keep-dir': [("            st.direction = 1 - st.direction;\n", "")],
    'pick-free-leave-kept': [("            if st.theta == inf && entry.free {\n                st.leave = -1;",
                              "            if false {\n                st.leave = -1;")],
    'pick-flip-keeps-leave': [("        st.leave = -1; // 0x1c293b", "        // 0x1c293b")],
    'flip-no-iter': [("            st.iterations += 1.0;\n            let s = push_flip_state",
                      "            let s = push_flip_state")],
    'flip-free-shift-sign': [("            } else if down {\n                st.xs[j]\n            } else {\n                -st.xs[j]",
                              "            } else if down {\n                -st.xs[j]\n            } else {\n                st.xs[j]")],
    'flip-shift-other-gap': [("if down { pick.entry.dl } else { pick.entry.du }", "if down { pick.entry.du } else { pick.entry.dl }")],
    'pivot-remove-keep-order': [("            idx.swap_remove(p);\n            val.swap_remove(p);",
                                 "            idx.remove(p);\n            val.remove(p);")],
    'pivot-width-unbounded': [("st.width[r] = if lp.ub[j] < inf { lp.ub[j] - lp.lb[j] } else { inf };",
                               "st.width[r] = lp.ub[j] - lp.lb[j];")],
    'pivot-no-unflip': [("            if down {\n                a = -a;\n            }\n", "")],
    'pivot-shift-sign': [("shift = if down { -b } else { b };", "shift = if down { b } else { -b };")],
    'pivot-leave-shift-ub': [("let b = if st.vstat[lv] == -1 { st.basic[r] } else { (st.basic[r] - lp.ub[lv]) + lp.lb[lv] };",
                              "let b = if st.vstat[lv] == -2 { st.basic[r] } else { (st.basic[r] - lp.ub[lv]) + lp.lb[lv] };")],
    'pivot-shift-order': [("(st.basic[r] - lp.ub[lv]) + lp.lb[lv]", "st.basic[r] - (lp.ub[lv] - lp.lb[lv])")],
    'pivot-no-lb-shift': [("                if -inf < lp.lb[j] {\n                    b -= lp.lb[j];\n                }\n", "")],
    'pivot-free-row-classified': [("} else { b'F' };", "} else { classify(b, st.width[r], h, inf, &mut st.violations) };")],
    'pivot-viol-no-decrement': [("            if matches!(st.marks[r], b'B' | b'A') {\n                st.violations -= 1;\n            }\n            let b",
                                 "            let b")],
    'rhs-pivot-shifted': [("        if !pivot {\n            match st.vstat[j]", "        {\n            match st.vstat[j]")],
    'rhs-flip-free-shift': [("                -3 => {}\n", "                -3 => d -= lp.lb[j],\n")],
    'rhs-no-leave-col': [("    if pivot && (st.leave_col as usize) < n {", "    if false {")],
    'rhs-leave-bound-ub': [("let bound = if st.vstat[lc] == -1 { lp.lb[lc] } else { lp.ub[lc] };",
                            "let bound = if st.vstat[lc] == -2 { lp.ub[lc] } else { lp.lb[lc] };")],
    'rhs-slack-skip': [("        st.rhs[j - n] += st.xs[j];", "")],
    'rhs-mul-order': [("rhs[i] += d * lp.values[q];", "rhs[i] += lp.values[q] * d;")],  # 동치 변이(교환법칙, 판별 불가)
    'loop-free-classified': [("            b'F' => {}\n            m => {", "            m => {")],
    'loop-viol-keep': [("                if matches!(m, b'B' | b'A') {\n                    st.violations -= 1;\n                }\n", "")],
    'classify-x-any-width': [("        } else if w == 0.0 {", "        } else if w <= 0.0 || w.is_nan() {")],
    'classify-n-strict': [("if v < w - h || inf <= w {", "if v < w - h || inf < w {")],
    'classify-a-strict': [("} else if w + h <= v {", "} else if w + h < v {")],
    'classify-b-edge': [("    if -h <= v {", "    if -h < v {")],
}


def mutate_rust(lines, steps, stats, text):
    original, result = RUST.read_text(), {}
    try:
        for name, edits in MUTANTS.items():
            src = original
            for old, new in edits:
                assert src.count(old) == 1, ('변조 대상 없음', name, old)
                src = src.replace(old, new)
            RUST.write_text(src)
            TRACE.unlink(missing_ok=True)
            run = subprocess.run(['cargo', 'test', '-q', '-p', 'rsolver-lp', '--test', 'push_chain'], cwd=ROOT,
                                 capture_output=True, text=True)
            try:
                assert run.returncode == 0, 'Rust 테스트 실패(컴파일·panic)'
                compare(lines, steps, stats, text, TRACE.read_text())
                result[name] = '생존'
            except AssertionError as error:
                result[name] = '거부: ' + str(error)[:240]
    finally:
        RUST.write_text(original)
        TRACE.unlink(missing_ok=True)
    return result


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    g = p.add_mutually_exclusive_group(required=True)
    g.add_argument('--write', action='store_true')
    g.add_argument('--compare', action='store_true')
    g.add_argument('--self-test', action='store_true')
    g.add_argument('--mutate-rust', type=Path, help='변이 결과 JSON 경로(덮어쓰기 거부)')
    args = p.parse_args()
    try:
        lines, steps, stats = collect()
        if args.write:
            INPUT.parent.mkdir(parents=True, exist_ok=True)
            INPUT.write_text(render(lines))
            print(f'{len(steps)}단계·{stats["segments"]}구간 저장: {INPUT.relative_to(ROOT)}')
            return
        text = INPUT.read_text()
        if args.mutate_rust:
            assert not args.mutate_rust.exists(), '검증 기록 덮어쓰기 거부'
            result = mutate_rust(lines, steps, stats, text)
            args.mutate_rust.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
            print(json.dumps(result, ensure_ascii=False, indent=1))
        elif args.compare:
            print(json.dumps(compare(lines, steps, stats, text, TRACE.read_text()), ensure_ascii=False))
        else:
            print(json.dumps({'rejected': self_test(lines, steps, stats, text, TRACE.read_text())}, ensure_ascii=False))
    except (AssertionError, KeyError, ValueError, IndexError, StopIteration) as error:
        print('불합격: ' + str(error), file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
