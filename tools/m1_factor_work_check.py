#!/usr/bin/env python3
"""M1-02q/M1-02r/M1-02s/M1-02v: Rust dual work at the factor (036ade), FT update and refactor-return (05b257, 9e60b), start
(534f7, 1bae9c) and post-05b257 (1bae9c, 1b7106, 1ca57, 16b0a) sites against the original. Contracts:
docs/m1/m1_02q_work.md, docs/m1/m1_02r_ft_work.md and docs/m1/m1_02s_post_work.md section 3.
Exit 0 only when every check passes.
Run with this worktree's build: tools/build_local.sh && PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_factor_work_check.py
  --self-test   also the tampers (T)
  --mutate      also the Rust mutations (R): each is built in a scratch copy and must be rejected
"""
import collections, copy, gzip, hashlib, json, os, shutil, struct, subprocess, sys, tempfile

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
D = os.path.join(ROOT, 'bench/oracle/m1_02q')
BIN = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
TOOL = os.path.join(ROOT, 'tools/m1_factor_work.js')
CAP = {'kb2': '61dc54471404280e5eb53c10d3442c1873d770277ce40a8e334c401e25c9890d',
       'degen2': 'b0dd087a2a1ad67d828b189c600dd41ea2dd0fe2b406c6bb4b9dd3ba20ec8b73'}  # gz SHA-256 of runs 1-3
WCAP = {'kb2': 'f4dfd4ba946a43b13e90cf73cd235b33d70932fe877c7caae2d567f3480e168b',
        'degen2': 'c476371aeb7317374cde3a43fc5bccb1299d0b788fe10f7cba67bd716f4d4401'}  # M1-02p watchpoint captures
NEED = {'kb2': 5, 'degen2': 9}  # refactors on the original path (contract section 1)
MODEL = {n: os.path.join(ROOT, '../re/bench/data/netlib', n + '.mps.gz') for n in CAP}
SITES = {0xe84b9, 0xc391f, 0xd4ded, 0xd5300, 0xd5e39, 0xd85e6, 0xdc7b2, 0xdcb7b, 0xf48ae, 0x1327dc, 0x1329e6, 0x1329c1,
         0xf4f9b, 0xf4ffd, 0xf507a, 0xf515b, 0xf51af}
FT = {0xf1beb, 0xf1d02, 0xf1f37, 0xf1f82, 0xf2044, 0xf207a, 0xf2521, 0xf2969, 0xf2a51, 0xf2e3c, 0xf3ca4, 0xf3d83, 0xf3eac,
      0xf3f71, 0xf4073, 0xf461f}  # M1-02r: 05b257 FT update path and the refactor path's spike clear
SITES |= FT
POST = {0xc0410, 0xc06c7, 0xc0c24, 0xc09bd, 0xc26ec, 0xc2761, 0xc2842, 0xc2ad4, 0xc142b, 0xc4329, 0xc1fe3, 0xbf981, 0xbfed2,
        0xbe793}  # M1-02s: 1bae9c per pivot / list rebuild / after a refactor, 1b7106, 1ca57, 16b0a
SITES |= POST
ITER = {0x135e92, 0xc7bcd, 0xc7c4f, 0xc82ee, 0xc8b67, 0xc92bd, 0xc9436, 0xcbcae, 0xc561a,
        0xcce72, 0xcd052, 0xcd191, 0xbe039, 0xbe0a1, 0xbe17e, 0xbe34d, 0xcdbb5, 0xce1ef, 0xce316}
SITES |= ITER
POST_RUN = POST - {0xc09bd}  # 0xc09bd (mode 1 with spx+0x2c8 set) is not expected to run (contract section 1)
UPD = {'kb2': (35, 35), 'degen2': (300, 10 ** 9)}  # FT updates (0xf2e3c) in range: (min, max)
F = lambda h: struct.unpack('<d', int(h, 16).to_bytes(8, 'little'))[0]
H = lambda x: struct.pack('<d', x)[::-1].hex()
sha = lambda p: hashlib.sha256(open(p, 'rb').read()).hexdigest()
load = lambda p: json.loads(gzip.decompress(open(p, 'rb').read()))


def rust_run(name, seeds, pyroot=None):
    """Rust lines (one per refactor) on the model with the original refactor-entry work as the seed."""
    model = MODEL[name]
    with tempfile.TemporaryDirectory() as td:
        out, seed = os.path.join(td, 'work.jsonl'), os.path.join(td, 'seed.txt')
        open(seed, 'w').write('\n'.join(seeds) + '\n')
        code = (f"import rsolver; m = rsolver.read({model!r}); m.Params['Method'] = 1; m.Params['Presolve'] = 0; "
                f"m.Params['OutputFlag'] = 0; m.optimize()")
        env = dict(os.environ, RSOLVER_WORK=out, RSOLVER_WORK_SEED=seed)
        if pyroot:
            env['PYTHONPATH'] = pyroot
        subprocess.run([sys.executable, '-c', code], env=env, check=True)
        return [json.loads(l) for l in open(out)] if os.path.exists(out) else []


def windows(events):
    """Original 036ade calls: (F event, its T terms)."""
    out, cur = [], None
    for e in events:
        if e['type'] == 'F':
            cur = (e, [])
        elif e['type'] == 'T' and cur:
            cur[1].append(e)
        elif e['type'] == 'X' and cur:
            out.append(cur)
            cur = None
    return out


def terms(ts):
    """Original terms as (kind, a, b, c): c/r singleton count, g = (ns, ecx, r10d) from 0xd6ea4 + 0xd7157."""
    out, ns = [], None
    for t in ts:
        pc = t['pc']
        if pc == 0xd6077:
            out.append(('c', t['cnt'], 0, 0))
        elif pc == 0xd647b:
            out.append(('r', t['cnt'], 0, 0))
        elif pc == 0xd6ea4:
            ns = t['ns']
        elif pc == 0xd7157:
            out.append(('g', ns, t['ecx'], t['r10']))
            ns = None
    return out


def check(caps, wcaps, runs, files_ok=True):
    """M N A K W F G on loaded data; returns (failures, report)."""
    bad, rep = [], []
    reach = collections.Counter()
    for name in CAP:
        c, wc, rust = caps[name], wcaps[name], runs[name]
        if c.get('binary_sha256') != BIN or wc.get('binary_sha256') != BIN or not c.get('ok') or c.get('errors'):
            bad.append(f'M {name}: binary hash / ok / errors')
        if c.get('model_sha256') != wc.get('model_sha256') or (files_ok and sha(MODEL[name]) != c.get('model_sha256')):
            bad.append(f'M {name}: model sha differs (file Rust reads, this capture, M1-02p capture)')
        # N: refactor and update entries from 06268b with the work bits equal M1-02p
        u1 = [(e['k'], e['reason'], e['work']) for e in c['events'] if e['type'] == 'U']
        u0 = [(e['k'], e['reason'], e['work']) for e in wc['events'] if e['type'] == 'U']
        if u1 != u0 or not u1:
            i = next((i for i in range(min(len(u1), len(u0))) if u1[i] != u0[i]), min(len(u1), len(u0)))
            bad.append(f'N {name}: 05b257 entry {i}: new {u1[i] if i < len(u1) else None} 02p {u0[i] if i < len(u0) else None}')
        if c.get('output') != wc.get('output'):
            bad.append(f'N {name}: output {c.get("output")!r} vs {wc.get("output")!r}')
        # A: aligned range
        wins = windows(c['events'])
        n = 0
        for (f, _), r in zip(wins, rust):
            # the original factors the crash basis by row at refactor 1 and renumbers after it; Rust's start.rs applies
            # that renumbering before (start.rs first factorization), so only the set is compared there (F2 on: order)
            same = r['head'] == f['head'] or (n == 0 and sorted(r['head']) == sorted(f['head']))
            if n == 0 and same and r['head'] != f['head']:
                rep.append(f'{name}: refactor 1 basis = original as a set, position order differs (Rust start.rs renumbers before it)')
            if not same or r['iter'] != f['k'] or r['tol'] != f['x390'] or not f['work']:
                why = 'head' if not same else 'iteration' if r['iter'] != f['k'] else 'tol' if r['tol'] != f['x390'] else 'work'
                rep.append(f'{name}: alignment ends at refactor {n + 1} ({why}: Rust iteration {r["iter"]}, original k {f["k"]})')
                break
            n += 1
        if n < NEED[name]:
            bad.append(f'A {name}: {n} aligned refactors, need {NEED[name]}')
        # K: Markowitz terms
        kt = kg = 0
        for i in range(n):
            (f, ts), r = wins[i], rust[i]
            o = terms(ts)
            m = [(t[0], t[1], t[2], t[3]) if t[0] == 'g' else (t[0], t[1], 0, 0) for t in r['terms']]
            if o != m:
                j = next((j for j in range(min(len(o), len(m))) if o[j] != m[j]), min(len(o), len(m)))
                bad.append(f'K {name} refactor {i + 1} term {j}: original {o[j] if j < len(o) else None} Rust {m[j] if j < len(m) else None}')
            if sum(1 for t in ts if t['pc'] == 0xd85c7) != 1:
                bad.append(f'K {name} refactor {i + 1}: fold point count')
            early = sum(1 for t in ts if t['pc'] == 0xd5e16)
            if early != sum(1 for s in r['stores'] if s[0] == 0xd5e39):
                bad.append(f'K {name} refactor {i + 1}: early folds original {early}')
            kt += len(o)
            kg += sum(1 for t in o if t[0] == 'g')
        # W: stores of the ported sites. Range: with refactor n + 1 not aligned, every iteration before its k (the FT
        # updates in between included, M1-02r); with all aligned, up to the n-th refactor's 05b257 return
        ev = wc['events']
        kstop = wins[n][0]['k'] if n < len(wins) else None
        nref, stop = 0, len(ev)
        for j, e in enumerate(ev):
            if e['type'] == 'U' and e['reason'] != 0:
                nref += 1
            if kstop is None and e['type'] == 'V' and nref == n:
                stop = j
                break
            if kstop is not None and e['k'] >= kstop:
                stop = j
                break
        prev, ow = None, []
        for e in ev[:stop]:
            if e['type'] == 'E':
                prev = e['v']
            elif e['type'] == 'W':
                if e['pc'] in SITES:
                    ow.append((e['pc'], prev, e['v'], e['k']))
                prev = e['v']
        rw = [s for r in rust[:n] for s in r['stores']]
        if kstop is not None and n < len(rust):
            rw += [s for s in rust[n]['stores'] if s[3] < kstop]
        sites = collections.Counter()
        if [s[0] for s in rw] != [o[0] for o in ow]:
            j = next((j for j in range(min(len(rw), len(ow))) if rw[j][0] != ow[j][0]), min(len(rw), len(ow)))
            bad.append(f'W {name}: store {j} site Rust {hex(rw[j][0]) if j < len(rw) else None} k {rw[j][3] if j < len(rw) else None}'
                       f' original {hex(ow[j][0]) if j < len(ow) else None} k {ow[j][3] if j < len(ow) else None}')
        else:
            for j, ((site, val, adds, k), (_, p, v, ok)) in enumerate(zip(rw, ow)):
                x = F(p)
                for a in adds:
                    x = F(a) + x
                if H(x) != v or k != ok:
                    bad.append(f'W {name}: store {j} k {ok} (Rust {k}) site {hex(site)}: original prev {p} + Rust {[F(a) / 1.3 for a in adds]}'
                               f' x 1.3 = {H(x)}, original {v} (amount {(F(v) - F(p)) / 1.3} x 1.3)')
                    break
                sites[hex(site)] += 1
                if site in FT or site in POST or site in ITER:
                    reach[site] += 1
        # G: FT updates in range
        nu = sites[hex(0xf2e3c)]
        if not UPD[name][0] <= nu <= UPD[name][1]:
            bad.append(f'G {name}: {nu} FT updates compared, need {UPD[name]}')
        # G2: one DSE update store per FT update (beta is never 0 here)
        if sites[hex(0xc0410)] != nu:
            bad.append(f'G2 {name}: {sites[hex(0xc0410)]} 0xc0410 stores for {nu} FT updates')
        # F: spx+0x158 at each 05b257 return
        v158 = [e['w158'] for e in ev if e['type'] == 'V' and e['reason'] != 0][:n]
        for i in range(n):
            if rust[i]['w158'] != v158[i]:
                bad.append(f'F {name} refactor {i + 1}: 0x158 Rust {rust[i]["w158"]} original {v158[i]}')
                break
        rep.append(f'{name}: N 05b257 entries {len(u1)}, A aligned refactors {n}/{len(wins)}, K terms {kt} (general {kg}), '
                   f'W stores {len(rw)} {dict(sites)}, F 0x158 {n}' + (f', W range k < {kstop}' if kstop is not None else ''))
    miss = sorted(hex(s) for s in FT - set(reach))
    if miss:
        bad.append(f'G: FT sites never compared {miss}')
    miss = sorted(hex(s) for s in POST_RUN - set(reach))
    if miss:
        bad.append(f'G2: post-05b257 sites never compared {miss}')
    miss = sorted(hex(s) for s in ITER - set(reach))
    if miss:
        bad.append(f'G3: iteration sites never compared {miss}')
    return bad, rep


def main():
    caps, wcaps, bad = {}, {}, []
    tool = sha(TOOL)
    for name in CAP:
        hs = [sha(os.path.join(D, f'{name}_fw_{r}.json.gz')) for r in (1, 2, 3)]
        if len(set(hs)) != 1 or hs[0] != CAP[name]:
            bad.append(f'M {name}: run hashes {[x[:12] for x in hs]}')
        caps[name] = load(os.path.join(D, f'{name}_fw_1.json.gz'))
        if caps[name]['tool_sha256'] != tool:
            bad.append(f'M {name}: capture tool sha {caps[name]["tool_sha256"][:12]} vs {tool[:12]}')
        wp = os.path.join(ROOT, 'bench/oracle/m1_02p', f'{name}_w_1.json.gz')
        if sha(wp) != WCAP[name]:
            bad.append(f'M {name}: M1-02p capture sha')
        wcaps[name] = load(wp)
    seeds = {n: [e['work'] for e in wcaps[n]['events'] if e['type'] == 'U' and e['reason'] != 0] for n in CAP}
    runs = {n: rust_run(n, seeds[n]) for n in CAP}
    b, rep = check(caps, wcaps, runs)
    bad += b
    print('\n'.join(rep))
    if '--self-test' in sys.argv:
        def flip(h):
            return '%016x' % (int(h, 16) ^ 1)

        def t_bin(c, w, r):
            c['kb2']['binary_sha256'] = '0' * 64

        def t_empty(c, w, r):
            c['degen2']['events'] = []

        def t_nof(c, w, r):
            c['kb2']['events'] = [e for e in c['kb2']['events'] if e['type'] != 'F']

        def t_nwork(c, w, r):
            next(e for e in c['degen2']['events'] if e['type'] == 'U' and e['k'] == 50)['work'] = '0' * 16

        def t_head(c, w, r):
            [e for e in c['degen2']['events'] if e['type'] == 'F'][4]['head'][7] += 1

        def t_head1(c, w, r):
            # refactor 1 is compared as a set: a different member must still fail
            f = next(e for e in c['kb2']['events'] if e['type'] == 'F')
            f['head'][3] = max(f['head']) + 1

        def t_term(c, w, r):
            [e for e in c['degen2']['events'] if e['type'] == 'T' and e['pc'] == 0xd647b][3]['cnt'] += 1

        def t_add(c, w, r):
            # one unit (x148) more: a last-bit change of an amount is absorbed by the sum's rounding and unobservable
            s = r['degen2'][6]['stores']
            s[0][2][0] = H(F(s[0][2][0]) + 1.3)

        def t_158(c, w, r):
            r['kb2'][3]['w158'] = flip(r['kb2'][3]['w158'])

        def t_site(c, w, r):
            s = next(s for s in r['degen2'][8]['stores'] if s[0] == 0x1329e6)
            s[0] = 0x1327dc

        def t_wval(c, w, r):
            e = next(e for e in w['kb2']['events'] if e['type'] == 'W' and e['pc'] == 0xd85e6 and e['k'] == 26)
            e['v'] = flip(e['v'])

        def t_ftadd(c, w, r):
            s = [s for s in r['kb2'][2]['stores'] if s[0] == 0xf1f82][1]
            s[2][-1] = H(F(s[2][-1]) + 1.3)

        def t_ftdrop(c, w, r):
            s = r['degen2'][5]['stores']
            s.remove(next(x for x in s if x[0] == 0xf3d83))

        def t_461f(c, w, r):
            next(s for s in r['degen2'][3]['stores'] if s[0] == 0xf461f)[0] = 0xf48ae

        def t_wft(c, w, r):
            e = [e for e in w['kb2']['events'] if e['type'] == 'W' and e['pc'] == 0xf1f82][7]
            e['v'] = flip(e['v'])

        def t_k(c, w, r):
            next(s for s in r['kb2'][3]['stores'] if s[0] == 0xf2e3c)[3] += 1

        # M1-02s
        def t_c0410(c, w, r):
            s = [s for s in r['kb2'][2]['stores'] if s[0] == 0xc0410][1]
            s[2][-1] = H(F(s[2][-1]) + 1.3)

        def t_c142b(c, w, r):
            s = r['degen2'][4]['stores']
            s.remove(next(x for x in s if x[0] == 0xc142b))

        def t_mode(c, w, r):
            next(s for s in r['kb2'][1]['stores'] if s[0] == 0xc06c7)[0] = 0xc0c24

        def t_wc1fe3(c, w, r):
            e = next(e for e in w['kb2']['events'] if e['type'] == 'W' and e['pc'] == 0xc1fe3)
            e['v'] = flip(e['v'])

        def t_be793(c, w, r):
            s = r['degen2'][3]['stores']
            s.remove(next(x for x in s if x[0] == 0xbe793))

        def t_iter_add(c, w, r):
            s = next(s for f in r['degen2'] for s in f['stores'] if s[0] == 0x135e92)
            s[2][-1] = H(F(s[2][-1]) + 1.3)

        def t_iter_drop(c, w, r):
            s = next(f['stores'] for f in r['kb2'] if any(s[0] == 0xcbcae for s in f['stores']))
            s.remove(next(s for s in s if s[0] == 0xcbcae))

        def t_iter_zero(c, w, r):
            for n in CAP:
                w[n]['events'] = [e for e in w[n]['events'] if e['type'] != 'W' or e['pc'] not in ITER]
                for f in r[n]:
                    f['stores'] = [s for s in f['stores'] if s[0] not in ITER]

        def t_iter_k(c, w, r):
            next(s for f in r['degen2'] for s in f['stores'] if s[0] == 0xce1ef)[3] -= 1

        def t_iter_site(c, w, r):
            next(s for f in r['kb2'] for s in f['stores'] if s[0] == 0xcd052)[0] = 0xcce72

        def t_iter_val(c, w, r):
            e = next(e for e in w['kb2']['events'] if e['type'] == 'W' and e['pc'] == 0xc9436)
            e['v'] = flip(e['v'])

        def t_model(c, w, r):
            c['kb2']['model_sha256'] = w['kb2']['model_sha256'] = '0' * 64

        for t in (t_iter_add, t_iter_drop, t_iter_zero, t_iter_k, t_iter_site, t_iter_val, t_model, t_bin, t_empty, t_nof, t_nwork, t_head, t_head1, t_term, t_add, t_158, t_site, t_wval, t_ftadd, t_ftdrop,
                  t_461f, t_wft, t_k, t_c0410, t_c142b, t_mode, t_wc1fe3, t_be793):
            c, w, r = copy.deepcopy(caps), copy.deepcopy(wcaps), copy.deepcopy(runs)
            t(c, w, r)
            b2, _ = check(c, w, r)
            print(f'T {t.__name__}: ' + (f'rejected ({b2[0]})' if b2 else 'NOT REJECTED'))
            if not b2:
                bad.append(f'T {t.__name__} not rejected')
    if '--mutate' in sys.argv:
        bad += mutate(caps, wcaps, seeds)
    print('\n'.join(bad) if bad else ('ACCEPTANCE PASS' + (' (+ self-test)' if '--self-test' in sys.argv else '')))
    return 1 if bad else 0


MUTANTS = [  # (name, file, original text, mutated text)
    ('d4ded without 100', 'crates/lp/src/lu.rs', '(nb * 5.0 + 100.0)', '(nb * 5.0 + 0.0)'),
    ('general term max', 'crates/lp/src/lu.rs', 'rsum.min(csum)', 'rsum.max(csum)'),
    ('0x158 without 0.5', 'crates/lp/src/work.rs', '(d + self.w158) * 0.5', '(d + self.w158)'),
    ('9e60b site swapped', 'crates/lp/src/lu.rs', 'if cost_mode == 2 { 0x1329e6 } else { 0x1327dc }', 'if cost_mode != 2 { 0x1329e6 } else { 0x1327dc }'),
    ('f507a store test', 'crates/lp/src/lu.rs', 'if d < m {\n            w.add(0xf507a', 'if d <= m {\n            w.add(0xf507a'),
    ('q without U', 'crates/lp/src/lu.rs', 'lu.lcol[k].len() + lu.urow[k].len()', 'lu.lcol[k].len()'),
    # M1-02r (contract section 3, fixed before the first run)
    ('f1f82 row length', 'crates/lp/src/lu.rs', '(false, at_kp)', '(false, self.urow[ki].len())'),
    ('f2044 test <=', 'crates/lp/src/lu.rs', 'if old_len < ns {', 'if old_len <= ns {'),
    ('section start swapped', 'crates/lp/src/lu.rs', '(0xf3f71, h - *mid) } else { (0xf2969, h) }', '(0xf3f71, h) } else { (0xf2969, h - *mid) }'),
    ('f3ca4 without 5', 'crates/lp/src/lu.rs', 'acc = (acc + self.ucol[x].len() as f64 * 2.0) + 5.0;', 'acc += self.ucol[x].len() as f64 * 2.0;'),
    ('f3d83 k + 1', 'crates/lp/src/lu.rs', 'w_cyc = Some((acc, ll - base));', 'w_cyc = Some((acc, ll - base + 1));'),
    ('f461f k + 1', 'crates/lp/src/simplex.rs', '(k as f64 + k as f64) * X148', '((k + 1) as f64 + (k + 1) as f64) * X148'),
    ('f2a51 with local_80 = 0', 'crates/lp/src/lu.rs', 'if l == p0 {\n                w_urow', 'if true {\n                w_urow'),
    ('f1beb association', 'crates/lp/src/lu.rs', '(X148 * 3.0) * n as f64', 'X148 * (3.0 * n as f64)'),
    # added after the first run (review): the no-store branch of a length-1 row at a position > local_40
    ('f1f37 without local_40 test', 'crates/lp/src/lu.rs', '} else if v as i64 <= local40 {', '} else {'),
    # M1-02s (contract section 3, fixed before the first run)
    ('c0410 N with the pivot', 'crates/lp/src/simplex.rs', 'self.work.add(0xc0410, (nq * 3.0) * X148)', 'self.work.add(0xc0410, ((nq + 1.0) * 3.0) * X148)'),
    ('x_B site mode test flipped', 'crates/lp/src/simplex.rs', '(0, _) => self.work.add(0xc06c7', '(1, _) => self.work.add(0xc06c7'),
    ('c142b S with zero bounds', 'crates/lp/src/simplex.rs', 'self.vstat[j] < 0 && self.x_nonbasic(j) != 0.0', 'self.vstat[j] < 0'),
    # moved (not added) before 48d5f's flips and perturbation; fixed after the first --mutate run, whose version added a
    # second store and was rejected for the duplicate only
    ('c142b before the perturbation', 'crates/lp/src/simplex.rs',
     '        if self.in_dual {\n            if flip {\n                if self.make_dual_feasible() {\n                    self.compute_xb();\n                }\n            } else {\n                self.shift_costs();\n            }\n            self.pt_rc += 1;\n            self.perturb_trigger();\n            // 48d5f C:2106-2111, after the perturbation; it stays on\n            // shortcut: the spx+0x178 != 0 disjunct is left out (no spx+0x178 here; 0 in every M1-02e capture)\n            if !self.nondeg_flip && (self.pt_lvl > 2 || (self.m as u64) < 3 * self.iter) {\n                self.nondeg_flip = true;\n                if std::env::var("RSOLVER_DEBUG").is_ok() {\n                    eprintln!("nondegflip on at it {}", self.iter);\n                }\n            }\n            self.work_xb();\n',
     '        if self.in_dual {\n            self.work_xb();\n            if flip {\n                if self.make_dual_feasible() {\n                    self.compute_xb();\n                }\n            } else {\n                self.shift_costs();\n            }\n            self.pt_rc += 1;\n            self.perturb_trigger();\n            // 48d5f C:2106-2111, after the perturbation; it stays on\n            // shortcut: the spx+0x178 != 0 disjunct is left out (no spx+0x178 here; 0 in every M1-02e capture)\n            if !self.nondeg_flip && (self.pt_lvl > 2 || (self.m as u64) < 3 * self.iter) {\n                self.nondeg_flip = true;\n                if std::env::var("RSOLVER_DEBUG").is_ok() {\n                    eprintln!("nondegflip on at it {}", self.iter);\n                }\n            }\n'),
    ('c1fe3 nInf = listed', 'crates/lp/src/simplex.rs', 'let ninf = viol.iter().flatten().count();', 'let ninf = self.pr.list.len();'),
    ('bf981 flag 0', 'crates/lp/src/simplex.rs', '((self.pt_lvl != 0) as i32 as f64 + 2.0)', '(0.0 + 2.0)'),
    ('c2761 k + s', 'crates/lp/src/simplex.rs', '((r.kept as f64 + r.kept as f64) + r.scanned as f64)', '(r.kept as f64 + r.scanned as f64)'),
    ('c26ec stored with an empty old list', 'crates/lp/src/simplex.rs', 'if r.scanned > 0 {\n                    self.work.add(0xc26ec', 'if true {\n                    self.work.add(0xc26ec'),
    ('be793 association', 'crates/lp/src/simplex.rs', '(X148 * 3.0) * n as f64', '(n as f64 * 3.0) * X148'),
]
# M1-02v: every new mutant is expected to be rejected; fixed before executing mutations.
MUTANTS += [
    ('a4109 sparse cost', 'crates/lp/src/simplex.rs', 'wf * 3.5', 'wf * 3.0'),
    ('a4109 single cost', 'crates/lp/src/simplex.rs', '((a + len) + 5.0) + len', '((a + len) + 4.0) + len'),
    ('a4109 big-M tail', 'crates/lp/src/simplex.rs', 'amount = (((rf + rf) + amount) + rf) + rf;', 'amount = ((rf + rf) + amount) + rf;'),
    ('partition length', 'crates/lp/src/bfrt.rs', '(hi - lo + 4) as f64', '(hi - lo + 3) as f64'),
    ('partition always store', 'crates/lp/src/bfrt.rs', 'if run < -feas_tol {', 'if true {'),
    ('single candidate store', 'crates/lp/src/bfrt.rs', 'if c.len() != 1 && thr <= -amin {', 'if thr <= -amin {'),
    ('second filter cost', 'crates/lp/src/bfrt.rs', '(c.len() - lo) as f64 * 3.0', '(c.len() - lo) as f64 * 2.0'),
    ('reselection cost', 'crates/lp/src/bfrt.rs', '(lo as f64 * 4.0 + flips.len() as f64)', '(lo as f64 * 4.0)'),
    ('full pricing feasible cost', 'crates/lp/src/pricing.rs', 'scanned - feasible', 'scanned'),
    ('full pricing rescan cost', 'crates/lp/src/pricing.rs', '((self.list.len() as f64 + m) + m)', '(self.list.len() as f64 + m)'),
    ('CHUZR rebuild scan', 'crates/lp/src/simplex.rs', 'self.work.add(0xbe0a1, a);', 'self.work.add(0xbe0a1, a + X148);'),
    ('flip matrix cost', 'crates/lp/src/simplex.rs', 'sum as f64 * 4.0', 'sum as f64 * 3.0'),
    ('flip result always dense', 'crates/lp/src/lu.rs', '(xb, !sparse)', '(xb, true || !sparse)'),
    ('flip result count', 'crates/lp/src/simplex.rs', '(X148 * 7.0) * nnz as f64', '(X148 * 7.0) * (nnz + 1) as f64'),
]
SURVIVE_OK = {'f1beb association', 'c142b before the perturbation', 'c1fe3 nInf = listed', 'c26ec stored with an empty old list',
              'be793 association'}  # survivors expected in the contract tables (last bit absorbed / branch maybe not run)  # last-bit amount changes are absorbed by the sum (M1-02q section 6)


def mutate(caps, wcaps, seeds):
    """Build each mutant in a scratch copy (shared target dir) and require the checks to fail."""
    bad = []
    scratch = tempfile.mkdtemp(prefix='m1_02q_mut_')
    try:
        for d in ('crates', 'python'):
            shutil.copytree(os.path.join(ROOT, d), os.path.join(scratch, d), ignore=shutil.ignore_patterns('target', '__pycache__'))
        for f in ('Cargo.toml', 'Cargo.lock'):
            shutil.copy(os.path.join(ROOT, f), scratch)
        venv_py = os.path.abspath(os.path.join(ROOT, '..', '.venv/bin/python'))
        for name, f, a, b in MUTANTS:
            p = os.path.join(scratch, f)
            src = open(p).read()
            assert src.count(a) == 1, (name, a)
            open(p, 'w').write(src.replace(a, b))
            env = dict(os.environ, PYO3_PYTHON=venv_py, CARGO_TARGET_DIR=os.path.join(scratch, 'target'))
            subprocess.run(['cargo', 'build', '--release', '-q', '-p', 'rsolver'], cwd=scratch, env=env, check=True)
            shutil.copy(os.path.join(scratch, 'target/release/lib_core.so'),
                        os.path.join(scratch, 'python/rsolver/_core.cpython-313-x86_64-linux-gnu.so'))
            runs = {n: rust_run(n, seeds[n], os.path.join(scratch, 'python')) for n in CAP}
            b2, _ = check(caps, wcaps, runs)
            print(f'R {name}: ' + (f'rejected ({b2[0]})' if b2 else 'NOT REJECTED'))
            if not b2 and name not in SURVIVE_OK:
                bad.append(f'R {name} not rejected')
            open(p, 'w').write(src)
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    return bad


if __name__ == '__main__':
    sys.exit(main())
