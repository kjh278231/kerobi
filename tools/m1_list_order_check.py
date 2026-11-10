#!/usr/bin/env python3
"""M1-02g: the mode-1 pricing list of the original dual simplex, replayed from stored snapshots only.
Contract: docs/m1/m1_02g_list_order.md section 3. Exit 0 only when every check passes.

For every mode-1 pivot k of kb2 (snapshots k and k+1 of bench/oracle/iter0-30), apply the CHUZR list cleanup of
2896a3 to snapshot k and the 1bae9c C:466-700 update loop over the final alpha_q list (spx+0x210, snapshot k+1 `aq`)
and compare the result with the list and marks of snapshot k+1.
  --self-test   also V (position-order variant) and T (tampers)
"""
import copy, gzip, hashlib, json, os, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ORACLE = os.path.join(ROOT, 'bench/oracle/iter0-30/kb2.jsonl.gz')
CAP = [os.path.join(ROOT, f'bench/oracle/m1_02a1_tau/kb2_{r}.json.gz') for r in (1, 2, 3)]
SHA = {
    ORACLE: '007d5a557fa84fcb73cdd7021f61e9fc4b5e97625615d9803cab6e7c96bc3bda',
    **{p: '64cc9dea89f809b601f7cb2e11636cc632f382318ceeaec0273a599687bdf393' for p in CAP},
}
FEAS = 1e-6  # local_b8 outside the artificial-bound phase (spx+0x2c8 = 0)
F = float.fromhex


class Fail(Exception):
    pass


def need(c, msg):
    if not c:
        raise Fail(msg)


def sha(p):
    return hashlib.sha256(open(p, 'rb').read()).hexdigest()


def load_oracle():
    g = {}
    for line in gzip.decompress(open(ORACLE, 'rb').read()).decode().splitlines():
        s = json.loads(line)
        g.setdefault(s['iter'], s)
    return g


def load_caps():
    return [{int(e['k']): e for e in json.loads(gzip.decompress(open(p, 'rb').read()))['events']} for p in CAP]


def chuzr(lst, mark, score, thr):
    """2896a3 mode 1 (Rust Pricing::full_pick): drop listed positions that turned feasible or fell under the
    threshold, take the best stored score; if none, lower the threshold tenfold and scan the unlisted positions."""
    best, r, kept = thr * 0.5, None, []
    for p in lst:
        if mark[p] < 2:
            if score[p] >= thr:
                kept.append(p)
                if best < score[p]:
                    best, r = score[p], p
            else:
                mark[p] = 0
        elif mark[p] < 5:
            mark[p] = 2
    if r is not None:
        return kept, thr, r, False
    lower, mx = thr / 10.0, 0.0
    for p in range(len(mark)):
        if mark[p] == 0:
            if mx < score[p]:
                mx, r = score[p], p
            if lower <= score[p]:
                kept.append(p)
                mark[p] = 1
    if kept:
        return kept, lower, r, True
    if r is not None:
        kept.append(r)
        mark[r] = 1
        return kept, mx, r, True
    return kept, 1e-13, r, True


def viol(s, p, head):
    j = head[p]
    x, lo, up = F(s['xB'][p]), F(s['l'][j]), F(s['u'][j])
    return FEAS < lo - x or FEAS < x - up  # 1bae9c C:576 (local_b0 == 0)


def replay(g, order=None):
    """Returns per-pivot results; raises Fail on the first difference. `order(aq)` reorders the loop (variant V)."""
    out = {}
    for k in sorted(g):
        if k + 1 not in g:
            continue
        a, b = g[k], g[k + 1]
        pa, pb = a['price'], b['price']
        if pa.get('mode') != 1 or pb.get('mode') != 1:
            continue
        if b['lu']['k'] != a['lu']['k'] + 1:
            # refactor after the pivot: positions are renumbered and mode 1 restarts (full_init), out of scope
            out[k] = dict(refactor=True)
            continue
        lst = pa['list'][:pa['allow']]
        mark = list(pa['flags'])
        score = [F(x) for x in pa['score']]
        kept, thr, r, scanned = chuzr(lst, mark, score, F(pa['thr']))
        need(r == b['r'], f'L pivot {k}: CHUZR picks position {r}, snapshot {k + 1} r {b["r"]}')
        flips = [j for j in range(len(a['vstat'])) if {a['vstat'][j], b['vstat'][j]} == {-1, -2}]
        thr1 = F(pb['thr'])
        raised = thr1 == thr * 1.1
        need(thr1 == thr or raised, f'L pivot {k}: threshold {thr1!r} is neither {thr!r} nor x1.1')
        head = {p: j for j, p in enumerate(b['vstat']) if p >= 0}
        sc1 = [F(x) for x in pb['score']]
        # r (C:466-543): violated -> mark 1 and a fresh score, else 2 -> 3 in mode 1
        mark[r] = 1 if viol(b, r, head) else 3
        idx = b['aq'][1]
        need(r not in idx, f'L pivot {k}: pivot position {r} still in the alpha_q list')
        app = []
        for p in (order(idx) if order else idx):
            if viol(b, p, head):
                if mark[p] == 3:
                    mark[p] = 1
                elif mark[p] & 1 == 0:
                    if sc1[p] < thr1:
                        mark[p] = 0
                    else:
                        kept.append(p)
                        app.append(p)
                        mark[p] = 1
            else:
                mark[p] |= 2
        want = pb['list'][:pb['allow']]
        if kept != want:
            t = next((i for i in range(min(len(kept), len(want))) if kept[i] != want[i]), min(len(kept), len(want)))
            raise Fail(f'L pivot {k}: list differs at {t}: snapshot {k + 1} {want[t:t + 5]} replay {kept[t:t + 5]} '
                       f'(appended {app}, alpha_q {idx})')
        for p in [r] + list(idx):
            need(mark[p] == pb['flags'][p], f'L pivot {k}: mark of position {p}: snapshot {pb["flags"][p]} replay {mark[p]}')
        out[k] = dict(appended=app, scanned=scanned, raised=raised, flips=len(flips), aq=list(idx))
    return out


def check(g, caps, order=None):
    # M: three runs of the 02a-1 capture give the same alpha_q list per pivot
    need(all(sorted(c) == sorted(caps[0]) for c in caps), 'M: 02a-1 runs differ in pivots')
    for k in caps[0]:
        lists = [(c[k]['aq']['idx'], c[k]['r']) for c in caps]
        need(all(x == lists[0] for x in lists), f'M: 02a-1 pivot {k} alpha_q list differs between runs')
    # X: snapshot k+1 aq = 02a-1 final alpha_q list of pivot k
    for k, e in caps[0].items():
        if k + 1 in g and g[k + 1]['lu']['k'] != 0:  # a refactor renumbers positions
            s = g[k + 1]
            need(s['aq'][1] == e['aq']['idx'] and s['r'] == int(e['r']),
                 f'X pivot {k}: snapshot aq {s["aq"][1][:6]} r {s["r"]} != 02a-1 {e["aq"]["idx"][:6]} r {e["r"]}')
    res = replay(g, order)
    # P: unsorted alpha_q lists only on sparse-U pivots
    for k, e in sorted(caps[0].items()):
        idx = e['aq']['idx']
        ker = [c.split('@')[0] for c in e['ft']['calls'] if c.split('@')[0] in ('21686f', '21841f', '2126cd')]
        if k in res and 'aq' in res[k] and idx != sorted(idx):
            need(any(x in ('21686f', '21841f') for x in ker), f'P pivot {k}: unsorted alpha_q list without a sparse U kernel {ker}')
    ftran_order(caps[0])
    return res


def ftran_order(cap, rule='orig', bad=None):
    """P2: 09d112 calls the sparse L kernel 21b9de iff LU+0x104 * nnz(a_q) <= m (LU+0x128 < 0 on every kb2 pivot).
    P3: on sparse-U pivots (21686f / 21841f) the 09d112 alpha_q output is [rows with LU+0xf8 = -1, as output] +
    [tail rows (U position >= t) by U position descending] + [prefix rows by LU+0xf8 ascending].
    Returns the pivots whose output that rule tells apart from index order; with `bad` (a list) a differing pivot is
    appended there instead of failing."""
    disc = []
    for k, e in sorted(cap.items()):
        ft = e['ft']
        lu, calls = ft['lu'], [c.split('@')[0] for c in ft['calls']]
        need(lu['x128'] < 0, f'P2 pivot {k}: LU+0x128 {lu["x128"]} >= 0 (level path, not covered)')
        need(('21b9de' in calls) == (lu['x104'] * ft['aq_in']['nnz'] <= ft['m']),
             f'P2 pivot {k}: 21b9de {"21b9de" in calls}, LU+0x104 {lu["x104"]} * nnz {ft["aq_in"]["nnz"]} vs m {ft["m"]}')
        # one 21b9de call per right-hand side (a_q, rho) that passes the same test
        nb = sum(lu['x104'] * v['nnz'] <= ft['m'] for v in (ft['aq_in'], ft['rho_in']) if v['nnz'] >= 0)
        need(calls.count('21b9de') == nb, f'P2 pivot {k}: {calls.count("21b9de")} 21b9de calls, test passes for {nb} RHS')
        if not any(c in ('21686f', '21841f') for c in calls):
            # P: without a sparse U kernel the final alpha_q list is in index order
            need(e['aq']['idx'] == sorted(e['aq']['idx']), f'P pivot {k}: no sparse U kernel but final list unsorted')
            continue
        pos = {r: i for i, r in enumerate(ft['ord'])}
        out, t, f8 = ft['aq_out']['idx'], ft['t'], ft['f8']
        neg = [r for r in out if pos[r] < t and f8[r] == -1]
        tail = [r for r in out if pos[r] >= t]
        pre = [r for r in out if pos[r] < t and f8[r] >= 0]
        if rule == 'neg-last':
            want = sorted(tail, key=lambda r: -pos[r]) + sorted(pre, key=lambda r: f8[r]) + neg
        elif rule == 'orig':
            want = neg + sorted(tail, key=lambda r: -pos[r]) + sorted(pre, key=lambda r: f8[r])
        elif rule == 'tail-asc':
            want = neg + sorted(tail, key=lambda r: pos[r]) + sorted(pre, key=lambda r: f8[r])
        elif rule == 'prefix-upos':
            want = neg + sorted(tail, key=lambda r: -pos[r]) + sorted(pre, key=lambda r: pos[r])
        else:  # 'index': the dense compaction order for the rows the rule orders
            want = neg + sorted(tail + pre)
        if out != want:
            if bad is None:
                raise Fail(f'P3 pivot {k}: 09d112 output {out} != rule {rule} {want}')
            bad.append(k)
        if want != neg + sorted(tail + pre):
            disc.append(k)
    return disc


def main():
    self_test = '--self-test' in sys.argv[1:]
    try:
        for p, h in SHA.items():
            need(sha(p) == h, f'M: {p} sha256 {sha(p)} != {h}')
        g, caps = load_oracle(), load_caps()
        allres = check(g, caps)
        res = {k: v for k, v in allres.items() if 'aq' in v}
        refac = sorted(k for k, v in allres.items() if 'aq' not in v)
        disc = [k for k, v in res.items() if v['appended'] != sorted(v['appended'])]
        for k, e in sorted(caps[0].items()):
            ker = [c.split('@')[0] for c in e['ft']['calls'] if c.split('@')[0] in ('21686f', '21841f', '2126cd')] or ['-']
            idx = e['aq']['idx']
            v = res.get(k)
            if k in refac:
                print(f'pivot {k:2d} r {int(e["r"]):2d} U {",".join(ker):7s} {"sorted  " if idx == sorted(idx) else "UNSORTED"} mode1 refactor after (not replayed)')
                continue
            print(f'pivot {k:2d} r {int(e["r"]):2d} U {",".join(ker):7s} {"sorted  " if idx == sorted(idx) else "UNSORTED"} '
                  + (f'mode1 appended {v["appended"]}{" scan" if v["scanned"] else ""}{" x1.1" if v["raised"] else ""}'
                     f'{" flips " + str(v["flips"]) if v["flips"] else ""}' if v else ''))
        print(f'L: {len(res)} mode-1 pivots ({min(res)}-{max(res)}) replayed, list and marks equal; '
              f'scan {sum(v["scanned"] for v in res.values())}, x1.1 {sum(v["raised"] for v in res.values())}, '
              f'flip pivots {sum(1 for v in res.values() if v["flips"])}; refactor pivots {refac} not replayed; '
              f'discriminating pivots {disc}')
        fdisc = ftran_order(caps[0])
        print(f'P2: 21b9de iff LU+0x104*nnz(a_q) <= m on {len(caps[0])} pivots; P3: sparse-U output order rule on '
              f'{sum(1 for e in caps[0].values() if any(c.split("@")[0] in ("21686f", "21841f") for c in e["ft"]["calls"]))} '
              f'pivots, discriminating {fdisc}')
        if self_test:
            need(fdisc, 'V: no pivot tells the output order rule apart')
            mixed = [k for k, e in caps[0].items() if any(c.split('@')[0] in ('21686f', '21841f') for c in e['ft']['calls'])
                     and any(e['ft']['f8'][r] == -1 and e['ft']['ord'].index(r) < e['ft']['t'] for r in e['ft']['aq_out']['idx'])
                     and any(e['ft']['f8'][r] != -1 for r in e['ft']['aq_out']['idx'])]
            for v in ('tail-asc', 'prefix-upos', 'index', 'neg-last'):
                bad = []
                ftran_order(caps[0], v, bad)
                need(bad, f'V {v}: output order variant passed')
                need(set(bad) <= set(fdisc) if v != 'neg-last' else set(bad) <= set(mixed),
                     f'V {v}: rejected at {bad}, outside the pivots the rule decides')
                if v == 'index':
                    need(bad == fdisc, f'V index: rejected at {bad} != discriminating {fdisc}')
                print(f'V {v}: rejected at pivots {bad}')
            need(disc, 'V: no discriminating pivot')
            try:
                replay(g, sorted)
                raise Fail('V: position-order variant passed')
            except Fail as ex:
                if str(ex).startswith('V:'):
                    raise
                need(f'pivot {disc[0]}:' in str(ex), f'V: variant rejected elsewhere: {ex}')
                print(f'V position order: rejected at {str(ex)[:150]}')
            # the variant agrees with the original everywhere else
            for k in res:
                if k not in disc:
                    gg = {i: g[i] for i in (k, k + 1)}
                    replay(gg, sorted)

            def t_aq(gg, cc):
                idx = gg[22]['aq'][1]
                i, j = idx.index(20), idx.index(5)
                idx[i], idx[j] = idx[j], idx[i]
                for c in cc:
                    c[21]['aq']['idx'] = list(idx)

            def t_list(gg, cc):
                lst = gg[22]['price']['list']
                lst[8], lst[9] = lst[9], lst[8]

            def t_mark(gg, cc):
                fl = gg[21]['price']['flags']
                fl[fl.index(3)] = 1

            def t_cap(gg, cc):
                idx = cc[1][21]['aq']['idx']
                idx[0], idx[1] = idx[1], idx[0]

            for name, fn in (('aq order', t_aq), ('list order', t_list), ('mark', t_mark), ('02a-1 run', t_cap)):
                gg, cc = copy.deepcopy(g), copy.deepcopy(caps)
                fn(gg, cc)
                try:
                    check(gg, cc)
                    raise Fail(f'T {name}: tampered input passed')
                except Fail as ex:
                    if str(ex).startswith('T '):
                        raise
                    print(f'T {name}: rejected: {str(ex)[:150]}')
        print('PASS' + (' (+ self-test)' if self_test else ''))
        return 0
    except Fail as ex:
        print('FAIL', ex)
        return 1


if __name__ == '__main__':
    sys.exit(main())
