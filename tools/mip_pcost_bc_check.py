#!/usr/bin/env python3
"""M2-06e-2: pseudocost table B update (5fb8ec C:800-858) and table C writers/readers against captures of
tools/mip_pcost_bc.js. Contract: docs/m2/mip_pseudocost_bc.md section 3.

(default)          B1-B7, C1-C2, R on bench/oracle/m2_06e2
--write-manifest   record the SHA-256 of every capture (once, after the capture)
--self-test        also T (capture tampers) and V (rule variants, outcomes fixed in section 3)
--sweep FILE...    replay on single captures and print the branch counts (not acceptance)
Exit code 0 only when every requested check passes.
"""
import collections, copy, json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mip_pseudocost_check import Fail, need, sha, f, hx, load, maxsd, upd_rule, ZERO  # noqa: E402  (06e rules)

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = os.path.join(ROOT, 'bench/oracle/m2_06e2')
OLD = os.path.join(ROOT, 'bench/oracle/m2_06e')  # same cases and params, B2 non-interference
TOOL = os.path.join(ROOT, 'tools/mip_pcost_bc.js')
BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
PARAMS = {'Threads': 1, 'Seed': 0, 'LogToConsole': 0, 'NodeLimit': 2000}
RUNS = (1, 2, 3)
CASES = ('markshare_4_0', 'pk1', 'neos5', 'tr12-30', 'fhnw-binpack4-4', 'neos-860300')
NEW_EV = ('bgate', 'bblk', 'bend')
RA_UP, RA_DN = 0x514de8, 0x514e2f
C_TYPE = 0x43
EMPTY = [0, 0, ZERO, ZERO]


def neg(h):
    """-x by flipping the sign bit (xorps with 0x2a6c460)."""
    return '%016x' % (int(h, 16) ^ (1 << 63))


def apply(cur, part):
    if 'full' in part:
        return list(part['full'])
    out = list(cur)
    for j, v in part['d']:
        out[j] = v
    return out


def predict(a, ent, V, st=None):
    """Section 2.2 loop over j < n: list of (j, ra, fx, delta). ent(j) is the replayed table B entry."""
    calls = []
    order = range(a['n'] - 1, -1, -1) if 'V10' in V else range(a['n'])
    cap = 4 if 'V1' in V else 6 if 'V2' in V else 5
    fu = 0.5 if 'V6' in V else 1.0
    for j in order:
        vt, b = a['vt'][j], a['vb'][j]
        lb, ub, rc = f(a['lb'][j]), f(a['ub'][j]), a['rc'][j]
        c_ok = vt != C_TYPE or 'V4' in V
        bnd = (ub >= lb) if 'V5' in V else (ub > lb)
        if st is not None and b == -3:
            st['basis_m3'] += 1
        if st is not None and b in (-1, -2):  # branch counts of the original rule (V uses None)
            d = 1 if b == -1 else 0
            cnt = ent(j)[d]
            if vt == C_TYPE and ub > lb and cnt < 5:
                st['C_pass'] += 1
            if vt != C_TYPE and not ub > lb and ub >= lb and cnt < 5:
                st['fixed_pass'] += 1
        if not c_ok:
            if st is not None:
                st['skip_C'] += 1
            continue
        if not bnd:
            if st is not None:
                st['skip_bound'] += 1
            continue
        if b not in (-1, -2):
            if st is not None:
                st['skip_basic' if b >= 0 else 'skip_other'] += 1
            continue
        up = (b == -1) != ('V3' in V)
        cnt = ent(j)[1 if up else 0]
        if not cnt < cap:
            if st is not None:
                st['skip_cap'] += 1
            continue
        if up:
            dx = f(rc) if 'V8' in V else maxsd(f(rc), 0.0)
            calls.append((j, RA_UP, fu, dx))
        else:
            src = rc if 'V7' in V else neg(rc)
            dx = f(src) if 'V8' in V else maxsd(f(src), 0.0)
            calls.append((j, RA_DN, -fu, dx))
        if st is not None:
            st['call_up' if up else 'call_dn'] += 1
            if cnt == 4:
                st['cnt4'] += 1
            if up and not f(rc) > 0:
                st['up_rc_le0'] += 1
            if not up and not f(neg(rc)) > 0:
                st['dn_rc_ge0'] += 1
            if up and f(rc) < 0:
                st['up_rc_neg'] += 1  # report only: rc sign against the bound (delta floors to 1e-10)
            if not up and f(rc) > 0:
                st['dn_rc_pos'] += 1
            if not up and maxsd(maxsd(f(rc), 0.0), 1e-10) != maxsd(maxsd(f(neg(rc)), 0.0), 1e-10):
                st['dn_rc_big'] += 1
    return calls


def replay(d, V=frozenset(), name=''):
    """B3-B6 over one capture. Returns the branch counts of the original rule (only meaningful with V empty)."""
    st = collections.Counter()
    slot = {}                 # table serial -> slot
    tabB = {}                 # table B serial -> {j: entry}
    arrs = {}                 # table B serial -> reconstructed column arrays
    gate = blk = None
    exp = None                # predicted calls still expected in the open window
    last_gate = None          # 06e node-update gate (report only: same node as the next bgate)
    for i, e in enumerate(d['events']):
        k = e['e']
        if k == 'gate':
            last_gate = e
        where = f'{name} event {i} ({k})'
        if k == 'new' and e.get('t', -2) >= 0:
            slot[e['t']] = e.get('slot') if e['f'] == '6ec3da' else 'copy'
            if slot[e['t']] == 0x3258:
                need(not e['nz'], f'B6 {where}: new table B not empty')
                tabB[e['t']] = {}
            need(not (e['f'] != '6ec3da' and e.get('slot') == 0x3258), f'B6 {where}: table B copy (out of scope)')
        elif k == 'bgate':
            need(gate is None, f'B3 {where}: bgate inside an open block')
            gate, blk, exp = e, None, None
            st['bgate'] += 1
            same = last_gate is not None and (last_gate['id'], last_gate['nest'], last_gate['st']) == (e['id'], e['nest'], e['st'])
            st['bgate_same_node' if same else 'bgate_other_node'] += 1
            last_gate = None
            if e['work'] is None:
                st['work_null'] += 1
            if e['st'] != 2:
                st['gate_st_ne2'] += 1
            elif not e['has98']:
                st['gate_no98'] += 1
            elif e['tB'] < 0:
                st['gate_noB'] += 1
        elif k == 'bblk':
            need(gate is not None and blk is None, f'B3 {where}: bblk without its bgate')
            need(gate['st'] == 2 and gate['has98'] and gate['tB'] >= 0, f'B3 {where}: bblk for gate st {gate["st"]} has98 {gate["has98"]}')
            need(e['t'] == e['tB'] == gate['tB'] and e['t'] in tabB, f'B3 {where}: table rdi {e["t"]} != M table B {e["tB"]}')
            a = arrs.setdefault(e['t'], {'vt': [], 'lb': [], 'ub': [], 'vb': [], 'rc': []})
            a['n'] = e['n']
            for key in ('vt', 'lb', 'ub', 'vb', 'rc'):
                if e['n'] > 0:
                    a[key] = apply(a[key], e[key])
                need(len(a[key]) == max(e['n'], 0) or e['n'] <= 0, f'B3 {where}: {key} length')
            t = tabB[e['t']]
            calls = predict(a, lambda j: t.get(j, EMPTY), V, None if V else st)
            exp = collections.deque(calls)
            blk = (e, len(calls))
            st['bblk'] += 1
            st['bblk_nest%d' % e['nest']] += 1
            if e['n'] <= 0:
                st['n_le0'] += 1
            if not calls:
                st['blk_nocall'] += 1
            if len(calls) >= 2:
                st['blk_2calls'] += 1
            if gate['work'] is not None and len(calls) != e['n']:
                st['calls_ne_n'] += 1
        elif k == 'upd' and slot.get(e['t']) == 0x3258:
            need(e['from'] == '5fb8ec' and e['ra'] in (RA_UP, RA_DN), f'B4 {where}: table B written from {e["from"]} {e["ra"]:#x}')
            need(exp is not None, f'B4 {where}: table B update outside a block window')
            need(exp, f'B4 {where}: unpredicted table B update j {e["j"]} ra {e["ra"]:#x}')
            j, ra, fx, dx = exp.popleft()
            need((e['j'], e['ra']) == (j, ra), f'B4 {where}: got j {e["j"]} ra {e["ra"]:#x}, rule j {j} ra {ra:#x}')
            t = tabB[e['t']]
            cur = t.get(j, EMPTY)
            need(e['before'] == cur, f'B4 {where}: before {e["before"]} != replay {cur}')
            want = upd_rule(cur, fx, dx, set())
            need(e['after'] == want, f'B4 {where}: after {e["after"]} != rule {want} (f {fx} delta {dx!r})')
            t[j] = want
        elif k == 'bend':
            need(gate is not None, f'B3 {where}: bend without bgate')
            ran = blk is not None
            need(ran == (gate['st'] == 2 and bool(gate['has98']) and gate['tB'] >= 0), f'B3 {where}: block ran {ran}')
            if ran:
                need(not exp, f'B4 {where}: {len(exp)} predicted calls missing, first {list(exp)[:1]}')
            if gate['work'] is None:
                need(e['work'] is None, f'B5 {where}: work appeared')
            else:
                w = f(gate['work'])
                if ran:
                    w = (float(blk[1]) if 'V9' in V else float(blk[0]['n'])) * 5.0 + w
                need(e['work'] == hx(w), f'B5 {where}: work {e["work"]} != rule {hx(w)}')
            gate = blk = exp = None
        elif k == 'free' and slot.get(e['t']) == 0x3258:
            t = tabB.pop(e['t'])
            got = [[j] + list(v) for j, v in sorted(t.items()) if v != EMPTY]
            if got != e['nz']:
                first = next((a, b) for a, b in zip(got + [None], e['nz'] + [None]) if a != b)
                raise Fail(f'B6 {where}: free snapshot != replay (first replay/capture {first})')
            st['free_B'] += 1
    need(gate is None, f'B3 {name}: capture ends inside a block')
    need(not tabB, f'B6 {name}: {len(tabB)} table B never freed')
    return st


def table_c(d, name, st):
    """C1-C2: the table C updates as 6edfcf calls with the section 2.3 deltas."""
    slot = {e['t']: e.get('slot') for e in d['events'] if e['e'] == 'new' and e.get('t', -2) >= 0 and e['f'] == '6ec3da'}
    for i, e in enumerate(d['events']):
        if e['e'] != 'upd' or slot.get(e['t']) != 0x3260:
            continue
        b, a, where = e['before'], e['after'], f'{name} event {i} ({e["from"]} {e["ra"]:#x})'
        if e['from'] == '67eb3e':
            fx = {0x5634af: 1.0, 0x5634cb: -1.0}.get(e['ra'])
            need(fx is not None, f'C1 {where}: unknown 67eb3e site')
            need(a == upd_rule(b, fx, 0.0, set()), f'C1 {where}: after {a} != 6edfcf({fx}, 0)')
            st['c_67eb3e'] += 1
            continue
        need(e['from'] == '6755af', f'C2 {where}: table C written by an unmapped caller')
        up = a[1] == b[1] + 1 and a[0] == b[0]
        need(up or (a[0] == b[0] + 1 and a[1] == b[1]), f'C2 {where}: not a one-direction step {b} -> {a}')
        d_ = 1 if up else 0
        fx = 1.0 if up else -1.0
        s0, s1 = f(b[2 + d_]), f(a[2 + d_])
        if b[d_] == 0 and s1 == s0 and s0 != 0:
            st['c_ambiguous'] += 1  # max kept the old sum
            continue
        g = s1 if b[d_] == 0 else s1 - s0
        k0 = max(0, round(g * 1.001))
        cands = [0.0, 2e9] + [float(x) for x in range(max(0, k0 - 2), k0 + 3)]
        hit = [x for x in cands if upd_rule(b, fx, x, set()) == a]
        need(hit, f'C2 {where}: no delta in {{0..1e7, 2e9}} reproduces {b} -> {a} (g {g!r})')
        x = hit[0]
        need(x <= 1e7 or x == 2e9, f'C2 {where}: delta {x}')
        st['c_6755af_' + ('2e9' if x == 2e9 else 'zero' if x <= 0 else 'int')] += 1
        st['c_6755af_site_%x' % e['ra']] += 1


def project(d):
    ev = [e for e in d['events'] if e['e'] not in NEW_EV]
    cnt = {k: v for k, v in d['counts'].items() if not k.startswith('rd:')}
    return ev, cnt, d['output']


def check_case(c, d, old):
    need(d['binary_sha256'] == BINARY, f'B1 {c}: binary sha256 {d["binary_sha256"]}')
    need(d['ok'] and not d['errors'] and d['params'] == PARAMS, f'B1 {c}: ok/errors/params')
    pe, pc, po = project(d)
    oe, oc, oo = project(old)
    need(po == oo, f'B2 {c}: output {po!r} != 06e {oo!r}')
    need(pc == oc, f'B2 {c}: counts {pc} != 06e {oc}')
    need(len(pe) == len(oe), f'B2 {c}: {len(pe)} events != 06e {len(oe)}')
    for i, (x, y) in enumerate(zip(pe, oe)):
        need(x == y, f'B2 {c}: event {i} differs from 06e: {json.dumps(x)[:160]} vs {json.dumps(y)[:160]}')
    st = replay(d, name=c)
    table_c(d, c, st)
    return st


REQUIRED = ('call_up', 'call_dn', 'skip_cap', 'cnt4', 'skip_C', 'skip_bound', 'skip_basic', 'up_rc_le0', 'dn_rc_ge0',
            'gate_st_ne2', 'blk_nocall', 'C_pass', 'fixed_pass', 'dn_rc_big', 'blk_2calls', 'c_67eb3e', 'c_6755af_int')
VARIANTS = {'V1': ('cnt4',), 'V2': ('skip_cap',), 'V3': ('call_up', 'call_dn'), 'V4': ('C_pass',),
            'V5': ('fixed_pass',), 'V6': ('call_up', 'call_dn'), 'V7': ('dn_rc_big',), 'V8': (),
            'V9': ('calls_ne_n',), 'V10': ('blk_2calls',)}


def a1(manifest):
    out, tool = {}, sha(TOOL)
    for c in CASES:
        hs = []
        for r in RUNS:
            p = os.path.join(DATA, f'{c}_{r}.json.gz')
            need(os.path.exists(p), f'B1: missing {p}')
            hs.append(sha(p))
            need(manifest['files'].get(os.path.basename(p)) == hs[-1], f'B1: {p} sha256 != manifest')
        need(len(set(hs)) == 1, f'B1: {c}: runs differ {hs}')
        d = load(os.path.join(DATA, f'{c}_1.json.gz'))
        need(d['tool_sha256'] == tool, f'B1: {c}: capture tool sha256 {d["tool_sha256"]} != current tool {tool}')
        out[c] = d
    return out


def report_map(docs):
    agg = collections.Counter()
    for d in docs.values():
        for k, v in d['counts'].items():
            if k.startswith('rd:'):
                _, fn, frm, ra, t, t2 = k.split(':')
                agg[(fn, frm, t, t2)] += v
    print('R reader map (6 cases, run 1): function caller T T2 calls')
    for (fn, frm, t, t2), v in sorted(agg.items()):
        print(f'  {fn} {frm} T={t} T2={t2} {v}')


def acceptance(docs, olds, quiet=False):
    stats = {}
    for c in CASES:
        stats[c] = check_case(c, docs[c], olds[c])
        if not quiet:
            print(c, 'PASS', json.dumps(dict(sorted(stats[c].items()))))
    tot = {k: sum(st[k] for st in stats.values()) for k in REQUIRED}
    if not quiet:
        print('B7 branch totals', json.dumps(tot))
    need(all(tot.values()), f'B7: a required branch never ran: {[k for k, v in tot.items() if not v]}')
    if not quiet:
        report_map(docs)
    return stats


def tampers(docs, olds):
    """T: each tamper of a run 1 copy must make the acceptance of that case fail."""
    def idx(d, pred):
        return next(i for i, e in enumerate(d['events']) if pred(e))

    def b_upd(e):
        return e['e'] == 'upd' and e['from'] == '5fb8ec' and e['ra'] in (RA_UP, RA_DN)

    def called(d, big=False):
        """(bblk index, j, upd index) of the first called column; with big, the first whose |rc| > 1e-9 (None if none)."""
        def pred(e):  # first update of that direction with g > 1e-9 means |rc| > 1e-9
            if not b_upd(e):
                return False
            u = e['ra'] == RA_UP
            return not big or e['before'][u] == 0 and f(e['after'][2 + u]) > 1e-9
        i = next((k for k, e in enumerate(d['events']) if pred(e)), None)
        if i is None:
            return None
        j = d['events'][i]['j']
        b = max(k for k in range(i) if d['events'][k]['e'] == 'bblk')
        return b, j, i

    def setcol(d, b, key, j, v):
        part = d['events'][b][key]
        if 'full' in part:
            part['full'][j] = v
        else:
            part['d'] = [x for x in part['d'] if x[0] != j] + [[j, v]]
            part['d'].sort()
        # keep the following diffs of the same M relative to the original value
        t = d['events'][b]['t']
        for e in d['events'][b + 1:]:
            if e['e'] == 'bblk' and e['t'] == t:
                if 'd' in e[key] and not any(x[0] == j for x in e[key]['d']):
                    e[key]['d'].append([j, orig[key]])
                    e[key]['d'].sort()
                break

    def arr_at(d, b, key):
        t, cur = d['events'][b]['t'], []
        for e in d['events'][:b + 1]:
            if e['e'] == 'bblk' and e['t'] == t and e['n'] > 0:
                cur = apply(cur, e[key])
        return cur

    orig = {}

    def t1(d):
        e = d['events'][idx(d, b_upd)]
        k = 3 if e['ra'] == RA_UP else 2
        e['after'][k] = hx(f(e['after'][k]) * (1 + 2 ** -52))

    def t2(d):
        del d['events'][idx(d, b_upd)]

    def t3(d):
        b, j, _ = called(d)
        orig['vb'] = arr_at(d, b, 'vb')[j]
        setcol(d, b, 'vb', j, -3 - orig['vb'])  # -1 <-> -2

    def t4(d):
        e = d['events'][idx(d, lambda e: e['e'] == 'bend' and e['work'] is not None)]
        e['work'] = hx(f(e['work']) * (1 + 2 ** -52))

    def t5(d):
        _, _, i = called(d)
        g = max(k for k in range(i) if d['events'][k]['e'] == 'bgate')
        d['events'][g]['st'] = 1

    def t6(d):
        if called(d, big=True) is None:
            return False  # no call with |rc| > 1e-10 (fhnw: every table B delta is 0)
        b, j, _ = called(d, big=True)
        orig['rc'] = arr_at(d, b, 'rc')[j]
        setcol(d, b, 'rc', j, neg(orig['rc']))

    def t7(d):
        i = idx(d, b_upd)
        e = d['events'].pop(i)
        k = idx(d, lambda x: x['e'] == 'bend')
        d['events'].insert(k + 1, e)

    def t8(d):
        d['binary_sha256'] = '0' * 64

    def t9(d):
        d['events'] = [e for e in d['events'] if e['e'] != 'bblk']

    def t10(d):  # target zero times: no table B traffic at all (applied to every case and its 06e reference)
        d['events'] = [e for e in d['events'] if e['e'] not in NEW_EV and not b_upd(e)]
        tb = {e['t'] for e in d['events'] if e['e'] == 'new' and e.get('slot') == 0x3258}
        for e in d['events']:
            if e['e'] == 'free' and e['t'] in tb:
                e['nz'] = []

    def t11(d):
        b, j, _ = called(d)
        orig['ub'] = arr_at(d, b, 'ub')[j]
        setcol(d, b, 'ub', j, arr_at(d, b, 'lb')[j])

    def t13(d):  # free snapshot of a table B
        tb = {e['t'] for e in d['events'] if e['e'] == 'new' and e.get('slot') == 0x3258}
        e = d['events'][idx(d, lambda e: e['e'] == 'free' and e['t'] in tb and e['nz'])]
        e['nz'][0][2] += 1

    def c_upd(fr):
        def g(d):
            tc = {e['t'] for e in d['events'] if e['e'] == 'new' and e.get('slot') == 0x3260}
            i = next((k for k, e in enumerate(d['events']) if e['e'] == 'upd' and e['t'] in tc and e['from'] == fr), None)
            if i is None:
                return False
            e = d['events'][i]
            u = 1 if e['after'][1] != e['before'][1] else 0
            e['after'][2 + u] = hx(f(e['after'][2 + u]) * 1.5 + 0.25)  # off any integer or 2e9 delta
        return g

    def t12(d):
        d['events'][idx(d, lambda e: e['e'] == 'ins')]['heap_n'] += 1

    # (name, tamper, check that must reject). Tampers of 06e events (upd) are run through the B3-B6 replay
    # directly, because B2 (06e projection) would reject them first and hide whether B4 can.
    plan = [('T1 upd after bit', t1, 'B4'), ('T2 drop upd', t2, 'B4'), ('T3 basis -1<->-2', t3, 'B4'),
            ('T4 work bit', t4, 'B5'), ('T5 gate state', t5, 'B3'), ('T6 rc sign', t6, 'B4'),
            ('T7 upd outside window', t7, 'B4'), ('T8 binary sha', t8, 'B1'), ('T9 no bblk', t9, 'B4'),
            ('T11 ub = lb', t11, 'B4'), ('T12 06e event', t12, 'B2'), ('T13 free snapshot B', t13, 'B6'),
            ('T14 67eb3e after', c_upd('67eb3e'), 'C1'), ('T15 6755af after', c_upd('6755af'), 'C2')]
    for name, fn, code in plan:
        na = []
        for c in CASES:
            d = copy.deepcopy(docs[c])
            orig.clear()
            if fn(d) is False:
                na.append(c)
                continue
            try:
                if name[:3] in ('T1 ', 'T2 ', 'T7 ', 'T13', 'T14', 'T15'):  # 06e events: bypass B2
                    table_c(d, c, replay(d, name=c))
                else:
                    check_case(c, d, olds[c])
            except Fail as ex:
                msg = str(ex)
                need(msg.startswith(code), f'{name} ({c}): rejected by the wrong check: {msg[:160]}')
                continue
            raise Fail(f'{name} ({c}): tampered capture passed')
        need(len(na) < len(CASES), f'{name}: applicable to no case')
        print(f'{name}: rejected by {code} in {len(CASES) - len(na)} cases' + (f', not applicable in {na}' if na else '')
              + f' (e.g. {msg[:140]})')
    # T10: the same zero-target input on both sides (so B2 still holds) must fail B7
    td, to = {}, {}
    for c in CASES:
        td[c], to[c] = copy.deepcopy(docs[c]), copy.deepcopy(olds[c])
        t10(td[c])
        t10(to[c])
    try:
        acceptance(td, to, quiet=True)
    except Fail as ex:
        print(f'T10 no table B traffic: rejected ({str(ex)[:140]})')
    else:
        raise Fail('T10 no table B traffic: zero-target input passed')


def variants(docs, stats):
    for v, keys in VARIANTS.items():
        for c in CASES:
            want = sum(stats[c][k] for k in keys) > 0
            try:
                replay(docs[c], V=frozenset([v]), name=c)
                got = False
            except Fail:
                got = True
            need(got == want, f'V {v} ({c}): rejected {got}, expected {want} from {keys}')
        rej = [c for c in CASES if sum(stats[c][k] for k in keys) > 0]
        need(bool(rej) == (v != 'V8'), f'V {v}: rejected in {rej}; only V8 may survive everywhere')
        print(f'V {v}: rejected in {rej}')


def main():
    args = sys.argv[1:]
    if args[:1] == ['--sweep']:
        bad = 0
        for p in args[1:]:
            try:
                d = load(p)
                st = replay(d, name=os.path.basename(p))
                table_c(d, os.path.basename(p), st)
                print(os.path.basename(p), 'PASS', json.dumps(dict(sorted(st.items()))))
                report_map({p: d})
            except Fail as ex:
                bad += 1
                print(os.path.basename(p), 'FAIL', ex)
        return 1 if bad else 0
    mpath = os.path.join(DATA, 'manifest.json')
    try:
        if args[:1] == ['--write-manifest']:
            files = {}
            for c in CASES:
                for r in RUNS:
                    p = os.path.join(DATA, f'{c}_{r}.json.gz')
                    files[os.path.basename(p)] = sha(p)
            with open(mpath, 'w') as fh:
                json.dump({'task': 'M2-06e-2', 'schema': 1, 'tool': 'tools/mip_pcost_bc.js', 'tool_sha256': sha(TOOL),
                           'binary_sha256': BINARY, 'params': PARAMS,
                           'models': '../re/bench/data/miplib/<case>.mps.gz', 'files': files}, fh, indent=1)
            print('manifest written', mpath)
            return 0
        need(os.path.exists(mpath), 'B1: no manifest (run --write-manifest once after the capture)')
        docs = a1(json.load(open(mpath)))
        olds = {c: load(os.path.join(OLD, f'{c}_1.json.gz')) for c in CASES}
        stats = acceptance(docs, olds)
        if '--self-test' in args:
            tampers(docs, olds)
            variants(docs, stats)
        print('ACCEPTANCE PASS' + (' (+ self-test)' if '--self-test' in args else ''))
        return 0
    except Fail as ex:
        print('FAIL', ex)
        return 1


if __name__ == '__main__':
    sys.exit(main())
