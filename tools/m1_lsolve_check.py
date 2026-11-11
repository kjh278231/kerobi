#!/usr/bin/env python3
"""M1-02j: the sparse-L kernel 21b9de and the row-eta order of 09d112 vs the original (degen2, kb2 dual pivots).
Contract: docs/m1/m1_02j_lsolve.md section 3. Exit 0 only when every check passes.
  --rust        also this worktree's build (RSOLVER_SNAP): U kernel, alpha_q -1 rows and L+R order (contract section 7)
  --self-test   also the tampers and model variants (T), each must be rejected
"""
import collections, copy, gzip, hashlib, json, os, struct, sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
D = os.path.join(ROOT, 'bench/oracle/m1_02j')
BIN = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
CAP = {'degen2': ('db3454311c46f9cffbe41b84000c212c1c84af2d9b1aa199e989783568709db4',
                  '47ad212b08b5c517cbe169bd6711cdc1051391bf1b30531bcf9893abae498346'),
       'kb2': ('6160a0c3af29d61e53d54f7eca5bb828ed7f3a2d7de5ede5710685dd559d4621',
               'fd4e89375a650a3a5eb25c1d57c293e1cf9b1791e0d6fa61a0d586b76ba5d516')}
# 02a-1 format captures (tools/m1_tau_path.js) of the same runs: N projection
TAU = {'degen2': os.path.join(D, 'degen2_tau_1.json.gz'), 'kb2': os.path.join(ROOT, 'bench/oracle/m1_02a1_tau/kb2_1.json.gz')}
TOL = 1e-13  # Rust ZERO; 21b9de's tol arrives in xmm0, which the hooks cannot read
F = lambda h: struct.unpack('<d', int(h, 16).to_bytes(8, 'little'))[0]
sha = lambda p: hashlib.sha256(open(p, 'rb').read()).hexdigest()
load = lambda p: json.loads(gzip.decompress(open(p, 'rb').read()))


def l21b9de(L, m, idx, val, var=None, tol=TOL):
    """21b9de (C:60-897) on the captured L; returns (path, [(row, value)]) in output order. var: model mutant."""
    beg, row, lv = L['beg'], L['row'], L['lv']
    c0 = beg[0]
    col = lambda j: range(beg[j] - c0, beg[j + 1] - c0)
    e4, x = L['xe4'], collections.defaultdict(float)
    dm = m - e4
    if dm < 6 or dm * (10.0 * dm) <= m or (beg[m] - beg[e4]) <= 10.0 * dm:
        x128 = L['x128']
        if x128 < 0 or 0.1 * m <= x128 or var == 'kahn':  # A1: Kahn FIFO (C:81-257)
            cnt, lst = collections.Counter(), list(idx)
            for i in idx:
                cnt[i] += 1
            p = 0
            while p < len(lst):
                for e in col(lst[p]):
                    if cnt[row[e]] == 0:
                        lst.append(row[e])
                    cnt[row[e]] += 1
                p += 1
            q = []
            for i, v in zip(idx, val):
                x[i] = v
                cnt[i] -= 1
                if cnt[i] == 0:
                    q.append(i)
            out, p, small = [], 0, 0
            while p < len(q):
                j = q[p]
                p += 1
                v, x[j] = x[j], 0.0
                big = tol <= v or v <= -tol
                if big:
                    out.append((j, v))
                else:
                    small += 1
                for e in col(j):
                    i = row[e]
                    cnt[i] -= 1
                    if big:
                        x[i] = -(lv[e] * v) + x[i]
                    if cnt[i] == 0:
                        q.append(i)
            return ('A1', small > 0), out
        lev = L['lev']  # A2: level buckets (C:259-605)
        bk, mark, lo = collections.defaultdict(list), set(), x128 + 1
        for i, v in zip(idx, val):
            bk[lev[i]].append(i)
            mark.add(i)
            lo, x[i] = min(lo, lev[i]), v
        pend, out, l = len(idx), [], lo
        while pend > 0:
            b, t = bk[l], 0
            if var == 'lifo':
                b.reverse()
            while t < len(b):
                j = b[t]
                t += 1
                pend -= 1
                v, x[j] = x[j], 0.0
                mark.discard(j)
                if tol <= abs(v):
                    out.append((j, v))
                    for e in col(j):
                        i = row[e]
                        if i not in mark:
                            mark.add(i)
                            pend += 1
                            bk[lev[i]].append(i)
                        x[i] = -(lv[e] * v) + x[i]
            l += 1
        return ('A2', False), out
    # B (C:607-893): Kahn below e4, then the tail swept in index order
    cnt, lst, lo = collections.Counter(), [], m
    for i, v in zip(idx, val):
        x[i] = v
        if i < e4:
            cnt[i] += 1
            lst.append(i)
        elif i < lo:
            lo = i
    nh, p = len(lst), 0
    while p < len(lst):
        for e in col(lst[p]):
            i = row[e]
            if i < e4:
                if cnt[i] == 0:
                    lst.append(i)
                cnt[i] += 1
        p += 1
    q = []
    for i in lst[:nh]:
        cnt[i] -= 1
        if cnt[i] == 0:
            q.append(i)
    out, p = [], 0
    while p < len(q):
        j = q[p]
        p += 1
        v, x[j] = x[j], 0.0
        if abs(v) > tol:
            out.append((j, v))
        for e in col(j):
            i = row[e]
            if abs(v) > tol:
                x[i] = -(lv[e] * v) + x[i]
            if i < e4:
                cnt[i] -= 1
                if cnt[i] == 0:
                    q.append(i)
            elif abs(v) > tol and i < lo:
                lo = i
    for j in range(lo, m):
        v, x[j] = x[j], 0.0
        if tol < abs(v):
            out.append((j, v))
            for e in col(j):
                x[row[e]] = -(lv[e] * v) + x[row[e]]
    return ('B', False), out


def check(caps, taus, var=None):
    """caps/taus: model -> loaded capture. Returns (failures, report lines)."""
    bad, rep = [], []
    for name, cap in caps.items():
        if cap['binary_sha256'] != BIN or not cap['ok'] or not cap['events']:
            bad.append(f'M {name}: binary {cap["binary_sha256"][:12]} ok {cap["ok"]} events {len(cap["events"])}')
            continue
        Ls, P = {}, []
        for e in cap['events']:
            if e['type'] == 'L':
                Ls[e['id']] = dict(e, lv=[F(v) for v in e['val']])
            else:
                P.append(e)
        tau = {e['k']: e for e in taus[name]['events']}
        paths, nL, nR, nM, rk = collections.Counter(), 0, collections.Counter(), 0, set()
        for e in P:
            k = e['k']
            if not e['ft']:
                continue
            ft = e['ft'][0]
            tf = tau.get(k, {}).get('ft')
            if tf is None or (ft['aq_out'].get('idx') if ft['aq_out']['nnz'] >= 0 else -1) != (tf['aq_out']['idx'] if tf['aq_out']['nnz'] >= 0 else -1):
                bad.append(f'N {name} pivot {k}: aq_out differs from the 02a-1 capture')
                continue
            for c in ft['calls']:
                if c['f'] == '21b9de':
                    path, out = l21b9de(Ls[ft['lid']], c['m'], c['in']['idx'], [F(v) for v in c['in']['val']], var)
                    paths[path] += 1
                    nL += 1
                    gi, gv = c['out']['idx'], [F(v) for v in c['out']['val']]
                    mi, mv = [i for i, _ in out], [v for _, v in out]
                    if gi != mi or gv != mv:
                        t = next((t for t in range(min(len(gi), len(mi))) if gi[t] != mi[t] or gv[t] != mv[t]), None)
                        bad.append(f'L {name} pivot {k} site {c["site"]} {path[0]}: first diff at {t}: '
                                   f'orig {gi[t:t + 4] if t is not None else len(gi)} model {mi[t:t + 4] if t is not None else len(mi)}')
                elif c['f'] == '21f83a':
                    b, a = c['before'][0], c['after'][1]
                    if a['nnz'] < 0 or b['nnz'] < 0:
                        bad.append(f'R {name} pivot {k}: dense sparse side')
                        continue
                    sa, sb = set(a['idx']), set(b['idx'])
                    keep = [i for i in b['idx'] if i in sa]
                    ok = a['idx'][:len(keep)] == keep and not (set(a['idx'][len(keep):]) & sb) and c['after'][3]['nnz'] < 0
                    nR['same' if a['idx'] == b['idx'] else 'appended/dropped'] += 1
                    if not ok:
                        bad.append(f'R {name} pivot {k} site {c["site"]}: L order {b["idx"][:8]} -> {a["idx"][:8]}')
            # -1 rows of the final alpha_q list (sparse U, sparse L on the alpha_q side) = the L output's -1 rows
            aqL = next((c for c in ft['calls'] if c['f'] == '21b9de' and c['site'] in ('1261af', '126284')), None)
            uk = next((c['f'] for c in ft['calls'] if c['f'] in ('21686f', '21841f', '2126cd')), 'inline')
            if aqL and ft['aq_out']['nnz'] >= 0 and uk in ('21686f', '21841f'):
                f8 = tf['f8']
                lo = [i for i in aqL['out']['idx'] if f8[i] == -1]
                fin = [i for i in ft['aq_out']['idx'] if f8[i] == -1]
                nM += 1
                if lo != fin:
                    bad.append(f'U {name} pivot {k}: -1 rows {fin[:6]} vs L output {lo[:6]}')
                if len(lo) >= 2:
                    rk.add(k)
        rep.append(f'{name}: 21b9de {nL} calls, paths {dict((p + ('+small' if s else ''), n) for (p, s), n in paths.items())}; '
                   f'21f83a {dict(nR)}; -1 rows = L output on {nM} sparse-U pivots ({len(rk)} with >= 2 such rows)')
    return bad, rep


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


def check_rust(caps, taus, runs):
    """Rust (RSOLVER_SNAP) vs the captures on the pivots before the first path difference: U kernel (K), the
    alpha_q list's LU+0xf8 = -1 rows (U) and the alpha_q side's L + row-eta order (S)."""
    bad, rep = [], []
    for name, (mine, path) in runs.items():
        g = json.load(gzip.open(os.path.join(ROOT, 'bench/oracle/path1', name + '.json.gz')))
        gp = [[e[0], l[0]] for _, e, l, *_ in g['path'][1:] if e or l]
        P = next((k for k, (a, b) in enumerate(zip(gp, path)) if a != b), min(len(gp), len(path)))
        tau = {e['k']: e for e in taus[name]['events']}
        ls = {e['k']: e for e in caps[name]['events'] if e['type'] == 'P'}
        # the same factor state: LU+0x18c at the 09d112 entry = Rust's update count before the pivot
        R = next((k for k in range(P) if ls.get(k, {}).get('ft') and k in mine and mine[k]['klast'] != ls[k]['ft'][0]['upd']), P)
        n = collections.Counter()
        for k in range(R):
            if k + 1 not in mine or 'aqo' not in mine[k + 1] or not ls.get(k, {}).get('ft') or tau.get(k, {}).get('ft') is None:
                n['no record'] += 1
                continue
            s1, ft, tf = mine[k + 1], ls[k]['ft'][0], tau[k]['ft']
            ck = [c for c in ft['calls']]
            uk = next((c['f'] for c in ck if c['f'] in ('21686f', '21841f', '2126cd')), 'inline')
            if s1['uk'] != uk:
                bad.append(f'K {name} pivot {k}: Rust {s1["uk"]} original {uk}')
                continue
            n['K'] += 1
            f8 = tf['f8']
            if uk in ('21686f', '21841f'):
                g1 = [i for i in tau[k]['aq']['idx'] if f8[i] == -1]
                m1 = [i for i in s1['aqo'] if f8[i] == -1]
                n['U'] += 1
                n['U >= 2'] += len(g1) >= 2
                if g1 != m1:
                    bad.append(f'U {name} pivot {k}: -1 rows Rust {m1[:6]} original {g1[:6]}')
            aqL = next((c for c in ck if c['f'] == '21b9de' and c['site'] in ('1261af', '126284')), None)
            r1 = next((c for c in ck if c['f'] == '21f83a' and c['site'] == '1279e5'), None)
            dense = aqL is None or any(c['f'] == '21f83a' and c['site'] == '127a7e' for c in ck)
            if dense:
                want = []
            elif ft['x180'] <= 0:
                want = aqL['out']['idx']
            elif r1 is not None:
                want = r1['after'][1]['idx']
            else:
                n['S inline R (no capture)'] += 1
                continue
            got = s1.get('lro')
            if got is None:
                bad.append(f'S {name} pivot {k}: Rust snapshot lacks lro')
                break
            n['S'] += 1
            if got != want:
                t = next((t for t in range(min(len(got), len(want))) if got[t] != want[t]), None)
                bad.append(f'S {name} pivot {k}: L+R order Rust {got[t:t + 4] if t is not None else len(got)} '
                           f'original {want[t:t + 4] if t is not None else len(want)}')
        stop = f' (pivot {R}: Rust {mine[R]["klast"]} updates, original {ls[R]["ft"][0]["upd"]})' if R < P else ''
        rep.append(f'{name} Rust: path = original on pivots 0-{P - 1}, update count on 0-{R - 1}{stop}; {dict(n)}')
    return bad, rep


def main():
    st = '--self-test' in sys.argv
    caps, taus, bad = {}, {}, []
    for name, (h, mh) in CAP.items():
        for r in (1, 2, 3):
            p = os.path.join(D, f'{name}_ls_{r}.json.gz')
            if sha(p) != h:
                bad.append(f'M {name}_ls_{r}: sha {sha(p)[:12]}')
        caps[name] = load(os.path.join(D, f'{name}_ls_1.json.gz'))
        if caps[name]['model_sha256'] != mh:
            bad.append(f'M {name}: model sha {caps[name]["model_sha256"][:12]}')
        taus[name] = load(TAU[name])
    b, rep = check(caps, taus)
    bad += b
    print('\n'.join(rep))
    runs = {}
    if '--rust' in sys.argv:
        runs = {name: rust_run(name) for name in CAP}
        b, rep = check_rust(caps, taus, runs)
        bad += b
        print('\n'.join(rep))
    if st:
        def tamper(desc, f, var=None):
            c2 = copy.deepcopy(caps)
            f(c2)
            b2, _ = check(c2, taus, var)
            print(f'T {desc}: {"rejected" if b2 else "NOT rejected"}' + (f' ({b2[0][:90]})' if b2 else ''))
            if not b2:
                bad.append(f'T {desc} not rejected')

        def calls(c2, name, f):
            return next(c for e in c2[name]['events'] if e['type'] == 'P' for ft in e['ft'] for c in ft['calls'] if f(c))

        def swap(c2):
            c = calls(c2, 'degen2', lambda c: c['f'] == '21b9de' and c['out']['nnz'] >= 3)
            c['out']['idx'][1], c['out']['idx'][2] = c['out']['idx'][2], c['out']['idx'][1]

        def bit(c2):
            c = calls(c2, 'kb2', lambda c: c['f'] == '21b9de' and c['out']['nnz'] >= 2)
            c['out']['val'][1] = format(int(c['out']['val'][1], 16) ^ 1, '016x')

        def rorder(c2):
            c = calls(c2, 'degen2', lambda c: c['f'] == '21f83a' and c['after'][1]['nnz'] > c['before'][0]['nnz'] >= 2)
            a = c['after'][1]['idx']
            a[0], a[-1] = a[-1], a[0]

        tamper('binary hash 0', lambda c2: c2['kb2'].__setitem__('binary_sha256', '0' * 64))
        tamper('no events', lambda c2: c2['degen2'].__setitem__('events', []))
        tamper('two L outputs swapped (degen2)', swap)
        tamper('one L output value bit (kb2)', bit)
        tamper('21f83a appended row moved to the front', rorder)
        tamper('model: degen2 A2 as Kahn FIFO', lambda c2: None, 'kahn')
        tamper('model: level buckets in reverse', lambda c2: None, 'lifo')
        if runs:
            def rtamper(desc, f):
                r2 = copy.deepcopy(runs)
                f(r2)
                b2, _ = check_rust(caps, taus, r2)
                print(f'T {desc}: {"rejected" if b2 else "NOT rejected"}' + (f' ({b2[0][:90]})' if b2 else ''))
                if not b2:
                    bad.append(f'T {desc} not rejected')

            def lro_swap(r2):
                s = next(s for k, s in sorted(r2['degen2'][0].items()) if len(s.get('lro') or []) >= 3)
                s['lro'][1], s['lro'][2] = s['lro'][2], s['lro'][1]

            rtamper('Rust L+R order: two entries swapped (degen2)', lro_swap)
            rtamper('Rust snapshots without lro', lambda r2: [s.pop('lro', None) for s in r2['kb2'][0].values()])
            rtamper('Rust U kernel of kb2 pivot 21', lambda r2: r2['kb2'][0][22].__setitem__('uk', 'inline'))
    for x in bad:
        print('FAIL', x)
    print('PASS' if not bad else f'FAIL ({len(bad)})')
    sys.exit(1 if bad else 0)


if __name__ == '__main__':
    main()
