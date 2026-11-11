#!/usr/bin/env python3
"""M2-06c-3 checker (docs/m2/mip_plunge_r47.md): replay the 6e8eb3 rule 4/7/9 picks, the rule 1/6 body 5fb13c queries and
the RNG slot after rule 5 from bench/oracle/m2_06c3 (tools/mip_plunge_score.js = the 06c tool + ents/x4/ranc/r5 events).

  python3 tools/mip_plunge_r47_check.py                   C0 controls, C1 (06c and 06c-2 checkers on the projections),
                                                          S4/S7/S9/SA/S5, variants
  python3 tools/mip_plunge_r47_check.py --self-test       the above, then tampers that must each be rejected
  python3 tools/mip_plunge_r47_check.py --write-manifest  once, after the captures
"""
import copy, json, math, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mip_plunge_rule_check as R  # noqa: E402
import mip_plunge_score_check as S  # noqa: E402
from mip_plunge_rule_check import Fail, need, f, h, one, calls, lcg, maxsd, sha, load, M100  # noqa: E402

ROOT = R.ROOT
DATA = os.path.join(ROOT, 'bench/oracle/m2_06c3')
TOOL = os.path.join(ROOT, 'tools/mip_plunge_score.js')
CASES = dict(R.CASES, **{'pk1-bd1': ('pk1', {'BranchDir': 1}), 'pk1-bd-1': ('pk1', {'BranchDir': -1})})
CONTROL = {'pk1-bd1': 'control/pk1-bd1_06c.json.gz', 'pk1-bd-1': 'control/pk1-bd-1_06c.json.gz'}
NEWE = {'ents', 'x4', 'ranc', 'r5'}
VARIANTS = {
    'X1': 'rule 4 floor/ceil swapped', 'X2': 'rule 4 no rounding', 'X3': 'rule 4 divisor 10', 'X4': 'rule 4 no depth term',
    'X5': 'rule 4 tie >= (last)', 'X6': 'rule 4 I+0x34 < 0 not skipped (diff 0)', 'X7': 'rule 7 direction byte ignored',
    'X8': 'rule 7 direction byte sign -', 'X9': 'rule 7 depth factor 1', 'X10': 'rule 9 sign flipped',
    'X11': 'rule 9 direction byte ignored', 'X12': 'rule 1/6 first loop: last deepest item', 'X13': 'rule 1/6 second loop tie >=',
    'X14': 'rule 1/6 second loop queries every item', 'X15': 'rule 5 slot after = seed (no LCG step)'}


def sx8(b):
    return b - 256 if b >= 128 else b


def proj(d):
    return [e for e in d['events'] if e['e'] not in NEWE]


# ---------------------------------------------------------------- loading and controls
def load_all():
    manifest = json.load(open(os.path.join(DATA, 'manifest.json')))
    caps, base = {}, {}
    for case in CASES:
        runs = []
        for r in (1, 2, 3):
            fn = f'{case}_{r}.json.gz'
            need(manifest['files'][fn] == sha(os.path.join(DATA, fn)), f'{fn}: file hash differs from the manifest')
            runs.append(load(os.path.join(DATA, fn)))
        caps[case] = runs
        if case in CONTROL:
            fn = CONTROL[case]
            need(manifest['control_files'][fn] == sha(os.path.join(DATA, fn)), f'{fn}: file hash differs from the manifest')
            base[case] = load(os.path.join(DATA, fn))
        else:                         # 06c run 1, checked against the 06c manifest (added after the review)
            fn = os.path.join(R.DATA, f'{case}_1.json.gz')
            need(json.load(open(os.path.join(R.DATA, 'manifest.json')))['files'][f'{case}_1.json.gz'] == sha(fn),
                 f'{case}: 06c baseline hash differs from the 06c manifest')
            base[case] = load(fn)
    return caps, base, manifest


def controls(caps, base, manifest):
    old_tool = json.load(open(os.path.join(R.DATA, 'manifest.json')))['tool_sha256']
    need(manifest['binary_sha256'] == R.BINARY, 'manifest binary hash')
    need(manifest['tool_sha256'] == sha(TOOL), 'manifest tool hash differs from tools/mip_plunge_score.js')
    for case, runs in caps.items():
        model, extra = CASES[case]
        params = dict(R.BASE, **extra)
        for r, d in enumerate(runs, 1):
            nm = f'{case}_{r}'
            need(d['binary_sha256'] == R.BINARY and d['tool_sha256'] == manifest['tool_sha256'], f'{nm}: binary/tool hash')
            need(d['kind'] == 'mip_plunge_score', f'{nm}: kind')
            need(d['model_sha256'] == manifest['models'][model], f'{nm}: model hash')
            need(d['params'] == params, f'{nm}: params')
            need(d['ok'] and not d['errors'] and d['output'].startswith('DONE ') and d['output'] == runs[0]['output'],
                 f'{nm}: run output {d["output"]}')
            need(d['events'], f'{nm}: no events')
            need(d['counts'] == runs[0]['counts'], f'{nm}: counts differ from run 1')
            for i, (a, b) in enumerate(zip(d['events'], runs[0]['events'])):
                need(a == b, f'{nm}: event {i} differs from run 1: {a} != {b}')
            need(len(d['events']) == len(runs[0]['events']), f'{nm}: event count differs from run 1')
        o = base[case]
        need(o['tool_sha256'] == old_tool and o['params'] == params and o['output'] == runs[0]['output'],
             f'{case}: 06c-tool baseline tool/params/output')
        need(o['binary_sha256'] == R.BINARY and o['model_sha256'] == manifest['models'][model] and o['ok'] and not o['errors'],
             f'{case}: 06c-tool baseline binary/model/ok')
        need(runs[0]['counts'] == o['counts'], f'{case}: counts differ from the 06c-tool run')
        ev = proj(runs[0])
        for i, (a, b) in enumerate(zip(ev, o['events'])):
            need(a == b, f'{case}: projection event {i} differs from the 06c-tool run: {a} != {b}')
        need(len(ev) == len(o['events']), f'{case}: projection length differs from the 06c-tool run')


def regress(caps):
    """C1: the 06c (R/G/S/T) and 06c-2 (S2/W) checkers on each run-1 projection."""
    out = {}
    for case, runs in caps.items():
        d = dict(runs[0], events=proj(runs[0]))
        R.check_capture(d, case)
        out[case] = S.check_capture(d, case)
    return out


# ---------------------------------------------------------------- replays
def rule4(ents, en, x4, V, st):
    need(x4 is not None, 'S4: x4 missing in a rule 4 call')
    need(x4['null'] == 0, 'S4: 5e6f57 returned null in a rule 4 call')
    xs = {j: (v, f(hx)) for j, v, hx in x4['x']}
    best, idx, read = M100, -1, []
    for j, e in enumerate(ents):
        if e[3]:
            continue
        v34, v38, b40 = en[j]
        if v34 < 0:
            if 'X6' not in V:
                continue
            diff = 0.0
        else:
            read.append(j)
            need(j in xs and xs[j][0] == v34, f'S4: x4 has no x[{v34}] for entry {j}')
            val = f(v38)
            if 'X2' in V:
                b = val
            else:
                lt = (b40 == 0x3c) != ('X1' in V)
                b = float(math.floor(val) if lt else math.ceil(val))
                st['r4path']['floor' if b40 == 0x3c else 'ceil'] += 1
            diff = abs(xs[j][1] - b) / (10.0 if 'X3' in V else 100.0)
        sc = (0.0 if 'X4' in V else float(e[2])) - diff
        if sc > best or ('X5' in V and sc == best):
            idx = j
        best = maxsd(sc, best)
    need(sorted(xs) == read, f'S4: x4 entries {sorted(xs)} != entries read {read}')
    return idx


def rule79(rule, ents, en, r10d, V):
    m = S.min_est([f(e[1]) for e in ents])
    best, idx = M100, -1
    for j, e in enumerate(ents):
        if e[3]:
            continue
        b = sx8(en[j][2])
        q = (f(e[1]) - m) / 1e8
        if rule == 7:
            b = 0 if 'X7' in V else -b if 'X8' in V else b
            sc = float(b + (1 if 'X9' in V else 10) * e[2]) - q
        else:
            base = float(10 * e[2]) - q
            bb = 0.0 if 'X11' in V else float(b)
            neg = (r10d == -1) != ('X10' in V)
            sc = base - bb if neg else base + bb
        if sc > best:
            idx = j
        best = maxsd(sc, best)
    return idx


def rule16(rule, ents, ranc, ch, V, st):
    """Rules 1/6 with the body 5fb13c results taken from ranc; the query sequence (site, start id, d) must match."""
    q = iter(ranc)
    sa, sb = ('6a', '6b') if rule == 6 else ('1a', '1b')

    def query(site, e, d):
        r = next(q, None)
        need(r is not None, f'SA: 5fb13c query ({site}, node {e[0]}, d {d}) not captured')
        need((r['at'], r['node'], r['d']) == (site, e[0], d),
             f'SA: query ({site}, {e[0]}, {d}) != captured ({r["at"]}, {r["node"]}, {r["d"]})')
        need(r['depth'] == e[2], 'SA: start depth differs from the pick list')
        if (d | (e[2] - d)) < 0:
            need(r['res'] is None, 'SA: 5fb13c must return null for (d | depth - d) < 0')
        elif ch is not None and e[0] in ch and d >= ch[e[0]][2]:
            try:
                need(S.anc_of(ch, e, d) == r['res'], 'SA: 5fb13c result differs from the tail chain walk')
                st['chain_checked'] += 1
            except S.Unrep:
                pass
        return r['res']

    maxd, anc = 0, None
    for e in ents:
        if e[3]:
            continue
        if e[2] > maxd or ('X12' in V and e[2] == maxd and maxd > 0):
            maxd = e[2]
            anc = query(sa, e, e[2] - 2)
    ests = [f(x[1]) for x in ents]
    m = S.min_est(ests)
    best, idx = M100, -1
    for j, e in enumerate(ents):
        if e[3]:
            continue
        if rule == 6:
            sc = -ests[j]
        else:
            sc = float(-e[2]) - (ests[j] - m) / 1e8
        hit = sc > best or ('X13' in V and sc == best)
        if hit or 'X14' in V:
            a = query(sb, e, maxd - 2)
            if hit and a == anc:
                idx, best = j, sc
    need(next(q, None) is None, 'SA: extra captured 5fb13c queries')
    return idx


def new_stats():
    return {'pick': {}, 'bd': {1: 0, -1: 0}, 'r4path': {'floor': 0, 'ceil': 0}, 'b40': {}, 'x4': 0, 'r5': 0,
            'unrep06c2': 0, 'chain_checked': 0, 'ranc': 0, 'x_int': 0, 'x_frac': 0}


def check_capture(d, name, V=frozenset()):
    st = new_stats()
    for i, c in enumerate(calls(d)):
        r = one(c, 'rule')
        if r is None:
            need(not any(e['e'] in NEWE for e in c['ev']), 'new events in an empty call')
            continue
        try:
            rule, pk, en = r['ecx'], one(c, 'pick'), one(c, 'ents')
            ents = pk['list']
            need(en is not None and len(en['ents']) == len(ents), 'ents missing or wrong size')
            en = en['ents']
            for x in en:
                st['b40'][x[2]] = st['b40'].get(x[2], 0) + 1
            x4, r5 = one(c, 'x4'), one(c, 'r5')
            ranc = [e for e in c['ev'] if e['e'] == 'ranc']
            need(rule == 4 or x4 is None, 'x4 outside rule 4')
            need(rule == 5 or r5 is None, 'r5 outside rule 5')
            need(rule in (1, 6) or not ranc, 'ranc outside rule 1/6')
            need(r['r10d'] == c['pin']['bdir'], 'rule r10d differs from BranchDir')
            if rule == 4:
                idx = rule4(ents, en, x4, V, st)
                st['x4'] += 1
                for _, _, hx in x4['x']:
                    st['x_int' if f(hx) == math.floor(f(hx)) else 'x_frac'] += 1
            elif rule in (7, 9):
                idx = rule79(rule, ents, en, r['r10d'], V)
                if rule == 9 and len(ents) > 1:
                    st['bd'][r['r10d']] += 1
            elif rule in (1, 6):
                ch = S.chains(c)
                idx = rule16(rule, ents, ranc, ch, V, st)
                st['ranc'] += len(ranc)
                try:
                    S.replay(rule, ents, 0, ch, frozenset())
                except S.Unrep:
                    st['unrep06c2'] += 1
            elif rule == 5:
                need(r5 is not None, 'S5: r5 missing in a rule 5 call')
                after = int(r['st'], 16) if 'X15' in V else lcg(int(r['st'], 16))
                need(int(r5['st'], 16) == after, f'S5: slot {r5["st"]} != {after:016x}')
                need(r5['eax'] == S.tint(after), 'S5: a7b4d6 return differs from the tempered state')
                idx = S.replay(5, ents, int(r['st'], 16), None, frozenset())
                st['r5'] += 1
            else:
                continue
            idx = 0 if idx < 0 else idx
            need(pk['idx'] == idx, f'S{rule}: pick {idx} != captured {pk["idx"]}')
            key = rule if len(ents) > 1 else f'{rule}(n=1)'
            st['pick'][key] = st['pick'].get(key, 0) + 1
        except Fail as ex:
            raise Fail(f'{name} call {i} (k {c["pin"]["k"]}, rule {r["ecx"]}): {ex}')
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


TARGETS = [('rule 4 picks (n>1)', lambda t: t['pick'].get(4, 0)), ('rule 7 picks (n>1)', lambda t: t['pick'].get(7, 0)),
           ('rule 9 picks r10d=1 (n>1)', lambda t: t['bd'][1]), ('rule 9 picks r10d=-1 (n>1)', lambda t: t['bd'][-1]),
           ('rule 4 floor entries', lambda t: t['r4path']['floor']), ('rule 4 ceil entries', lambda t: t['r4path']['ceil']),
           ('rule 1/6 calls unreplayable in 06c-2', lambda t: t['unrep06c2']), ('r5', lambda t: t['r5'])]


def acceptance(caps, quiet=False):
    stats = {case: check_capture(runs[0], case) for case, runs in caps.items()}
    tot = total(stats)
    if not quiet:
        for case, s in stats.items():
            print(f'  {case}: PASS picks {dict(sorted(s["pick"].items(), key=str))} rule9 n>1 {s["bd"]} '
                  f'rule4 entries {s["r4path"]} x int/frac {s["x_int"]}/{s["x_frac"]} ranc {s["ranc"]} '
                  f'(chain-checked {s["chain_checked"]}) 06c-2-unreplayable now replayed {s["unrep06c2"]} r5 {s["r5"]} '
                  f'I+0x40 {dict(sorted(s["b40"].items()))}')
    for label, fn in TARGETS:
        need(fn(tot) > 0, f'acceptance: no {label}')
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


# ---------------------------------------------------------------- self-test
def tampers(caps, base, manifest):
    def flips(c, mutate):             # does mutating a copy of call c change the replayed pick?
        c2 = copy.deepcopy(c)
        mutate(c2)
        try:
            check_capture({'events': [c2['pin']] + c2['ev']}, 'probe')
            return False
        except Fail:
            return True

    def bump(c):
        pk = one(c, 'pick')
        pk['idx'] = (pk['idx'] + 1) % len(pk['list'])

    def x_sel(c):
        pk, x4 = one(c, 'pick'), one(c, 'x4')
        for row in x4['x']:
            if row[0] == pk['idx']:
                row[2] = h(f(row[2]) + 1e6)

    def b40_sel(c):
        e = one(c, 'ents')['ents'][one(c, 'pick')['idx']]
        e[2] = 0x80

    def r10d(c):
        r = one(c, 'rule')
        r['r10d'] = -r['r10d']
        c['pin']['bdir'] = r['r10d']

    def ranc_res(c):                  # picked item's second-loop query no longer matches; only in calls without a tail chain
        if S.chains(c) is not None:   # (06c-2 unreplayable), so the pick replay alone must reject it
            return
        sel = one(c, 'pick')['list'][one(c, 'pick')['idx']][0]
        for e in c['ev']:
            if e['e'] == 'ranc' and e['at'] in ('1b', '6b') and e['node'] == sel and e['res'] is not None:
                e['res'] = -7.0

    def x4_del(c):
        c['ev'] = [e for e in c['ev'] if e['e'] != 'x4']

    def x4_null(c):
        one(c, 'x4')['null'] = 1

    def ranc_del(c):
        i = next(i for i, e in enumerate(c['ev']) if e['e'] == 'ranc')
        del c['ev'][i]

    def r5_st(c):
        x = one(c, 'r5')
        x['st'] = f'{lcg(int(x["st"], 16)):016x}'

    def on_calls(rules, mutate, flip=True):
        def g(caps2):                 # run 1 only (acceptance reads run 1): first call (n > 1) where the change flips the pick
            for d in (runs[0] for runs in caps2.values()):
                cs = calls(d)
                for c in cs:
                    r = one(c, 'rule')
                    if r and r['ecx'] in rules and len(one(c, 'pick')['list']) > 1 and (not flip or flips(c, mutate)):
                        mutate(c)
                        return rebuild(d, cs)
            raise Fail('no target')
        return g

    def rebuild(d, cs):               # flat event list from call groups (events outside calls dropped: acceptance ignores them)
        d['events'] = [x for c in cs for x in [c['pin']] + c['ev']]

    def drop4(caps2):
        for runs in caps2.values():
            for d in runs[:1]:
                keep, cur = [], []
                for e in d['events']:
                    if e['e'] == 'pin':
                        cur = [e]
                        continue
                    if cur:
                        cur.append(e)
                        if e['e'] == 'leave':
                            r = next((x for x in cur if x['e'] == 'rule'), None)
                            if not (r and r['ecx'] == 4):
                                keep += cur
                            cur = []
                        continue
                    keep.append(e)
                d['events'] = keep

    def proj_est(caps2):
        for d in caps2['pk1']:        # every run, so only the projection check can see it
            pk = next(e for e in d['events'] if e['e'] == 'pick')
            pk['list'][0][1] = h(12345.0) if pk['list'][0][1] != h(12345.0) else h(1.0)

    def run2(caps2):
        e = next(e for e in caps2['pk1'][1]['events'] if e['e'] == 'ents')
        e['ents'][0][0] += 1

    cases = [('rule 4 pick index', on_calls((4,), bump)), ('rule 7 pick index', on_calls((7,), bump)),
             ('rule 9 pick index', on_calls((9,), bump)), ('rule 4 x of the picked entry', on_calls((4,), x_sel)),
             ('rule 4 x4 deleted', on_calls((4,), x4_del, False)), ('rule 4 x4 null', on_calls((4,), x4_null, False)),
             ('rule 7 I+0x40 of the picked entry', on_calls((7,), b40_sel)), ('rule 9 r10d', on_calls((9,), r10d)),
             ('ranc result of the picked entry', on_calls((1, 6), ranc_res)), ('ranc event deleted', on_calls((1, 6), ranc_del, False)),
             ('r5 slot', on_calls((5,), r5_st, False)), ('rule 4 calls deleted', drop4)]
    ctrl = [('projection: pick.list est (all runs)', proj_est), ('run 2 event', run2)]
    bad = []
    for label, fn in cases:
        c2 = copy.deepcopy(caps)
        try:
            fn(c2)
        except Fail as ex:
            bad.append(label + ' (no target)')
            print(f'  tamper {label}: NO TARGET ({ex})')
            continue
        try:
            acceptance(c2, quiet=True)
            bad.append(label)
            print(f'  tamper {label}: NOT rejected')
        except Fail as ex:
            print(f'  tamper {label}: rejected ({str(ex)[:100]})')
    for label, fn in ctrl:
        c2 = copy.deepcopy(caps)
        fn(c2)
        try:
            controls(c2, base, manifest)
            bad.append(label)
            print(f'  tamper {label}: NOT rejected')
        except Fail as ex:
            print(f'  tamper {label}: rejected ({str(ex)[:100]})')
    m2 = copy.deepcopy(manifest)
    m2['binary_sha256'] = '0' * 64
    try:
        controls(caps, base, m2)
        bad.append('binary hash zero')
    except Fail:
        print('  tamper binary hash zero: rejected')
    need(not bad, f'tampers not rejected: {bad}')


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
    models = {m: sha(os.path.join(R.MODELS, m + '.mps.gz')) for m, _ in CASES.values()}
    json.dump({'task': 'M2-06c-3', 'schema': 1, 'tool': 'tools/mip_plunge_score.js', 'tool_sha256': tool, 'binary_sha256': R.BINARY,
               'base_params': R.BASE, 'cases': {c: {'model': m + '.mps.gz', **dict(R.BASE, **x)} for c, (m, x) in CASES.items()},
               'models': models, 'model_paths': '../re/bench/data/miplib/<model>.mps.gz', 'files': files,
               'control_tool': 'tools/mip_plunge_rule.js (06c, pinned)',
               'control_files': {fn: sha(os.path.join(DATA, fn)) for fn in CONTROL.values()}},
              open(mp, 'w'), indent=1)
    print('wrote', mp)


def main():
    a = sys.argv[1:]
    if a[:1] == ['--write-manifest']:
        write_manifest()
        return 0
    try:
        caps, base, manifest = load_all()
        controls(caps, base, manifest)
        print('C0 PASS (manifest hashes, 3 identical runs per case, projections equal to the 06c-tool runs)')
        reg = regress(caps)
        print('C1 PASS (06c R/G/S/T and 06c-2 S2/W on every projection; 06c-2 unreplayable '
              f'{sum(sum(s["unrep"].values()) for s in reg.values())})')
        stats, tot = acceptance(caps)
        print('  totals: ' + ', '.join(f'{lbl} {fn(tot)}' for lbl, fn in TARGETS) + f', picks {dict(sorted(tot["pick"].items(), key=str))}')
        variants(caps)
        if a[:1] == ['--self-test']:
            print('self-test:')
            tampers(caps, base, manifest)
        print('ACCEPTANCE PASS' + (' (+ self-test)' if a[:1] == ['--self-test'] else ''))
    except Fail as ex:
        print('FAIL', ex)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
