#!/usr/bin/env python3
"""M2-06c-2 checker (docs/m2/mip_plunge_score.md): replay the 6e8eb3 rule 1/2/5/6 picks and every rule's body work from
the M2-06c captures (bench/oracle/m2_06c, tools/mip_plunge_rule.js). No new capture.

  python3 tools/mip_plunge_score_check.py              06c controls + R/G/S/T, then S2 (rule 1/2/5/6 pick) + W (work) + variants
  python3 tools/mip_plunge_score_check.py --self-test  the above, then tampers that must each be rejected
"""
import copy, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mip_plunge_rule_check as R  # noqa: E402
from mip_plunge_rule_check import Fail, need, f, h, one, calls, lcg, temper, minsd, maxsd, M100, E100  # noqa: E402

WORK = {1: (3.0, 4.0), 2: (3.0,), 3: (4.0,), 4: (7.0,), 5: (), 6: (3.0, 3.0), 7: (5.0,), 9: (5.0,), 11: (3.0,), 12: (4.0,)}
VARIANTS = {
    'W1': 'rule 1/6 ancestor comparison off', 'W2': 'maxd/anc from the last deepest item', 'W3': 'ancestor depth maxd-1',
    'W4': 'ancestor depth maxd-3', 'W5': 'null ancestor never matches', 'W6': 'rule 1 depth sign +', 'W7': 'rule 1 q sign +',
    'W8': 'rule 1/6 best also updated on a mismatch', 'W9': 'rule 2 tie goes to last', 'W10': 'rule 5 start = trunc(u*n)',
    'W11': 'rule 1/6 work added once (6n / 7n)'}
NULL_ANC_06C = 58                     # 06c null-ancestor calls over the five cases (docs/m2/mip_plunge_rule.md §6.2)


class Unrep(Exception):               # the call's tail did not record the chain this pick needs
    pass


def min_est(ests):                    # 0x5b3541-0x5b35fc, pairs: minsd then comisd/ja (same as 06c pick_replay)
    m, j = E100, 0
    while j + 1 < len(ests):
        m = minsd(m, ests[j])
        b = ests[j + 1]
        m = m if b > m else b
        j += 2
    if j < len(ests):
        m = minsd(m, ests[j])
    return m


def chains(c):
    """Node id -> (depth, id chain upward, tail d') from the call's tail 5fb13c events; None if the tail ancestor is null."""
    ancs = [e for e in c['ev'] if e['e'] == 'anc']
    need(ancs and ancs[0]['w'] == 'sel', 'chain: selected 5fb13c missing')
    if ancs[0]['res'] is None:
        return None
    out = {}
    for a in ancs:
        need(a['chain'][0] not in out, 'chain: node twice')
        out[a['chain'][0]] = (a['depth'], a['chain'], a['d'])
    return out


def anc_of(ch, ent, d):
    """5fb13c(N, d) for a pick-list entry: (d | depth - d) < 0 -> null, else depth - d parent steps (null stops)."""
    steps = ent[2] - d
    if (d | steps) < 0:
        return None
    if ch is None:
        raise Unrep('null tail ancestor')
    need(ent[0] in ch, f'chain: node {ent[0]} has no tail chain')
    depth, chain, dt = ch[ent[0]]
    need(depth == ent[2], 'chain: depth differs from the pick list')
    need(d >= dt, 'chain: needed depth above the tail d\'')
    if steps < len(chain):
        return chain[steps]
    if len(chain) >= 64:
        raise Unrep('chain cap')
    return None


def tint(s):                          # a7b4d6: 31-bit tempered integer; temper() returns it times 2^-31 exactly
    return int(temper(s) * 2147483648.0)


def replay(rule, ents, slot, ch, V):
    n = len(ents)
    ests = [f(e[1]) for e in ents]
    best, idx = M100, -1
    if rule == 2:
        for j, e in enumerate(ents):
            if e[3]:
                continue
            v = -ests[j]
            if v > best or ('W9' in V and v == best):
                idx = j
            best = maxsd(v, best)
    elif rule in (1, 6):
        off = 1 if 'W3' in V else 3 if 'W4' in V else 2
        maxd, top = 0, None           # the asm calls 5fb13c at every new maximum; only the last result is kept
        for j, e in enumerate(ents):
            if e[3]:
                continue
            if e[2] > maxd or ('W2' in V and e[2] == maxd and maxd > 0):
                maxd, top = e[2], e
        d = maxd - off
        anc = None if top is None else anc_of(ch, top, d)
        m = min_est(ests)
        for j, e in enumerate(ents):
            if e[3]:
                continue
            if rule == 6:
                sc = -ests[j]
            else:
                q = (ests[j] - m) / 1e8
                dep = float(e[2]) if 'W6' in V else float(-e[2])
                sc = dep + q if 'W7' in V else dep - q
            if sc > best:
                a = anc_of(ch, e, d)
                match = a == anc and not ('W5' in V and anc is None)
                if match or 'W1' in V:
                    idx, best = j, sc
                elif 'W8' in V:
                    best = sc
    elif rule == 5:
        after = lcg(slot)
        s = int(temper(after) * n) if 'W10' in V else tint(after) % n
        for i in range(n):
            j = (s + i) % n
            if not ents[j][3]:
                idx = j
                break
    else:
        raise Fail(f'rule {rule} pick is out of scope')
    return 0 if idx < 0 else idx


def work_after(rule, n, w, V):
    steps = WORK[rule]
    if 'W11' in V and rule in (1, 6):
        steps = (sum(steps),)
    for k in steps:
        w = float(n) * k + w
    return w


def new_stats():
    return {'pick': {}, 'unrep': {}, 'work': {}, 'filter': {1: 0, 6: 0}, 'r5_nonzero': 0}


def check_capture(d, name, V=frozenset()):
    st = new_stats()
    for i, c in enumerate(calls(d)):
        r = one(c, 'rule')
        if r is None:
            continue
        try:
            rule, pk = r['ecx'], one(c, 'pick')
            ents = pk['list']
            need(len(ents) == r['n'], 'list size')
            if r['work'] is None:
                need(pk['work'] is None, 'W: work pointer')
            else:
                w = work_after(rule, len(ents), f(r['work']), V)
                need(h(w) == pk['work'], f'W: rule {rule} work {h(w)} != captured {pk["work"]}')
                st['work'][rule] = st['work'].get(rule, 0) + 1
            if rule not in (1, 2, 5, 6):
                continue
            ch = chains(c) if rule in (1, 6) else None
            slot = int(r['st'], 16)
            try:
                idx = replay(rule, ents, slot, ch, V)
            except Unrep as ex:
                need(rule in (1, 6), 'unreplayable outside rule 1/6')
                st['unrep'][str(ex)] = st['unrep'].get(str(ex), 0) + 1
                continue
            need(pk['idx'] == idx, f'S2: rule {rule} pick {idx} != captured {pk["idx"]}')
            st['pick'][rule] = st['pick'].get(rule, 0) + 1
            if rule in (1, 6) and not V and replay(rule, ents, slot, ch, {'W1'}) != idx:
                st['filter'][rule] += 1
            if rule == 5 and tint(lcg(slot)) % len(ents) != 0:
                st['r5_nonzero'] += 1
        except Fail as ex:
            raise Fail(f'{name} call {i} (k {c["pin"]["k"]}): {ex}')
    return st


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


TARGETS = [('rule 1 picks', lambda t: t['pick'].get(1, 0)), ('rule 2 picks', lambda t: t['pick'].get(2, 0)),
           ('rule 5 picks', lambda t: t['pick'].get(5, 0)), ('rule 6 picks', lambda t: t['pick'].get(6, 0)),
           ('rule 1 picks changed by the ancestor test', lambda t: t['filter'][1]),
           ('rule 6 picks changed by the ancestor test', lambda t: t['filter'][6]),
           ('rule 5 picks with start != 0', lambda t: t['r5_nonzero']),
           ('rule 4 work', lambda t: t['work'].get(4, 0)), ('rule 7 work', lambda t: t['work'].get(7, 0))]


def acceptance(caps, quiet=False):
    stats = {case: check_capture(runs[0], case) for case, runs in caps.items()}
    tot = total(stats)
    if not quiet:
        for case, s in stats.items():
            print(f'  {case}: S2/W PASS picks {dict(sorted(s["pick"].items()))} work {dict(sorted(s["work"].items()))} '
                  f'ancestor-decided {s["filter"]} rule5 start!=0 {s["r5_nonzero"]} unreplayable {s["unrep"]}')
    for label, fn in TARGETS:
        need(fn(tot) > 0, f'acceptance: no {label}')
    nu = sum(tot['unrep'].values())
    need(nu <= NULL_ANC_06C, f'acceptance: {nu} unreplayable > {NULL_ANC_06C}')
    return stats, tot


def variants(caps):
    print('variants (rejected cases; survival = clause not discriminated by these runs):')
    for v, desc in VARIANTS.items():
        rej = []
        for case, runs in caps.items():
            try:
                check_capture(runs[0], case, frozenset([v]))
            except Fail:
                rej.append(case)
        print(f'  {v} {desc}: ' + (f'rejected {", ".join(rej)}' if rej else 'SURVIVES'))


def tampers(caps, old, manifest):
    def calls_of(d, rules):
        return [c for c in calls(d) if one(c, 'rule') and one(c, 'rule')['ecx'] in rules]

    def first_pick(d, rule, pred=lambda c: True):
        for c in calls_of(d, (rule,)):
            if pred(c):
                return c
        raise Fail(f'tamper target for rule {rule} not found')

    def bump_idx(rule):
        def g(d):
            pk = one(first_pick(d, rule, lambda c: one(c, 'pick')['list'].__len__() > 1), 'pick')
            pk['idx'] = (pk['idx'] + 1) % len(pk['list'])
        return g

    def decided(c, rule):
        r, pk = one(c, 'rule'), one(c, 'pick')
        ch = chains(c)
        try:
            return ch is not None and pk['idx'] != 0 and replay(rule, pk['list'], 0, ch, {'W1'}) != pk['idx']
        except Unrep:
            return False

    def chain_anc(d):                 # rule 6: change the first-deepest node's ancestor id in its tail chain
        c = first_pick(d, 6, lambda c: decided(c, 6))
        ents = one(c, 'pick')['list']
        maxd = max(e[2] for e in ents if not e[3])
        deep = next(e for e in ents if not e[3] and e[2] == maxd)
        a = next(e for e in c['ev'] if e['e'] == 'anc' and e['chain'][0] == deep[0])
        a['chain'][2] = -12345.0

    def est1(d):                      # rule 1: make the picked item's est huge
        c = first_pick(d, 1, lambda c: decided(c, 1))
        pk = one(c, 'pick')
        pk['list'][pk['idx']][1] = h(1e30)

    def slot5(d):                     # first rule 5 call where one extra LCG step moves the start
        def moves(c):
            s, n = int(one(c, 'rule')['st'], 16), len(one(c, 'pick')['list'])
            return tint(lcg(lcg(s))) % n != tint(lcg(s)) % n
        c = first_pick(d, 5, moves)
        r = one(c, 'rule')
        r['st'] = f'{lcg(int(r["st"], 16)):016x}'

    def work(rule):
        def g(d):
            pk = one(first_pick(d, rule), 'pick')
            pk['work'] = h(f(pk['work']) + 1.0)
        return g

    def drop16(d):
        keep, cur = [], []
        for e in d['events']:
            if e['e'] == 'pin':
                cur = [e]
                continue
            if cur:
                cur.append(e)
                if e['e'] == 'leave':
                    r = next((x for x in cur if x['e'] == 'rule'), None)
                    if not (r and r['ecx'] in (1, 6)):
                        keep += cur
                    cur = []
                continue
            keep.append(e)
        d['events'] = keep

    cases = [('rule 1 pick index', 'pk1', bump_idx(1)), ('rule 2 pick index', 'pk1', bump_idx(2)),
             ('rule 5 pick index', 'markshare_4_0', bump_idx(5)), ('rule 6 pick index', 'pk1', bump_idx(6)),
             ('rule 6 ancestor id in a tail chain', None, chain_anc), ('rule 1 est', None, est1),
             ('rule 5 RNG slot', 'markshare_4_0', slot5), ('rule 4 work', 'markshare_4_0', work(4)),
             ('rule 7 work', 'markshare_4_0', work(7)), ('rule 1/6 calls deleted', 'all', drop16)]
    bad = []
    for label, case, fn in cases:
        c2 = {k: [copy.deepcopy(v[0])] for k, v in caps.items()}
        targets = list(c2) if case in (None, 'all') else [case]
        done = False
        for k in targets:
            try:
                fn(c2[k][0])
                done = True
                if case is None:
                    break
            except (Fail, StopIteration):
                continue
        need(done, f'tamper {label}: no target')
        try:
            acceptance(c2, quiet=True)
            bad.append(label)
            print(f'  tamper {label}: NOT rejected')
        except Fail as ex:
            print(f'  tamper {label}: rejected ({str(ex)[:90]})')
    m2 = copy.deepcopy(manifest)
    m2['binary_sha256'] = '0' * 64
    try:
        R.controls(caps, m2, old)
        bad.append('binary hash zero')
    except Fail:
        print('  tamper binary hash zero: rejected')
    need(not bad, f'tampers not rejected: {bad}')


def main():
    selftest = '--self-test' in sys.argv[1:]
    try:
        caps, old, manifest = R.load_all()
        R.controls(caps, manifest, old)
        print('C0 PASS (06c captures, 3 identical runs per case, 06a projections)')
        R.acceptance(caps, quiet=True)
        print('06c R/G/S/T PASS')
        acceptance(caps)
        variants(caps)
        if selftest:
            tampers(caps, old, manifest)
        print('ACCEPTANCE PASS' + (' (+ self-test)' if selftest else ''))
    except Fail as ex:
        print('FAIL:', ex)
        sys.exit(1)


if __name__ == '__main__':
    main()
