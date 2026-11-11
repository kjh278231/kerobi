#!/usr/bin/env python3
"""M2-06g: pseudocost readers (6ef12f, 6eed2e, 6ee4ac), SB cache writes (6ee5d1) and the reliability threshold eta /
SB iteration limit (62f909 eta block, 6edd89, 6edda1) against captures of tools/mip_sb_read.js.
Contract: docs/m2/mip_sb_read.md section 3.

(default)          S1-S2, F, E, A, W, WS, K, H, EB, IT, Q on bench/oracle/m2_06g
--write-manifest   record the SHA-256 of every capture (once, after the capture)
--self-test        also T (capture tampers) and V (rule variants, outcomes fixed in section 3)
--sweep FILE...    replay single captures and print the branch counts (not acceptance)
Exit code 0 only when every requested check passes.
"""
import collections, copy, json, math, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mip_pseudocost_check import Fail, need, sha, f, hx, load, maxsd  # noqa: E402  (06e helpers)

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = os.path.join(ROOT, 'bench/oracle/m2_06g')
OLD = os.path.join(ROOT, 'bench/oracle/m2_06e2')  # same cases and params, S2 non-interference (rd counts)
TOOL = os.path.join(ROOT, 'tools/mip_sb_read.js')
BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
PARAMS = {'Threads': 1, 'Seed': 0, 'LogToConsole': 0, 'NodeLimit': 2000}
RUNS = (1, 2, 3)
CASES = ('markshare_4_0', 'pk1', 'neos5', 'tr12-30', 'fhnw-binpack4-4', 'neos-860300')
INT_MIN = -2 ** 31
# eta block 6edd89 call sites (return address = call + 5) and the 62f909 restore
EB_SITE = {'2e9': '543729', 'one': '543717', 'c': '5436f8', 'cap': '543705'}
RESTORE = '542fb0'
INL_ONE = ('i541ce1', 'i541f90')  # 62f909 inline T+0x60 = 1; the other inline sites write the saved eta


def bits(x):
    return hx(x)


def fl(x):
    """floor() of a double, as a double (exact)."""
    return float(math.floor(x)) if math.isfinite(x) else x


def cvttsd2si(x):
    if not math.isfinite(x) or not (INT_MIN <= math.trunc(x) <= 2 ** 31 - 1):
        return INT_MIN
    return int(math.trunc(x))


def cdiv(a, b):
    """C int division (truncation toward zero)."""
    q = abs(a) // abs(b)
    return q if (a >= 0) == (b >= 0) else -q


# ---- rules (section 2); V is a set of variant names (section 3 V), empty for the original
def hit(V, c, w30, w38, x):
    """cache hit of 6ef12f / 6eed2e / 6ee4ac: comisd tests, NaN -> miss."""
    if c is None or w30 is None or c[0] != w30:
        return False
    if len(c) < 5:
        raise Fail('cache entries missing although stamp == W+0x30')
    if 'V2' not in V and not (f(w38) - f(c[1]) <= 100.0):
        return False
    d = abs(x - f(c[2]))
    return d < 0.01 if 'V3' in V else d <= 0.01


def psi(V, h, cval, ent, ent2, k, st, side):
    """one direction of 6ef12f: k = 0 (down) or 1 (up)."""
    if h and 'V5' not in V:
        st[f'f_hit_{side}'] += 1
        return f(cval)
    cnt, s = ent[k], f(ent[2 + k])
    if ent2 is not None and 'V4' not in V:
        tot = ent2[k] + cnt
        if tot > 0:
            st['f_t2'] += 1
            return (f(ent2[2 + k]) + s) / float(tot)
    if cnt > 0:
        st['f_tab'] += 1
        return s / float(cnt)
    st['f_zero'] += 1
    return 0.0


def rule_f(e, V, st):
    _, ra, t, t2, j, xh, w30, w38, ent, ent2, c, dn, up = e
    x = f(xh)
    fr = x - (round(x) if 'V15' in V else fl(x))
    h = hit(V, c, w30, w38, x)
    p_dn = psi(V, h, c[3] if h else None, ent, ent2, 0, st, 'dn')
    p_up = psi(V, h, c[4] if h else None, ent, ent2, 1, st, 'up')
    lo = 0.0 if 'V1' in V else 1e-6
    if p_dn <= 1e-6 or p_up <= 1e-6:
        st['f_floor'] += 1
    return bits(maxsd(p_dn, lo) * fr), bits((1.0 - fr) * maxsd(p_up, lo))


def rule_e(e, V, st):
    _, ra, t, j, root, lnn, xlnn, wnn, w30, w38, xl, cnt, eta, c, ret = e
    if root and cnt[0] > 0 and cnt[1] > 0 and 'V6' not in V:
        st['e_root'] += 1
        return 0
    if lnn and xlnn and wnn and 'V7' not in V and hit(V, c, w30, w38, f(xl)):
        st['e_cache'] += 1
        return 0
    if 'V9' in V:
        r = 1 if cnt[0] < eta and cnt[1] < eta else 0
    elif 'V8' in V:
        r = 1 if cnt[0] <= eta or cnt[1] <= eta else 0
    else:
        r = 1 if cnt[0] < eta or cnt[1] < eta else 0
    st['e_unrel' if r else 'e_rel'] += 1
    return r


def rule_a(e, V, st):
    _, ra, t, j, xh, w30, w38, c, ret = e
    need(xh is not None, f'A: 6ee4ac from an unmapped call site {ra}')
    r = 1 if hit(V, c, w30, w38, f(xh)) else 0
    st['a_hit' if r else 'a_miss'] += 1
    return r


def rule_w(args, V, st):
    """6ee5d1 cache entries 2 and 3: maxsd(delta, 1e-10) / (|f| + 0.001)."""
    x, fd, dd, fu, du = (f(a) for a in args)
    lo = 0.0 if 'V10' in V else 1e-10
    if dd <= 1e-10 or du <= 1e-10:
        st['w_floor'] += 1
    den = (lambda v: v + 0.001) if 'V11' in V else (lambda v: abs(v) + 0.001)
    return bits(maxsd(dd, lo) / den(fd)), bits(maxsd(du, lo) / den(fu))


def rule_eb(e, V, st):
    """62f909 eta block (asm 0x543677-0x543736): (site, value, sbit set to -1)."""
    _, t, eta0, cost, it, t0, run, w, m8c0 = e
    tm = f(t0) + f(w) if run else f(t0)
    if 'V16' not in V and 0.01 * f(m8c0) > tm:
        st['eb_2e9'] += 1
        return EB_SITE['2e9'], 2000000000, True
    if f(cost) == 0.0:
        st['eb_cost0'] += 1
        c = 2000000000
    else:
        q = f(it) / f(cost)
        c = int(round(q)) if 'V12' in V else cvttsd2si(q)
    if c <= 0:
        st['eb_one'] += 1
        return EB_SITE['one'], 1, False
    if (c <= -eta0) if 'V13' in V else (c < -eta0):
        st['eb_c'] += 1
        return EB_SITE['c'], c, False
    st['eb_cap'] += 1
    return EB_SITE['cap'], -eta0, False


def rule_it(e, V, st):
    """6edda1 (asm 0x5b7220-0x5b7328): T+0x64 after."""
    _, frm, t, a, b, p, sbmax, g808, m618, focus, nodes, before, after = e
    if sbmax < -1:
        st['it_neg'] += 1
        return -1
    if sbmax >= 0:
        st['it_param'] += 1
        return max(1, sbmax)
    if g808 == 0:
        st['it_off'] += 1
        return 1
    if m618 and g808 > 0:
        a = a * 3
    nd = f(nodes)
    if 'V14' not in V:
        if focus == 3 or nd > 100000.0:
            p = cdiv(p, 10)
        elif focus == 2 or nd > 10000.0:
            p = cdiv(p, 3)
    it = cdiv(a + 10 + cdiv(b, 10), p + 1)
    if p > 1000:
        st['it_p1000'] += 1
        return 1
    if p < 100:
        st['it_free'] += 1
        return -1
    if it >= 10:
        st['it_gt30' if it > 30 else 'it_val'] += 1
        return -1 if it > 30 else it
    if p > 500 or it <= 4:
        st['it_small'] += 1
        return 1
    st['it_val'] += 1
    return it


# ---- replay of one capture
def replay(d, V=frozenset(), name=''):
    st = collections.Counter()
    ev = d['events']
    eta, sbit, cache = {}, {}, {}  # per live table: eta, T+0x64, cache (None = no arrays, else {j: entries})
    sb_stack, pend_eb, pend_ws, inl_saved = [], None, None, {}
    prev = None
    for i, e in enumerate(ev):
        where = f'{name} event {i}'
        if isinstance(e, dict):
            if e['e'] == 'new' and e['t'] >= 0:
                need(e['c20'] == 0, f'K: {where}: new table with cache arrays')
                eta[e['t']], sbit[e['t']], cache[e['t']] = e['info'][0], e['info'][1], None
                st['new'] += 1
            elif e['e'] == 'free' and e['t'] >= 0:
                for m in (eta, sbit, cache):
                    m.pop(e['t'], None)
            prev = e
            continue
        k = e[0]
        if pend_eb is not None:
            need(k == 'set', f'EB: {where}: eta block not followed by 6edd89 ({k})')
        if pend_ws is not None:
            need(k == 'w' and e[1] == '542884', f'WS: {where}: 0x542863 probe not followed by the 6ee5d1 call')
        if k == 'f':
            st['f'] += 1
            got = rule_f(e, V, st)
            need(got == (e[11], e[12]), f'F: {where}: 6ef12f {e[:7]} dn/up {e[11:13]} != rule {got}')
            check_cache(cache, e[2], e[4], e[10], where)
        elif k == 'e':
            st['e'] += 1
            need(e[12] == eta.get(e[2]), f'H: {where}: 6eed2e eta {e[12]} != chain {eta.get(e[2])}')
            r = rule_e(e, V, st)
            need(r == e[14], f'E: {where}: 6eed2e {e[:8]} returned {e[14]}, rule {r}')
            check_cache(cache, e[2], e[3], e[13], where)
        elif k == 'a':
            st['a'] += 1
            r = rule_a(e, V, st)
            need(r == e[8], f'A: {where}: 6ee4ac returned {e[8]}, rule {r}')
            need(isinstance(prev, list) and prev[0] == 'f' and prev[1] == '542d6c' and e[1] == '542d89'
                 and prev[2] == e[2] and prev[4] == e[3] and prev[5] == e[4],
                 f'A: {where}: 6ee4ac not paired with the preceding 6ef12f call (same T, j, x)')
            check_cache(cache, e[2], e[3], e[7], where)
        elif k == 'ws':
            st['ws'] += 1
            pend_ws = e
        elif k == 'w':
            st['w'] += 1
            _, ra, t, j, w30, w38, args, had, n, negc, wr, rc = e
            need(rc == 0, f'W: {where}: 6ee5d1 rc {rc}')
            need((cache.get(t) is not None) == bool(had), f'K: {where}: cache arrays {had} but chain says {cache.get(t) is not None}')
            if not had:
                need(n > 0, f'W: {where}: allocation with n = {n}')
                need(negc == n - (1 if w30 != -1 else 0), f'W: {where}: {negc} stamps -1 after allocation, n = {n}')
                cache[t] = {}
                st['w_alloc'] += 1
            c2, c3 = rule_w(args, V, st)
            need(wr == [w30, w38, args[0], c2, c3], f'W: {where}: written {wr} != rule {[w30, w38, args[0], c2, c3]}')
            cache[t][j] = wr
            if pend_ws is not None:
                _, wx, wfd, dobj, p, sc, wfu, uobj, wj = pend_ws
                dd, du = bits((f(dobj) - f(p)) * f(sc)), bits((f(uobj) - f(p)) * f(sc))
                need([wx, wfd, dd, wfu, du] == args and wj == j, f'WS: {where}: call-site args {[wx, wfd, dd, wfu, du]} != {args}')
                x = f(wx)
                fr = x - fl(x)
                if wfd == bits(-fr) and wfu == bits(1.0 - fr):
                    st['ws_int'] += 1
                elif wfd == bits(-abs(x)) and f(wfu) == 1.0:
                    st['ws_cont'] += 1
                else:
                    raise Fail(f'WS: {where}: f- {wfd} f+ {wfu} match neither C:944-945 nor C:948-951 for x {wx}')
                pend_ws = None
            else:
                st['w_nows'] += 1
        elif k == 'sb':
            t = e[3]
            if e[1] == 'in':
                need(e[4] == eta.get(t) and e[5] == sbit.get(t), f'H: {where}: 62f909 entry eta/sbit {e[4:6]} != chain {eta.get(t)}/{sbit.get(t)}')
                sb_stack.append(e)
            else:
                ent = sb_stack.pop()
                need(e[4:6] == ent[4:6], f'H: {where}: 62f909 leave eta/sbit {e[4:6]} != entry {ent[4:6]}')
                need(e[4] == eta.get(t) and e[5] == sbit.get(t), f'H: {where}: 62f909 leave eta/sbit != chain')
                st['sb'] += 1
        elif k == 'eb':
            st['eb'] += 1
            need(e[2] < 0 and sb_stack and sb_stack[-1][4] == e[2], f'EB: {where}: eta0 {e[2]} != 62f909 entry eta')
            need(e[3] == sb_stack[-1][6], f'EB: {where}: sbcost differs from 62f909 entry')
            pend_eb = (e, rule_eb(e, V, st))
        elif k == 'set':
            _, ra, frm, t, v = e
            st['set:' + frm + ':' + ra] += 1
            if ra.startswith('i'):  # 62f909 inline writes: 1 around the root loops, then the saved eta (C:454-577)
                need(bool(sb_stack) and sb_stack[-1][3] == t, f'H: {where}: inline eta write outside 62f909 or to another table')
                if ra in INL_ONE:
                    need(v == 1, f'H: {where}: inline eta write {v} at {ra}')
                    inl_saved[t] = eta.get(t)
                    st['h_inl_one'] += 1
                else:
                    want = inl_saved.pop(t, eta.get(t))
                    need(v == want, f'H: {where}: inline eta restore {v} at {ra}, saved {want}')
                    st['h_inl_restore'] += 1
            elif pend_eb is not None:
                (_, (site, val, s1)) = pend_eb
                need(ra == site and v == val, f'EB: {where}: 6edd89 at {ra} value {v}, rule {site} {val}')
                if s1:
                    sbit[t] = -1
                pend_eb = None
            elif ra == RESTORE:
                need(sb_stack and v == sb_stack[-1][4], f'H: {where}: restore value {v} != 62f909 entry eta')
                if 'V17' not in V:
                    sbit[t] = sb_stack[-1][5]  # 0x542fba: T+0x64 = saved value right after this call
            else:
                need(ra not in EB_SITE.values(), f'EB: {where}: eta block 6edd89 without the block probe')
            if t in eta:
                eta[t] = v
        elif k == 'it':
            st['it'] += 1
            t = e[2]
            need(e[11] == sbit.get(t), f'H: {where}: 6edda1 sees T+0x64 {e[11]}, chain {sbit.get(t)}')
            r = rule_it(e, V, st)
            need(r == e[12], f'IT: {where}: 6edda1 {e[3:11]} wrote {e[12]}, rule {r}')
            sbit[t] = e[12]
        else:
            raise Fail(f'{where}: unknown event {k}')
        prev = e
    need(not sb_stack and pend_eb is None and pend_ws is None, f'{name}: unbalanced 62f909 / pending probe at the end')
    return st


def check_cache(cache, t, j, c, where):
    """K: what a reader saw equals the chain of 6ee5d1 writes (stamp -1 after allocation)."""
    ch = cache.get(t, 'gone')
    need(ch != 'gone', f'K: {where}: read of a table not alive in the chain')
    if ch is None:
        need(c is None, f'K: {where}: reader saw cache arrays the chain never allocated')
        return
    need(c is not None, f'K: {where}: reader saw no cache arrays after an allocation')
    w = ch.get(j)
    if w is None:
        need(c == [-1], f'K: {where}: unwritten cache entry j={j} seen as {c}')
    else:
        need(c[0] == w[0] and (len(c) == 1 or c == w), f'K: {where}: cache entry j={j} seen as {c}, last write {w}')


# ---- acceptance
# branches that must run at least once over the 6 cases (section 4, fixed from the exploration capture)
REQUIRED = ('f_hit_dn', 'f_hit_up', 'f_t2', 'f_tab', 'f_zero', 'f_floor', 'e_root', 'e_cache', 'e_rel', 'e_unrel',
            'a_hit', 'a_miss', 'w_alloc', 'w_floor', 'ws_int', 'eb_c', 'eb_cap', 'it_off', 'it_free', 'it_val',
            'h_inl_one', 'h_inl_restore')


def project_rd(d):
    return {k: v for k, v in d['counts'].items() if k.startswith('rd:')}


def check_case(c, d, old):
    need(d['ok'] and not d['errors'], f'S1: {c}: capture not ok')
    need(d['binary_sha256'] == BINARY and d['params'] == PARAMS, f'S1: {c}: binary or params differ')
    need(d['output'] == old['output'], f'S2: {c}: output {d["output"]!r} != 06e-2 {old["output"]!r}')
    need(project_rd(d) == project_rd(old), f'S2: {c}: rd counts differ from bench/oracle/m2_06e2')
    return replay(d, name=c)


def s1(manifest):
    out, tool = {}, sha(TOOL)
    for c in CASES:
        hs = []
        for r in RUNS:
            p = os.path.join(DATA, f'{c}_{r}.json.gz')
            need(os.path.exists(p), f'S1: missing {p}')
            hs.append(sha(p))
            need(manifest['files'].get(os.path.basename(p)) == hs[-1], f'S1: {p} sha256 != manifest')
        need(len(set(hs)) == 1, f'S1: {c}: runs differ {hs}')
        d = load(os.path.join(DATA, f'{c}_1.json.gz'))
        need(d['tool_sha256'] == tool, f'S1: {c}: capture tool sha256 {d["tool_sha256"]} != current tool {tool}')
        out[c] = d
    return out


def acceptance(docs, olds, quiet=False):
    stats = {}
    for c in CASES:
        stats[c] = check_case(c, docs[c], olds[c])
        if not quiet:
            print(c, 'PASS', json.dumps(dict(sorted(stats[c].items()))))
    tot = collections.Counter()
    for st in stats.values():
        tot.update(st)
    if not quiet:
        print('Q branch totals', json.dumps(dict(sorted(tot.items()))))
    need(all(tot[k] for k in REQUIRED), f'Q: a required branch never ran: {[k for k in REQUIRED if not tot[k]]}')
    return stats


# ---- self-test
def tampers(docs, olds):
    """T: each tamper must be rejected by the intended check (message prefix)."""
    def first(d, pred):
        for i, e in enumerate(d['events']):
            if isinstance(e, list) and pred(e):
                return i
        raise Fail('tamper target not found')

    def flip(h):
        return '%016x' % (int(h, 16) ^ 1)

    def t_f_dn(d):
        i = first(d, lambda e: e[0] == 'f'); d['events'][i][11] = flip(d['events'][i][11])

    def t_f_x(d):
        i = first(d, lambda e: e[0] == 'f' and e[1] != '542d6c'); d['events'][i][5] = bits(f(d['events'][i][5]) + 0.25)

    def t_f_cnt(d):
        i = first(d, lambda e: e[0] == 'f' and e[10] is None and e[8][0] > 0); d['events'][i][8][0] += 1

    def t_e_ret(d):
        i = first(d, lambda e: e[0] == 'e'); d['events'][i][14] ^= 1

    def t_e_eta(d):
        i = first(d, lambda e: e[0] == 'e'); d['events'][i][12] += 1

    def t_a_ret(d):
        i = first(d, lambda e: e[0] == 'a'); d['events'][i][8] ^= 1

    def t_a_x(d):
        i = first(d, lambda e: e[0] == 'a'); d['events'][i][4] = flip(d['events'][i][4])

    def t_w_c(d):
        i = first(d, lambda e: e[0] == 'w'); d['events'][i][10][3] = flip(d['events'][i][10][3])

    def t_ws(d):
        i = first(d, lambda e: e[0] == 'ws'); d['events'][i][2] = flip(d['events'][i][2])

    def t_k_stamp(d):
        i = first(d, lambda e: e[0] == 'f' and e[10] is not None and len(e[10]) == 1 and e[10][0] != -1)
        d['events'][i][10] = [-1]

    def t_k_drop(d):
        i = first(d, lambda e: e[0] == 'w' and e[7] == 1); del d['events'][i]; del d['events'][i - 1]

    def t_set(d):
        i = first(d, lambda e: e[0] == 'set' and e[1] == RESTORE); d['events'][i][4] += 1

    def t_eb(d):
        i = first(d, lambda e: e[0] == 'eb'); d['events'][i][4] = bits(f(d['events'][i][4]) * 3.0 + 50.0)

    def t_it(d):
        i = first(d, lambda e: e[0] == 'it' and e[7] != 0); d['events'][i][3] += 1000

    def t_rd(d):
        k = next(k for k in d['counts'] if k.startswith('rd:')); d['counts'][k] += 1

    plan = [('T1', t_f_dn, 'F:', 'markshare_4_0'), ('T2', t_f_x, 'F:', 'pk1'), ('T3', t_f_cnt, 'F:', 'neos5'),
            ('T4', t_e_ret, 'E:', 'markshare_4_0'), ('T5', t_e_eta, 'H:', 'pk1'), ('T6', t_a_ret, 'A:', 'neos5'),
            ('T7', t_a_x, 'A:', 'tr12-30'), ('T8', t_w_c, 'W:', 'pk1'), ('T9', t_ws, 'WS:', 'neos5'),
            ('T10', t_k_stamp, 'K:', 'tr12-30'), ('T11', t_k_drop, 'K:', 'pk1'), ('T12', t_set, 'H:', 'markshare_4_0'),
            ('T13', t_eb, 'EB:', 'pk1'), ('T14', t_it, 'IT:', 'tr12-30'), ('T15', t_rd, 'S2:', 'fhnw-binpack4-4')]
    for name, fn, want, c in plan:
        d = copy.deepcopy(docs[c])
        fn(d)
        try:
            check_case(c, d, olds[c])
            raise Fail(f'T {name}: tamper accepted')
        except Fail as ex:
            msg = str(ex)
            need(msg.startswith(want), f'T {name}: rejected by the wrong check: {msg[:200]}')
            print(f'T {name} ({c}): rejected [{msg[:110]}]')


ALL = frozenset(CASES)
NO_MS = ALL - {'markshare_4_0'}
FOUR = frozenset({'pk1', 'neos5', 'tr12-30', 'fhnw-binpack4-4'})
# variant -> cases where it must be rejected (section 3, fixed from the exploration capture before the reference capture)
VARIANTS = {
    'V1': ALL, 'V2': NO_MS, 'V3': frozenset(), 'V4': ALL - {'fhnw-binpack4-4'}, 'V5': ALL - {'fhnw-binpack4-4'},
    'V6': FOUR, 'V7': frozenset({'neos5', 'tr12-30', 'fhnw-binpack4-4'}), 'V8': ALL, 'V9': FOUR, 'V10': ALL, 'V11': ALL,
    'V12': FOUR, 'V13': frozenset({'neos5', 'tr12-30', 'fhnw-binpack4-4'}), 'V14': frozenset(), 'V15': ALL,
    # added after the reference capture (independent review): clauses the cases never decide
    'V16': frozenset(), 'V17': frozenset()}
SURVIVORS = ('V3', 'V14', 'V16', 'V17')  # undiscriminable in these cases (section 3)


def variants(docs):
    for v, want in VARIANTS.items():
        rej = set()
        for c in CASES:
            try:
                replay(docs[c], V=frozenset([v]), name=c)
            except Fail:
                rej.add(c)
        need(rej == set(want), f'V {v}: rejected in {sorted(rej)}, expected {sorted(want)}')
        need(bool(rej) == (v not in SURVIVORS), f'V {v}: survivor list {SURVIVORS} disagrees')
        print(f'V {v}: rejected in {sorted(rej)}')


def main():
    args = sys.argv[1:]
    if args[:1] == ['--sweep']:
        bad = 0
        for p in args[1:]:
            try:
                st = replay(load(p), name=os.path.basename(p))
                print(os.path.basename(p), 'PASS', json.dumps(dict(sorted(st.items()))))
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
                json.dump({'task': 'M2-06g', 'schema': 1, 'tool': 'tools/mip_sb_read.js', 'tool_sha256': sha(TOOL),
                           'binary_sha256': BINARY, 'params': PARAMS,
                           'models': '../re/bench/data/miplib/<case>.mps.gz', 'files': files}, fh, indent=1)
            print('manifest written', mpath)
            return 0
        need(os.path.exists(mpath), 'S1: no manifest (run --write-manifest once after the capture)')
        docs = s1(json.load(open(mpath)))
        olds = {c: load(os.path.join(OLD, f'{c}_1.json.gz')) for c in CASES}
        stats = acceptance(docs, olds)
        if '--self-test' in args:
            tampers(docs, olds)
            variants(docs)
        print('ACCEPTANCE PASS' + (' (+ self-test)' if '--self-test' in args else ''))
        return 0
    except Fail as ex:
        print('FAIL', ex)
        return 1


if __name__ == '__main__':
    sys.exit(main())
