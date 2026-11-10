#!/usr/bin/env python3
"""M2-06b: L1 replay of the Rust global node heap (crates/mip/src/node_heap.rs NodeHeap) on the M2-06a traces.
Contract: docs/m2/mip_node_heap_rust.md section 3.

(default)    M (run 1 traces = M2-06a manifest), X (replay input from trace events only), run
             `cargo test --release -p rsolver-mip --test node_heap_l1 -- --ignored`, then P/R/S/K against the trace
--self-test  also T (Rust output tampers) and V (code mutants on a patched copy of the workspace, per-case outcome fixed
             in section 3 before the first run)
Exit code 0 only when every requested check passes.
"""
import copy, gzip, hashlib, json, math, os, shutil, struct, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = os.path.join(ROOT, 'bench/oracle/m2_06a')
WORK = os.path.join(ROOT, 'target/mip_node_heap_rust')
SRC = 'crates/mip/src/node_heap.rs'
# case: (pops, inserts, pruned nodes, array snapshots), from docs/m2/mip_node_select_spec.md section 6.2
CASES = {'markshare_4_0': (216, 407, 25, 435), 'pk1': (44, 1521, 549, 91), 'neos5': (52, 1260, 447, 106),
         'tr12-30': (27, 1501, 608, 96)}
TIES = {'markshare_4_0': True, 'pk1': True, 'neos5': True, 'tr12-30': False}  # rejected where the case has ties
ALL = dict.fromkeys(CASES, True)
PRUNE_LOOP = '''        let all = std::mem::take(&mut self.h);
        for e in all {
            if cutoff <= e.1 {
                dropped(e.2);
            } else {
                self.h.push(e);
            }
        }'''
UNSTABLE = '''        let mut i = 0;
        while i < self.h.len() {
            if cutoff <= self.h[i].1 {
                dropped(self.h.swap_remove(i).2);
            } else {
                i += 1;
            }
        }'''
# (name, [(old, new)], expected rejection per case or None = report only); section 3 V
MUTANTS = [
    ('V1 pop tie to the left child', [('let j = if k2 <= k1 { c2 }', 'let j = if k2 < k1 { c2 }')], TIES),
    ('V2 insert moves up on equal keys', [('if self.h[p].0 <= key {', 'if self.h[p].0 < key {')], TIES),
    ('V3 pop moves down on equal keys', [('if x <= m {', 'if x < m {')], TIES),
    ('P2 unstable prune removal', [(PRUNE_LOOP, UNSTABLE)], ALL),
    ('C1 prune drops only cutoff < bound (synthetic cutoff)', [('if cutoff <= e.1 {', 'if cutoff < e.1 {')], ALL),
    ('H1 prune heapify from index 0 up', [('for i in (0..self.h.len()).rev() {', 'for i in 0..self.h.len() {')], None),
    ('O1 one child moves on equal keys', [('if k1 <= x && x != k1 {', 'if k1 <= x {')], None),
]


class Fail(Exception):
    pass


def need(cond, msg):
    if not cond:
        raise Fail(msg)


def sha(p):
    with open(p, 'rb') as f:
        return hashlib.sha256(f.read()).hexdigest()


def f(h):
    return struct.unpack('>d', bytes.fromhex(h))[0]


def load_inputs():
    man = json.load(open(os.path.join(DATA, 'manifest.json')))
    data = {}
    for c in CASES:
        p = os.path.join(DATA, f'{c}_1.json.gz')
        h = sha(p)
        need(man['files'].get(f'{c}_1.json.gz') == h, f'M: {p} sha256 != M2-06a manifest')
        data[c] = (h, json.loads(gzip.decompress(open(p, 'rb').read()))['events'])
    return data


def export(case, h, ev):
    """Replay input and the expected outputs, from trace events only (no heap model). Returns (text, exp, counts)."""
    ops, exp = [f'case {case} {h}'], []
    cnt = dict.fromkeys(('pop', 'ins', 'pruned', 'arrays'), 0)
    bound, cur, pr = {}, None, None
    for e in ev:  # pair each enter with its leave (the pop count M+0xd0 is read at leave)
        if e['e'] == 'enter':
            op = e
        elif e['e'] == 'leave':
            op['_leave'] = e

    def snap(tree, where, n, mn=None, heap=None):
        ops.append(f'snap {tree}')
        exp.append(('snap', where, (n, mn, heap)))
        cnt['arrays'] += heap is not None

    def close_prune(x):
        nonlocal pr
        if pr is None:
            return
        # the cutoff (xmm0) is not captured: the smallest dropped bound, +inf when nothing is dropped (section 1)
        cut = min((f(bound[i]) for i in pr['drop']), default=math.inf)
        ops.append(f'prune {pr["tree"]} {struct.pack(">d", cut).hex()}')
        exp.append(('prune', f'event {x} prune', pr['drop']))
        cnt['pruned'] += len(pr['drop'])
        pr = None

    def pops(x):
        cur['popdone'] = True
        en, lv = cur['enter'], cur['enter']['_leave']
        want = cur['defer'] + ([lv['node'][0]] if lv['node'] is not None and lv['st']['pops'] > en['st']['pops'] else [])
        need(len(want) == lv['st']['pops'] - en['st']['pops'], f'X: event {x} sel {en["sel"]}: pops vs selected/deferred')
        for i in want:
            ops.append(f'pop {en["st"]["tree"]}')
            exp.append(('pop', f'event {x} sel {en["sel"]}', i))
            cnt['pop'] += 1

    for x, e in enumerate(ev):
        t = e['e']
        if t != 'acct':
            close_prune(x)
        if t == 'tnew':
            ops.append(f'tree {e["tree"]}')
        elif t == 'tfree':
            snap(e['tree'], f'event {x} tfree', e['heap_n'])
            ops.append(f'free {e["tree"]}')
        elif t == 'enter':
            need(cur is None, f'X: event {x}: enter inside a selection')
            st = e['st']
            snap(st['tree'], f'event {x} sel {e["sel"]} enter', st['heap_n'], st['heap_min'], st.get('heap'))
            cur = {'enter': e, 'defer': [], 'popdone': False}
        elif t == 'defer':
            raise Fail(f'X: event {x}: deferred add (out of scope, section 1)')
        elif t == 'ins':
            nd = e['node']
            need(nd[6] != 10, f'X: event {x}: status 10 insert (out of scope, section 1)')
            need(not (math.isnan(f(nd[1])) or math.isnan(f(nd[2]))), f'X: event {x}: NaN key or bound')
            if cur is not None and e['in_sel'] and not cur['popdone']:
                pops(x)
            ops.append(f'ins {e["tree"]} {nd[0]} {nd[2]} {nd[1]}')
            bound[nd[0]] = nd[1]
            cnt['ins'] += 1
        elif t == 'prune':
            pr = {'tree': e['tree'], 'drop': []}
        elif t == 'acct':
            if e['from'] == '6e460c':
                need(pr is not None and pr['tree'] == e['tree'], f'X: event {x}: 6e460c removal outside a prune')
                pr['drop'].append(e['node'])
            else:
                need(e['from'] == '6a6563', f'X: event {x}: unmodeled removal by {e["from"]}')
        elif t == 'leave':
            need(cur is not None and cur['enter']['sel'] == e['sel'], f'X: event {x}: leave without enter')
            if not cur['popdone']:
                pops(x)
            st = e['st']
            snap(st['tree'], f'event {x} sel {e["sel"]} leave', st['heap_n'], st['heap_min'], st.get('heap'))
            cur = None
        elif t == 'pruned':
            snap(e['tree'], f'event {x} pruned', len(e['heap']), e['heap_min'], e['heap'])
        elif t == 'flush':
            need(e['n'] == 0, f'X: event {x}: deferred list flush with entries (out of scope)')
        else:
            raise Fail(f'X: event {x}: unknown event {t}')
    close_prune(len(ev))
    need(cur is None, 'X: trace ends inside a selection')
    ops.append('end')
    return '\n'.join(ops) + '\n', exp, cnt


def compare(case, head, text, exp):
    """P/R/S: first difference with case, event, field, index, original and Rust values."""
    lines = text.rstrip('\n').split('\n')
    need(lines[0] == head, f'{case}: Rust output head {lines[0]!r} vs input {head!r}')
    out = lines[1:]
    for k, (kind, where, want) in enumerate(exp):
        need(k < len(out), f'{case}: Rust output ends after {k} of {len(exp)} lines')
        got = out[k].split()
        need(got[0] == kind, f'{case} {where}: Rust line {k} is {got[0]!r}, expected {kind}')
        if kind == 'pop':
            need(got[1] == str(want), f'P {case} {where}: popped {got[1]}, original {want}')
        elif kind == 'prune':
            need(got[1:] == [str(i) for i in want],
                 f'R {case} {where}: dropped {got[1:][:8]}..., original {want[:8]}... (first diff '
                 f'{next((i for i, (a, b) in enumerate(zip(got[1:], map(str, want))) if a != b), min(len(got) - 1, len(want)))})')
        else:
            n, mn, heap = want
            need(int(got[1]) == n, f'S {case} {where}: heap size {got[1]}, original {n}')
            if mn is not None:
                need(got[2] == mn, f'S {case} {where}: T+0x170 {got[2]}, original {mn}')
            if heap is not None:
                arr = [g.split(':') for g in got[3:]]
                for i, (a, b) in enumerate(zip(arr, heap)):
                    need(a == [str(b[0]), b[1], b[2]], f'S {case} {where}: heap[{i}] Rust {a}, original {b}')
                need(len(arr) == len(heap), f'S {case} {where}: heap length {len(arr)} vs {len(heap)}')
    need(len(out) == len(exp), f'{case}: Rust output has {len(out)} lines, expected {len(exp)}')


def cargo(ws, target, din, dout):
    env = dict(os.environ, NODE_HEAP_IN=din, NODE_HEAP_OUT=dout, CARGO_TARGET_DIR=target,
               PATH=os.path.expanduser('~/.cargo/bin') + os.pathsep + os.environ.get('PATH', ''))
    shutil.rmtree(dout, ignore_errors=True)  # no stale trace can pass
    r = subprocess.run(['cargo', 'test', '--release', '-p', 'rsolver-mip', '--test', 'node_heap_l1', '--', '--ignored'],
                       cwd=ws, env=env, capture_output=True, text=True)
    need(r.returncode == 0 and '1 passed' in r.stdout, f'cargo test failed ({r.returncode}):\n{r.stdout[-2000:]}{r.stderr[-2000:]}')


def traces(dout):
    return {c: open(os.path.join(dout, f'{c}.trace.txt')).read() for c in CASES}


def outcomes(heads, exps, tr):
    """Per case: None when equal, else the first difference."""
    res = {}
    for c in CASES:
        try:
            compare(c, heads[c], tr[c], exps[c])
            res[c] = None
        except Fail as x:
            res[c] = str(x)
    return res


def tampers(heads, exps, tr):
    def edit(case, fn):
        t = copy.deepcopy(tr)
        lines = t[case].split('\n')
        fn(lines)
        t[case] = '\n'.join(lines)
        return t

    def find(lines, ok):
        return next(i for i, ln in enumerate(lines) if ok(ln.split()))

    def swap_entries(lines):
        # a snapshot whose original array is captured (selections with pops, prune returns), two different entries
        i = next(k + 1 for k, (kind, _, w) in enumerate(exps['markshare_4_0']) if kind == 'snap' and w[2] is not None
                 and len(w[2]) >= 2 and w[2][0][1:] != w[2][1][1:])
        p = lines[i].split()
        p[3], p[4] = p[4], p[3]
        lines[i] = ' '.join(p)

    def pop_id(lines):
        i = find(lines, lambda p: p[:1] == ['pop'])
        lines[i] = 'pop 999999999'

    def prune_order(lines):
        i = find(lines, lambda p: p[:1] == ['prune'] and len(p) >= 3)
        p = lines[i].split()
        p[1], p[2] = p[2], p[1]
        lines[i] = ' '.join(p)

    def drop_line(lines):
        lines.pop(find(lines, lambda p: p[:1] == ['pop']))

    def min_bits(lines):
        i = find(lines, lambda p: p[:1] == ['snap'] and int(p[1]) > 0)
        p = lines[i].split()
        p[2] = p[2][:-1] + ('0' if p[2][-1] != '0' else '1')
        lines[i] = ' '.join(p)

    for name, case, fn in (('heap entries swapped', 'markshare_4_0', swap_entries), ('pop id', 'pk1', pop_id),
                           ('prune order', 'tr12-30', prune_order), ('line deleted', 'neos5', drop_line),
                           ('T+0x170 bits', 'pk1', min_bits)):
        why = outcomes(heads, exps, edit(case, fn))[case]
        need(why is not None, f'T: tamper not rejected: {name}')
        print(f'T: rejected: {name}: {why}')


def mutants(heads, exps, din):
    base = os.path.join(WORK, 'mutant')
    wsp = os.path.join(base, 'ws')
    shutil.rmtree(wsp, ignore_errors=True)
    os.makedirs(wsp)
    for p in ('Cargo.toml', 'Cargo.lock'):
        shutil.copy(os.path.join(ROOT, p), wsp)
    shutil.copytree(os.path.join(ROOT, 'crates'), os.path.join(wsp, 'crates'))
    orig = open(os.path.join(ROOT, SRC)).read()
    for name, subs, want in MUTANTS:
        s = orig
        for a, b in subs:
            need(s.count(a) == 1, f'V: mutant {name}: pattern {a!r} found {s.count(a)} times')
            s = s.replace(a, b)
        with open(os.path.join(wsp, SRC), 'w') as fh:
            fh.write(s)
        dout = os.path.join(base, 'out')
        cargo(wsp, os.path.join(base, 'target'), din, dout)
        res = outcomes(heads, exps, traces(dout))
        for c in CASES:
            tag = 'rejected' if res[c] else 'passes'
            print(f'V: {name}: {c}: {tag}' + (f': {res[c][:200]}' if res[c] else ''))
            if want is not None:
                need(bool(res[c]) == want[c], f'V: mutant {name} on {c}: {tag}, section 3 expects '
                     f'{"rejected" if want[c] else "passes"}')
    shutil.rmtree(wsp)


def main():
    self_test = '--self-test' in sys.argv[1:]
    data = load_inputs()
    print(f'M: run 1 traces match the M2-06a manifest ({len(data)} cases)')
    din, dout = os.path.join(WORK, 'in'), os.path.join(WORK, 'out')
    shutil.rmtree(din, ignore_errors=True)
    os.makedirs(din)
    heads, exps = {}, {}
    for c, (h, ev) in data.items():
        text, exps[c], cnt = export(c, h, ev)
        heads[c] = text.split('\n', 1)[0]
        got = (cnt['pop'], cnt['ins'], cnt['pruned'], cnt['arrays'])
        need(got == CASES[c], f'K: {c}: (pops, inserts, pruned, array snapshots) {got} vs M2-06a {CASES[c]}')
        with open(os.path.join(din, f'{c}.input.txt'), 'w') as fh:
            fh.write(text)
    print(f'X: replay inputs in {os.path.relpath(din, ROOT)}')
    cargo(ROOT, os.path.join(ROOT, 'target'), din, dout)
    tr = traces(dout)
    res = outcomes(heads, exps, tr)
    for c in CASES:
        need(res[c] is None, res[c] or '')
        p, i, r, a = CASES[c]
        print(f'P/R/S/K: {c}: pops {p}, inserts {i}, pruned {r}, array snapshots {a}, '
              f'{sum(1 for e in exps[c] if e[0] == "snap")} snapshots: rust = original')
    tot = [sum(v[k] for v in CASES.values()) for k in range(4)]
    print(f'K: pops {tot[0]}, inserts {tot[1]}, pruned {tot[2]}, array snapshots {tot[3]}')
    if self_test:
        tampers(heads, exps, tr)
        mutants(heads, exps, din)
    print('PASS')


if __name__ == '__main__':
    try:
        main()
    except Fail as x:
        print('FAIL', x)
        sys.exit(1)
