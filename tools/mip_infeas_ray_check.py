#!/usr/bin/env python3
"""M2-06f-4: 95da98 (infeasible-node vector) replayed from the inputs captured by tools/mip_infeas_ray.js.
Contract: docs/m2/mip_infeas_ray.md section 3.

(default)          controls + acceptance on bench/oracle/m2_06f4
--write-manifest   record the SHA-256 of every capture (once, after the capture)
--self-test        also T (capture tampers) and V (rule variants; expected outcome fixed from the branch counts)
--sweep FILE...    replay single exploratory captures, print branch counts (not acceptance)
Exit code 0 only when every requested check passes.
"""
import collections, copy, gzip, hashlib, json, math, os, struct, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = os.path.join(ROOT, 'bench/oracle/m2_06f4')
OLD = os.path.join(ROOT, 'bench/oracle/m2_06f2')  # same runs (NodeLimit 2000), non-interference projection
TOOL = os.path.join(ROOT, 'tools/mip_infeas_ray.js')
BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
RUNS = (1, 2, 3)
CASES = ('markshare_4_0', 'pk1', 'neos-860300', 'tr12-30', 'fhnw-binpack4-4', 'neos5')
PARAMS = {'Threads': 1, 'Seed': 0, 'LogToConsole': 0, 'NodeLimit': 2000}
TOL = 1e-10  # xmm0 at the 5ce67b call (0x3ddb7cdfd9d7bdbb)
# rule variants: (name, branch-count key that must be > 0 for the variant to be detectable)
VARIANTS = {
    'sign': 'ok',          # s = +1 above the upper bound
    'noflipj': 'flip_j',   # ignore flip of the basic variable
    'nobndflip': 'flip_k', # do not negate bnd of flipped columns
    'nomin1': 'scale_lt1', # divide by the scale itself, not min(1, scale)
    'tol0': 'ray_small',   # keep entries with |alpha| <= tol
    'basic1': 'ok',        # basic entry keeps f*alpha instead of 1
    'norm_all': 'ok',      # norm over n+m instead of n
    'nonorm60': 'ok',      # L+0x60 not divided by the norm
    'rhs_noR': 'rowscale', # rho without the row scale
    'act_swap': 'ok',      # lb/ub choice swapped in the activity
    'act_inf': 'ub_inf',   # infinite ub kept instead of 1e100
    'age_first': 'age_multi',  # first cut value instead of the max
    'age_all': 'age_zero',     # max also over zero ray entries
    'f6_100': 'ok',        # fail 6 threshold 100 instead of 1000
    'work_bnd3': 'colscale',   # bnd fill work 3 per column even with column scales
    'tiny200': 'fm2',      # no 1e-200 start for flip mode 2
}


class Fail(Exception):
    pass


def need(cond, msg):
    if not cond:
        raise Fail(msg)


def sha(p):
    with open(p, 'rb') as fh:
        return hashlib.sha256(fh.read()).hexdigest()


def load(path):
    return json.loads(gzip.decompress(open(path, 'rb').read()))


def f(h):
    return struct.unpack('>d', bytes.fromhex(h))[0]


def h(x):
    return struct.pack('>d', x).hex()


def calls(d):
    """Group the events of one capture into 95da98 calls: fk, the inner events, fk_."""
    out, cur = [], None
    for i, e in enumerate(d['events']):
        if e['e'] == 'fk':
            need(cur is None, f'event {i}: fk inside fk')
            cur = {'fk': e, 'i': i, 'inner': []}
        elif e['e'] == 'fk_':
            need(cur is not None, f'event {i}: fk_ without fk')
            cur['ret'], cur['j'] = e, i
            out.append(cur)
            cur = None
        elif e['e'] in ('row', 'q', 'rs'):
            need(cur is not None, f'event {i}: {e["e"]} outside fk')
            cur['inner'].append(e)
    need(cur is None, 'capture ends inside fk')
    return out


class Out(Exception):
    def __init__(self, code):
        self.code = code


def replay(call, arrays, V=frozenset(), c=None):
    """Section 2.3 rule. Returns the outputs; c collects branch counts."""
    c = c if c is not None else collections.Counter()
    fk = call['fk']
    rows = [e for e in call['inner'] if e['e'] == 'row']
    qs = [e for e in call['inner'] if e['e'] == 'q']
    rs = [e for e in call['inner'] if e['e'] == 'rs']
    need(fk.get('lim') == 2, 'retry limit is not 2')
    c['rs'] += len(rs)
    o = {'fail': 0, 'rc': 0, 'l58': -1, 'l54': 0, 'l60': 0.0, 'l5c': 0, 'l68': 0.0, 'ray': None, 'bnd': None, 'w': None}
    # entry gate (first loop pass; a re-solve pass is not replayed)
    if not rs:
        st = fk['st']
        if st in (2, 5):
            c['gate_st25'] += 1
            o['fail'] = 1
            return o
        need(st == 3 and fk['spx'], f'entry state {st} spx {fk["spx"]} without a re-solve event')
        if fk['q'] >= 0 and f(fk['qb0']) + f(fk['delta']) <= f(fk['qa8']):
            c['gate_q'] += 1
            o['fail'] = 3
            return o
        need(fk['fac'] and fk['r'] >= 0, 'no factor or r < 0 without a re-solve event')
    need(len(rows) == 1, f'{len(rows)} row events')
    row = rows[0]
    need(row['rc'] == 0, f'12ea3f rc {row["rc"]}')
    need(row['quad'] == 0, 'quad precision path (out of scope)')
    need(row['lpsame'] == 1 and call['ret']['same'] == 1, 'lp arrays moved during the call')
    need(row['ctxS'] == 1, 'ctx null or ctx+8 != S (out of scope)')
    need(row['spxr'] == row['r'], 'row argument != spx+0x2f8')
    n, m = row['n'], row['m']
    N = n + m
    A = lambda k: arrays[row[k]] if row.get(k) is not None else None
    lb, ub, rhs = [f(x) for x in A('lb')], [f(x) for x in A('ub')], [f(x) for x in A('rhs')] if m > 0 else []
    C = [f(x) for x in A('C')] if row.get('C') else None
    R = [f(x) for x in A('R')] if row.get('R') else None
    status = A('status')
    fm = row['fm']
    flip = A('flip') if fm > 0 else None
    cut = A('cut')
    delta, inf = f(row['delta']), f(row['inf'])
    j, x = row['j'], f(row['x'])
    for v in [x] + lb + ub + rhs + (C or []) + (R or []):
        need(not math.isnan(v), 'NaN input (out of scope)')
    w = f(row['w'])
    c['colscale' if C else 'nocolscale'] += 1
    c['rowscale' if R else 'norowscale'] += 1
    c[f'fm{fm}'] += 1
    c['slack_j' if j >= n else 'col_j'] += 1
    c['dense' if row['cnt'] < 0 else 'sparse'] += 1
    o['l58'] = j
    fl = 1.0
    try:
        above = ub[j] + delta <= x
        if not above and not (x <= lb[j] - delta):
            raise Out(1)
        s = -1 if above else 1
        if 'sign' in V:
            s = -s
        if j < n and fm > 0 and flip[j] == 1:
            c['flip_j'] += 1
            if 'noflipj' not in V:
                fl, s = -1.0, -s
        o['l54'] = s
        xs = x
        if C:
            if j < n:
                xs = x * C[j]
                if C[j] > 1e7:
                    raise Out(3)
            else:
                xs = x / R[j - n]
                if R[j - n] < 1e-7:
                    raise Out(3)
        bnd = [0.0] * N
        for k in range(n):
            b = lb[k] if status[k] in (-1, -4) else ub[k] if status[k] in (-2, -5) else None
            bnd[k] = 0.0 if b is None else (b * C[k] if C else b)
        if n > 0:
            w = n * (4.0 if C and 'work_bnd3' not in V else 3.0) + w
        if m > 0:
            w = m + w
        bnd[j] = xs
        if fm > 0:
            for k in range(n):
                if flip[k] == 1 and 'nobndflip' not in V:
                    c['flip_k'] += 1
                    bnd[k] = -bnd[k]
        o['bnd'] = bnd
        # ray
        ray = [0.0] * N
        o['ray'] = ray
        val = [f(v) for v in row['val']] if row['val'] else []
        ent = list(enumerate(val)) if row['cnt'] < 0 else list(zip(row['idx'] or [], val))
        if row['cnt'] >= 0:
            w = w + N
        qi = 0
        done = 0
        try:
            for t, (k, a) in enumerate(ent):
                done = t
                if k < n:
                    sc = C[k] if C else 1.0
                else:
                    sc = R[k - n] if R else 1.0
                if sc < 1.0:
                    c['scale_lt1'] += 1
                if 'nomin1' not in V and 1.0 <= sc:
                    sc = 1.0
                d = abs(a) / sc
                if k >= n and d != 0.0:
                    need(qi < len(qs) and qs[qi]['row'] == k - n, f'7889ec call {qi} for row {k - n} missing')
                    c['q'] += 1
                    qi += 1
                    if qs[qi - 1]['ret']:
                        raise Out(4)
                if d > 1e7:
                    raise Out(3)
                if TOL < d or ('tol0' in V and a != 0.0):
                    if not TOL < d:
                        c['ray_small'] += 1
                    if status[k] < 0:
                        ray[k] = fl * a
                        if status[k] in (-3, -6):
                            raise Out(2)
                        if k >= n:
                            c['ray_slack'] += 1
                    elif k == j:
                        ray[k] = fl * a if 'basic1' in V else 1.0
                        if abs(a - fl) > TOL:
                            raise Out(2)
                    else:
                        ray[k] = fl * a
                        raise Out(2)
                else:
                    if a != 0.0:
                        c['ray_small'] += 1
                    ray[k] = 0.0
                done = t + 1
        except Out:
            w = done * 4.0 + w
            raise
        need(qi == len(qs), f'{len(qs) - qi} unused 7889ec calls')
        w = done * 4.0 + w
        # rho
        rho = 0.0
        for i in range(m):
            y = ray[n + i]
            rho = rho + ((rhs[i] * y) / R[i] if R and 'rhs_noR' not in V else rhs[i] * y)
            if abs(y) > (100.0 if 'f6_100' in V else 1000.0):
                raise Out(6)
        if m > 0:
            w = m * 3.0 + w if R else (m + m) + w
        o['l60'] = rho
        res = rho
        for k in range(N):
            res = res - bnd[k] * ray[k]
        o['res'] = abs(res / 100.0)
        if N > 0:
            w = (N + N) + w
        if abs(res / 100.0) > TOL:
            raise Out(2)
        sg = float(s)
        nu2 = 1e-200 if fm == 2 and 'tiny200' not in V else 0.0
        act = 0.0
        for k in range(N if 'norm_all' in V else n):
            r = ray[k]
            nu2 = nu2 + r * r
            if k >= n:
                continue
            fk_ = fm == 1 and flip[k] == 1
            if (sg * r <= 0.0) != ('act_swap' in V):
                if fk_:
                    b = -lb[k] * C[k] if C else -lb[k]
                else:
                    b = ub[k]
                    if b < inf:
                        b = b * C[k] if C else b
                    else:
                        c['ub_inf'] += 1
                        b = b if 'act_inf' in V else 1e100
            else:
                if fk_ or lb[k] <= -inf:
                    b = -1e100
                else:
                    b = lb[k] * C[k] if C else lb[k]
            act = act + r * b
        if n > 0:
            w = n * (4.0 if C else 3.0) + w
        if nu2 < 1e-4:
            raise Out(5)
        nu = math.sqrt(nu2)
        o['l68'] = (sg * (act - rho)) / nu
        for k in range(N):
            ray[k] = ray[k] / nu
        if N > 0:
            w = N + w
        o['l60'] = rho if 'nonorm60' in V else rho / nu
        m0 = row['m0']
        mx = 0
        if m > m0:
            nzc = 0
            for t in range(m - m0):
                if abs(ray[n + m0 + t]) > 1e-10 or 'age_all' in V:
                    if abs(ray[n + m0 + t]) <= 1e-10:
                        c['age_zero'] += 1
                    nzc += 1
                    need(cut[t] is not None, f'cut {t} unreadable')
                    if 'age_first' in V:
                        if nzc == 1:
                            mx = max(cut[t], 0)
                    elif cut[t] > mx:
                        mx = cut[t]
            if nzc > 1:
                c['age_multi'] += 1
            if nzc:
                c['age_cut'] += 1
            cnt = m
        else:
            cnt = m0
        w = (cnt + cnt) + w
        o['l5c'] = mx
        o['w'] = w
        c['ok'] += 1
    except Out as e:
        o.update(fail=e.code, l58=-1, l54=0, l60=0.0, l5c=0, l68=0.0)
        o['w'] = w
        c[f'fail{e.code}'] += 1
    return o


def compare(call, o, where):
    r = call['ret']
    need(r['rc'] == o['rc'], f'{where}: rc {r["rc"]} != {o["rc"]}')
    need(r['f354'] == o['fail'], f'{where}: fail code {r["f354"]} != {o["fail"]}')
    for k in ('l58', 'l54', 'l5c'):
        need(r[k] == o[k], f'{where}: L+0x{k[1:]} {r[k]} != {o[k]}')
    for k in ('l60', 'l68'):
        need(r[k] == h(o[k]), f'{where}: L+0x{k[1:]} {r[k]} ({f(r[k])!r}) != {h(o[k])} ({o[k]!r})')
    if o['w'] is not None and r['w'] is not None:
        need(r['w'] == h(o['w']), f'{where}: work {f(r["w"])!r} != {o["w"]!r}')
    if o['fail'] == 0:
        for k in ('ray', 'bnd'):
            got = r[k]
            exp = {str(i): h(v) for i, v in enumerate(o[k]) if h(v) != '0' * 16}
            if got != exp:
                diff = sorted(set(got) ^ set(exp) | {i for i in set(got) & set(exp) if got[i] != exp[i]}, key=int)
                i = diff[0]
                raise Fail(f'{where}: {k}[{i}] {got.get(i, "0" * 16)} != {exp.get(i, "0" * 16)} ({len(diff)} entries differ)')


def check_capture(d, name, V=frozenset(), c=None):
    c = c if c is not None else collections.Counter()
    arrays = d['arrays']
    cs = calls(d)
    c['calls'] += len(cs)
    margin = None
    for n, call in enumerate(cs):
        o = replay(call, arrays, V, c)
        compare(call, o, f'{name} call {n} (event {call["i"]})')
        if o.get('res') is not None:
            margin = o['res'] if margin is None else max(margin, o['res'])
    c['_res_max'] = margin
    # decay input: the vector the decay reads equals its last writer (fk_ or pp)
    for i, e in enumerate(d['events']):
        if e['e'] == 'dec':
            need(e['k'] >= 0, f'{name} event {i}: decay on an L no successful fk filled')
            need(e.get('vsame') == 1 and e.get('diff') == 0, f'{name} event {i}: decay vector differs from its last writer')
            c['dec_' + e['src']] += 1
        elif e['e'] == 'pp' and e.get('d'):
            c['pp_changed'] += 1
        elif e['e'] == 'pp' and e.get('moved'):
            raise Fail(f'{name} event {i}: 6684cd moved the vector')
        elif e['e'] == 'oth':
            c['oth'] += 1
        elif e['e'] == 'x2a':
            c['x2a'] += 1
    return c


def project(d):
    """The M2-06f-2 fk / fk_ fields, in order."""
    out = []
    for e in d['events']:
        if e['e'] == 'fk':
            out.append(('fk', e['r9'], e['v70']))
        elif e['e'] == 'fk_':
            out.append(('fk_', e['rc'], e['f354'], e['l58'], e['l54'], e['v70']))
    return out


def controls(caps, manifest, old):
    need(manifest['binary_sha256'] == BINARY, 'manifest binary hash')
    need(manifest['tool_sha256'] == sha(TOOL), 'manifest tool hash differs from tools/mip_infeas_ray.js')
    for case in CASES:
        runs = [caps[(case, r)] for r in RUNS]
        for r, d in zip(RUNS, runs):
            name = f'{case}_{r}'
            need(d['binary_sha256'] == BINARY, f'{name}: binary hash')
            need(d['tool_sha256'] == manifest['tool_sha256'], f'{name}: tool hash')
            need(d['params'] == PARAMS, f'{name}: params')
            need(d['ok'] and not d['errors'], f'{name}: run not ok')
            need(len(d['events']) > 0 or case == 'neos5', f'{name}: no events')
            need(d['model_sha256'] == manifest['models'][case], f'{name}: model hash')
        for r, d in zip(RUNS[1:], runs[1:]):
            need(d['events'] == runs[0]['events'] and d['arrays'] == runs[0]['arrays'] and d['output'] == runs[0]['output'],
                 f'{case}: run {r} differs from run 1')
        o = old[case]
        need(runs[0]['output'] == o['output'], f'{case}: output line {runs[0]["output"]!r} != M2-06f-2 {o["output"]!r}')
        p, q = project(runs[0]), project(o)
        if p != q:
            i = next((i for i, (a, b) in enumerate(zip(p, q)) if a != b), min(len(p), len(q)))
            raise Fail(f'{case}: fk/fk_ projection differs from M2-06f-2 run 1 at {i}: {p[i:i + 1]} != {q[i:i + 1]}')


def load_all():
    caps = {(c, r): load(os.path.join(DATA, f'{c}_{r}.json.gz')) for c in CASES for r in RUNS}
    old = {c: load(os.path.join(OLD, f'{c}_1.json.gz')) for c in CASES}
    manifest = json.load(open(os.path.join(DATA, 'manifest.json')))
    for c in CASES:
        for r in RUNS:
            fn = f'{c}_{r}.json.gz'
            need(manifest['files'][fn] == sha(os.path.join(DATA, fn)), f'{fn}: file hash differs from the manifest')
    return caps, old, manifest


def acceptance(caps):
    tot = collections.Counter()
    for case in CASES:
        c = check_capture(caps[(case, 1)], case)
        res = c.pop('_res_max')
        tot.update(c)
        print(f'  {case}: ' + ' '.join(f'{k}={v}' for k, v in sorted(c.items())) + (f' res_max={res:.3g}' if res is not None else ''))
    need(tot['calls'] > 0 and tot['ok'] > 0, 'no successful call replayed')
    need(tot['dec_fk_'] > 0, 'no decay read a 95da98 vector')
    return tot


def tampers(caps, old, manifest):
    base = caps[('pk1', 1)]
    fk_i = next(i for i, e in enumerate(base['events']) if e['e'] == 'fk_' and e['f354'] == 0)
    T = []

    def ev(fn):
        def t():
            d = copy.deepcopy(base)
            fn(d)
            check_capture(d, 'pk1')
        return t

    def setf(k, v):
        return ev(lambda d: d['events'][fk_i].__setitem__(k, v))

    T.append(('l58', setf('l58', base['events'][fk_i]['l58'] + 1)))
    T.append(('l54', setf('l54', -base['events'][fk_i]['l54'])))
    T.append(('l5c', setf('l5c', base['events'][fk_i]['l5c'] + 1)))
    T.append(('l60 last bit', setf('l60', h(f(base['events'][fk_i]['l60'])) [:-1] + format(int(base['events'][fk_i]['l60'][-1], 16) ^ 1, 'x'))))
    T.append(('l68 last bit', setf('l68', base['events'][fk_i]['l68'][:-1] + format(int(base['events'][fk_i]['l68'][-1], 16) ^ 1, 'x'))))
    T.append(('work last bit', setf('w', base['events'][fk_i]['w'][:-1] + format(int(base['events'][fk_i]['w'][-1], 16) ^ 1, 'x'))))
    T.append(('fail code', setf('f354', 6)))

    def rayflip(d):
        r = d['events'][fk_i]['ray']
        k = next(iter(r))
        r[k] = r[k][:-1] + format(int(r[k][-1], 16) ^ 1, 'x')
    T.append(('ray bit', ev(rayflip)))
    T.append(('ray extra', ev(lambda d: d['events'][fk_i]['ray'].__setitem__('0' if '0' not in d['events'][fk_i]['ray'] else '1', '3ff0000000000000'))))
    T.append(('bnd bit', ev(lambda d: d['events'][fk_i]['bnd'].__setitem__(next(iter(d['events'][fk_i]['bnd'])), '3ff0000000000001'))))
    T.append(('row dropped', ev(lambda d: d['events'].pop(next(i for i, e in enumerate(d['events']) if e['e'] == 'row')))))
    T.append(('q dropped', ev(lambda d: d['events'].pop(next(i for i, e in enumerate(d['events']) if e['e'] == 'q')))))
    T.append(('quad', ev(lambda d: next(e for e in d['events'] if e['e'] == 'row').__setitem__('quad', 1))))
    T.append(('decay vector', ev(lambda d: next(e for e in d['events'] if e['e'] == 'dec').__setitem__('diff', 8))))
    T.append(('decay without fk', ev(lambda d: next(e for e in d['events'] if e['e'] == 'dec').__setitem__('k', -1))))

    def mani(fn):
        def t():
            m2 = copy.deepcopy(manifest)
            fn(m2)
            controls(caps, m2, old)
        return t
    T.append(('binary hash', mani(lambda m: m.__setitem__('binary_sha256', '0' * 64))))
    T.append(('tool hash', mani(lambda m: m.__setitem__('tool_sha256', '0' * 64))))

    def capt(fn):
        def t():
            c2 = dict(caps)
            c2[('pk1', 2)] = copy.deepcopy(caps[('pk1', 2)])
            fn(c2[('pk1', 2)])
            controls(c2, manifest, old)
        return t
    T.append(('run 2 event', capt(lambda d: d['events'][fk_i].__setitem__('l5c', 99))))
    T.append(('run 2 empty', capt(lambda d: d.__setitem__('events', []))))
    T.append(('model hash', capt(lambda d: d.__setitem__('model_sha256', '0' * 64))))

    def proj():
        o2 = dict(old)
        o2['pk1'] = copy.deepcopy(old['pk1'])
        e = next(e for e in o2['pk1']['events'] if e['e'] == 'fk_')
        e['l54'] = -e['l54']
        controls(caps, manifest, o2)
    T.append(('projection', proj))
    bad = []
    for name, t in T:
        try:
            t()
            bad.append(name)
            print(f'  T {name}: NOT REJECTED')
        except Fail as e:
            print(f'  T {name}: rejected ({str(e)[:110]})')
    return bad


def variants(caps, tot):
    bad = []
    for v, key in VARIANTS.items():
        expect = tot[key] > 0
        caught = None
        for case in CASES:
            try:
                check_capture(caps[(case, 1)], case, frozenset([v]))
            except Fail as e:
                caught = f'{case}: {str(e)[:90]}'
                break
        ok = (caught is not None) == expect
        print(f'  V {v} ({key}={tot[key]}): {"rejected " + caught if caught else "survives"}' + ('' if ok else '  <-- UNEXPECTED'))
        if not ok:
            bad.append(v)
    return bad


def write_manifest():
    files = {}
    models = {}
    for c in CASES:
        for r in RUNS:
            fn = f'{c}_{r}.json.gz'
            files[fn] = sha(os.path.join(DATA, fn))
            models[c] = load(os.path.join(DATA, fn))['model_sha256']
    m = {'task': 'M2-06f-4', 'schema': 1, 'tool': 'tools/mip_infeas_ray.js', 'tool_sha256': sha(TOOL), 'binary_sha256': BINARY,
         'params': PARAMS, 'models': models, 'model_paths': '../re/bench/data/miplib/<case>.mps.gz', 'files': files}
    p = os.path.join(DATA, 'manifest.json')
    if os.path.exists(p):
        raise SystemExit('manifest exists')
    json.dump(m, open(p, 'w'), indent=1)
    print('wrote', p)


def main(argv):
    if argv[:1] == ['--write-manifest']:
        write_manifest()
        return 0
    if argv[:1] == ['--sweep']:
        bad = 0
        for p in argv[1:]:
            d = load(p)
            try:
                c = check_capture(d, os.path.basename(p))
                res = c.pop('_res_max')
                print(os.path.basename(p), d['output'], ' '.join(f'{k}={v}' for k, v in sorted(c.items())),
                      f'res_max={res:.3g}' if res is not None else '')
            except Fail as e:
                bad += 1
                print(os.path.basename(p), 'FAIL', e)
        return 1 if bad else 0
    try:
        caps, old, manifest = load_all()
        controls(caps, manifest, old)
        print('controls: PASS (manifest, 3 runs equal, output line and fk/fk_ projection equal to M2-06f-2 run 1)')
        tot = acceptance(caps)
    except Fail as e:
        print('FAIL', e)
        return 1
    print('ACCEPTANCE PASS')
    if '--self-test' in argv:
        bt = tampers(caps, old, manifest)
        bv = variants(caps, tot)
        if bt or bv:
            print('SELF-TEST FAIL', bt, bv)
            return 1
        print('ACCEPTANCE PASS (+ self-test)')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
