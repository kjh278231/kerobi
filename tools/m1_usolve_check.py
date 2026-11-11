#!/usr/bin/env python3
"""M1-02m: the sparse-U kernels 21686f / 21841f of 09d112 (alpha_q side) vs the original (degen2, kb2 dual pivots).
Contract: docs/m1/m1_02m_usolve.md section 3. Exit 0 only when every check passes.
  --rust        also this worktree's build (RSOLVER_SNAP): U column state (C), kernel output order (O), input order (I)
  --self-test   also the tampers and model variants (T), each must be rejected
"""
import collections, copy, gzip, hashlib, json, os, struct, sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
D = os.path.join(ROOT, 'bench/oracle/m1_02m')
BIN = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
CAP = {'degen2': ('337407830c58b08784c0a1ba11bc87d2971524410794bd6e50c49ff8369b1890',
                  '47ad212b08b5c517cbe169bd6711cdc1051391bf1b30531bcf9893abae498346'),
       'kb2': ('f0d201e8510aa83d90c0826ceec074bbaac5b1949e6ccea7431b087e192a62ab',
               'fd4e89375a650a3a5eb25c1d57c293e1cf9b1791e0d6fa61a0d586b76ba5d516')}
# M1-02j captures of the same runs (tools/m1_lsolve.js): N projection
LS = {n: os.path.join(ROOT, 'bench/oracle/m1_02j', n + '_ls_1.json.gz') for n in CAP}
LS_SHA = {'degen2': 'db3454311c46f9cffbe41b84000c212c1c84af2d9b1aa199e989783568709db4',
          'kb2': '6160a0c3af29d61e53d54f7eca5bb828ed7f3a2d7de5ede5710685dd559d4621'}
TOL = 1e-13  # Rust ZERO; the kernels' tol arrives in xmm0
F = lambda h: struct.unpack('<d', int(h, 16).to_bytes(8, 'little'))[0]
sha = lambda p: hashlib.sha256(open(p, 'rb').read()).hexdigest()
load = lambda p: json.loads(gzip.decompress(open(p, 'rb').read()))


def ukernel(c, var=None, tol=TOL):
    """21686f / 21841f alpha_q side (docs/m1/m1_02m_usolve.md section 2) on the captured state. Returns (path,
    [(index, value)] in output order, count of outputs with |x| <= tol < |x/d|). var: model mutant."""
    m, t, ec, f8, ordr = c['m'], c['t'], c['ec'], c['f8'], c['ord']
    d = lambda i: F(c['diag'][i])
    D = F(c['D'][0]) + F(c['D'][1])
    D2 = D * D
    cols = {e[0]: e for e in c['cols']}
    part = lambda i, w: zip(cols[i][1 + w], map(F, cols[i][3 + w]))  # w = 0 tail part, 1 prefix part
    idx, val = c['in']['idx'], [F(v) for v in c['in']['val']]
    x, out, sm = collections.defaultdict(float), [], [0]

    def emit(i):
        y = x[i] / d(i)
        x[i] = 0.0
        if tol < abs(y):
            sm[0] += abs(y * d(i)) <= tol
            out.append((i, y))
            return y
        return None

    def direct(i, v):
        x[i] = v
        emit(i)

    if float(m) <= len(idx) * D2:
        path = 'S1'
        for i, v in zip(idx, val):
            if f8[i] == -1:
                direct(i, v)
            else:
                x[i] = v
    else:
        npre, ntail, sec2, mark = 0, 0, [], set()
        for i, v in zip(idx, val):
            if f8[i] == -1:
                direct(i, v)
            elif f8[i] < 0:
                x[i], ntail = v, ntail + 1
            else:
                x[i], npre = v, npre + 1
                sec2.append(i)
                mark.add(i)
        if D2 * (npre + ntail) <= float(ec + m - t):
            b8, tout = npre, []
            for p in range(m - 1, t - 1, -1):
                i = ordr[p]
                y = emit(i)
                if y is not None:
                    tout.append((i, y))
                    b8 += len(cols[i][2])
                    for r, u in part(i, 0):
                        x[r] = x[r] - u * y
            h2 = b8 * (0.5 if var == 'coef' else 2.5) <= ec and var != 'list'
            first = []
            for i, y in tout:
                for r, u in part(i, 1):
                    if h2 and r not in mark:
                        first.append(r)
                        mark.add(r)
                    x[r] = -(u * y) + x[r]
            order = (first + sec2 if var == 'prelast' else sec2 + first) if h2 else c['f0']
            for i in order:
                emit(i)
            return ('H2' if h2 else 'H1'), out, sm[0]
        path = 'S2'
    for p in range(m - 1, t - 1, -1):
        i = ordr[p]
        y = emit(i)
        if y is not None:
            for w in (0, 1):
                for r, u in part(i, w):
                    x[r] = x[r] - u * y
    for i in c['f0']:
        emit(i)
    return path, out, sm[0]


def calls(cap):
    """(pivot, 09d112 record, kernel call or None) per dual pivot with an FTRAN."""
    for e in cap['events']:
        if e['type'] == 'P' and e['ft']:
            ft = e['ft'][0]
            yield e['k'], ft, (ft['calls'][0] if ft['calls'] else None)


def check(caps, lss, var=None):
    bad, rep = [], []
    for name, cap in caps.items():
        if cap['binary_sha256'] != BIN or not cap['ok'] or not cap['events']:
            bad.append(f'M {name}: binary {cap["binary_sha256"][:12]} ok {cap["ok"]} events {len(cap["events"])}')
            continue
        ls = {e['k']: e['ft'][0] for e in lss[name]['events'] if e['type'] == 'P' and e['ft']}
        paths, fs, nP, nsm, pre_nonslot = collections.Counter(), collections.Counter(), 0, 0, 0
        for k, ft, c in calls(cap):
            if k not in ls or ls[k]['aq_out'].get('idx') != ft['aq_out'].get('idx') or ls[k]['aq_out']['nnz'] != ft['aq_out']['nnz']:
                bad.append(f'N {name} pivot {k}: aq_out differs from the M1-02j capture')
                continue
            if c is None:
                continue
            fs[c['f']] += 1
            if c['uoff'] != 0x20 or c['out']['idx'] != ft['aq_out'].get('idx'):
                bad.append(f'A {name} pivot {k}: U at LU+{c["uoff"]:#x} or kernel output != 09d112 aq_out')
            # P: the tail / prefix split of the U columns
            pos = {i: p for p, i in enumerate(c['ord'])}
            for col in c['cols']:
                nP += 1
                if any(pos[r] < c['t'] for r in col[1]) or any(pos[r] >= c['t'] or c['f8'][r] < 0 for r in col[2]):
                    bad.append(f'P {name} pivot {k}: column {col[0]} split')
                    break
            if c['pre_nonempty']:
                bad.append(f'P {name} pivot {k}: {c["pre_nonempty"]} non-empty columns below t')
            path, out, sm = ukernel(c, var)
            paths[(c['f'], path)] += 1
            nsm += sm
            gi, gv = c['out']['idx'], [F(v) for v in c['out']['val']]
            mi, mv = [i for i, _ in out], [v for _, v in out]
            pre = [i for i in gi if c['f8'][i] >= 0]
            pre_nonslot += [c['f8'][i] for i in pre] != sorted(c['f8'][i] for i in pre)
            if gi != mi or gv != mv:
                t = next((t for t in range(min(len(gi), len(mi))) if gi[t] != mi[t] or gv[t] != mv[t]), None)
                bad.append(f'A {name} pivot {k} {c["f"]} {path}: first diff at {t}: '
                           f'orig {gi[t:t + 4] if t is not None else len(gi)} model {mi[t:t + 4] if t is not None else len(mi)}')
        rep.append(f'{name}: kernels {dict(fs)}, paths {dict((f"{a} {b}", n) for (a, b), n in sorted(paths.items()))}, '
                   f'prefix not in slot order {pre_nonslot}, |x| <= tol < |x/d| {nsm}, columns split-checked {nP}')
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


def check_rust(caps, runs):
    """Rust snapshots vs the captures on the pivots before the first path difference with the same update count:
    U columns' prefix parts and the LU+0xf0 list (C), kernel output order (O, only where C holds), input order (I)."""
    bad, rep = [], []
    for name, (mine, path) in runs.items():
        g = json.load(gzip.open(os.path.join(ROOT, 'bench/oracle/path1', name + '.json.gz')))
        gp = [[e[0], l[0]] for _, e, l, *_ in g['path'][1:] if e or l]
        P = next((k for k, (a, b) in enumerate(zip(gp, path)) if a != b), min(len(gp), len(path)))
        cs = {k: (ft, c) for k, ft, c in calls(caps[name])}
        R = next((k for k in range(P) if k in cs and k in mine and mine[k]['klast'] != cs[k][0]['upd']), P)
        n, firstC = collections.Counter(), None
        for k in range(R):
            if k not in cs or cs[k][1] is None:
                continue
            c = cs[k][1]
            s1 = mine.get(k + 1)
            if s1 is None or 'aqk' not in s1 or 'ucp' not in s1:
                bad.append(f'O {name} pivot {k}: Rust snapshot lacks aqk/ucp')
                break
            if s1['uk'] != c['f']:
                bad.append(f'K {name} pivot {k}: Rust {s1["uk"]} original {c["f"]}')
                continue
            n['K'] += 1
            gcols = [[col[0], col[2]] for col in reversed(c['cols'])]
            if s1['f0k'] != c['f0'] or s1['t'] != c['t']:
                # the prefix t or the LU+0xf0 list differs (05b257 prefix rule, M1-02n): out of this contract
                n['C excluded (t / f0)'] += 1
                if firstC is None:
                    firstC = f'{k} (t Rust {s1["t"]} original {c["t"]}, LU+0xec Rust {len(s1["f0k"])} original {c["ec"]})'
                continue
            if s1['ucp'] != gcols:
                j = next((j for j, (a, b) in enumerate(zip(s1['ucp'], gcols)) if a != b), min(len(s1['ucp']), len(gcols)))
                bad.append(f'C {name} pivot {k}: tail column {j}: Rust {s1["ucp"][j:j + 1]} original {gcols[j:j + 1]}')
                continue
            n['C'] += 1
            if s1['aqk'] != c['out']['idx']:
                t = next((t for t in range(min(len(s1['aqk']), len(c['out']['idx']))) if s1['aqk'][t] != c['out']['idx'][t]), None)
                bad.append(f'O {name} pivot {k} {c["f"]}: Rust {s1["aqk"][t:t + 4] if t is not None else len(s1["aqk"])} '
                           f'original {c["out"]["idx"][t:t + 4] if t is not None else len(c["out"]["idx"])} at {t}')
            n['O'] += 1
            if s1['lro'] != c['in']['idx']:
                bad.append(f'I {name} pivot {k}: kernel input Rust {s1["lro"][:6]} original {c["in"]["idx"][:6]}')
            n['I'] += 1
        stop = f' (pivot {R}: Rust {mine[R]["klast"]} updates, original {cs[R][0]["upd"]})' if R < P and R in cs else ''
        cdiff = f'; first excluded pivot {firstC}' if firstC is not None else ''
        rep.append(f'{name} Rust: path = original on pivots 0-{P - 1}, update count on 0-{R - 1}{stop}; {dict(n)}{cdiff}')
    return bad, rep


def main():
    st = '--self-test' in sys.argv
    caps, lss, bad = {}, {}, []
    for name, (h, mh) in CAP.items():
        for r in (1, 2, 3):
            p = os.path.join(D, f'{name}_us_{r}.json.gz')
            if sha(p) != h:
                bad.append(f'M {name}_us_{r}: sha {sha(p)[:12]}')
        caps[name] = load(os.path.join(D, f'{name}_us_1.json.gz'))
        if caps[name]['model_sha256'] != mh:
            bad.append(f'M {name}: model sha {caps[name]["model_sha256"][:12]}')
        if sha(LS[name]) != LS_SHA[name]:
            bad.append(f'M {os.path.basename(LS[name])}: sha {sha(LS[name])[:12]}')
        lss[name] = load(LS[name])
    b, rep = check(caps, lss)
    bad += b
    print('\n'.join(rep))
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
            b2, _ = check(c2, lss, var)
            print(f'T {desc}: {"rejected" if b2 else "NOT rejected"}' + (f' ({b2[0][:100]})' if b2 else ''))
            if not b2:
                bad.append(f'T {desc} not rejected')

        def kcall(c2, name, f):
            return next(c for _, _, c in calls(c2[name]) if c is not None and f(c))

        def swap(c2):
            c = kcall(c2, 'degen2', lambda c: c['out']['nnz'] >= 3)
            c['out']['idx'][1], c['out']['idx'][2] = c['out']['idx'][2], c['out']['idx'][1]

        def bit(c2):
            c = kcall(c2, 'kb2', lambda c: c['out']['nnz'] >= 2)
            c['out']['val'][1] = format(int(c['out']['val'][1], 16) ^ 1, '016x')

        tamper('binary hash 0', lambda c2: c2['kb2'].__setitem__('binary_sha256', '0' * 64))
        tamper('no events', lambda c2: c2['degen2'].__setitem__('events', []))
        tamper('two kernel outputs swapped (degen2)', swap)
        tamper('one kernel output value bit (kb2)', bit)
        tamper('model: H2 in LU+0xf0 list order', lambda c2: None, 'list')
        tamper('model: section 2 with the prefix inputs last', lambda c2: None, 'prelast')
        tamper('model: 2.5 -> 0.5', lambda c2: None, 'coef')
        if runs:
            def rtamper(desc, f):
                r2 = copy.deepcopy(runs)
                f(r2)
                b2, _ = check_rust(caps, r2)
                print(f'T {desc}: {"rejected" if b2 else "NOT rejected"}' + (f' ({b2[0][:100]})' if b2 else ''))
                if not b2:
                    bad.append(f'T {desc} not rejected')

            def aqk_swap(r2):
                s = next(s for k, s in sorted(r2['degen2'][0].items()) if len(s.get('aqk') or []) >= 3 and s.get('uk') in ('21686f', '21841f'))
                s['aqk'][0], s['aqk'][-1] = s['aqk'][-1], s['aqk'][0]

            rtamper('Rust kernel output: first and last swapped (degen2)', aqk_swap)
            rtamper('Rust snapshots without ucp', lambda r2: [s.pop('ucp', None) for s in r2['kb2'][0].values()])

            def ucp_swap(r2):
                s = next(s for k, s in sorted(r2['degen2'][0].items()) if any(len(c[1]) >= 2 for c in s.get('ucp', [])))
                c = next(c for c in s['ucp'] if len(c[1]) >= 2)
                c[1][0], c[1][1] = c[1][1], c[1][0]

            rtamper('Rust U column prefix part: two entries swapped (degen2)', ucp_swap)
    for x in bad:
        print('FAIL', x)
    print('PASS' if not bad else f'FAIL ({len(bad)})')
    sys.exit(1 if bad else 0)


if __name__ == '__main__':
    main()
