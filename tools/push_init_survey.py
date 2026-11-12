#!/usr/bin/env python3
"""M3-04c-1 push 초기화 조사: 21h 원본 캡처의 push:enter 입력에서 `134a87` C 150–1142 규칙으로
후보 집합·정렬 순서, rhs S+0x200, 폭, 표시·PInf, 반복·지원 flag를 다시 계산해 첫 supply:enter와 비교한다.

basic 값(FTRAN 결과)은 비교하지 않고 진단 잔차만 남긴다. 새 원본 실행은 하지 않는다.

  python3 tools/push_init_survey.py --check       # 저장 집계와 바이트 비교, 불일치면 종료 1
  python3 tools/push_init_survey.py --write       # 집계 작성
  python3 tools/push_init_survey.py --self-test   # 입력 변조·규칙 변이 판별력, 필수 변조 통과 시 종료 1
"""
import argparse, copy, gzip, hashlib, json, math, os, struct, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = os.path.join(ROOT, 'bench/oracle/m3_push_block')
OUT = os.path.join(ROOT, 'bench/validation/m3_push_init/survey.json')
CASES = ['arms', 'retry', 'interior', 'adlittle', 'scsd1', 'capri']
BINARY_SHA = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
CAP = 0.99  # C 621·631 상수, asm 0x1c10b0 (0x2a613e8)


def f64(h):
    return struct.unpack('<d', bytes.fromhex(h))[0]


def i64(h):
    return struct.unpack('<q', bytes.fromhex(h))[0]


def hx(v):
    return struct.pack('<d', v).hex()


def minsd(d, c):
    """minsd d,c: d < c 이면 d, 아니면(같음·NaN 포함) c."""
    return d if d < c else c


# ---------------------------------------------------------------- aabbe3 (crates/lp/src/common.rs 이식)
def sort_desc(key, idx):
    _quick(0, len(key), 0, key, idx)


def _quick(lo, hi, depth, key, idx):
    while depth < 50:
        n = hi - lo
        if n < 100:
            break
        d1, d2, d3 = key[lo], key[hi - 1], key[lo + n // 2]
        p = d1
        if ((d3 < d1) or (d1 < d2)) and ((d1 < d3) or (d2 < d1)):
            p = d3
            if ((d1 < d3) or (d3 < d2)) and ((d3 < d1) or (d2 < d3)):
                p = d2
        i, g, j = lo, lo, hi
        if key[lo] > p:
            while i < hi and key[i] > p:
                i += 1
            g = i
        while i < j:
            x = key[i]
            if x > p:
                key[g], key[i] = key[i], key[g]
                idx[g], idx[i] = idx[i], idx[g]
                g += 1
                i += 1
            elif x == p:
                i += 1
            else:
                j -= 1
                y = key[j]
                done = False
                while y <= p:
                    if y == p:
                        if i < j:
                            key[i], key[j] = key[j], key[i]
                            idx[i], idx[j] = idx[j], idx[i]
                            i += 1
                        done = True
                        break
                    if j <= i:
                        done = True
                        break
                    j -= 1
                    y = key[j]
                if not done and i < j:
                    xi, gi, ji = idx[i], idx[g], idx[j]
                    key[i] = key[g]; idx[i] = gi
                    key[g] = y; idx[g] = ji
                    key[j] = x; idx[j] = xi
                    g += 1
                    i += 1
        depth += 1
        if hi - j < g - lo:
            _quick(j, hi, depth, key, idx)
            hi = g
        else:
            _quick(lo, g, depth, key, idx)
            lo = j
    _shell(lo, hi, key, idx)


def _shell(lo, hi, key, idx):
    gap = (hi - lo) // 2
    while gap > 0:
        while True:
            k = lo + gap
            while k < hi:
                x, xi = key[k], idx[k]
                t = k
                while t >= lo + gap and x > key[t - gap]:
                    key[t] = key[t - gap]; idx[t] = idx[t - gap]
                    t -= gap
                key[t] = x; idx[t] = xi
                k += 1
            if gap == 2 and lo + 1 < hi:
                gap = 1
            else:
                break
        gap = int(gap / 2.2)


# ---------------------------------------------------------------- 자료 읽기
def load(path):
    with (gzip.open(path, 'rt') if path.endswith('.gz') else open(path)) as fh:
        return json.load(fh)


def sha(path):
    return hashlib.sha256(open(path, 'rb').read()).hexdigest()


def case_paths(case):
    out = []
    for r in (1, 2, 3):
        p = os.path.join(DATA, f'{case}-{r}.json')
        out.append(p if os.path.exists(p) else p + '.gz')
    return out


def extract(doc):
    """첫 push의 초기화 경계 사건을 뽑는다."""
    ev = doc['events']
    get = {}
    for e in ev:
        k = e['event']
        if k in ('push:enter', 'factor:enter', 'factor:leave', 'roundctl:pass:enter', 'roundctl:pass:ready') and k not in get:
            get[k] = e
        if k == 'supply:enter':
            get[k] = e
            break
    leave = next((e for e in ev if e['event'] == 'push:leave'), None)
    return get, leave


def decode(get):
    pe, se, pa = get['push:enter'], get['supply:enter'], get['roundctl:pass:enter']
    md, st, ss = pe['model'], pe['state'], se['state']
    n, m = st['n'], st['m']
    inp = dict(
        n=n, m=m, first=pe['first'], attempt=pe['attempt'], model_status=pe['model_status'],
        alias=pe['alias_contiguous'],
        starts=[i64(h) for h in md['starts']], lengths=md['lengths'], rows=md['rows'],
        values=[f64(h) for h in md['values']], rhs=[f64(h) for h in md['rhs']],
        lb=[f64(h) for h in md['lb']], ub=[f64(h) for h in md['ub']],
        vstat=list(st['vstat']), x=[f64(h) for h in st['x']], slack=[f64(h) for h in st['slack']],
        inf=f64(st['tolerances']['370']), tau=f64(st['tolerances']['380']),
        markowitz=st['tolerances']['390'], iterations=f64(st['counters']['iterations']),
        numeric=st['counters']['numeric'], limit=st['factor']['limit'])
    out = dict(
        candidates=list(pa['state']['candidates']), total=pa['state']['total'], support=pa['state']['support'],
        supply_candidates=list(se['candidates']), vstat=list(ss['vstat']), header=list(ss['header']),
        rhs=list(ss['rhs']), basic=[f64(h) for h in ss['basic']], width=list(ss['width']),
        marks=list(ss['marks']), pinf=ss['pinf'], iterations=f64(ss['counters']['iterations']),
        numeric=ss['counters']['numeric'], request=ss['counters']['request'],
        markowitz=ss['tolerances']['390'], limit=ss['factor']['limit'],
        factor_request=get['factor:enter']['request'], factor_caller=get['factor:enter']['factor_ctl']['caller'],
        factor_ret=get['factor:leave']['ret'])
    return inp, out


# ---------------------------------------------------------------- C 규칙 재계산
DEFAULT_RULES = dict(sort='aabbe3', cap=True, slack_m2='stale', dist='midpoint', order='cols-first',
                     tau_strict=True, m4='lb')


def build(inp, rules=DEFAULT_RULES):
    """C 401–640: rhs·후보·key·정렬(기준 조정 없는 경로)."""
    n, m = inp['n'], inp['m']
    lb, ub, x, sl, vs = inp['lb'], inp['ub'], inp['x'], inp['slack'], inp['vstat']
    r = list(inp['rhs'])
    cand, dist = [], []
    xmm0 = 0.0  # asm 0x1c0b68 pxor
    stats = dict(col={}, slack={}, capped=0, stale_nonzero=0)
    col_c, sl_c = [], []
    for j in range(n):
        s = vs[j]
        s_key = s if s < 0 else 'basic'
        stats['col'][s_key] = stats['col'].get(s_key, 0) + 1
        if s >= 0:
            continue
        if s == -1:
            v = lb[j]
        elif s == -2:
            v = ub[j]
        elif s == -3:
            v = x[j]
            if rules['dist'] == 'midpoint':
                d = (x[j] - lb[j]) if (ub[j] + lb[j] > x[j] + x[j]) else (ub[j] - x[j])
            else:
                d = x[j] - lb[j]
            col_c.append((j, d))
        elif s == -4:
            v = lb[j] if rules['m4'] == 'lb' else ub[j]
        elif s == -5:
            v = ub[j]
        else:
            v = 0.0
        xmm0 = v
        if v != 0.0 or v != v:
            for k in range(inp['starts'][j], inp['starts'][j] + inp['lengths'][j]):
                r[inp['rows'][k]] = r[inp['rows'][k]] - v * inp['values'][k]
    for i in range(m):
        s = vs[n + i]
        s_key = s if s < 0 else 'basic'
        stats['slack'][s_key] = stats['slack'].get(s_key, 0) + 1
        if s >= 0 or s < -3:
            continue
        if s == -1:
            v = 0.0
        elif s == -3:
            v = sl[i]
            sl_c.append((n + i, sl[i]))
        else:  # -2: asm 0x1c4f3d → 0x1c4f68, xmm0 갱신 없음
            v = xmm0 if rules['slack_m2'] == 'stale' else 0.0
            if v != 0.0:
                stats['stale_nonzero'] += 1
        xmm0 = v
        r[i] = r[i] - v
    pairs = col_c + sl_c if rules['order'] == 'cols-first' else sl_c + col_c
    idx = [p[0] for p in pairs]
    key = []
    for j, d in pairs:
        c = minsd(d, CAP) if rules['cap'] else d
        if rules['cap'] and not d < CAP:
            stats['capped'] += 1
        key.append(-(float(inp['lengths'][j]) + c) if j < n else -1.0 - c)
    unsorted = list(zip(idx, key))
    if rules['sort'] == 'aabbe3':
        sort_desc(key, idx)
    else:
        order = sorted(range(len(idx)), key=lambda t: -key[t])
        idx = [idx[t] for t in order]
        key = [key[t] for t in order]
    groups = {}
    for k in key:
        groups[k] = groups.get(k, 0) + 1
    stats['ties'] = sum(c for c in groups.values() if c > 1)
    stats['candidates'] = len(idx)
    stats['quick_path'] = len(idx) >= 100
    # 안정 정렬과 다른지(원본 정렬의 판별 가능성)
    st_order = [u[0] for u in sorted(unsorted, key=lambda t: -t[1])]
    stats['stable_differs'] = st_order != idx
    return dict(rhs=r, candidates=idx, keys=key), stats


def widths(inp, header, basic_in):
    """C 800–815: lower shift는 원본 basic에 이미 반영됐으므로 폭만 다시 만든다."""
    I = inp['inf']
    w = []
    for p, h in enumerate(header):
        l, u = inp['lb'][h], inp['ub'][h]
        if l <= -I or I <= u:
            w.append(I)
        else:
            w.append(u - l)
    return w


def classify(inp, header, basic, width, rules=DEFAULT_RULES):
    """C 816–866."""
    I, tau = inp['inf'], inp['tau']
    marks, viol, pinf = [], 0, 0.0
    for p, h in enumerate(header):
        l, u = inp['lb'][h], inp['ub'][h]
        if (-I < l) or (u < I):
            v, w = basic[p], width[p]
            low = (-tau <= v) if rules['tau_strict'] else (-tau < v)
            if low:
                if tau <= v:
                    if (v < w - tau) or (I <= w):
                        marks.append('4e')
                    elif w + tau <= v:
                        marks.append('41'); viol += 1; pinf = pinf + (v - w)
                    else:
                        marks.append('55')
                elif w == 0.0:
                    marks.append('58')
                else:
                    marks.append('4c')
            else:
                marks.append('42'); viol += 1; pinf = pinf - v
        else:
            marks.append('46')
    return marks, viol, pinf


def residual(inp, header, basic):
    n, m = inp['n'], inp['m']
    acc = [0.0] * m
    for p, h in enumerate(header):
        val = basic[p] + (inp['lb'][h] if inp['lb'][h] > -inp['inf'] else 0.0)
        if h < n:
            for k in range(inp['starts'][h], inp['starts'][h] + inp['lengths'][h]):
                acc[inp['rows'][k]] += val * inp['values'][k]
        else:
            acc[h - n] += val
    return acc


def compare(inp, out, rules=DEFAULT_RULES):
    """사례 하나의 비교. 실패 목록과 집계를 돌려준다."""
    fails = []
    b, stats = build(inp, rules)
    # 기저 위치 번호는 초기화 재분해(요청 10)의 순열로 바뀐다. 기저 집합과 header 일치만 확인한다.
    changes = [(k, a, c) for k, (a, c) in enumerate(zip(inp['vstat'], out['vstat'])) if (a < 0 or c < 0) and a != c]
    adjusted = bool(changes)
    bad_change = [t for t in changes if not (t[1] in (-1, -2) and t[2] == -3)]
    if bad_change:
        fails.append(('vstat', bad_change[0]))
    if sorted(out['header']) != sorted(k for k, a in enumerate(inp['vstat']) if a >= 0):
        fails.append(('basic-set',))
    if any(out['vstat'][h] != p for p, h in enumerate(out['header'])):
        fails.append(('header-vstat',))
    renumbered = sum(1 for a, c in zip(inp['vstat'], out['vstat']) if a >= 0 and a != c)
    if not adjusted:
        if b['candidates'] != out['candidates']:
            k = next((t for t, (a, c) in enumerate(zip(b['candidates'], out['candidates'])) if a != c),
                     min(len(b['candidates']), len(out['candidates'])))
            fails.append(('candidates', k))
        rh = [hx(v) for v in b['rhs']]
        if rh != out['rhs']:
            k = next(t for t, (a, c) in enumerate(zip(rh, out['rhs'])) if a != c)
            fails.append(('rhs', k, rh[k], out['rhs'][k]))
    if out['supply_candidates'] != out['candidates']:
        fails.append(('supply_candidates',))
    if out['total'] != len(out['candidates']):
        fails.append(('total',))
    w = [hx(v) for v in widths(inp, out['header'], out['basic'])]
    if w != out['width']:
        fails.append(('width', next(t for t, (a, c) in enumerate(zip(w, out['width'])) if a != c)))
    marks, viol, pinf = classify(inp, out['header'], out['basic'], [f64(h) for h in out['width']], rules)
    if marks != out['marks']:
        fails.append(('marks', next(t for t, (a, c) in enumerate(zip(marks, out['marks'])) if a != c)))
    if hx(pinf) != out['pinf']:
        fails.append(('pinf', hx(pinf), out['pinf']))
    if out['iterations'] != inp['iterations'] + 1.0:
        fails.append(('iterations', inp['iterations'], out['iterations']))
    sup = 1 if math.sqrt(float(inp['m'])) <= float(out['total']) else 0
    if sup != out['support']:
        fails.append(('support', sup, out['support']))
    if out['request'] != 0:
        fails.append(('request', out['request']))
    if inp['first'] and (out['factor_request'] != 10 or out['factor_caller'] != '0x1c07c6'):
        fails.append(('init-factor', out['factor_request'], out['factor_caller']))
    res = residual(inp, out['header'], out['basic'])
    rmax = max([abs(a - c) for a, c in zip(res, b['rhs'])] + [0.0])
    summary = dict(
        n=inp['n'], m=inp['m'], first=inp['first'], attempt=inp['attempt'], model_status=inp['model_status'],
        alias_contiguous=inp['alias'],
        vstat_cols={str(k): v for k, v in stats['col'].items()},
        vstat_slacks={str(k): v for k, v in stats['slack'].items()},
        candidates=stats['candidates'], capped=stats['capped'], key_ties=stats['ties'],
        quick_path=stats['quick_path'], stable_sort_differs=stats['stable_differs'],
        slack_m2_stale_nonzero=stats['stale_nonzero'], adjusted=adjusted, vstat_changes=len(changes), basic_renumbered=renumbered,
        markowitz_enter=inp['markowitz'], markowitz_after=out['markowitz'],
        markowitz_tightened=inp['markowitz'] != out['markowitz'], limit_enter=inp['limit'], limit_after=out['limit'],
        numeric_enter=inp['numeric'], numeric_after=out['numeric'], support=out['support'],
        violations=viol, pinf=out['pinf'], pinf_le_1=f64(out['pinf']) <= 1.0,
        marks={k: marks.count(k) for k in sorted(set(marks))}, factor_ret=out['factor_ret'],
        diag_residual_max=hx(rmax), compared_fields=len(out['candidates']) + inp['m'] * 3 + 6)
    return fails, summary


INIT_FIELDS = ('push:enter', 'roundctl:pass:enter', 'supply:enter')


def strip(e):
    e = copy.deepcopy(e)
    for k in ('pid', 'run_id'):
        e.pop(k, None)
    if 'supplier' in e:
        e.pop('supplier')
    return e


def survey(rules=DEFAULT_RULES, tamper=None):
    manifest = json.load(open(os.path.join(DATA, 'manifest.json')))
    errs = []
    if manifest.get('binary_sha256') != BINARY_SHA:
        errs.append('manifest binary_sha256')
    result = dict(task='M3-04c-1', source='bench/oracle/m3_push_block', binary_sha256=BINARY_SHA,
                  tool_sha256=sha(os.path.abspath(__file__)), cases={})
    for case in CASES:
        runs = []
        for p in case_paths(case):
            rel = os.path.relpath(p, ROOT)
            if manifest['captures'].get(rel) != sha(p):
                errs.append(f'hash {rel}')
            doc = load(p)
            if doc.get('binary_sha256') != BINARY_SHA:
                errs.append(f'binary {rel}')
            runs.append(extract(doc))
        base = [strip(runs[0][0][k]) for k in INIT_FIELDS]
        for r, (g, _) in enumerate(runs[1:], 2):
            if [strip(g[k]) for k in INIT_FIELDS] != base:
                errs.append(f'{case} run{r} 초기화 사건 불일치')
        get, leave = runs[0]
        inp, out = decode(get)
        if tamper:
            tamper(inp, out)
        fails, summ = compare(inp, out, rules)
        summ['push_leave'] = None if leave is None else dict(ret=leave['ret'], retry=leave['retry'])
        summ['failures'] = [list(map(str, f)) for f in fails]
        result['cases'][case] = summ
        errs += [f'{case}: {f}' for f in fails]
    result['errors'] = errs
    return result


def dumps(obj):
    return json.dumps(obj, ensure_ascii=False, indent=1, sort_keys=True) + '\n'


# ---------------------------------------------------------------- 변조·변이
def _swap_cand(inp, out):
    c = out['candidates']
    if len(c) > 1:
        c[0], c[1] = c[1], c[0]


def _flip_rhs(inp, out):
    h = bytearray(bytes.fromhex(out['rhs'][0])); h[0] ^= 1; out['rhs'][0] = h.hex()


def _flip_mark(inp, out):
    out['marks'][0] = '41' if out['marks'][0] != '41' else '4e'


def _iter(inp, out):
    out['iterations'] += 1.0


def _input_x(inp, out):
    j = next((j for j in range(inp['n']) if inp['vstat'][j] == -3), None)
    if j is not None:
        inp['x'][j] = math.nextafter(inp['x'][j], math.inf)


def _input_rhs(inp, out):
    inp['rhs'][0] = math.nextafter(inp['rhs'][0], math.inf)


def _width(inp, out):
    h = bytearray(bytes.fromhex(out['width'][0])); h[0] ^= 1; out['width'][0] = h.hex()


def _support(inp, out):
    out['support'] ^= 1


TAMPERS = [('후보 앞 두 개 교환', _swap_cand), ('rhs[0] 1비트', _flip_rhs), ('표시[0] 변경', _flip_mark),
           ('반복 수 +1', _iter), ('입력 rhs[0] 1ulp', _input_rhs), ('폭[0] 1비트', _width),
           ('지원 flag 반전', _support), ('후보 열 x 1ulp', _input_x)]
def _cut_supply(d):
    k = next(i for i, e in enumerate(d['events']) if e['event'] == 'supply:enter')
    d['events'] = d['events'][:k]


DOC_TAMPERS = [('바이너리 해시 0', lambda d: d.__setitem__('binary_sha256', '0' * 64)),
               ('사건 비우기', lambda d: d.__setitem__('events', [])),
               ('첫 공급 사건 제거', _cut_supply)]
MUTATIONS = [('안정 정렬', dict(sort='stable')), ('0.99 상한 제거', dict(cap=False)),
             ('slack -2 = 0', dict(slack_m2='zero')), ('거리 항상 x-lb', dict(dist='lower')),
             ('slack 후보 먼저', dict(order='slacks-first')), ('-τ 경계 strict', dict(tau_strict=False)),
             ('-4를 ub로', dict(m4='ub'))]


def self_test():
    ok = True
    base = survey()
    if base['errors']:
        print('기준 실패:', base['errors'][:5]); return False
    print('입력 변조(필수 거부):')
    for name, fn in TAMPERS:
        r = survey(tamper=fn)
        rej = bool(r['errors'])
        print(f'  {"거부" if rej else "통과!"}  {name}  {r["errors"][:1]}')
        ok &= rej
    print('자료 변조(필수 거부):')
    global load
    real = load
    for name, edit in DOC_TAMPERS:
        def patched(path, edit=edit):
            d = real(path)
            if path.endswith(('arms-1.json', 'arms-1.json.gz')):
                edit(d)
            return d
        load = patched
        try:
            r = survey()
            rej, why = bool(r['errors']), r['errors'][:1]
        except (KeyError, StopIteration, IndexError, TypeError) as e:
            rej, why = True, [f'예외 {type(e).__name__}']
        finally:
            load = real
        print(f'  {"거부" if rej else "통과!"}  {name}  {why}')
        ok &= rej
    print('규칙 변이(자연 자료 판별력):')
    for name, ch in MUTATIONS:
        rules = dict(DEFAULT_RULES, **ch)
        r = survey(rules=rules)
        print(f'  {"거부" if r["errors"] else "생존"}  {name}  {r["errors"][:1]}')
    return ok


def main():
    ap = argparse.ArgumentParser()
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument('--check', action='store_true')
    g.add_argument('--write', action='store_true')
    g.add_argument('--self-test', action='store_true')
    a = ap.parse_args()
    if a.self_test:
        sys.exit(0 if self_test() else 1)
    res = survey()
    text = dumps(res)
    for e in res['errors']:
        print('오류:', e)
    if a.write:
        os.makedirs(os.path.dirname(OUT), exist_ok=True)
        open(OUT, 'w').write(text)
        print('작성', os.path.relpath(OUT, ROOT))
        sys.exit(1 if res['errors'] else 0)
    if not os.path.exists(OUT) or open(OUT).read() != text:
        print('저장 집계와 다름:', os.path.relpath(OUT, ROOT)); sys.exit(1)
    if res['errors']:
        sys.exit(1)
    for c, s in res['cases'].items():
        print(f"{c}: n={s['n']} m={s['m']} 후보={s['candidates']} 조정={s['adjusted']} Markowitz강화={s['markowitz_tightened']} 위반={s['violations']}")
    print('통과')


if __name__ == '__main__':
    main()
