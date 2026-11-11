#!/usr/bin/env python3
"""M1-02q: Rust dual work at the factor (036ade), refactor-return (05b257, 9e60b) and start (534f7, 1bae9c) sites
against the original. Contract: docs/m1/m1_02q_work.md section 3. Exit 0 only when every check passes.
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
SITES = {0xe84b9, 0xc391f, 0xd4ded, 0xd5300, 0xd5e39, 0xd85e6, 0xdc7b2, 0xdcb7b, 0xf48ae, 0x1327dc, 0x1329e6, 0x1329c1,
         0xf4f9b, 0xf4ffd, 0xf507a, 0xf515b, 0xf51af}
F = lambda h: struct.unpack('<d', int(h, 16).to_bytes(8, 'little'))[0]
H = lambda x: struct.pack('<d', x)[::-1].hex()
sha = lambda p: hashlib.sha256(open(p, 'rb').read()).hexdigest()
load = lambda p: json.loads(gzip.decompress(open(p, 'rb').read()))


def rust_run(name, seeds, pyroot=None):
    """Rust lines (one per refactor) on the model with the original refactor-entry work as the seed."""
    model = os.path.join(ROOT, '../re/bench/data/netlib', name + '.mps.gz')
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
    """M N A K W F on loaded data; returns (failures, report)."""
    bad, rep = [], []
    for name in CAP:
        c, wc, rust = caps[name], wcaps[name], runs[name]
        if c.get('binary_sha256') != BIN or wc.get('binary_sha256') != BIN or not c.get('ok') or c.get('errors'):
            bad.append(f'M {name}: binary hash / ok / errors')
        if c.get('model_sha256') != wc.get('model_sha256'):
            bad.append(f'M {name}: model sha differs from the M1-02p capture')
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
            # the first factor's input order is the start basis' position order, which Rust's start.rs assigns
            # differently (same set; the factor renumbers positions, F2 on are equal in order): reported, not compared
            same = r['head'] == f['head'] or (n == 0 and sorted(r['head']) == sorted(f['head']))
            if n == 0 and same and r['head'] != f['head']:
                rep.append(f'{name}: refactor 1 basis = original as a set, position order differs (start basis)')
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
        # W: stores of the ported sites, original range = before the n-th refactor's 05b257 return
        ev = wc['events']
        nref, stop = 0, len(ev)
        for j, e in enumerate(ev):
            if e['type'] == 'U' and e['reason'] != 0:
                nref += 1
            if e['type'] == 'V' and nref == n:
                stop = j
                break
        prev, ow = None, []
        for e in ev[:stop]:
            if e['type'] == 'E':
                prev = e['v']
            elif e['type'] == 'W':
                if e['pc'] in SITES:
                    ow.append((e['pc'], prev, e['v']))
                prev = e['v']
        rw = [s for r in rust[:n] for s in r['stores']]
        sites = collections.Counter()
        if [s[0] for s in rw] != [o[0] for o in ow]:
            j = next((j for j in range(min(len(rw), len(ow))) if rw[j][0] != ow[j][0]), min(len(rw), len(ow)))
            bad.append(f'W {name}: store {j} site Rust {hex(rw[j][0]) if j < len(rw) else None} original {hex(ow[j][0]) if j < len(ow) else None}')
        else:
            for j, ((site, val, adds), (_, p, v)) in enumerate(zip(rw, ow)):
                x = F(p)
                for a in adds:
                    x = F(a) + x
                if H(x) != v:
                    bad.append(f'W {name}: store {j} site {hex(site)}: original prev {p} + Rust {adds} = {H(x)}, original {v}')
                    break
                sites[hex(site)] += 1
        # F: spx+0x158 at each 05b257 return
        v158 = [e['w158'] for e in ev if e['type'] == 'V' and e['reason'] != 0][:n]
        for i in range(n):
            if rust[i]['w158'] != v158[i]:
                bad.append(f'F {name} refactor {i + 1}: 0x158 Rust {rust[i]["w158"]} original {v158[i]}')
                break
        rep.append(f'{name}: N 05b257 entries {len(u1)}, A aligned refactors {n}/{len(wins)}, K terms {kt} (general {kg}), '
                   f'W stores {len(rw)} {dict(sites)}, F 0x158 {n}')
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

        for t in (t_bin, t_empty, t_nof, t_nwork, t_head, t_term, t_add, t_158, t_site, t_wval):
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
]


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
            if not b2:
                bad.append(f'R {name} not rejected')
            open(p, 'w').write(src)
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    return bad


if __name__ == '__main__':
    sys.exit(main())
