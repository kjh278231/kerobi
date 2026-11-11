#!/usr/bin/env python3
"""M2-06c checker (docs/m2/mip_plunge_rule.md): replay the 6e8eb3 plunge rule number, its RNG draw, the rule 3/12 pick
and the tail cleanup from bench/oracle/m2_06c captures (tools/mip_plunge_rule.js).

  python3 tools/mip_plunge_rule_check.py                    controls (C0) + acceptance (R G S T) + variants
  python3 tools/mip_plunge_rule_check.py --self-test        the above, then tampers that must each be rejected
  python3 tools/mip_plunge_rule_check.py --write-manifest   record the SHA-256 of every capture (once, after the capture)
  python3 tools/mip_plunge_rule_check.py --sweep F.json.gz  replay one capture (exploration, not a verdict)
"""
import copy, gzip, hashlib, json, math, os, struct, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = os.path.join(ROOT, 'bench/oracle/m2_06c')
OLD = os.path.join(ROOT, 'bench/oracle/m2_06a')
TOOL = os.path.join(ROOT, 'tools/mip_plunge_rule.js')
MODELS = os.path.join(ROOT, '../re/bench/data/miplib')
BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
BASE = {'Threads': 1, 'Seed': 0, 'LogToConsole': 0, 'NodeLimit': 2000}
CASES = {'markshare_4_0': ('markshare_4_0', {}), 'pk1': ('pk1', {}), 'neos5': ('neos5', {}), 'tr12-30': ('tr12-30', {}),
         'pk1-pool': ('pk1', {'PoolSearchMode': 2})}
NEW = {'pin', 's70', 'drw', 'rpre', 'rule', 'pick', 'tail', 'anc', 'thr', 'rm', 'tend'}
VARIANTS = {
    'V1': 'rule 3/12 sign swapped', 'V2': 'lexicographic (depth, est)', 'V3': 'minEst over non-excluded only',
    'V4': 'tie goes to last', 'V5': 'seed from post-pop k', 'V6': 'draw cutoff 0.5', 'V7': 'delta=2 clause off',
    'V8': 'delta=6 clause off', 'V9': 'log branch rounds', 'V10': 'keep ignores ancestor', 'V11': 'keep ignores bound',
    'V12': 'removal always to heap', 'V13': 'delta=10 clause off', 'V14': 'delta=3 clause off', 'V15': "rule clause s==0 off",
    'V16': 'k%7 ignores 5e6efd (always 4)'}
E100, M100 = 1e100, -1e100


class Fail(Exception):
    pass


def need(cond, msg):
    if not cond:
        raise Fail(msg)


def sha(p):
    with open(p, 'rb') as fh:
        return hashlib.sha256(fh.read()).hexdigest()


def load(p):
    with gzip.open(p) as fh:
        return json.load(fh)


def f(hx):
    need(hx is not None, 'null double')
    return struct.unpack('>d', bytes.fromhex(hx))[0]


def h(x):
    return struct.pack('>d', x).hex()


M64 = (1 << 64) - 1


def lcg(s):
    return (s * 0x5851f42d4c957f2d + 0x14057b7ef767814f) & M64


def temper(s):
    u = s >> 32
    u ^= u >> 11
    u ^= (u & 0x13a58ad) << 7
    u &= 0xffffffff
    u ^= (u & 0x1df8c) << 15
    u &= 0xffffffff
    return ((u ^ (u >> 18)) & 0x7fffffff) * 4.656612873077393e-10


def seed_of(k, w28):
    x = float(k) + w28                        # cvtsi2sd + addsd
    if x < 9.223372036854776e18:
        return int(x) & M64                    # cvttsd2si (x >= 0 here)
    return (int(x - 9.223372036854776e18) | (1 << 63)) & M64


def cmod(a, b):                                # C remainder (sign of the dividend)
    r = abs(a) % b
    return -r if a < 0 else r


def minsd(a, b):                               # minsd a, b -> a < b ? a : b
    return a if a < b else b


def maxsd(a, b):
    return a if a > b else b


# ---------------------------------------------------------------- call records
def calls(d):
    """Group the top-level 6e8eb3 events: pin, enter, ..., leave."""
    out, cur = [], None
    for e in d['events']:
        t = e['e']
        if t == 'pin':
            need(cur is None, 'pin inside a call')
            cur = {'pin': e, 'ev': []}
            continue
        if cur is None:
            need(t not in NEW, f'{t} outside a call')
            continue
        cur['ev'].append(e)
        if t == 'leave':
            out.append(cur)
            cur = None
    need(cur is None, 'unterminated call')
    return out


def one(c, t):
    xs = [e for e in c['ev'] if e['e'] == t]
    need(len(xs) <= 1, f'{t} more than once')
    return xs[0] if xs else None


# ---------------------------------------------------------------- R / G
def rule_replay(c, V):
    p, r = c['pin'], one(c, 'rule')
    k, ke = r['esi'], p['k']
    s70 = [e for e in c['ev'] if e['e'] == 's70']
    calls_seen = [(e['f'], e['r']) for e in s70]
    need(calls_seen and calls_seen[0][0] == '5e70c1', '5e70c1 not called first')
    s = calls_seen[0][1]
    expect_calls = ['5e70c1']
    hint = p['ign_hint'] == 0 and p['hint'] != 0 and ke < 10
    seed = seed_of(r['esi'] if 'V5' in V else ke, f(p['w28']))
    draw = None
    mx40 = f(p['mx40'])
    if mx40 == M100 and p['ncols'] == p['nint']:
        rr = 11
    elif hint:
        rr = 3
    elif p['m848'] != 0:
        rr = 6
    elif p['bdir'] != 0:
        rr = 9
    elif k <= 8:
        rr = 6 if p['mx808'] == 0 else 1
    elif k <= 16 or (s == 0 and 'V15' not in V) or cmod(k, 11) == 0:
        rr = 2
    elif cmod(k, 7) == 0:
        expect_calls.append('5e6efd')
        need(len(calls_seen) == 2, '5e6efd call missing')
        rr = 4 if (calls_seen[1][1] != 0 or 'V16' in V) else 2
    elif cmod(k, 13) == 0:
        rr = 5
    elif cmod(k, 19) == 0:
        rr = 6 if p['mx808'] == 0 else 1
    elif cmod(k, 43) == 9:
        rr = 7
    elif cmod(k, 2) != 0:
        after = lcg(seed)
        u = temper(after)
        draw = (seed, after, u)
        rr = 3 if u < (0.5 if 'V6' in V else 0.9) else 12
    else:
        rr = 3
    need([x[0] for x in calls_seen] == expect_calls, f'helper calls {calls_seen} != {expect_calls}')
    pre = rr
    if p['focus'] == 4:
        rr = 5
    return pre, rr, seed, draw, hint


def check_rule(c, V, st):
    r, rp = one(c, 'rule'), one(c, 'rpre')
    need(r is not None and rp is not None, 'rule/rpre missing')
    pre, rr, seed, draw, hint = rule_replay(c, V)
    need(rp['ecx'] == pre, f'R: rule before focus {pre} != captured {rp["ecx"]}')
    need(r['ecx'] == rr, f'R: rule {rr} != captured {r["ecx"]}')
    need(r['r8b'] == (1 if hint else 0), 'R: hint flag')
    need(r['r8b'] == 0, 'hint path is out of scope')
    need(r['esi'] == rp['esi'], 'R: k changed between probes')
    drw = one(c, 'drw')
    if draw is None:
        need(drw is None, 'G: unexpected draw')
        need(int(r['st'], 16) == seed, f'G: state {r["st"]} != seed {seed:016x}')
    else:
        need(drw is not None, 'G: draw missing')
        need(int(drw['st'], 16) == draw[0], f'G: seed {drw["st"]} != {draw[0]:016x}')
        need(int(r['st'], 16) == draw[1], 'G: state after the draw')
        st['draws'] += 1
    st['rule'][rr] = st['rule'].get(rr, 0) + 1
    return rr


# ---------------------------------------------------------------- S
def pick_replay(ents, rule, V):
    n = len(ents)
    m = E100
    ests = [f(e[1]) for e in ents]
    src = [x for x, e in zip(ests, ents) if not e[3]] if 'V3' in V else ests
    j = 0
    while j + 1 < len(src):                    # pairs: minsd, then comisd/ja
        m = minsd(m, src[j])
        b = src[j + 1]
        m = m if b > m else b
        j += 2
    if j < len(src):
        m = minsd(m, src[j])
    best, idx = M100, -1
    bkey = None
    for j, e in enumerate(ents):
        if e[3]:
            continue
        dep = float(e[2])
        if 'V2' in V:
            key = (e[2], -ests[j] if (rule == 3) != ('V1' in V) else ests[j])
            if bkey is None or key > bkey or ('V4' in V and key == bkey):
                bkey, idx = key, j
            continue
        q = (ests[j] - m) / 1e8
        sign = -1 if rule == 3 else 1
        if 'V1' in V:
            sign = -sign
        sc = dep + q if sign > 0 else dep - q
        if sc > best or ('V4' in V and sc == best):
            idx = j
        best = maxsd(sc, best)
    return (0 if idx < 0 else idx), n


def discriminating(ents):
    live = [e for e in ents if not e[3]]
    if len(live) < 2:
        return False
    mx = max(e[2] for e in live)
    top = {e[1] for e in live if e[2] == mx}
    return len(top) > 1


def check_pick(c, rule, V, st):
    pk, r, rp = one(c, 'pick'), one(c, 'rule'), one(c, 'rpre')
    need(pk is not None, 'pick missing')
    ents = pk['list']
    need(len(ents) == r['n'], 'S: list size')
    if rule not in (3, 12):
        return pk
    idx, n = pick_replay(ents, rule, V)
    need(pk['idx'] == idx, f'S: rule {rule} pick {idx} != captured {pk["idx"]}')
    if rp['work'] is not None:
        w0, w1, w2 = f(rp['work']), f(r['work']), f(pk['work'])
        need(float(n) * 3.0 + w0 == w1, 'S: minEst work')
        need(float(n) * 4.0 + w1 == w2, 'S: score work')
    st['r312'] += 1
    st['disc'] += discriminating(ents)
    return pk


# ---------------------------------------------------------------- T
def walk(chain, depth, d):
    steps = depth - d
    if (d | steps) < 0:
        return None
    return chain[steps] if len(chain) > steps else None


def delta(t, V):
    if t['esi'] <= t['m2c'] and 'V7' not in V:
        return 2, 'd2'
    if t['focus'] == 4 and 'V14' not in V:
        return 3, 'd3'
    if t['t138'] > 0 and t['t120'] >= 10000 and t['k'] >= 1000 and 'V13' not in V:
        return 10, 'd10'
    if cmod(t['k'], 23) == 7 and 'V8' not in V:
        return 6, 'd6'
    x = math.log(float(t['depth']) * 0.2 + 1.0) / math.log(2.0)
    v = int(round(x)) if 'V9' in V else int(x)
    return min(max(v, 3), 10), 'log'


def check_tail(c, pk, V, st):
    t, te = one(c, 'tail'), one(c, 'tend')
    need(t is not None and te is not None, 'tail/tend missing')
    need(t['rbp_m'] == 1, 'tail: rbp is not M')
    ents = [list(e) for e in pk['list']]
    idx = pk['idx']
    need(t['n'] == len(ents), 'T: n at the tail')
    sel = ents[idx]
    need(t['sel'] == sel[0] and t['depth'] == sel[2] and t['x41'] == sel[3], 'T: selected node')
    leave = c['ev'][-1]
    need(leave['node'] is not None and leave['node'][0] == sel[0], 'T: returned node')
    ents[idx] = ents[-1]
    ents.pop()
    dl, kind = delta(t, V)
    st['delta'][dl] = st['delta'].get(dl, 0) + 1
    st['dkind'][kind] = st['dkind'].get(kind, 0) + 1
    dp = t['depth'] - dl
    ancs = [e for e in c['ev'] if e['e'] == 'anc']
    need(ancs and ancs[0]['w'] == 'sel', 'T: selected 5fb13c missing')
    a0 = ancs[0]
    need(a0['d'] == dp, f'T: d\' {dp} != captured {a0["d"]}')
    need(a0['chain'][0] == sel[0] and a0['depth'] == t['depth'], 'T: selected chain start')
    anc = walk(a0['chain'], a0['depth'], dp)
    need(anc == a0['res'], f'T: anc {anc} != captured {a0["res"]}')
    thr_e = one(c, 'thr')
    rest = [e for e in c['ev'][c['ev'].index(a0) + 1:] if e['e'] in ('anc', 'rm', 'ins', 'thr')]
    if anc is None:
        need(thr_e is None and not rest, 'T: cleanup after a null ancestor')
        need(te['list'] == [e[0] for e in ents] and te['n'] == len(ents), 'T: list after a null ancestor')
        need(te['work'] == pk['work'], 'T: work after a null ancestor')
        st['null'] += 1
        return
    need(thr_e is not None and rest and rest[0] is thr_e, 'T: threshold probe')
    need(thr_e['d'] == dp and thr_e['anc'] == anc, 'T: threshold frame d\'/anc')
    for k in ('t20', 't28', 't00', 'v8239', 'f9a41'):
        need(thr_e.get(k) is not None, f'T: thr {k} null')
    thr0 = E100
    if thr_e['f9a41'] != 0 and f(thr_e['t00']) > f(thr_e['v8239']) - 1e-10:
        thr0 = minsd(thr0, f(thr_e['t00']))
    need(h(thr0) == thr_e['t20'], f'T: threshold {h(thr0)} != captured {thr_e["t20"]}')
    if thr0 < E100:
        st['thr_set'] += 1
    thr = M100 if t['x41'] else thr0
    cut = f(thr_e['t28'])
    rest = rest[1:]
    kept = []
    for e in ents:
        need(rest and rest[0]['e'] == 'anc' and rest[0]['w'] == 'it', 'T: item 5fb13c missing')
        a = rest.pop(0)
        need(a['chain'][0] == e[0] and a['d'] == dp and a['bound'] == e[4] and a['depth'] == e[2], 'T: item chain start')
        res = walk(a['chain'], a['depth'], dp)
        need(res == a['res'], f'T: item anc {res} != captured {a["res"]}')
        b = f(e[4])
        same = res == anc or 'V10' in V
        if same and (not (b > thr) or 'V11' in V):
            kept.append(e[0])
            st['kept'] += 1
            continue
        if res == anc:
            st['by_bound'] += 1
        if cut > b or 'V12' in V:
            need(len(rest) >= 2 and rest[0]['e'] == 'rm' and rest[0]['node'] == e[0] and rest[0]['stage'] == 4, 'T: stage-4 release')
            need(rest[1]['e'] == 'ins' and rest[1]['from'] == 'raw:5b47b7' and rest[1]['node'][0] == e[0], 'T: heap insert')
            rest = rest[2:]
            st['heap'] += 1
        else:
            need(rest and rest[0]['e'] == 'rm' and rest[0]['node'] == e[0] and rest[0]['stage'] == 0, 'T: drop')
            rest = rest[1:]
            st['drop'] += 1
    need(not rest, f'T: extra events {rest[:1]}')
    need(te['list'] == kept and te['n'] == len(kept), 'T: list after cleanup')
    if pk['work'] is not None:
        need(float(len(ents)) * 4.0 + f(pk['work']) == f(te['work']), 'T: tail work')


def new_stats():
    return {'rule': {}, 'draws': 0, 'r312': 0, 'disc': 0, 'delta': {}, 'dkind': {}, 'kept': 0, 'heap': 0, 'drop': 0,
            'null': 0, 'thr_set': 0, 'by_bound': 0, 'calls': 0, 'empty': 0}


def check_capture(d, name, V=frozenset()):
    st = new_stats()
    for i, c in enumerate(calls(d)):
        try:
            st['calls'] += 1
            if one(c, 'rule') is None:          # list empty after pops: nothing selected
                need(c['ev'][-1]['node'] is None and not any(e['e'] in NEW for e in c['ev']), 'empty call with events')
                st['empty'] += 1
                continue
            rule = check_rule(c, V, st)
            pk = check_pick(c, rule, V, st)
            check_tail(c, pk, V, st)
        except Fail as ex:
            raise Fail(f'{name} call {i} (sel {c["pin"].get("k")}): {ex}')
    return st


# ---------------------------------------------------------------- controls
def proj(d):
    ev = [e for e in d['events'] if e['e'] not in NEW]
    return ev, d['counts']


def controls(caps, manifest, old):
    need(manifest['binary_sha256'] == BINARY, 'manifest binary hash')
    need(manifest['tool_sha256'] == sha(TOOL), 'manifest tool hash differs from tools/mip_plunge_rule.js')
    for case, runs in caps.items():
        model, extra = CASES[case]
        need(len(runs) == 3, f'{case}: runs')
        for r, d in enumerate(runs, 1):
            nm = f'{case}_{r}'
            need(d['binary_sha256'] == BINARY, f'{nm}: binary hash')
            need(d['tool_sha256'] == manifest['tool_sha256'], f'{nm}: tool hash')
            need(d['model_sha256'] == manifest['models'][model], f'{nm}: model hash')
            need(d['params'] == dict(BASE, **extra), f'{nm}: params')
            need(d['ok'] and not d['errors'] and d['output'] == 'DONE 8 2000', f'{nm}: run output {d["output"]}')
            need(d['events'], f'{nm}: no events')
            need(d['events'] == runs[0]['events'] and d['counts'] == runs[0]['counts'], f'{nm}: differs from run 1')
        if case in old:
            ev, cnt = proj(runs[0])
            o = old[case]
            need(cnt == o['counts'], f'{case}: counts differ from m2_06a')
            for i, (a, b) in enumerate(zip(ev, o['events'])):
                need(a == b, f'{case}: projection event {i} differs from m2_06a: {a} != {b}')
            need(len(ev) == len(o['events']), f'{case}: projection length differs from m2_06a')


def load_all():
    manifest = json.load(open(os.path.join(DATA, 'manifest.json')))
    caps, old = {}, {}
    for case in CASES:
        runs = []
        for r in (1, 2, 3):
            fn = f'{case}_{r}.json.gz'
            need(manifest['files'][fn] == sha(os.path.join(DATA, fn)), f'{fn}: file hash differs from the manifest')
            runs.append(load(os.path.join(DATA, fn)))
        caps[case] = runs
        if case in ('markshare_4_0', 'pk1', 'neos5', 'tr12-30'):
            old[case] = load(os.path.join(OLD, f'{case}_1.json.gz'))
    return caps, old, manifest


TARGETS = [('rule 3 calls', lambda s: s['rule'].get(3, 0)), ('rule 12 calls', lambda s: s['rule'].get(12, 0)),
           ('draws', lambda s: s['draws']), ('discriminating 3/12 picks', lambda s: s['disc']),
           ('kept items', lambda s: s['kept']), ('heap items', lambda s: s['heap']),
           ('delta=2', lambda s: s['dkind'].get('d2', 0)), ('delta=6', lambda s: s['dkind'].get('d6', 0)),
           ('delta log > 3', lambda s: sum(v for k, v in s['delta'].items() if k > 3 and k != 6 and k != 10))]


def total(stats):
    t = new_stats()
    for s in stats.values():
        for k, v in s.items():
            if isinstance(v, dict):
                for a, b in v.items():
                    t[k][a] = t[k].get(a, 0) + b
            else:
                t[k] += v
    return t


def acceptance(caps, quiet=False):
    stats = {case: check_capture(runs[0], case) for case, runs in caps.items()}
    tot = total(stats)
    if not quiet:
        for case, s in stats.items():
            print(f'  {case}: R/G/S/T PASS calls {s["calls"]} (empty {s["empty"]}) rules {dict(sorted(s["rule"].items()))} '
                  f'draws {s["draws"]} 3/12 {s["r312"]} disc {s["disc"]} delta {dict(sorted(s["delta"].items()))} '
                  f'kinds {s["dkind"]} null-anc {s["null"]} kept {s["kept"]} heap {s["heap"]} drop {s["drop"]} '
                  f'thr<1e100 {s["thr_set"]} removed-by-bound {s["by_bound"]}')
    for label, fn in TARGETS:
        need(fn(tot) > 0, f'acceptance: no {label}')
    return stats, tot


def variants(caps):
    rows = {}
    for v in VARIANTS:
        rej = []
        for case, runs in caps.items():
            try:
                check_capture(runs[0], case, frozenset([v]))
            except Fail as ex:
                rej.append((case, str(ex)[:110]))
        rows[v] = rej
        print(f'  {v} {VARIANTS[v]}: ' + ('rejected in ' + ', '.join(c for c, _ in rej) + f' (first: {rej[0][1]})' if rej else 'SURVIVES'))
    return rows


# ---------------------------------------------------------------- self-test
def tampers(caps, old, manifest):
    def first(d, t, pred=lambda e: True):
        for e in d['events']:
            if e['e'] == t and pred(e):
                return e
        raise Fail(f'no {t} to tamper')

    def all_runs(fn):
        def g(c2):
            for d in c2['pk1']:
                fn(d)
        return g

    def bump_hex(hx):
        return '%016x' % (int(hx, 16) ^ 1)

    def is312(e):
        return e['ecx'] in (3, 12)

    def drop_312(d):
        out, skip, buf = [], False, []
        for e in d['events']:
            if e['e'] == 'pin':
                buf, skip = [e], False
                continue
            if buf:
                buf.append(e)
                if e['e'] == 'rule' and e['ecx'] in (3, 12):
                    skip = True
                if e['e'] == 'leave':
                    if not skip:
                        out.extend(buf)
                    buf = []
                continue
            out.append(e)
        d['events'] = out

    def tend_swap(d):
        e = first(d, 'tend', lambda e: len(e['list']) >= 2)
        e['list'][0], e['list'][1] = e['list'][1], e['list'][0]

    def chain_id(d):
        e = first(d, 'anc', lambda e: e['res'] is not None and len(e['chain']) > 1)
        e['chain'][-1] += 0.5

    def rm_del(d):
        e = first(d, 'rm')
        d['events'].remove(e)

    def pick_idx(d):
        e = first(d, 'pick', lambda e: len(e['list']) >= 2)
        e['idx'] = (e['idx'] + 1) % len(e['list'])

    def proj_change(d):
        first(d, 'ins')['heap_n'] += 1

    def s70_used(d):                           # first 5e70c1 return in a call whose rule step reads it (k > 16)
        ev = d['events']
        for i, e in enumerate(ev):
            if e['e'] == 'rule' and e['esi'] > 16 and cmod(e['esi'], 11) != 0:
                j = max(x for x in range(i) if ev[x]['e'] == 's70' and ev[x]['f'] == '5e70c1')
                ev[j]['r'] = 0
                return
        raise Fail('no call reads 5e70c1')

    def pick312(d):
        ev = d['events']
        for i, e in enumerate(ev):
            if e['e'] == 'rule' and e['ecx'] in (3, 12):
                pk = next(x for x in ev[i:] if x['e'] == 'pick')
                if len(pk['list']) >= 2:
                    pk['idx'] = (pk['idx'] + 1) % len(pk['list'])
                    return
        raise Fail('no rule 3/12 pick')

    def thr_null(d):
        e = first(d, 'thr')
        e['t20'] = None

    cases = [
        ('binary hash zero', 'mani', lambda m: m.__setitem__('binary_sha256', '0' * 64)),
        ('tool hash zero', 'mani', lambda m: m.__setitem__('tool_sha256', '0' * 64)),
        ('empty events', 'caps', all_runs(lambda d: d.__setitem__('events', []))),
        ('run 2 differs', 'caps', lambda c2: first(c2['pk1'][1], 'rule').__setitem__('esi', -5)),
        ('projection differs from m2_06a', 'caps', all_runs(proj_change)),
        ('rule ecx', 'caps', all_runs(lambda d: first(d, 'rule').__setitem__('ecx', 7))),
        ('rule before focus', 'caps', all_runs(lambda d: first(d, 'rpre').__setitem__('ecx', 1))),
        ('RNG slot', 'caps', all_runs(lambda d: first(d, 'drw').__setitem__('st', bump_hex(first(d, 'drw')['st'])))),
        ('RNG state after draw', 'caps', all_runs(lambda d: first(d, 'rule', lambda e: e['ecx'] in (3, 12)).__setitem__('st', '0' * 16))),
        ('pick index', 'caps', all_runs(pick_idx)),
        ('pick work', 'caps', all_runs(lambda d: first(d, 'pick', lambda e: e['work'] is not None).__setitem__('work', h(0.5)))),
        ("d'", 'caps', all_runs(lambda d: first(d, 'anc', lambda e: e['w'] == 'sel').__setitem__('d', first(d, 'anc', lambda e: e['w'] == 'sel')['d'] + 1))),
        ('anc chain id', 'caps', all_runs(chain_id)),
        ('tend order', 'caps', all_runs(tend_swap)),
        ('threshold value', 'caps', all_runs(lambda d: first(d, 'thr').__setitem__('t20', h(1.0)))),
        ('threshold null-null', 'caps', all_runs(thr_null)),
        ('rm deleted', 'caps', all_runs(rm_del)),
        ('5e70c1 return where k > 16 reads it', 'caps', all_runs(s70_used)),
        ('rule 3/12 pick index', 'caps', all_runs(pick312)),
        ('selected excluded flag', 'caps', all_runs(lambda d: first(d, 'tail').__setitem__('x41', 1))),
    ]
    bad = 0
    for label, kind, fn in cases:
        m2, c2 = copy.deepcopy(manifest), copy.deepcopy(caps)
        try:
            (fn(m2) if kind == 'mani' else fn(c2))
            controls(c2, m2, old)
            acceptance(c2, quiet=True)
            print(f'  NOT REJECTED: {label}')
            bad += 1
        except Fail as ex:
            print(f'  rejected: {label}: {str(ex)[:120]}')
    # all rule 3/12 calls removed from every case: the remaining calls replay, the acceptance count must fail
    c2 = copy.deepcopy(caps)
    for runs in c2.values():
        for d in runs:
            drop_312(d)
    try:
        acceptance(c2, quiet=True)
        print('  NOT REJECTED: no rule 3/12 calls')
        bad += 1
    except Fail as ex:
        print(f'  rejected: no rule 3/12 calls: {ex}')
    return bad


def write_manifest():
    mp = os.path.join(DATA, 'manifest.json')
    need(not os.path.exists(mp), 'manifest exists')
    files, tool = {}, None
    for case in CASES:
        for r in (1, 2, 3):
            fn = f'{case}_{r}.json.gz'
            d = load(os.path.join(DATA, fn))
            need(tool in (None, d['tool_sha256']), 'tool hash differs between captures')
            tool = d['tool_sha256']
            files[fn] = sha(os.path.join(DATA, fn))
    need(tool == sha(TOOL), 'captures were made with a different tool version')
    models = {m: sha(os.path.join(MODELS, m + '.mps.gz')) for m, _ in CASES.values()}
    json.dump({'task': 'M2-06c', 'schema': 1, 'tool': 'tools/mip_plunge_rule.js', 'tool_sha256': tool, 'binary_sha256': BINARY,
               'base_params': BASE, 'cases': {c: {'model': m + '.mps.gz', **dict(BASE, **x)} for c, (m, x) in CASES.items()},
               'models': models, 'model_paths': '../re/bench/data/miplib/<model>.mps.gz', 'files': files},
              open(mp, 'w'), indent=1, sort_keys=False)
    print('wrote', mp)


def main():
    a = sys.argv[1:]
    if a[:1] == ['--write-manifest']:
        write_manifest()
        return 0
    if a[:1] == ['--sweep']:
        for p in a[1:]:
            try:
                st = check_capture(load(p), os.path.basename(p))
                print(os.path.basename(p), 'replay equal', json.dumps(st, sort_keys=True))
            except Fail as ex:
                print(os.path.basename(p), 'FAIL', ex)
        return 0
    try:
        caps, old, manifest = load_all()
        controls(caps, manifest, old)
        print('C0 PASS (3 runs identical per case, manifest hashes, 06a projection equal for 4 cases)')
        stats, tot = acceptance(caps)
        print('  totals: ' + ', '.join(f'{lbl} {fn(tot)}' for lbl, fn in TARGETS) + f', drop {tot["drop"]}, thr<1e100 {tot["thr_set"]}, '
              f'removed-by-bound {tot["by_bound"]}, null-anc {tot["null"]}, delta {dict(sorted(tot["delta"].items()))}')
        print('ACCEPTANCE PASS')
        print('variants:')
        variants(caps)
    except Fail as ex:
        print('FAIL', ex)
        return 1
    if a[:1] == ['--self-test']:
        print('self-test:')
        bad = tampers(caps, old, manifest)
        if bad:
            print('SELF-TEST FAIL')
            return 1
        print('ACCEPTANCE PASS (+ self-test)')
    return 0


if __name__ == '__main__':
    sys.exit(main())
