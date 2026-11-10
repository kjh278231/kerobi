#!/usr/bin/env python3
"""M2-06e: pseudocost table A init / node update / decay rules against captures of tools/mip_pseudocost.js.
Contract: docs/m2/mip_pseudocost.md section 3.

(default)          C1-C7 on bench/oracle/m2_06e
--write-manifest   record the SHA-256 of every capture (once, after the capture)
--self-test        also T (capture tampers) and V (rule variants, outcomes fixed in section 3)
--sweep FILE...    replay the rules on single exploratory captures and print the branch counts (not acceptance)
Exit code 0 only when every requested check passes.
"""
import copy, gzip, hashlib, json, math, os, struct, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = os.path.join(ROOT, 'bench/oracle/m2_06e')
OLD = os.path.join(ROOT, 'bench/oracle/m2_06b2')  # same runs (NodeLimit 2000), C7 non-interference
TOOL = os.path.join(ROOT, 'tools/mip_pseudocost.js')
RUNS = (1, 2, 3)
CASES = ('markshare_4_0', 'pk1', 'neos5', 'tr12-30', 'fhnw-binpack4-4', 'neos-860300')
ZERO = '0' * 16
LT = 0x3c  # '<'
C_TYPE = 0x43  # 'C'


class Fail(Exception):
    pass


def need(cond, msg):
    if not cond:
        raise Fail(msg)


def sha(p):
    with open(p, 'rb') as fh:
        return hashlib.sha256(fh.read()).hexdigest()


def f(h):
    return struct.unpack('>d', bytes.fromhex(h))[0]


def hx(x):
    return struct.pack('>d', x).hex()


def load(path):
    return json.loads(gzip.decompress(open(path, 'rb').read()))


def maxsd(dst, src):
    """x86 maxsd dst, src: dst if dst > src else src (src on equality or NaN)."""
    return dst if dst > src else src


# ---- rules (section 2); V is a set of variant names (section 3 V), empty for the original
def upd_rule(e, fx, delta, V):
    """6edfcf on one entry e = [cd, cu, sd_hex, su_hex]; returns the new entry."""
    g = delta if 'P5' in V else maxsd(delta, 1e-10)
    eps = 0.0 if 'P3' in V else 0.001
    if fx > 0:
        d, g = 1, g / (eps + fx)
    elif fx < 0:
        d, g = 0, g / (eps - fx)
    else:
        return list(e)
    out = list(e)
    s = f(e[2 + d])
    s = g + s if (e[d] != 0 or 'P4' in V) else maxsd(s, g)
    out[2 + d], out[d] = hx(s), e[d] + 1
    return out


def node_update(g, V):
    """5fb8ec block: None (no call) or (site, f, delta, w, cutoff)."""
    cut = f(g['cut']) if g['cut'] is not None else 1e100
    if not g['parent'] or g['st'] == 7 or g['j'] < 0:
        return None
    w = 1.0 if 'P6' in V else math.pow(0.8 / (float(g['depth']) + (0.0 if 'P2' in V else 1.0)),
                                        (0.2 if 'P1' in V else 0.1) if 5e99 > cut else (0.1 if 'P1' in V else 0.2))
    x = f(g['x'])
    if g['dir'] == LT:
        fx = -0.5 if g['vt'] == C_TYPE else float(math.floor(x)) - x
    else:
        fx = 0.5 if g['vt'] == C_TYPE else float(math.ceil(x)) - x
    p = f(g['p'])
    if -1e100 >= p:
        return None
    if g['st'] & ~1 == 4:
        if not (5e99 > cut) and 'P8' not in V:
            return None
        return 'n45', fx, w * (cut - p), w, cut
    if g['st'] - 1 in (0, 1):
        return 'n12', fx, (f(g['lp']) - p) * w, w, cut
    return None


def decay_fires(g, V):
    if not g['parent'] or g['j'] < 0 or g['st'] == 9:
        return False
    p = f(g['p'])
    if -1e100 >= p:
        return False
    if g['st'] & ~1 == 4:
        return True
    return g['st'] - 1 in (0, 1) and ('G1' in V or f(g['lp']) - p > 1e-10)


def decay_rule(ev, ents, V, st):
    """6ee1d3 on the entries dict j -> entry (changed in place)."""
    if ev.get('noI') or ev.get('noL'):
        st['dec_noinfo'] += 1
        return
    if ev['l38'] == 6 or ev['l58'] >= 0:
        if not ev['l70']:
            st['dec_novec'] += 1
            return
    elif not ev['l18'] or not ev['l28']:
        st['dec_novec'] += 1
        return
    w, s = (0.5, 1.0) if ev['l58'] < 0 else (1.0, float(ev['l54']))
    st['l54_le0'] += ev['l58'] >= 0 and ev['l54'] <= 0
    if 'D4' in V:
        s = 1.0
    st['dec_l58neg' if ev['l58'] < 0 else 'dec_l58pos'] += 1
    for a in ev['anc']:
        if not (w > 0.001) or a is None:
            break
        if 'D1' not in V:
            w *= 0.5
        j, d, vh = a
        if j >= 0:
            s2 = s if d == LT else -s
            if ev['l58'] >= 0 and (-1e-10 > s2 * f(vh)) != (-1e-10 > (1.0 if d == LT else -1.0) * f(vh)):
                st['d4_eff'] += ents[j][2 if d == LT else 3] != ZERO
            if 'D3' not in V and -1e-10 > s2 * f(vh):
                st['dec_skip_sign'] += 1
                st['dec_skip_eff'] += ents[j][2 if d == LT else 3] != ZERO
            else:
                k = 0 if d == LT else 1
                e = ents[j]
                c = float(e[k]) + (0.0 if 'D2' in V else 1.0)
                st['dec_apply_eff'] += f(e[2 + k]) != 0
                e[2 + k] = hx((1.0 - w / c) * f(e[2 + k]))
                st['dec_apply'] += 1
        if 'D1' in V:
            w *= 0.5


def first(a, b):
    """First differing pair of two lists (None padding); None when equal."""
    return next(((x, y) for x, y in zip(a + [None] * len(b), b + [None] * len(a)) if x != y), None)


def sym_rule(model, ncols, cls, V, st=None):
    """6ef22e: per direction, each class takes the column with the largest sum/cnt (first index on ties) and every
    member gets that column's cnt and sum."""
    for d in (0, 1):
        best, rep = {}, {}
        for j in range(ncols):
            e = model.get(j, [0, 0, ZERO, ZERO])
            if e[d] > 0:
                c, v = cls[j], f(e[2 + d]) / float(e[d])
                if st is not None and c in rep and best[c] == v:
                    r = model.get(rep[c], [0, 0, ZERO, ZERO])
                    st['sym_tie'] += (r[d], r[2 + d]) != (e[d], e[2 + d])
                if c not in rep or (best[c] <= v if 'S1' in V else best[c] < v):
                    best[c], rep[c] = v, j
        if 'S2' in V:
            continue
        for j in range(ncols):
            r = rep.get(cls[j], -1)
            if r >= 0:
                src = model.get(r, [0, 0, ZERO, ZERO])
                e = list(model.get(j, [0, 0, ZERO, ZERO]))
                e[d], e[2 + d] = src[d], src[2 + d]
                model[j] = e


def nz(model):
    return sorted([j] + e for j, e in model.items() if e != [0, 0, ZERO, ZERO])


def replay(d, V=frozenset(), name=''):
    """Replays every table event of one capture; raises Fail at the first difference. Returns branch counts."""
    need(d.get('ok') and not d.get('errors'), f'{name}: capture not ok: {d.get("errors")}')
    ev = d['events']
    models = {}
    st = dict.fromkeys(('new', 'new_seed', 'new_copy', 'free', 'upd', 'upd_other', 'n12', 'n45', 'gate', 'gate_fire',
                        'nofire_parent', 'nofire_st7', 'nofire_j', 'nofire_p', 'nofire_cut', 'nofire_st', 'vt_C',
                        'cnt0', 'cnt0_keep', 'clamp', 'cut_inf_n12', 'ij0', 'dgate', 'dgate_fire', 'dgate_st45',
                        'dgate_st12', 'dec', 'dec_l58neg', 'dec_l58pos', 'dec_l38_6', 'dec_skip_sign', 'dec_apply',
                        'dec_wstop', 'ik0', 'dec_noinfo', 'dec_novec', 'sym', 'sym_changed', 'cnt0_nz', 'seed_break', 'l54_le0', 'seed_avg', 'dec_skip_eff', 'dec_apply_eff',
                        'sym_tie', 'g1', 'd4_eff'), 0)

    def ent(t, j):
        return list(models[t].get(j, [0, 0, ZERO, ZERO]))

    for i, e in enumerate(ev):
        k = e['e']
        where = f'{name}: event {i} ({k})'
        if k == 'new':
            need(e['t'] >= 0 and e['rc'] == 0, f'{where}: table creation failed {e}')
            snap = sorted(e['nz'])
            if e['f'] == '6eca1a':
                need(e['src'] in models, f'{where}: copy of unknown table {e["src"]}')
                want = nz(models[e['src']])
                st['new_copy'] += 1
            elif e['src'] >= 0 and e['map'] is not None and e['flag850'] == 0:
                need(e['src'] in models, f'{where}: seed from unknown table {e["src"]}')
                src, want = models[e['src']], {}
                for j in range(e['ncols']):
                    m = e['map'][j]
                    if m < 0:
                        st['seed_break'] += j < e['ncols'] - 1
                        if 'Z1' in V:
                            continue
                        break
                    s = src.get(m, [0, 0, ZERO, ZERO])
                    st['seed_avg'] += (s[0] > 1 and f(s[2]) != 0) + (s[1] > 1 and f(s[3]) != 0)
                    sd = (s[2] if 'Z2' in V else hx(f(s[2]) / float(s[0]))) if s[0] != 0 else ZERO
                    su = (s[3] if 'Z2' in V else hx(f(s[3]) / float(s[1]))) if s[1] != 0 else ZERO
                    want[j] = [0, 0, sd, su]
                want = nz(want)
                st['new_seed'] += 1
            else:
                want = []
                need((e['eta'], e['sbit'], e['sbcost']) == (1, 0, ZERO), f'{where}: fresh header {e["eta"], e["sbit"], e["sbcost"]}')
            need(snap == want, f'{where}: table {e["t"]} initial entries differ: first '
                 f'{first(snap, want)}')
            models[e['t']] = {r[0]: list(r[1:]) for r in snap}
            st['new'] += 1
        elif k == 'free':
            if e['t'] < 0:
                continue
            need(e['t'] in models, f'{where}: free of unknown table {e["t"]}')
            got, want = sorted(e['nz']), nz(models[e['t']])
            need(got == want, f'{where}: table {e["t"]} at free differs from the replayed chain: first '
                 f'{first(got, want)}')
            del models[e['t']]
            st['free'] += 1
        elif k == 'sym':
            need(e['t'] in models and e['t'] == e['tA'], f'{where}: symmetry step on table {e["t"]} (table A {e["tA"]})')
            got = sorted(e['before'])
            need(got == nz(models[e['t']]), f'{where}: before differs from the chain: first {first(got, nz(models[e["t"]]))}')
            if e['S'] and e['norb'] > 0 and e['cls'] is not None:
                sym_rule(models[e['t']], len(e['cls']), e['cls'], V, st)
                st['sym'] += 1
                st['sym_changed'] += sorted(e['after']) != got
            want, got = nz(models[e['t']]), sorted(e['after'])
            need(got == want, f'{where}: after differs from the rule: first {first(got, want)}')
            models[e['t']] = {r[0]: list(r[1:]) for r in got}
        elif k == 'upd':
            need(e['t'] in models, f'{where}: update of unknown table {e["t"]}')
            before = ent(e['t'], e['j'])
            need(e['before'] == before, f'{where}: table {e["t"]} j {e["j"]} before {e["before"]} != chain {before}')
            a, b = e['before'], e['after']
            dirs = [dd for dd in (0, 1) if b[dd] != a[dd]]
            need(len(dirs) == 1 and b[dirs[0]] == a[dirs[0]] + 1 and b[1 - dirs[0]] == a[1 - dirs[0]]
                 and b[3 - dirs[0]] == a[3 - dirs[0]], f'{where}: not a one-direction count step {a} -> {b}')
            if 'site' in e:
                g = ev[i - 1] if i else None
                need(g is not None and g['e'] == 'gate', f'{where}: 5fb8ec update without the gate event before it')
                r = node_update(g, V)
                need(r is not None and r[0] == e['site'], f'{where}: gate {g} predicts {r and r[0]}, call site {e["site"]}')
                site, fx, delta, w, cut = r
                need(e['t'] == g['tA'], f'{where}: table {e["t"]} is not the M table A {g["tA"]}')
                need(e['w'] == hx(w), f'{where}: w {e["w"]} != pow {hx(w)} (depth {g["depth"]}, cut {hx(cut)})')
                need(e['cut'] == hx(cut), f'{where}: cutoff slot {e["cut"]} != gate {hx(cut)}')
                want = upd_rule(a, fx, delta, V)
                need(b == want, f'{where}: {site} j {e["j"]} f {fx!r} delta {delta!r}: after {b} != rule {want}')
                st[site] += 1
                st['vt_C'] += g['vt'] == C_TYPE
                dd = 0 if fx < 0 else 1
                if a[dd] == 0:
                    st['cnt0'] += 1
                    st['cnt0_nz'] += a[2 + dd] != ZERO
                    st['cnt0_keep'] += b[2 + dd] == a[2 + dd] and a[2 + dd] != ZERO
                st['clamp'] += not (delta > 1e-10)
                st['cut_inf_n12'] += site == 'n12' and not (5e99 > cut)
            else:
                st['upd_other'] += 1
            models[e['t']][e['j']] = list(b)
            st['upd'] += 1
        elif k == 'gate':
            st['gate'] += 1
            st['ij0'] += not e['ij']
            r = node_update(e, V)
            nxt = ev[i + 1] if i + 1 < len(ev) else {}
            called = nxt.get('e') == 'upd' and 'site' in nxt
            need(called == (r is not None), f'{where}: gate {e} predicts {r}, next event {nxt.get("e")} {nxt.get("site")}')
            if r is not None:
                st['gate_fire'] += 1
            else:
                cut = f(e['cut']) if e['cut'] is not None else 1e100
                why = ('nofire_parent' if not e['parent'] else 'nofire_st7' if e['st'] == 7 else 'nofire_j' if e['j'] < 0
                       else 'nofire_p' if -1e100 >= f(e['p']) else 'nofire_cut' if e['st'] & ~1 == 4 and not (5e99 > cut)
                       else 'nofire_st')
                st[why] += 1
        elif k == 'dgate':
            st['dgate'] += 1
            st['ik0'] += not e['ik']
            fire = decay_fires(e, V)
            st['g1'] += not fire and decay_fires(e, frozenset(['G1']))
            nxt = ev[i + 1] if i + 1 < len(ev) else {}
            need((nxt.get('e') == 'dec') == fire, f'{where}: decay gate {e} predicts {fire}, next event {nxt.get("e")}')
            if fire:
                need(nxt['t'] == e['tA'], f'{where}: decay table {nxt["t"]} is not table A {e["tA"]}')
                st['dgate_fire'] += 1
                st['dgate_st45' if e['st'] & ~1 == 4 else 'dgate_st12'] += 1
        elif k == 'dec':
            need(e['t'] in models, f'{where}: decay of unknown table {e["t"]}')
            need(i and ev[i - 1]['e'] == 'dgate', f'{where}: decay without the gate event before it')
            st['dec'] += 1
            st['dec_l38_6'] += e.get('l38') == 6
            ents = {r[0]: ent(e['t'], r[0]) for r in e.get('before', [])}
            for r in e.get('before', []):
                need(r[1:] == ents[r[0]], f'{where}: table {e["t"]} j {r[0]} before {r[1:]} != chain {ents[r[0]]}')
            decay_rule(e, ents, V, st)
            w, n = (0.5 if e.get('l58', -1) < 0 else 1.0), 0
            for a in e.get('anc', []):
                if not (w > 0.001) or a is None:
                    break
                w *= 0.5
                n += 1
            st['dec_wstop'] += n < len(e.get('anc', [])) and e['anc'][n] is not None
            for r in e.get('after', []):
                need(r[1:] == ents[r[0]], f'{where}: decay j {r[0]} after {r[1:]} != rule {ents[r[0]]} (anc {e["anc"]})')
                models[e['t']][r[0]] = list(r[1:])
    need(not models, f'{name}: tables never freed {sorted(models)}')
    bulk = {k: v for k, v in d['counts'].items() if k.startswith('bulk')}
    need(not bulk, f'{name}: parallel merge/reset paths ran {bulk}')
    return st


def ins_stream(d):
    return [(e['in_sel'], e['from'], e['tree'], e['node'], e['heap_n']) for e in d['events'] if e['e'] == 'ins']


def check_case(c, d, old):
    st = replay(d, name=c)
    need(d['output'] == old['output'], f'C7 {c}: output {d["output"]!r} != M2-06b-2 {old["output"]!r}')
    a, b = ins_stream(d), ins_stream(old)
    need(a == b, f'C7 {c}: heap inserts differ from M2-06b-2 ({len(a)} vs {len(b)}): first {first(a, b)}')
    return st


# variant -> branch counts whose sum > 0 means the variant must be rejected in that case (section 3 V); fixed before
# the acceptance capture from the exploratory sweep (NodeLimit 500)
VARIANTS = {
    'P1': ('n12', 'n45'), 'P2': ('n12', 'n45'), 'P3': ('n12', 'n45'), 'P6': ('n12', 'n45'),
    'P4': ('cnt0_nz',), 'P5': ('clamp',), 'P8': ('nofire_cut',), 'G1': ('g1',),
    'D1': ('dec_apply_eff',), 'D2': ('dec_apply_eff',), 'D3': ('dec_skip_eff',), 'D4': ('d4_eff',),
    'S1': ('sym_tie',), 'S2': ('sym_changed',), 'Z1': ('seed_break',), 'Z2': ('seed_avg',),
}


def rejected(d, v, c):
    try:
        replay(d, frozenset([v]), c)
        return False
    except Fail:
        return True


def variant_matrix(docs):
    """name -> set of cases where the variant is rejected."""
    return {v: {c for c, d in docs.items() if rejected(d, v, c)} for v in VARIANTS}


def a1(manifest):
    """C1 reproducibility and identity; returns run 1 of every case."""
    tool = sha(TOOL)
    out = {}
    for c in CASES:
        hs = []
        for r in RUNS:
            p = os.path.join(DATA, f'{c}_{r}.json.gz')
            need(os.path.exists(p), f'C1: missing {p}')
            hs.append(sha(p))
            if manifest is not None:
                need(manifest['files'].get(os.path.basename(p)) == hs[-1], f'C1: {p} sha256 != manifest')
        need(len(set(hs)) == 1, f'C1: {c}: runs differ {hs}')
        d = load(os.path.join(DATA, f'{c}_1.json.gz'))
        need(d['tool_sha256'] == tool, f'C1: {c}: capture tool sha256 {d["tool_sha256"]} != current tool {tool}')
        need(d['params'] == {'Threads': 1, 'Seed': 0, 'LogToConsole': 0, 'NodeLimit': 2000}, f'C1: {c}: params {d["params"]}')
        out[c] = d
    return out


REQUIRED = ('n12', 'n45', 'nofire_cut', 'clamp', 'cut_inf_n12', 'dec_apply', 'dec_skip_sign', 'dec_l58pos',
            'dec_l58neg', 'dec_l38_6', 'dec_novec', 'dec_noinfo', 'sym_changed', 'new_seed', 'cnt0', 'upd_other')


def acceptance(docs, olds):
    stats = {}
    for c in CASES:
        stats[c] = check_case(c, docs[c], olds[c])
        print(c, 'PASS', json.dumps({k: v for k, v in stats[c].items() if v}))
    tot = {k: sum(st[k] for st in stats.values()) for k in REQUIRED}
    print('C8 branch totals', json.dumps(tot))
    need(all(tot.values()), f'C8: a required branch never ran: {[k for k, v in tot.items() if not v]}')
    return stats


def tampers(docs, olds):
    """T: each tamper of a run 1 copy must make the named check fail."""
    def ev_of(d, pred):
        return next(i for i, e in enumerate(d['events']) if pred(e))

    def t_upd(d):  # node update result bit
        i = ev_of(d, lambda e: e['e'] == 'upd' and e.get('site') == 'n12')
        e = d['events'][i]
        k = 2 if e['after'][0] != e['before'][0] else 3
        e['after'][k] = hx(f(e['after'][k]) * (1 + 2 ** -52))

    def t_gate_depth(d):
        i = ev_of(d, lambda e: e['e'] == 'gate' and e['parent'])
        d['events'][i]['depth'] += 1

    def t_gate_cut(d):  # no incumbent -> pretend one: the 4/5 gate now predicts a call
        i = ev_of(d, lambda e: e['e'] == 'gate' and e['st'] & ~1 == 4 and e['cut'] is not None and f(e['cut']) >= 5e99
                  or e['e'] == 'gate' and e['st'] & ~1 == 4 and e['cut'] is None)
        d['events'][i]['cut'] = hx(1.0)

    def t_drop_dec(d):
        i = ev_of(d, lambda e: e['e'] == 'dec' and e.get('after') and e['after'] != e['before'])
        del d['events'][i]

    def t_dec_v(d):  # flip the sign of the vector entry of an applied ancestor
        for e in d['events']:
            if e['e'] == 'dec' and e.get('after') and e['after'] != e['before']:
                for a in e['anc']:
                    if a and a[0] >= 0 and a[2] is not None and f(a[2]) != 0:
                        a[2] = hx(-f(a[2]) * 1e12)
                        return

    def t_free(d):
        i = ev_of(d, lambda e: e['e'] == 'free' and e['t'] >= 0 and e['nz'])
        d['events'][i]['nz'][0][1] += 1

    def t_new(d):
        i = ev_of(d, lambda e: e['e'] == 'new' and e['src'] < 0)
        d['events'][i]['nz'].append([0, 1, 0, ZERO, ZERO])

    def t_sym(d):
        i = ev_of(d, lambda e: e['e'] == 'sym' and sorted(e['after']) != sorted(e['before']))
        d['events'][i]['after'] = d['events'][i]['before']

    def t_other(d):  # a strong-branching update that steps both counts
        i = ev_of(d, lambda e: e['e'] == 'upd' and 'site' not in e)
        d['events'][i]['after'][0] += 1
        d['events'][i]['after'][1] += 1

    def t_ins(d):
        i = ev_of(d, lambda e: e['e'] == 'ins')
        d['events'][i]['heap_n'] += 1

    plan = [('upd result', 'markshare_4_0', t_upd), ('gate depth', 'pk1', t_gate_depth),
            ('gate cutoff', 'fhnw-binpack4-4', t_gate_cut), ('drop decay', 'tr12-30', t_drop_dec),
            ('decay vector', 'pk1', t_dec_v), ('free snapshot', 'neos5', t_free), ('new snapshot', 'markshare_4_0', t_new),
            ('symmetry after', 'neos5', t_sym), ('other caller step', 'tr12-30', t_other), ('heap insert', 'pk1', t_ins)]
    for name, c, fn in plan:
        d = copy.deepcopy(docs[c])
        fn(d)
        try:
            check_case(c, d, olds[c])
            raise Fail(f'T {name} ({c}): tampered capture passed')
        except Fail as ex:
            if str(ex).startswith('T '):
                raise
            print(f'T {name} ({c}): rejected: {str(ex)[:160]}')


def variants(docs, stats):
    for v, keys in VARIANTS.items():
        for c in CASES:
            want = sum(stats[c][k] for k in keys) > 0
            got = rejected(docs[c], v, c)
            need(got == want, f'V {v} ({c}): rejected {got}, expected {want} from {keys}')
        print(f'V {v}: rejected in {[c for c in CASES if sum(stats[c][k] for k in keys) > 0]}')


def main():
    args = sys.argv[1:]
    if args[:1] == ['--variants']:
        docs = {os.path.basename(p): load(p) for p in args[1:]}
        for v, rej in variant_matrix(docs).items():
            print(v, 'rejected:', sorted(rej), 'rule:', sorted(c for c, d in docs.items()
                                                               if sum(replay(d, name=c)[k] for k in VARIANTS[v]) > 0))
        return 0
    if args[:1] == ['--sweep']:
        bad = 0
        for p in args[1:]:
            try:
                st = replay(load(p), name=os.path.basename(p))
                print(os.path.basename(p), 'PASS', json.dumps({k: v for k, v in st.items() if v}))
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
                json.dump({'task': 'M2-06e', 'schema': 1, 'tool': 'tools/mip_pseudocost.js', 'tool_sha256': sha(TOOL),
                           'binary_sha256': '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c',
                           'params': {'Threads': 1, 'Seed': 0, 'LogToConsole': 0, 'NodeLimit': 2000},
                           'models': '../re/bench/data/miplib/<case>.mps.gz', 'files': files}, fh, indent=1)
            print('manifest written', mpath)
            return 0
        manifest = json.load(open(mpath)) if os.path.exists(mpath) else None
        need(manifest is not None, 'C1: no manifest (run --write-manifest once after the capture)')
        docs = a1(manifest)
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
