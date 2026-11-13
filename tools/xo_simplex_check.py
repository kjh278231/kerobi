#!/usr/bin/env python3
"""M3-04f-4c: cleanup simplex(0cda5b) 관측 자료 검사·집계.

  python3 tools/xo_simplex_check.py --controls     # 사례당 독립 3회 사건열·결과 동일
  python3 tools/xo_simplex_check.py --summary      # 경로·반복·재분해·basis 변화 집계(run 1)
  python3 tools/xo_simplex_check.py --self-test    # 변조를 controls·분류가 잡는지

자료: bench/oracle/m3_xo_simplex/x<C>/<모델>-<r>.json.gz, cases/<특수>_x<C>-<r>.json.gz.
"""
import argparse, collections, copy, glob, gzip, json, os, re, struct, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
D = ROOT / 'bench/oracle/m3_xo_simplex'
BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
TRAMP = 0x4000000  # 모듈 밖 반환 주소(Frida 꼬리 호출 trampoline)는 실행마다 다르다
# 반환값을 쓰지 않는 함수의 rax: 0b16d3은 void(C 3줄), 048d5f는 힙 포인터를 돌려주지만 06268b C 176·431·920이 버린다
NORET = {'0b16d3', '048d5f'}


def load(p):
    return json.loads(gzip.decompress(Path(p).read_bytes()))


def runs():
    """{사례: {rep: 경로}}"""
    out = collections.defaultdict(dict)
    for p in glob.glob(str(D / 'x*' / '*.json.gz')) + glob.glob(str(D / 'cases' / '*.json.gz')):
        rel = Path(p).relative_to(D)
        m = re.match(r'(.+)-(\d+)\.json\.gz$', rel.name)
        case = f'{rel.parent}/{m.group(1)}' if rel.parent.name != 'cases' else f'cases/{m.group(1)}'
        out[case][int(m.group(2))] = p
    return out


def norm(d):
    """재현성 비교 대상: 사건열(실행마다 다른 trampoline 반환 주소만 표시로 바꿈)과 Python 결과."""
    ev = []
    for e in d['events']:
        e = dict(e)
        if 'ra' in e and e['ra'] is not None and not (0 <= e['ra'] < TRAMP):
            e['ra'] = 'T'
        if e.get('fn') in NORET and e.get('ph') == 'leave':
            e['ret'] = 'void'
        ev.append(e)
    return {'events': ev, 'result': d['result']}


def first_diff(a, b, path=''):
    if type(a) is not type(b):
        return path, a, b
    if isinstance(a, dict):
        for k in sorted(set(a) | set(b)):
            if k not in a or k not in b:
                return f'{path}.{k}', a.get(k, '<없음>'), b.get(k, '<없음>')
            r = first_diff(a[k], b[k], f'{path}.{k}')
            if r:
                return r
        return None
    if isinstance(a, list):
        if len(a) != len(b):
            return f'{path}[len]', len(a), len(b)
        for i, (x, y) in enumerate(zip(a, b)):
            r = first_diff(x, y, f'{path}[{i}]')
            if r:
                return r
        return None
    return None if a == b else (path, a, b)


def controls(allruns, quiet=False):
    bad = 0
    for case, reps in sorted(allruns.items()):
        if sorted(reps) != [1, 2, 3]:
            print('회차 부족', case, sorted(reps)); bad += 1; continue
        ds = {r: load(p) for r, p in reps.items()}
        for r, d in ds.items():
            if d['binary_sha256'] != BINARY or d['schema'] != 'M3-04f-4c-v1' or d['errors']:
                print('식별값·오류', case, r); bad += 1
        base = norm(ds[1])
        for r in (2, 3):
            fd = first_diff(base, norm(ds[r]))
            if fd:
                print('재현 실패', case, f'run1↔run{r}', fd[0], fd[1], fd[2]); bad += 1
    if not quiet:
        print(f'controls: 사례 {len(allruns)} × 3회, 실패 {bad}')
    return bad


def f64(h):
    return struct.unpack('<d', bytes.fromhex(h))[0]


def classify(d):
    """한 실행의 cleanup 요약."""
    ev = d['events']
    enter = [e for e in ev if e['event'] == 'cs:enter']
    leave = [e for e in ev if e['event'] == 'cs:leave']
    if len(enter) != 1 or len(leave) != 1:
        raise ValueError(f'cleanup 호출 수 {len(enter)}/{len(leave)}')
    a, b = enter[0], leave[0]
    n, m = a['n'], a['m']
    va, vb = a['vstat'], b['vstat']
    basic_a = {j for j, s in enumerate(va) if s >= 0}
    basic_b = {j for j, s in enumerate(vb) if s >= 0}
    nb_change = sum(1 for j in range(n + m) if va[j] < 0 and vb[j] < 0 and va[j] != vb[j])
    calls = [(e['fn'], e['ph'], e['depth']) for e in ev if e['event'] == 'call']
    top = [f for f, ph, dep in calls if ph == 'enter' and dep <= 1]
    eng = [e for e in ev if e['event'] == 'call' and e['fn'] in ('0478d3', '06268b') and e['ph'] == 'leave']
    c = b['counts']
    # 최종 VBasis/CBasis와 반환 vstat의 대응: 기저(≥ 0) → 0, −1·−2 그대로, −3 → −3, −4(고정 비기저) → −1 또는 −2
    # (−4의 하한/상한 보고 규칙은 basis 출력 단계 a83b2f·08b129 몫이라 여기서는 둘 다 허용하고 따로 센다)
    res = d['result']
    gb = (res['vbasis'] or []) + (res['cbasis'] or [])
    gmatch = None
    fixed = collections.Counter()
    if gb:
        gmatch = 0
        for g, s in zip(gb, vb):
            if s == -4:
                fixed[g] += 1
                gmatch += g not in (-1, -2)
            else:
                gmatch += g != (0 if s >= 0 else s)
    return {
        'alg': a['alg'], 'basis_arg': not a['vb_null'], 'n': n, 'm': m,
        'it_enter': f64(a['it']), 'it_leave': f64(b['it']),
        'refactor': c.get('036ade', 0), 'ft': c.get('05b257', 0),
        'primal': c.get('ev_0478d3', 0), 'dual': c.get('ev_06268b', 0),
        'pass_primal': c.get('ev_07373e', 0) + c.get('ev_06899a', 0), 'iter_primal': c.get('ev_0b44ce', 0),
        'iter_dual': c.get('ev_028c93', 0), 'perturb': c.get('ev_048d5f', 0), 'shift': c.get('ev_0b16d3', 0),
        'i325': c.get('ev_0325b4', 0), 'sifting': c.get('ev_12e738', 0), 'quad': c.get('ev_02d1de', 0) + c.get('ev_02d3b2', 0),
        'unscaled': c.get('ev_02e9ae', 0),
        'basis_same': basic_a == basic_b, 'basis_in': len(basic_b - basic_a), 'nb_change': nb_change,
        'ret': b['ret'], 'status': b['status'], 'model_status': b['model_status'],
        'enter_status': a['status'], 'enter_model_status': a['model_status'], 'alg_358': a['alg_358'],
        'env_2454': a['env_2454'], 's180': a['s180'], 's4d8': a['s4d8'], 's3ec_leave': b['s3ec'],
        'top': top, 'gurobi_vs_leave': gmatch, 'fixed_report': dict(fixed), 'iter_total': res.get('iter'),
    }


def summary(allruns):
    rows = {}
    for case, reps in sorted(allruns.items()):
        rows[case] = classify(load(reps[1]))
    by = collections.Counter()
    for case, r in rows.items():
        x = case.split('/')[0] if not case.startswith('cases/') else 'cases'
        key = (x, 'primal' if r['alg'] == 0 else 'dual', 'arg' if r['basis_arg'] else 'noarg',
               'it0' if r['it_leave'] == 0 else 'it>0', 'same' if r['basis_same'] else 'changed')
        by[key] += 1
    print('사례별(run 1):')
    for case, r in rows.items():
        print(f"  {case:28s} alg={r['alg']} arg={int(r['basis_arg'])} it={r['it_leave']:.0f} refac={r['refactor']} ft={r['ft']} "
              f"P={r['primal']} D={r['dual']} pass={r['pass_primal']} b44ce={r['iter_primal']} 28c93={r['iter_dual']} pert={r['perturb']} "
              f"shift={r['shift']} 325={r['i325']} quad={r['quad']} basis_same={int(r['basis_same'])} in={r['basis_in']} nb={r['nb_change']} "
              f"st={r['enter_status']}/{r['enter_model_status']}→{r['status']}/{r['model_status']} 358={r['alg_358']} 180={r['s180']} "
              f"3ec={r['s3ec_leave']} vb≠={r['gurobi_vs_leave']} iter={r['iter_total']}")
    print('집계(Crossover 묶음, 엔진, basis 인자, 반복, basis 집합):')
    for k, v in sorted(by.items()):
        print('  ', k, v)
    tops = collections.Counter(tuple(r['top']) for r in rows.values())
    print('경로(깊이 ≤ 1 진입 순서):')
    for t, v in tops.most_common():
        print(f'  {v:4d}', ' '.join(t))
    return rows


def self_test(allruns):
    """변조: (1) run 3의 반환 반복 수·(2) 재분해 횟수·(3) 사건 순서·(4) 반환 vstat 기저 교체를 바꾸면 controls/분류가 잡아야 한다."""
    case = next(c for c, r in sorted(allruns.items()) if c.startswith('x3/'))
    d1, d3 = load(allruns[case][1]), load(allruns[case][3])
    ok = True

    def leave_of(d):
        return next(e for e in d['events'] if e['event'] == 'cs:leave')

    muts = []
    m = copy.deepcopy(d3); leave_of(m)['it'] = struct.pack('<d', 1.0).hex(); muts.append(('반복 수', m))
    m = copy.deepcopy(d3); leave_of(m)['counts']['036ade'] = leave_of(m)['counts'].get('036ade', 0) + 1; muts.append(('재분해 수', m))
    m = copy.deepcopy(d3); i = next(k for k, e in enumerate(m['events']) if e['event'] == 'call'); m['events'][i], m['events'][i + 1] = m['events'][i + 1], m['events'][i]; muts.append(('사건 순서', m))
    m = copy.deepcopy(d3); m['result']['vbasis'][0] = -9; muts.append(('최종 VBasis', m))
    m = copy.deepcopy(d3); e = next(e for e in m['events'] if e['event'] == 'call' and e['ph'] == 'leave' and e['fn'] not in NORET); e['ret'] += 1
    muts.append(('사용하는 반환값(' + e['fn'] + ')', m))
    for name, m in muts:
        fd = first_diff(norm(d1), norm(m))
        print(f'  변조 {name}:', '거부' if fd else '생존', fd[0] if fd else '')
        ok &= bool(fd)
    # 분류 판별: 반환 vstat에서 기저 하나를 비기저로 바꾸면 basis_same이 거짓이 돼야 한다
    m = copy.deepcopy(d1); lv = leave_of(m)
    j = next(k for k, s in enumerate(lv['vstat']) if s >= 0); k2 = next(k for k, s in enumerate(lv['vstat']) if s < 0)
    lv['vstat'][j], lv['vstat'][k2] = -1, lv['vstat'][j]
    r0, r1 = classify(d1), classify(m)
    det = r0['basis_same'] != r1['basis_same'] or r0['basis_in'] != r1['basis_in']
    print('  변조 basis 교체 분류:', '거부' if det else '생존'); ok &= det
    # trampoline 정규화가 모듈 안 반환 주소 차이를 지우지 않는지
    m = copy.deepcopy(d3); e = next(e for e in m['events'] if e['event'] == 'call' and e.get('ra') is not None and 0 <= e['ra'] < TRAMP); e['ra'] += 1
    fd = first_diff(norm(d1), norm(m)); print('  변조 모듈 안 반환 주소:', '거부' if fd else '생존'); ok &= bool(fd)
    print('self-test', '통과' if ok else '실패', f'(사례 {case})')
    return ok


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--controls', action='store_true')
    ap.add_argument('--summary', action='store_true')
    ap.add_argument('--self-test', action='store_true')
    ap.add_argument('--json')
    a = ap.parse_args()
    allruns = runs()
    if not allruns:
        sys.exit('자료 없음')
    rc = 0
    if a.controls and controls(allruns):
        rc = 1
    if a.summary:
        rows = summary(allruns)
        if a.json:
            Path(a.json).write_text(json.dumps(rows, indent=1, ensure_ascii=False) + '\n')
    if a.self_test and not self_test(allruns):
        rc = 1
    sys.exit(rc)


if __name__ == '__main__':
    main()
