#!/usr/bin/env python3
"""M1-02n: the FT update's new prefix bound (05b257 C:463-512) and the refactor decision (06268b LAB_001f6eb5) vs the
original (degen2, kb2 dual runs). Contract: docs/m1/m1_02n_update.md section 3. Exit 0 only when every check passes.
  --rust        also this worktree's build (RSOLVER_SNAP): FTRAN-time t, LU+0xf0, LU+0xe8, empty U columns
  --self-test   also the tampers and model variants (T), each must be rejected
"""
import collections, copy, gzip, hashlib, json, os, struct, sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
D = os.path.join(ROOT, 'bench/oracle/m1_02n')
BIN = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
CAP = {'degen2': ('10646348f90e674f7522f1f3c23716c36299e3398db652c2246af509de46c205', '47ad212b08b5c517cbe169bd6711cdc1051391bf1b30531bcf9893abae498346'),
       'kb2': ('613052312e6327da6d15c05284bcd17924e852ba0108598c773214e1504e3ca6', 'fd4e89375a650a3a5eb25c1d57c293e1cf9b1791e0d6fa61a0d586b76ba5d516')}
# M1-02m captures of the same runs (tools/m1_usolve.js): N projection
US = {n: os.path.join(ROOT, 'bench/oracle/m1_02m', n + '_us_1.json.gz') for n in CAP}
US_SHA = {'degen2': '337407830c58b08784c0a1ba11bc87d2971524410794bd6e50c49ff8369b1890',
          'kb2': 'f0d201e8510aa83d90c0826ceec074bbaac5b1949e6ccea7431b087e192a62ab'}
F = lambda h: struct.unpack('<d', int(h, 16).to_bytes(8, 'little'))[0]
sha = lambda p: hashlib.sha256(open(p, 'rb').read()).hexdigest()
load = lambda p: json.loads(gzip.decompress(open(p, 'rb').read()))


def decide(e, var=None):
    """06268b C:1273-1303 on the probe state: (reason, fill predicate, work predicate). var: model mutant."""
    L, k = e['lu'], e['lu']['upd']
    if k >= L['per']:
        return 2, False, False
    m, m15, m4 = F(e['s78']), F(e['s80']), F(e['s88'])
    if k < e['c46c']:
        f = 0.0
    else:
        d = float(L['c1e8'] + L['c1e0']) - m15
        d = d * d
        g = float(e['c46c']) * m4
        if d <= g:
            g = d
        f = (float(L['c190']) * 1.5 + float(L['c184']) + float(L['c184'])) * m - g
    fill = 0.0 < f
    w158 = F(e['w158'])
    work = (k % 10 == 9 or var == 'nomod') and 0.0 < w158 and \
        w158 * (8.0 if var == 'w8' else 14.0) + float(e['c3d0'] * 100) < (F(e['work']) if e['work'] else 0.0) - F(e['w150'])
    return (10 if fill or work else 0), fill, work


def bound(e, var=None):
    """05b257 C:268-302 and C:463-512 on the entry state: (new t, branch, searched LU+0xe8 or None)."""
    I, r = e['in'], e['r']
    pos, t, emp = I['pos'], I['t'], set(I['emp'])
    q = pos[r]
    p = max(q, t - 1)
    if var == 'nval':
        oth = [i for i, v in zip(e['spike'], e['spv']) if i != r and abs(F(v)) > 1e-13]
    else:
        oth = [i for i in e['spike'] if i != r]
    l = max([p] + [pos[i] for i in oth])
    nz = (len(e['spike']) - (r in e['spike'])) != 0 if var != 'nval' else len(oth) != 0
    tm1, e8 = t - 1, None
    if l == p:
        br, b = 'l=p', tm1 - nz if (q < p or p == tm1) else tm1
    elif q < p:
        e8 = I['e8']
        found = False
        while p < e8:
            if e8 in emp:
                found = True
                break
            e8 -= 1
        if found:
            br, b = 'q<p empty column', tm1
        else:
            br, b = 'q<p no empty column', t - 2 if p < l else tm1
    elif p < l and p == tm1:
        br, b = 'q>=p, p=t-1<l', t - 2 + (not nz)
    else:
        br, b = 'q>=p other', tm1
    return b + 1, br, e8


def check(caps, uss, var=None):
    bad, rep = [], []
    for name, cap in caps.items():
        if cap['binary_sha256'] != BIN or not cap['ok'] or not cap['events']:
            bad.append(f'M {name}: binary {cap["binary_sha256"][:12]} ok {cap["ok"]} events {len(cap["events"])}')
            continue
        ev = cap['events']
        n = collections.Counter()
        # R: each decision against the reason at the next 05b257 entry
        for j, e in enumerate(ev):
            if e['type'] != 'D':
                continue
            nxt = next((x for x in ev[j + 1:] if x['type'] == 'U'), None)
            want, fill, work = decide(e, var)
            n['R decisions'] += 1
            n['R reason %d' % want] += 1
            n['R fill'] += fill
            n['R work'] += work
            n['R k%10=9, work false'] += e['lu']['upd'] % 10 == 9 and e['lu']['upd'] < e['lu']['per'] and not work
            if nxt is None:
                n['R last decision (no 05b257 after)'] += 1
                continue
            if e['reason'] != 0 or nxt['reason'] != want:
                bad.append(f'R {name} pivot {e["k"]}: update count {e["lu"]["upd"]}, probe reason {e["reason"]}, '
                           f'model {want} (fill {fill} work {work}), next 05b257 reason {nxt["reason"]}')
        # B: every FT update
        for e in ev:
            if e['type'] != 'U' or e['reason'] != 0 or e['x290'] != 0 or e['ret'] == 1:
                continue
            t1, br, e8 = bound(e, var)
            n['B ' + br] += 1
            if t1 != e['out']['t']:
                bad.append(f'B {name} pivot {e["k"] - 1} update {e["in"]["upd"]} ({br}): model t {t1} original {e["out"]["t"]}')
            elif e8 is not None and e['out']['e8'] < e8:
                bad.append(f'B {name} pivot {e["k"] - 1}: searched LU+0xe8 {e8} above the returned {e["out"]["e8"]}')
            n['B e8 searched = returned'] += e8 is not None and e['out']['e8'] == e8
            n['B spike entries |v| <= ZERO'] += sum(abs(F(v)) <= 1e-13 for i, v in zip(e['spike'], e['spv']) if i != e['r'])
        # N: the 02m kernel-time (t, ec, f0) = the last 05b257 return before that pivot
        leave = {}
        for e in ev:
            if e['type'] == 'U':
                leave[e['k']] = e['out']
        for e in uss[name]['events']:
            if e['type'] != 'P' or not e['ft'] or not e['ft'][0]['calls']:
                continue
            c = e['ft'][0]['calls'][0]
            o = leave.get(e['k'])
            if o is None:
                n['N pivot without a 05b257 before'] += 1
                continue
            if (o['t'], o['ec'], o['f0'], o['upd']) != (c['t'], c['ec'], c['f0'], e['ft'][0]['upd']):
                bad.append(f'N {name} pivot {e["k"]}: 05b257 return t {o["t"]} ec {o["ec"]} upd {o["upd"]}, 02m kernel t {c["t"]} '
                           f'ec {c["ec"]} upd {e["ft"][0]["upd"]}')
            n['N'] += 1
        rep.append(f'{name}: {dict(sorted(n.items()))}')
    return bad, rep


def answers(caps):
    """Q: the degen2 events the contract names (printed, not judged)."""
    ev = caps['degen2']['events']
    out = []
    for e in ev:
        if e['type'] == 'U' and e['k'] - 1 in (313, 326) and e['reason'] == 0:
            I = e['in']
            q = I['pos'][e['r']]
            t1, br, e8 = bound(e)
            out.append(f'Q degen2 pivot {e["k"] - 1} update {I["upd"]}: r {e["r"]} q {q} t {I["t"]} LU+0xe8 {I["e8"]} '
                       f'n\' {len(e["spike"]) - (e["r"] in e["spike"])}, U row r {e["urow_r"]} entries, {br}, searched to {e8}, '
                       f'new t {e["out"]["t"]} (LU+0xe8 after {e["out"]["e8"]})')
        if e['type'] == 'D' and e['k'] == 356:
            w, fill, work = decide(e)
            out.append(f'Q degen2 decision before pivot 356: update count {e["lu"]["upd"]}, reason {w} (fill {fill}, work {work}): '
                       f'work since factor start {F(e["work"]) - F(e["w150"])!r} > 14*{F(e["w158"])!r} + 100*{e["c3d0"]}')
    return out


def rust_run(name):
    """This worktree's build on the model with RSOLVER_SNAP: (first snapshot per iteration, pivot path)."""
    import subprocess, tempfile
    fd, snap = tempfile.mkstemp(suffix='.jsonl')
    os.close(fd)
    fd, pth = tempfile.mkstemp(suffix='.json')
    os.close(fd)
    model = os.path.join(ROOT, '../re/bench/data/netlib', name + '.mps.gz')
    code = (f"import json, rsolver; m = rsolver.read({model!r}); m.Params['Method'] = 1; m.Params['Presolve'] = 0; "
            f"m.Params['OutputFlag'] = 0; m.optimize(); json.dump([[q, o] for (_, q, o) in m._path], open({pth!r}, 'w'))")
    subprocess.run([sys.executable, '-c', code], env=dict(os.environ, RSOLVER_SNAP=snap), check=True)
    mine = {}
    for line in open(snap):
        s = json.loads(line)
        mine.setdefault(s['iter'], s)['klast'] = s['k']  # a refactor writes the iteration again: the FTRAN state
    path = json.load(open(pth))
    os.unlink(snap)
    os.unlink(pth)
    return mine, path


def check_rust(caps, runs):
    """Rust FTRAN-time (t, LU+0xf0, LU+0xe8, empty columns) vs the original 05b257 return before each pivot, on the
    pivots before the first path difference with the same update count."""
    bad, rep = [], []
    for name, (mine, path) in runs.items():
        g = json.load(gzip.open(os.path.join(ROOT, 'bench/oracle/path1', name + '.json.gz')))
        gp = [[e[0], l[0]] for _, e, l, *_ in g['path'][1:] if e or l]
        P = next((k for k, (a, b) in enumerate(zip(gp, path)) if a != b), min(len(gp), len(path)))
        leave = {}
        for e in caps[name]['events']:
            if e['type'] == 'U':
                leave[e['k']] = e['out']
        R = next((k for k in range(P) if k in leave and k in mine and mine[k]['klast'] != leave[k]['upd']), P)
        n, first = collections.Counter(), None
        for k in range(R):
            o, s1 = leave.get(k), mine.get(k + 1)
            if o is None or s1 is None or 't' not in s1:
                n['no state (no 05b257 before or no DSE snapshot)'] += 1
                continue
            if 'e8' not in s1:
                bad.append(f'{name} pivot {k}: Rust snapshot lacks e8/emp')
                break
            for f, a, b in (('t', s1['t'], o['t']), ('LU+0xec', len(s1['f0k']), o['ec']), ('LU+0xf0', s1['f0k'], o['f0']),
                            ('LU+0xe8', s1['e8'], o['e8']), ('empty U columns', s1['emp'], o['emp'])):
                if a != b:
                    bad.append(f'{name} pivot {k}: {f} Rust {a if not isinstance(a, list) else len(a)} original '
                               f'{b if not isinstance(b, list) else len(b)}' + (
                                   f' (first at {next((i for i in range(min(len(a), len(b))) if a[i] != b[i]), min(len(a), len(b)))})'
                                   if isinstance(a, list) else ''))
                    break
            else:
                n['state equal'] += 1
        why = f'pivot {R}: Rust {mine[R]["klast"]} updates, original {leave[R]["upd"]}' if R < P else 'path difference'
        rep.append(f'{name} Rust: first path difference {P}, same update count on 0-{R - 1} (stop: {why}); {dict(n)}')
    return bad, rep


def main():
    st = '--self-test' in sys.argv
    caps, uss, bad = {}, {}, []
    for name, (h, mh) in CAP.items():
        hs = [sha(os.path.join(D, f'{name}_up_{r}.json.gz')) for r in (1, 2, 3)]
        if len(set(hs)) != 1 or (h is not None and hs[0] != h):
            bad.append(f'M {name}: run hashes {[x[:12] for x in hs]}, pinned {h and h[:12]}')
        caps[name] = load(os.path.join(D, f'{name}_up_1.json.gz'))
        if caps[name]['model_sha256'] != mh:
            bad.append(f'M {name}: model sha {caps[name]["model_sha256"][:12]}')
        if sha(US[name]) != US_SHA[name]:
            bad.append(f'M {os.path.basename(US[name])}: sha {sha(US[name])[:12]}')
        uss[name] = load(US[name])
    b, rep = check(caps, uss)
    bad += b
    print('\n'.join(rep + answers(caps)))
    runs = {}
    if '--rust' in sys.argv:
        runs = {name: rust_run(name) for name in CAP}
        b, rep = check_rust(caps, runs)
        bad += b
        print('\n'.join(rep))
    if st:
        def tamper(desc, f, var=None):
            c2 = copy.deepcopy(caps)
            f(c2)
            b2, _ = check(c2, uss, var)
            print(f'T {desc}: {"rejected" if b2 else "NOT rejected"}' + (f' ({b2[0][:110]})' if b2 else ''))
            if not b2:
                bad.append(f'T {desc} not rejected')

        def ev(c2, name, f):
            return next(e for e in c2[name]['events'] if f(e))

        def tplus(c2):
            ev(c2, 'degen2', lambda e: e['type'] == 'U' and e['k'] == 314 and e['reason'] == 0)['out']['t'] += 1

        def reason(c2):
            ev(c2, 'degen2', lambda e: e['type'] == 'U' and e['k'] == 356 and e['reason'] == 10)['reason'] = 0

        def workbit(c2):
            e = ev(c2, 'degen2', lambda e: e['type'] == 'D' and e['k'] == 356)
            e['work'] = format(int(e['work'], 16) ^ (1 << 62), '016x')

        tamper('binary hash 0', lambda c2: c2['kb2'].__setitem__('binary_sha256', '0' * 64))
        tamper('no events', lambda c2: c2['degen2'].__setitem__('events', []))
        tamper('returned t + 1 (degen2 pivot 313)', tplus)
        tamper('refactor reason 10 -> 0 (degen2 before pivot 356)', reason)
        tamper('work one bit (degen2 decision before pivot 356)', workbit)
        def nswap(c2):
            # the 05b257 return before a 02m kernel pivot: only N reads its LU+0xf0 list
            ks = {e['k'] for e in uss['degen2']['events'] if e['type'] == 'P' and e['ft'] and e['ft'][0]['calls']}
            e = [e for e in c2['degen2']['events'] if e['type'] == 'U' and e['k'] in ks and len(e['out']['f0']) >= 2][-1]
            e['out']['f0'][0], e['out']['f0'][1] = e['out']['f0'][1], e['out']['f0'][0]

        tamper('05b257 return LU+0xf0: two entries swapped (degen2, N)', nswap)
        tamper('model: work constant 14 -> 8', lambda c2: None, 'w8')
        tamper('model: no k % 10 == 9 gate', lambda c2: None, 'nomod')
        # list count = value count unless a listed spike entry is <= ZERO: the variant is undiscriminated (not a failure)
        # only while the captures hold no such entry
        small = sum(int(x.split(': ')[1]) for r in rep for x in r.split('{')[1].rstrip('}').split(', ')
                    if x.startswith("'B spike entries |v| <= ZERO'"))
        if small:
            tamper('model: n\' and l from spike values > ZERO', lambda c2: None, 'nval')
        else:
            b2, _ = check(caps, uss, 'nval')
            print(f'T model: n\' and l from spike values > ZERO: {"rejected" if b2 else "survives"} (listed spike entries <= ZERO: 0, undiscriminated)')
        if runs:
            def rtamper(desc, f):
                r2 = copy.deepcopy(runs)
                f(r2)
                b2, _ = check_rust(caps, r2)
                print(f'T {desc}: {"rejected" if b2 else "NOT rejected"}' + (f' ({b2[0][:110]})' if b2 else ''))
                if not b2:
                    bad.append(f'T {desc} not rejected')

            def f0swap(r2):
                s = r2['degen2'][0][315]
                s['f0k'][0], s['f0k'][1] = s['f0k'][1], s['f0k'][0]

            def e8plus(r2):
                r2['degen2'][0][328]['e8'] += 1

            rtamper('Rust LU+0xf0: two entries swapped (degen2 pivot 314)', f0swap)
            rtamper('Rust LU+0xe8 + 1 (degen2 pivot 327)', e8plus)
    for x in bad:
        print('FAIL', x)
    print('PASS' if not bad else f'FAIL ({len(bad)})')
    sys.exit(1 if bad else 0)


if __name__ == '__main__':
    main()
