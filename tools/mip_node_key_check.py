#!/usr/bin/env python3
"""M2-06b-2: heap key formula of 627f3e (captures of tools/mip_node_key.js) and its Rust port
(crates/mip/src/node_heap.rs child_key/deep_key). Contract: docs/m2/mip_node_key.md section 3.

(default)          A1-A6 on bench/oracle/m2_06b2, then R: `cargo test --release -p rsolver-mip --test node_key_l1 --
                   --ignored` on the run 1 inputs, keys compared bit for bit
--write-manifest   record the SHA-256 of every capture (once, after the capture)
--self-test        also T (capture tampers) and V (code mutants on a copy of the workspace, outcomes fixed in section 3)
Exit code 0 only when every requested check passes.
"""
import copy, gzip, hashlib, json, os, shutil, struct, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = os.path.join(ROOT, 'bench/oracle/m2_06b2')
OLD = os.path.join(ROOT, 'bench/oracle/m2_06a')
WORK = os.path.join(ROOT, 'target/mip_node_key')
TOOL = os.path.join(ROOT, 'tools/mip_node_key.js')
SRC = 'crates/mip/src/node_heap.rs'
RUNS = (1, 2, 3)
# case -> branches that must run (all others must not): 'A', 'A-inf', 'B', 'B-inf', 'C', 'D', 'E', 'deep'
CASES = {'markshare_4_0': {'A'}, 'pk1': {'A'}, 'neos5': {'A'}, 'tr12-30': {'A'}, 'fhnw-binpack4-4': {'A-inf'},
         'neos-860300': {'A', 'B'}}
BRANCHES = ('A', 'A-inf', 'B', 'B-inf', 'C', 'C-inf', 'D', 'D-inf', 'E', 'E-inf', 'deep')
OLD_CASES = ('markshare_4_0', 'pk1', 'neos5', 'tr12-30')  # M2-06a captures of the same runs (A6)
NOT_FHNW = {c: c != 'fhnw-binpack4-4' for c in CASES}
NONE = dict.fromkeys(CASES, False)
MUTANTS = [  # (name, old, new, expected rejection per case); section 3 V
    # K1 first expected 'all but fhnw' from a count that included n == a4 (ratio exactly 1); corrected to the cases
    # with n > a4 after the first self-test (docs/m2/mip_node_key.md section 6)
    ('K1 no min(ratio, 1)', '(n / f64::from(root_nfrac.max(1))).min(1.0)', '(n / f64::from(root_nfrac.max(1)))',
     {c: c in ('markshare_4_0', 'neos5', 'tr12-30', 'neos-860300') for c in CASES}),
    ('K2 2^-48 -> 2^-45', '.min(1.0) * 2f64.powi(-48)', '.min(1.0) * 2f64.powi(-45)', NOT_FHNW),
    ('K3 max(|b|, 1) -> |b|', 'bound.abs().max(1.0)', 'bound.abs()', {c: c in ('markshare_4_0', 'pk1') for c in CASES}),
    ('K4 no -1e100 branch', 'if obj_step == -1e100 {', 'if obj_step == 1e300 {', {c: c == 'fhnw-binpack4-4' for c in CASES}),
    ('K5 max(1, a4) -> a4', 'f64::from(root_nfrac.max(1))', 'f64::from(root_nfrac)', NONE),
    ('K6 no depth term', 'if f64::from(parent_depth) > f64::from(ncols) * 100.0 {', 'if false {', NONE),
]
NEG = struct.pack('>d', -1e100).hex()


class Fail(Exception):
    pass


def need(cond, msg):
    if not cond:
        raise Fail(msg)


def sha(p):
    with open(p, 'rb') as fh:
        return hashlib.sha256(fh.read()).hexdigest()


def f(h):
    return struct.unpack('>d', bytes.fromhex(h))[0]


def hx(x):
    return struct.pack('>d', x).hex()


def load(path):
    return json.loads(gzip.decompress(open(path, 'rb').read()))


def a1(manifest):
    """Reproducibility and identity; returns run 1 of every case."""
    tool = sha(TOOL)
    out = {}
    for c in CASES:
        hs = []
        for r in RUNS:
            p = os.path.join(DATA, f'{c}_{r}.json.gz')
            need(os.path.exists(p), f'A1: missing {p}')
            hs.append(sha(p))
            if manifest is not None:
                need(manifest['files'].get(os.path.basename(p)) == hs[-1], f'A1: {p} sha256 != manifest')
        need(len(set(hs)) == 1, f'A1: {c}: runs differ {hs}')
        d = load(os.path.join(DATA, f'{c}_1.json.gz'))
        need(d['ok'] and not d['errors'], f'A1: {c}: capture not ok: {d["errors"][:1]}')
        need(d['tool_sha256'] == tool, f'A1: {c}: captured with another tool version')
        need(d['params'] == {'Threads': 1, 'Seed': 0, 'LogToConsole': 0, 'NodeLimit': 2000} and not d['options'],
             f'A1: {c}: params {d["params"]} {d["options"]}')
        out[c] = d
    return out


def branch(w, k):
    """Branch tag of one key write, and whether its depth term fires."""
    inf = w['m40'] == NEG
    deep = w['p'] in 'AD' and w['pdepth'] > w['ncols'] * 100.0
    return w['p'] + ('-inf' if inf else ''), deep


def py_key(w, k):
    """Section 2.1 formulas in the asm order (independent of the Rust code)."""
    p, m40 = w['p'], f(w['m40'])
    if p in 'AD':
        n, b = f(w['nf']), f(k['b'])
        if m40 == -1e100:
            key = n
        else:
            key = min(n / max(1, w['a4']), 1.0) * 2.0 ** -48 * max(abs(b), 1.0) + b
        if w['pdepth'] > w['ncols'] * 100.0:
            key = key + max(abs(key), 1.0) * 2.0 ** -45
        return key
    if p == 'B':
        b = f(w['b'])
        if m40 == -1e100:
            return float(k['i90']) / float(w['n20']) + float(w['n48'])
        return min(w['n48'] / max(1, w['a4']), 1.0) * 2.0 ** -48 * max(abs(b), 1.0) + b
    if p == 'C':
        return 0.0 if m40 == -1e100 else f(k['b'])
    return float(w['n28']) if m40 == -1e100 else f(w['b'])  # E


def writes(ev):
    for x, e in enumerate(ev):
        if e['e'] == 'call':
            for w in e['w']:
                for k in w['kids']:
                    yield x, e, w, k


def check(c, d, old):
    """A2-A6 on one case; returns (stats, Rust input text, expected key bits)."""
    ev = d['events']
    need(not d['counts'].get('probe_outside:A') and not any(k.startswith('probe_outside') for k in d['counts']),
         f'A2: {c}: probe outside an open call {d["counts"]}')
    st = dict.fromkeys(BRANCHES, 0)
    st.update(writes=0, moved=0, ratio_lt1=0, ratio_gt1=0, absb_lt1=0, a4_lt1=0, root_par=0, root_par_eq=0)
    a4s, m40s, lines, keys = [], set(), [], []
    for x, e, w, k in writes(ev):
        tag, deep = branch(w, k)
        st[tag] += 1
        st['deep'] += deep
        st['writes'] += 1
        st['moved'] += k['moved']
        if e['nest'] == 1:
            m40s.add(w['m40'])
        if w['p'] == 'A':
            # A3: the stack count is the parent LP's +0x80 field, read through the parent node
            need(w['nf'] == hx(float(w['nf_lp'])), f'A3: {c}: event {x}: n {w["nf"]} vs parent LP +0x80 {w["nf_lp"]}')
            need(w['parent_same'] == 1, f'A3: {c}: event {x}: parent N+0x18 != rdi')
            st['ratio_lt1'] += w['nf_lp'] < max(1, w['a4'])
            st['ratio_gt1'] += w['nf_lp'] > max(1, w['a4']) and tag == 'A'
            st['absb_lt1'] += abs(f(k['b'])) < 1 and tag == 'A'
            st['a4_lt1'] += w['a4'] < 1
            if e['nest'] == 1:
                if not a4s or a4s[-1] != w['a4']:
                    a4s.append(w['a4'])
                if w['pdepth'] == 0:
                    st['root_par'] += 1
                    st['root_par_eq'] += w['nf_lp'] == w['a4']
            lines.append(f'A {k["b"]} {w["nf_lp"]} {w["a4"]} {w["m40"]} {w["pdepth"]} {w["ncols"]}')
        elif w['p'] == 'B':
            need(tag == 'B', f'R: {c}: event {x}: B -1e100 side is not ported (section 1)')
            st['ratio_gt1'] += w['n48'] > max(1, w['a4'])
            lines.append(f'B {w["b"]} {w["n48"]} {w["a4"]} {w["m40"]}')
        else:
            raise Fail(f'R: {c}: event {x}: path {w["p"]} is not ported (section 1)')
        keys.append((f'event {x} call {e["c"]} {w["p"]} node {k["id"]}', k['key']))
        # A4
        got = hx(py_key(w, k))
        need(got == k['key'], f'A4: {c}: event {x} call {e["c"]} path {w["p"]} node {k["id"]}: key {k["key"]}, formula {got}')
    ran = {b for b in BRANCHES if st[b]}
    need(ran == CASES[c], f'A2: {c}: branches run {sorted(ran)}, section 4 expects {sorted(CASES[c])}')
    if c == 'fhnw-binpack4-4':
        need(m40s == {NEG}, f'A2: {c}: top-level M+0x40 {m40s}, expected -1e100 (zero objective)')
    else:
        need(NEG not in m40s, f'A2: {c}: top-level M+0x40 is -1e100')
    # A5: every top-tree insert except the root is joined to its latest top-level key write
    last, ins, pending = {}, 0, []
    for x, e in enumerate(ev):
        if e['e'] == 'call' and e['nest'] == 1:
            for w in e['w']:
                for k in w['kids']:
                    last[k['id']] = k
        elif e['e'] == 'ins' and e['from'] != '5daa82':
            nd = e['node']
            k = last.get(nd[0])
            need(k is not None, f'A5: {c}: event {x}: insert of node {nd[0]} without a top-level key write')
            need(k['key'] == nd[2], f'A5: {c}: event {x}: node {nd[0]} insert key {nd[2]} vs written {k["key"]}')
            need(k['bound'] == nd[1], f'A5: {c}: event {x}: node {nd[0]} insert bound {nd[1]} vs {k["bound"]}')
            ins += 1
    st['ins'] = ins
    # A6: non-interference against the M2-06a capture of the same run
    if old is not None:
        mine = [e for e in ev if e['e'] == 'ins']
        theirs = [e for e in old['events'] if e['e'] == 'ins']
        need(len(mine) == len(theirs), f'A6: {c}: {len(mine)} inserts vs M2-06a {len(theirs)}')
        for i, (a, b) in enumerate(zip(mine, theirs)):
            need(a == b, f'A6: {c}: insert {i}: {a} vs M2-06a {b}')
        need(d['output'] == old['output'], f'A6: {c}: output {d["output"]!r} vs M2-06a {old["output"]!r}')
    st['a4_seq'] = a4s
    text = '\n'.join([f'case {c}'] + lines + ['end']) + '\n'
    return st, text, keys


def cargo(ws, target, din, dout):
    env = dict(os.environ, NODE_KEY_IN=din, NODE_KEY_OUT=dout, CARGO_TARGET_DIR=target,
               PATH=os.path.expanduser('~/.cargo/bin') + os.pathsep + os.environ.get('PATH', ''))
    shutil.rmtree(dout, ignore_errors=True)  # no stale output can pass
    r = subprocess.run(['cargo', 'test', '--release', '-p', 'rsolver-mip', '--test', 'node_key_l1', '--', '--ignored'],
                       cwd=ws, env=env, capture_output=True, text=True)
    need(r.returncode == 0 and '1 passed' in r.stdout, f'cargo test failed ({r.returncode}):\n{r.stdout[-2000:]}{r.stderr[-2000:]}')


def compare(c, out, keys):
    lines = out.split('\n')
    need(lines[0] == f'case {c}', f'R: {c}: header {lines[0]!r}')
    got = lines[1:-1] if lines[-1] == '' else lines[1:]
    need(len(got) == len(keys), f'R: {c}: {len(got)} keys vs {len(keys)} writes')
    for g, (where, k) in zip(got, keys):
        need(g == k, f'R: {c}: {where}: rust {g}, original {k}')


def run_rust(ws, target, inputs):
    din, dout = os.path.join(WORK, 'in'), os.path.join(WORK, 'out')
    shutil.rmtree(din, ignore_errors=True)
    os.makedirs(din)
    for c, (text, _) in inputs.items():
        with open(os.path.join(din, f'{c}.input.txt'), 'w') as fh:
            fh.write(text)
    cargo(ws, target, din, dout)
    res = {}
    for c, (_, keys) in inputs.items():
        try:
            compare(c, open(os.path.join(dout, f'{c}.keys.txt')).read(), keys)
            res[c] = None
        except Fail as x:
            res[c] = str(x)
    return res


def tampers(data, olds):
    """Each edit of a capture copy must fail the named checks."""
    def first(ev, ok):
        return next(k for _, e, w, k in writes(ev) if ok(e, w, k))

    def key_bit(d):
        k = first(d['events'], lambda e, w, k: True)
        k['key'] = k['key'][:-1] + ('0' if k['key'][-1] != '0' else '1')

    def a4(d):
        w = next(w for _, e, w, k in writes(d['events']) if w['p'] == 'A' and w['nf_lp'] < w['a4'])
        w['a4'] += 1

    def ins_key(d):
        e = next(e for e in d['events'] if e['e'] == 'ins' and e['from'] != '5daa82')
        e['node'][2] = hx(f(e['node'][2]) * 2 + 1)

    def old_ins(o):
        e = next(e for e in o['events'] if e['e'] == 'ins')
        e['heap_n'] += 1

    for name, case, fn, on_old, want in (('key bit', 'pk1', key_bit, False, ('A4', 'R')),
                                         ('a4 + 1', 'neos5', a4, False, ('A4', 'R')),
                                         ('insert key', 'tr12-30', ins_key, False, ('A5',)),
                                         ('M2-06a insert', 'markshare_4_0', old_ins, True, ('A6',))):
        d, o = copy.deepcopy(data[case]), copy.deepcopy(olds.get(case))
        fn(o if on_old else d)
        seen = []
        try:
            check(case, d, o)
        except Fail as x:
            seen.append(str(x).split(':')[0])
            why = str(x)
        if 'R' in want:
            # the Python formula already fails; R is checked on a capture copy whose A4 is skipped
            w = copy.deepcopy(d)
            lines, keys = [], []
            for x, e, ww, k in writes(w['events']):
                if ww['p'] == 'A':
                    lines.append(f'A {k["b"]} {ww["nf_lp"]} {ww["a4"]} {ww["m40"]} {ww["pdepth"]} {ww["ncols"]}')
                else:
                    lines.append(f'B {ww["b"]} {ww["n48"]} {ww["a4"]} {ww["m40"]}')
                keys.append((f'event {x}', k['key']))
            res = run_rust(ROOT, os.path.join(ROOT, 'target'), {case: ('\n'.join([f'case {case}'] + lines + ['end']) + '\n', keys)})
            if res[case]:
                seen.append('R')
        for t in want:
            need(t in seen, f'T: tamper {name} on {case}: {t} did not fail (failed: {seen})')
        print(f'T: rejected: {name} on {case} by {"+".join(seen)}: {why[:160]}')


def mutants(inputs):
    base = os.path.join(WORK, 'mutant')
    wsp = os.path.join(base, 'ws')
    shutil.rmtree(wsp, ignore_errors=True)
    os.makedirs(wsp)
    for p in ('Cargo.toml', 'Cargo.lock'):
        shutil.copy(os.path.join(ROOT, p), wsp)
    shutil.copytree(os.path.join(ROOT, 'crates'), os.path.join(wsp, 'crates'))
    orig = open(os.path.join(ROOT, SRC)).read()
    for name, a, b, want in MUTANTS:
        need(orig.count(a) == 1, f'V: mutant {name}: pattern {a!r} found {orig.count(a)} times')
        with open(os.path.join(wsp, SRC), 'w') as fh:
            fh.write(orig.replace(a, b))
        res = run_rust(wsp, os.path.join(base, 'target'), inputs)
        for c in CASES:
            tag = 'rejected' if res[c] else 'passes'
            print(f'V: {name}: {c}: {tag}' + (f': {res[c][:160]}' if res[c] else ''))
            need(bool(res[c]) == want[c], f'V: mutant {name} on {c}: {tag}, section 3 expects '
                 f'{"rejected" if want[c] else "passes"}')
    shutil.rmtree(wsp)


def main():
    args = sys.argv[1:]
    mpath = os.path.join(DATA, 'manifest.json')
    if '--write-manifest' in args:
        need(not os.path.exists(mpath), f'{mpath} exists')
        files = {f'{c}_{r}.json.gz': sha(os.path.join(DATA, f'{c}_{r}.json.gz')) for c in CASES for r in RUNS}
        man = {'schema': 1, 'task': 'M2-06b-2', 'tool': 'tools/mip_node_key.js', 'tool_sha256': sha(TOOL),
               'binary_sha256': load(os.path.join(DATA, 'markshare_4_0_1.json.gz'))['binary_sha256'],
               'params': {'NodeLimit': 2000}, 'models': '../re/bench/data/miplib/<case>.mps.gz', 'files': files}
        with open(mpath, 'w') as fh:
            json.dump(man, fh, indent=1, sort_keys=True)
            fh.write('\n')
        print(f'wrote {mpath}')
    data = a1(json.load(open(mpath)))
    print(f'A1: {len(CASES)} cases x {len(RUNS)} runs byte-identical, manifest and tool hash match')
    oman = json.load(open(os.path.join(OLD, 'manifest.json')))
    olds = {}
    for c in OLD_CASES:
        p = os.path.join(OLD, f'{c}_1.json.gz')
        need(oman['files'].get(f'{c}_1.json.gz') == sha(p), f'A6: {p} sha256 != M2-06a manifest')
        olds[c] = load(p)
    inputs, tot = {}, {}
    for c in CASES:
        st, text, keys = check(c, data[c], olds.get(c))
        inputs[c] = (text, keys)
        print(f'A2-A6: {c}: ' + ', '.join(f'{k} {v}' for k, v in st.items() if v or k in ('deep', 'a4_lt1')))
        for k, v in st.items():
            if isinstance(v, int):
                tot[k] = tot.get(k, 0) + v
    print('A2-A6 total: ' + ', '.join(f'{k} {v}' for k, v in tot.items() if v or k in ('deep', 'a4_lt1')))
    res = run_rust(ROOT, os.path.join(ROOT, 'target'), inputs)
    for c in CASES:
        need(res[c] is None, res[c] or '')
    print(f'R: {sum(len(k) for _, k in inputs.values())} keys: rust = original on {len(CASES)} cases')
    if '--self-test' in args:
        tampers(data, olds)
        mutants(inputs)
    print('ACCEPTANCE PASS')


if __name__ == '__main__':
    try:
        main()
    except Fail as x:
        print('FAIL', x)
        sys.exit(1)
