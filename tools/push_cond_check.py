#!/usr/bin/env python3
"""M3-04c-1c: 조건수 116fd7 내부 경계 캡처 검사기.

  python3 tools/push_cond_check.py --controls --acceptance [--summary OUT]
  python3 tools/push_cond_check.py --self-test
  python3 tools/push_cond_check.py --write-manifest

controls: manifest·바이너리·도구 해시, 15사례 × 3회 사건 동일, 새 사건(cond:*)을 뺀 투영이 M3-04c-1b run 1과 같음(비간섭).
acceptance: 116fd7 호출마다 C·asm 규칙(docs/m3/primal_push_condition.md §2)으로 ‖B‖₁·FTRAN 입력·추정값·종료·부호 벡터·
argmax·출력을 캡처만으로 다시 계산해 비트/정확 비교한다. FTRAN/BTRAN 출력은 원본 값을 그대로 쓴다(그 수치는 Rust 쪽 비교).
"""
import argparse, copy, json, os, struct, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import push_init_check as pic  # noqa: E402
import push_init_survey as sv  # noqa: E402

ROOT = pic.ROOT
DATA = os.path.join(ROOT, 'bench/oracle/m3_push_condition')
MANIFEST = os.path.join(DATA, 'manifest.json')
TOOLS = ['tools/push_cond_agent.js', 'tools/push_cond_capture.js', 'tools/push_cond_runs.py']
SCHEMA = 'M3-04c-1c-v1'
RULES = dict(peel=True, lanes='pairs', bmax='gt', slack=1.0, sign='lt', amax='first', limit=5, stop='ge', copy_buf='x')


def f(h):
    return struct.unpack('<d', bytes.fromhex(h))[0]


def hx(v):
    return struct.pack('<d', v).hex()


def netlib():
    import push_init_runs as pir
    return [f'netlib/{x}' for x in pir.SURVEY45]


def cases():
    """1b 15사례(비간섭 대상) + Netlib 소형 45 자연(Crossover=3)."""
    return pic.cases() + netlib()


def path(case, r):
    p = os.path.join(DATA, f'{case}-{r}.json')
    return p if os.path.exists(p) else p + '.gz'


def load_all():
    return {c: [sv.load(path(c, r)) for r in (1, 2, 3)] for c in cases()}


# ---------------------------------------------------------------- 통제
def controls(docs, manifest):
    errs = []
    if manifest.get('binary_sha256') != sv.BINARY_SHA:
        errs.append('manifest binary_sha256')
    for t in TOOLS:
        if manifest.get('tools', {}).get(t) != sv.sha(os.path.join(ROOT, t)):
            errs.append(f'manifest tool {t}')
    for c, runs in docs.items():
        for r, d in enumerate(runs, 1):
            rel = os.path.relpath(path(c, r), ROOT)
            if manifest.get('captures', {}).get(rel) != sv.sha(path(c, r)):
                errs.append(f'{c}-{r} manifest 해시')
            if d.get('binary_sha256') != sv.BINARY_SHA:
                errs.append(f'{c}-{r} binary_sha256')
            if d.get('agent_sha256') != manifest.get('tools', {}).get('tools/push_cond_agent.js'):
                errs.append(f'{c}-{r} agent 해시')
            if d.get('errors') or d.get('reason') != 'process-terminated' or not isinstance((d.get('result') or {}).get('status'), int):
                errs.append(f'{c}-{r} 실행 실패')
            if not d.get('events'):
                errs.append(f'{c}-{r} 사건 없음')
            if d.get('schema') != SCHEMA:
                errs.append(f'{c}-{r} schema')
            if any(e['event'].startswith('unsupported') for e in d.get('events', [])):
                errs.append(f'{c}-{r} x87 경로')
        base = pic.strip_doc(runs[0])
        for r, d in enumerate(runs[1:], 2):
            s = pic.strip_doc(d)
            if s != base:
                k = next(k for k in sorted(set(base) | set(s)) if base.get(k) != s.get(k))
                detail = ''
                if k == 'events':
                    i = next((i for i, (a, b) in enumerate(zip(base['events'], s['events'])) if a != b),
                             min(len(base['events']), len(s['events'])))
                    detail = f' 사건 {i}'
                errs.append(f'{c} run{r}≠run1: {k}{detail}')
    return errs


def noninterference(docs):
    """cond:* 사건을 뺀 run 1이 1b run 1과 같다(사건열·설정·결과)."""
    errs = []
    for c in pic.cases():
        old = sv.load(pic.path(c, 1))
        mine = docs[c][0]
        evs = [e for e in mine['events'] if not e['event'].startswith('cond:')]
        if evs != old['events']:
            i = next((i for i, (a, b) in enumerate(zip(old['events'], evs)) if a != b), min(len(old['events']), len(evs)))
            errs.append(f'{c}: 1b 투영 사건 {i} 다름')
        for k in ('config', 'params', 'result', 'model_sha256', 'kind', 'origin'):
            if mine.get(k) != old.get(k):
                errs.append(f'{c}: 1b {k} 다름')
    return errs


# ---------------------------------------------------------------- C 규칙 재계산
def asum(vals, mod16, rules):
    """|v| 합. asm 0x1a9387–0x1a948b·0x1a96e3–0x1a97fd: 길이 < 8, 주소 & 7 ≠ 0, 길이 < peel+8이면 순차."""
    a = [abs(v) for v in vals]
    if mod16 % 8 != 0:
        raise ValueError('8바이트 비정렬 버퍼')
    p = 1 if (mod16 == 8 and rules['peel']) else 0
    if len(a) < 8 or len(a) < p + 8:
        s = 0.0
        for t in a:
            s += t
        return s
    body = len(a) - ((len(a) - p) & 7)
    lane = [0.0] * 8
    for t in a[:p]:
        lane[0] += t
    for k in range(p, body, 8):
        for q in range(8):
            lane[q] += a[k + q]
    if rules['lanes'] == 'pairs':
        w = ((lane[0] + lane[2]) + (lane[4] + lane[6])) + ((lane[1] + lane[3]) + (lane[5] + lane[7]))
    else:  # 순서 변이: lane 0..7 순차
        w = 0.0
        for t in lane:
            w += t
    for t in a[body:]:
        w += t
    return w


def norm_b(model, header, n, a16, rules):
    best = 0.0
    starts, lengths, values = model['starts'], model['lengths'], model['values']
    for j in header:
        if j >= n:
            s = rules['slack']
        else:
            b0 = sv.i64(starts[j])
            s = asum([f(h) for h in values[b0:b0 + lengths[j]]], (a16 + 8 * b0) % 16, rules)
        if rules['bmax'] == 'gt':
            best = s if s > best else best
        else:
            best = s if s >= best else best
    return best


def argmax(out, m, rules):
    vals = [abs(f(h)) for h in out['val']]
    idx = list(range(m)) if out['count'] < 0 else out['idx']
    best, j = 0.0, -1
    for k, v in enumerate(vals):
        if (v > best) if rules['amax'] == 'first' else (v >= best and v > 0):
            best, j = v, idx[k]
    return j


def check_cond(tag, evs, model, rules, summ):
    """cond:enter … cond:leave 한 호출. 반환: 오류 목록."""
    errs = []
    en = evs[0]
    m, n = en['m'], en['n']
    if en['f0c'] != 0 or en['precision'] != 0 or en['distinct'] != 4 or not en['out_is_param'] or en['work_null']:
        return [f'{tag}: 진입 전제(F+0xc·precision·버퍼·인자)']
    mod = en['mod16']
    nb = norm_b(model, en['header'], n, en['a_val_mod16'], rules)
    one = hx(1.0 / m)
    x = dict(count=-1, idx=None, val=[one] * m)
    est, prev_j, it, ests, js, reason, k = 0.0, -1, 0, [], [], None, 0
    rest = evs[1:-1]
    p = 0

    def take(kind):
        nonlocal p
        if p >= len(rest) or rest[p]['event'] != kind:
            raise AssertionError(f'{tag}: 사건 {p} {kind} 기대, {rest[p]["event"] if p < len(rest) else "끝"}')
        e = rest[p]
        p += 1
        if e['k'] != k:
            raise AssertionError(f'{tag}: {kind} k {e["k"]} != {k}')
        return e
    try:
        while True:
            fe = take('cond:ftran:enter')
            got = fe['input']
            if got['count'] != x['count'] or got['val'] != x['val'] or (x['idx'] is not None and got['idx'] != x['idx']):
                return [f'{tag}: k{k} FTRAN 입력 다름(count {got["count"]}/{x["count"]})']
            fl = take('cond:ftran:leave')
            y = fl['output']
            # x.count = y.count, 값·idx 복사: 합산 버퍼는 반환 뒤 x 구조의 val 버퍼
            buf = fl['input']['val_buf'] if rules['copy_buf'] == 'x' else y['val_buf']
            if buf not in mod:
                return [f'{tag}: k{k} x 버퍼 {buf}']
            old = est
            est = asum([f(h) for h in y['val']], mod[buf], rules) if y['count'] != 0 else 0.0
            ests.append(est)
            if it > 0 and ((old >= est) if rules['stop'] == 'ge' else (old > est)):
                reason = 'est'
                break
            sign = ['000000000000f03f'] * m
            pos = range(m) if y['count'] < 0 else y['idx']
            for kk, i in enumerate(pos):
                v = f(y['val'][kk])
                if (v < 0) if rules['sign'] == 'lt' else (v <= 0):
                    sign[i] = '000000000000f0bf'
            be = take('cond:btran:enter')
            if be['input']['count'] != -1 or be['input']['val'] != sign:
                return [f'{tag}: k{k} 부호 벡터 다름']
            bl = take('cond:btran:leave')
            it += 1
            if it >= rules['limit']:
                reason = 'limit'
                break
            j = argmax(bl['output'], m, rules)
            js.append(j)
            if j == prev_j:
                reason = 'repeat'
                break
            x = dict(count=1, idx=[j], val=['000000000000f03f'])
            prev_j = j
            k += 1
    except AssertionError as e:
        return [str(e)]
    if p != len(rest):
        return [f'{tag}: 종료({reason}) 뒤 사건 {len(rest) - p}개 남음']
    lv = evs[-1]
    want = hx(nb * est)
    if lv['condition'] != want or lv['ret'] != 0:
        errs.append(f'{tag}: condition {lv["condition"]} != 재계산 {want}')
    summ.append(dict(tag=tag, m=m, normb=hx(nb), iters=it, reason=reason, est=[hx(e) for e in ests], j=js,
                     ftran_out=[e['output']['count'] for e in rest if e['event'] == 'cond:ftran:leave'],
                     swaps=[e['output']['val_buf'] for e in rest if e['event'] == 'cond:ftran:leave'],
                     condition=lv['condition'], cond=nb * est))
    return errs


def acceptance(docs, rules=RULES, summary=None):
    errs, summ = [], [] if summary is None else summary
    for c in cases():
        d = docs[c][0]
        for call, evs in sorted(pic.calls_of(d).items()):
            pe = next((e for e in evs if e['event'] == 'push:enter'), None)
            if pe is None:
                errs.append(f'{c} call{call}: push:enter 없음')
                continue
            model = pe['model']
            lus = [e for e in evs if e['event'] == 'init:lu']
            clus = [e for e in evs if e['event'] == 'cond:lu']
            if len(lus) != len(clus) or any(len(e['block']) != 1 for e in clus):
                errs.append(f'{c} call{call}: cond:lu {len(clus)} != init:lu {len(lus)} 또는 036ade 재시도')
            leaves = [e for e in evs if e['event'] == 'init:condition:leave']
            starts = [i for i, e in enumerate(evs) if e['event'] == 'cond:enter']
            if len(starts) != len(leaves):
                errs.append(f'{c} call{call}: cond 호출 {len(starts)} != 12f282 {len(leaves)}')
                continue
            if not pe['first'] and starts:
                errs.append(f'{c} call{call}: first=0에서 116fd7')
            for r, i in enumerate(starts):
                end = next((q for q in range(i, len(evs)) if evs[q]['event'] == 'cond:leave'), None)
                if end is None:
                    errs.append(f'{c} call{call}#{r}: cond:leave 없음')
                    continue
                tag = f'{c} call{call}#{r}'
                errs += check_cond(tag, evs[i:end + 1], model, rules, summ)
                if evs[end]['condition'] != leaves[r]['condition']:
                    errs.append(f'{tag}: cond:leave != init:condition:leave')
                if evs[end + 1]['event'] != 'init:condition:leave':
                    errs.append(f'{tag}: cond:leave 뒤 init:condition:leave 아님')
    if not summ:
        errs.append('116fd7 호출 0')
    if summary is None:
        pass
    return errs, summ


# ---------------------------------------------------------------- self-test
def _ev(doc, kind, k=0):
    return [e for e in doc['events'] if e['event'] == kind][k]


def _flip(h):
    b = bytearray(bytes.fromhex(h)); b[0] ^= 1; return b.hex()


def _flip6(h):
    b = bytearray(bytes.fromhex(h)); b[6] ^= 1; return b.hex()


def _all(docs, case, fn):
    for d in docs[case]:
        fn(d)


TAMPERS = [
    ('바이너리 해시 0', 'controls', lambda D: D['arms'][0].__setitem__('binary_sha256', '0' * 64)),
    ('사건 비우기', 'controls', lambda D: D['interior'][1].__setitem__('events', [])),
    ('run2 BTRAN 출력 1비트', 'controls', lambda D: (lambda e: e['output']['val'].__setitem__(0, _flip(e['output']['val'][0])))(_ev(D['capri'][1], 'cond:btran:leave'))),
    ('1b 비간섭: push:enter x 1비트(3회 모두)', 'nonint', lambda D: _all(D, 'retry', lambda d: _ev(d, 'push:enter')['state']['x'].__setitem__(2, _flip(_ev(d, 'push:enter')['state']['x'][2])))),
    ('1b 비간섭: 12f282 출력 1비트(3회 모두)', 'nonint', lambda D: _all(D, 'adlittle', lambda d: _ev(d, 'init:condition:leave').__setitem__('condition', _flip(_ev(d, 'init:condition:leave')['condition'])))),
    ('condition 1비트', 'accept', lambda D: _ev(D['scsd1'][0], 'cond:leave').__setitem__('condition', _flip(_ev(D['scsd1'][0], 'cond:leave')['condition']))),
    ('header 최대 열 → 슬랙', 'accept', lambda D: (lambda e: e.__setitem__('header', [e['n'] + i for i in range(e['m'])]))(_ev(D['capri'][0], 'cond:enter'))),
    ('A 정렬 8(share2b)', 'accept', lambda D: _ev(D['netlib/share2b'][0], 'cond:enter').__setitem__('a_val_mod16', 8)),
    ('x·y 버퍼 정렬 8(전체)', 'accept', lambda D: [e['mod16'].update(X=8, Y=8) for c in D for e in D[c][0]['events'] if e['event'] == 'cond:enter']),
    ('첫 FTRAN 입력 1비트', 'accept', lambda D: (lambda e: e['input']['val'].__setitem__(3, _flip(e['input']['val'][3])))(_ev(D['adlittle'][0], 'cond:ftran:enter'))),
    ('FTRAN 출력 상위 가수 1비트', 'accept', lambda D: (lambda e: e['output']['val'].__setitem__(0, _flip6(e['output']['val'][0])))(_ev(D['adlittle'][0], 'cond:ftran:leave', 1))),
    ('부호 벡터 1개 반전', 'accept', lambda D: _ev(D['scsd1'][0], 'cond:btran:enter')['input']['val'].__setitem__(0, '000000000000f0bf' if _ev(D['scsd1'][0], 'cond:btran:enter')['input']['val'][0] != '000000000000f0bf' else '000000000000f03f')),
    ('e_j index +1', 'accept', lambda D: (lambda e: e['input']['idx'].__setitem__(0, e['input']['idx'][0] + 1))(_ev(D['adlittle'][0], 'cond:ftran:enter', 1))),
    ('마지막 BTRAN 사건 제거', 'accept', lambda D: D['capri'][0].__setitem__('events', [e for e in D['capri'][0]['events'] if e is not [x for x in D['capri'][0]['events'] if x['event'] == 'cond:btran:leave'][-1]])),
    ('cond:leave 제거', 'accept', lambda D: D['arms'][0].__setitem__('events', [e for e in D['arms'][0]['events'] if e['event'] != 'cond:leave'])),
    ('null-null: FTRAN 입력·출력 비우기', 'accept', lambda D: [e[k].__setitem__('val', []) for e, k in ((_ev(D['interior'][0], 'cond:ftran:enter'), 'input'), (_ev(D['interior'][0], 'cond:ftran:leave'), 'output'))]),
]
MUTATIONS = [
    # (이름, 규칙, 필수) — 필수는 자료가 구별해야 하는 것(결과를 보기 전에 정함)
    ('합산 순차(lane 무시)', dict(lanes='seq'), True),
    ('슬랙 열 노름 0', dict(slack=0.0), True),
    ('부호 ≤ 0', dict(sign='le'), False),
    ('반복 한도 4', dict(limit=4), False),
    ('추정 비증가 엄격 >', dict(stop='gt'), False),
    ('argmax 마지막 최대', dict(amax='last'), False),
    ('‖B‖₁ max ≥', dict(bmax='ge'), False),
    ('peel 무시', dict(peel=False), False),
    ('합산 버퍼 = y 출력 버퍼', dict(copy_buf='y'), False),
]


def self_test(docs, manifest):
    bad = 0
    base_c = controls(docs, manifest) + noninterference(docs)
    base_a, _ = acceptance(docs)
    if base_c or base_a:
        print('기준 자료가 먼저 실패:', (base_c + base_a)[:5])
        return 1
    for name, layer, fn in TAMPERS:
        D = copy.deepcopy(docs)
        try:
            fn(D)
            if layer == 'controls':
                errs = controls(D, manifest)
            elif layer == 'nonint':
                errs = noninterference(D)
            else:
                errs, _ = acceptance(D)
        except Exception as e:
            errs = [f'예외 {type(e).__name__}: {e}']
        print(('REJECTED     ' if errs else 'NOT REJECTED ') + name + (f'  ← {errs[0]}' if errs else ''))
        bad += not errs
    for name, mut, required in MUTATIONS:
        try:
            errs, _ = acceptance(docs, dict(RULES, **mut))
        except Exception as e:
            errs = [f'예외 {type(e).__name__}: {e}']
        tag = 'REJECTED     ' if errs else ('NOT REJECTED ' if required else 'SURVIVED     ')
        print(tag + '변이 ' + name + (f'  ← {errs[0]}' if errs else ''))
        bad += required and not errs
    print('필수 변조/변이 통과', bad)
    return 1 if bad else 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--controls', action='store_true')
    ap.add_argument('--acceptance', action='store_true')
    ap.add_argument('--self-test', action='store_true')
    ap.add_argument('--write-manifest', action='store_true')
    ap.add_argument('--summary')
    a = ap.parse_args()
    if a.write_manifest:
        if os.path.exists(MANIFEST):
            sys.exit('기존 manifest 덮어쓰기 거부')
        man = dict(task='M3-04c-1c', binary_sha256=sv.BINARY_SHA, schema=SCHEMA,
                   tools={t: sv.sha(os.path.join(ROOT, t)) for t in TOOLS},
                   captures={os.path.relpath(path(c, r), ROOT): sv.sha(path(c, r)) for c in cases() for r in (1, 2, 3)},
                   baseline='bench/oracle/m3_push_init (1b manifest)', baseline_sha256=sv.sha(pic.MANIFEST))
        open(MANIFEST, 'w').write(json.dumps(man, indent=1, ensure_ascii=False) + '\n')
        print('manifest 작성', len(man['captures']))
        return 0
    docs = load_all()
    manifest = json.load(open(MANIFEST))
    if manifest.get('baseline_sha256') != sv.sha(pic.MANIFEST):
        print('1b manifest 해시 불일치'); return 1
    if a.self_test:
        return self_test(docs, manifest)
    errs = []
    if a.controls:
        e = controls(docs, manifest) + noninterference(docs)
        print('controls', '통과' if not e else f'실패 {len(e)}', f'(사례 {len(docs)} × 3회, 1b 비간섭 {len(pic.cases())}사례)')
        errs += e
    if a.acceptance:
        e, summ = acceptance(docs)
        print('acceptance', '통과' if not e else f'실패 {len(e)}', f'(116fd7 호출 {len(summ)}개)')
        for s in summ:
            print(f'  {s["tag"]:22s} m {s["m"]:4d} ‖B‖₁ {f(s["normb"]):.6g} 반복 {s["iters"]} 종료 {s["reason"]:6s} '
                  f'j {s["j"]} FTRAN 출력 count {s["ftran_out"]} 버퍼 {s["swaps"]} cond {s["cond"]:.6g}')
        errs += e
        if a.summary:
            open(a.summary, 'w').write(json.dumps(summ, indent=1, ensure_ascii=False) + '\n')
    for x in errs[:30]:
        print('  ', x)
    return 1 if errs else 0


if __name__ == '__main__':
    sys.exit(main())
