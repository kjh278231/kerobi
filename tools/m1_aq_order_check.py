#!/usr/bin/env python3
"""M1-02h: Rust alpha_q list order of the dual DSE pivot vs the original 09d112 / 8b473 on kb2 pivots 0-37.
Contract: docs/m1/m1_02h_aq_order.md section 3. Exit 0 only when every check passes.

Rust side: RSOLVER_SNAP of this worktree's build (fields f0 = LU+0xf0 list by position at the snapshot, and for the
previous pivot aqo = final alpha_q list, uk = predicted U kernel, fill = spike rows ordered by the sparse-L
shortcut). Original side: the M1-02a-1 captures (09d112 entry state ft, 8b473 return aq).
  --snap FILE   use a stored Rust snapshot instead of running the build (code mutants)
  --self-test   also the tampers (T)
"""
import copy, gzip, json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from m1_list_order_check import CAP, ORACLE, SHA, sha  # noqa: E402

U = ('21686f', '21841f', '2126cd')
NP = 38


def load_caps():
    return [{int(e['k']): e for e in json.loads(gzip.decompress(open(p, 'rb').read()))['events']} for p in CAP]


def load_oracle():
    g = {}
    for line in gzip.decompress(open(ORACLE, 'rb').read()).decode().splitlines():
        s = json.loads(line)
        g.setdefault(s['iter'], s)
    return g


def rust_snapshot(path=None):
    if path is None:
        from m1_beta_check import rust_snapshot as run
        return run()
    mine = {}
    for line in open(path):
        s = json.loads(line)
        mine.setdefault(s['iter'], s)
    return mine


def listed(ft):
    f8 = ft['f8']
    return sorted((r for r in range(len(f8)) if f8[r] >= 0), key=lambda r: f8[r])


def kernel(ft):
    return next((c for c in (x.split('@')[0] for x in ft['calls']) if c in U), 'inline')


def check(caps, oracle, mine, shas=None):
    """Returns (failures, report lines)."""
    bad, rep = [], []
    for p in [ORACLE, *CAP]:
        got = (shas or {}).get(p) or sha(p)
        if got != SHA[p]:
            bad.append(f'M {os.path.basename(p)}: sha {got[:12]}')
    c0 = caps[0]
    if sorted(c0) != list(range(NP)):
        bad.append(f'M: capture pivots {sorted(c0)[:3]}.. != 0-{NP - 1}')
        return bad, rep
    for i, c in enumerate(caps[1:], 2):
        for k in range(NP):
            a, b = c0[k], c[k]
            if (a['ft']['f8'], a['aq']['idx'], a['ft']['calls'], a['r']) != (b['ft']['f8'], b['aq']['idx'], b['ft']['calls'], b['r']):
                bad.append(f'M: capture run {i} differs from run 1 at pivot {k}')
                break
    # N: same path. Snapshots 0-30 have stored oracle vstat; every pivot must have the same alpha_q support and r
    for k in range(min(31, NP + 1)):
        if mine.get(k, {}).get('vstat') != oracle[k]['vstat']:
            bad.append(f'N: Rust snapshot {k} vstat != stored oracle')
            break
    for k in range(NP):
        s1 = mine.get(k + 1, {})
        if 'aqo' not in s1 or 'f0' not in mine.get(k, {}):
            bad.append(f'N: Rust snapshot {k} or {k + 1} lacks f0 / aqo (old build?)')
            return bad, rep
        e = c0[k]
        if sorted(s1['aqo']) != sorted(e['aq']['idx']):
            bad.append(f'N pivot {k}: alpha_q support differs: Rust {sorted(s1["aqo"])} original {sorted(e["aq"]["idx"])}')
    rep.append(f'N: Rust vstat = stored oracle on snapshots 0-30, alpha_q support = capture on pivots 0-{NP - 1}')
    # scope (contract section 1): a pivot whose spike has >= 2 rows ordered by the sparse-L shortcut (fill) and whose
    # next list differs from the original taints the lists until the next refactor (Rust snapshot k = 0 updates)
    approx = [k for k in range(NP) if mine[k + 1].get('fill', 0) >= 2]
    f_same = {k: listed(c0[k]['ft']) == mine[k]['f0'] and len(mine[k]['f0']) == c0[k]['ft']['lu']['ec'] for k in range(NP)}
    tainted, live = set(), False
    for k in range(NP):
        if mine[k].get('k') == 0:
            live = False
        if k - 1 in approx and not f_same[k]:
            # only the rows that pivot k - 1 appended (in spike order) may be permuted
            want, got = listed(c0[k]['ft']), mine[k]['f0']
            n = len(set(want) - set(listed(c0[k - 1]['ft'])))
            live = n >= 2 and len(want) == len(got) and want[:-n] == got[:-n] and sorted(want[-n:]) == sorted(got[-n:])
        if live:
            tainted.add(k)
    # F: LU+0xf0 list
    f_ok, f_out = [], []
    for k in range(NP):
        if k in tainted:
            f_out.append((k, f_same[k]))
        elif f_same[k]:
            f_ok.append(k)
        else:
            bad.append(f'F pivot {k}: Rust list {mine[k]["f0"]} != original {listed(c0[k]["ft"])}')
    rep.append(f'F: LU+0xf0 list = capture on {len(f_ok)} pivots; outside the rule {[k for k, _ in f_out]} (fill >= 2 '
               f'spikes at {approx}), equal there {[k for k, s in f_out if s]}')
    # K: U kernel
    k_ok, cls = 0, {'sparse L, dense U': [], '21f83a, no sparse U': [], '2126cd': [], 'sparse U': []}
    for k in range(NP):
        ft, want, got = c0[k]['ft'], kernel(c0[k]['ft']), mine[k + 1].get('uk')
        if want != got:
            bad.append(f'K pivot {k}: Rust kernel {got} != original {want} (calls {ft["calls"]})')
            continue
        k_ok += 1
        calls = [c.split('@')[0] for c in ft['calls']]
        if want in ('21686f', '21841f'):
            cls['sparse U'].append(k)
        elif want == '2126cd':
            cls['2126cd'].append(k)
        elif '21f83a' in calls:
            cls['21f83a, no sparse U'].append(k)
        elif '21b9de' in calls:
            cls['sparse L, dense U'].append(k)
    rep.append(f'K: U kernel = capture on {k_ok}/{NP} pivots; ' + ', '.join(f'{n} {v}' for n, v in cls.items()))
    # A: final alpha_q list order
    a_ok, a_out = [], []
    for k in range(NP):
        e, s1 = c0[k], mine[k + 1]
        same = s1['aqo'] == e['aq']['idx']
        ft = e['ft']
        pos = {r: i for i, r in enumerate(ft['ord'])}
        neg = [r for r in e['aq']['idx'] if pos[r] < ft['t'] and ft['f8'][r] == -1]
        # sparse U: rows with LU+0xf8 = -1 keep the L/R order (shortcut when >= 2 of them with fill), listed rows the
        # list order
        if kernel(ft) in ('21686f', '21841f') and ((k in approx and len(neg) >= 2) or k in tainted):
            a_out.append((k, same))
        elif same:
            a_ok.append(k)
        else:
            bad.append(f'A pivot {k}: Rust alpha_q list {s1["aqo"]} != original {e["aq"]["idx"]}')
    disc = [k for k in a_ok if c0[k]['aq']['idx'] != sorted(c0[k]['aq']['idx'])]
    rep.append(f'A: final alpha_q list = capture on {len(a_ok)} pivots (not position order: {disc}); outside the rule '
               f'{[k for k, _ in a_out]}, equal there {[k for k, s in a_out if s]}')
    if not disc:
        bad.append('A: no pivot whose list differs from position order (nothing discriminated)')
    return bad, rep


def self_test(caps, oracle, mine):
    def each(fn):
        cc = copy.deepcopy(caps)
        for c in cc:
            fn(c)
        return cc

    def f8_swap(c):
        f8 = c[22]['ft']['f8']
        a, b = [r for r in range(len(f8)) if f8[r] >= 0][:2]
        f8[a], f8[b] = f8[b], f8[a]

    def drop_841(c):
        c[21]['ft']['calls'] = [x for x in c[21]['ft']['calls'] if not x.startswith('21841f')]

    def aq_swap(c):
        idx = c[21]['aq']['idx']
        idx[0], idx[1] = idx[1], idx[0]

    def dense_last(c):
        idx = c[10]['aq']['idx']
        idx.append(idx.pop(0))

    tampers = {
        'stored oracle file sha': (caps, {ORACLE: '0' * 64}),
        'pivot 22 list slots swapped (F)': (each(f8_swap), None),
        'pivot 21 21841f call removed (K)': (each(drop_841), None),
        'pivot 21 alpha_q entries swapped (A)': (each(aq_swap), None),
        'pivot 10 dense list rotated (A)': (each(dense_last), None),
        'no capture pivots': ([{} for _ in caps], None),
    }
    ok = True
    for name, (cc, shas) in tampers.items():
        b, _ = check(cc, oracle, mine, shas)
        print(f'T {name}: {"rejected: " + b[0] if b else "NOT REJECTED"}')
        ok &= bool(b)
    for name, f in (('Rust aqo of pivot 21 in position order', lambda m: m[22].__setitem__('aqo', sorted(m[22]['aqo']))),
                    ('Rust list f0 of snapshot 22 reversed', lambda m: m[22].__setitem__('f0', m[22]['f0'][::-1]))):
        mm = copy.deepcopy(mine)
        f(mm)
        b, _ = check(caps, oracle, mm)
        print(f'T {name}: {"rejected: " + b[0] if b else "NOT REJECTED"}')
        ok &= bool(b)
    return ok


def main():
    args = sys.argv[1:]
    snap = args[args.index('--snap') + 1] if '--snap' in args else None
    caps, oracle = load_caps(), load_oracle()
    mine = rust_snapshot(snap)
    bad, rep = check(caps, oracle, mine)
    for line in rep:
        print(line)
    for line in bad:
        print('FAIL', line)
    ok = not bad
    if '--self-test' in args:
        ok &= self_test(caps, oracle, mine)
    print('PASS' + (' (+ self-test)' if '--self-test' in args else '') if ok else 'FAIL')
    return 0 if ok else 1


if __name__ == '__main__':
    sys.exit(main())
