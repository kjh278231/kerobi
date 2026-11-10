#!/usr/bin/env python3
"""M2-03d-10: L1 replay of the Rust deactivated dive row list and D swap (crates/mip/src/dive_rows.rs RowList) on the
M2-03d-7/03d-8 captures. Contract: docs/m2/mip_dive_list_rust.md section 3.

(default)    M (captures = M2-03d-7/03d-8 manifests), X (replay inputs from operation fields only), run
             `cargo test --release -p rsolver-mip --test dive_list_l1 -- --ignored`, then T/P/K/S against the captures
--self-test  also trace tampers and code mutants (patched copy of the workspace): discriminated ones must be rejected,
             the ones the captures cannot discriminate (section 1) must pass
Exit code 0 only when every requested check passes.
"""
import copy, gzip, hashlib, json, os, shutil, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MAN8 = 'bench/oracle/mip_dive_rows_manifest.json'
MAN7 = 'bench/oracle/mip_blank_rows_manifest.json'
CAP7 = 'bench/oracle/mip_blank_rows/nidda_bnb_H_s1_1.json'               # M2-03d-7: ops and the target swap
API_LP = 'bench/oracle/mip_retry_node_lp/nidda_bnb_H_s1.node_lp.json'   # M2-03d-6: API LP at the target cda5b entry
CAP6 = 'bench/oracle/mip_retry_node_lp/nidda_bnb_H_s1_1.json'           # M2-03d-6: y:ws (reduced simplex ws) at k=2
WORK = os.path.join(ROOT, 'target/mip_dive_list_rust')
# case: capture, (lp 1 updates, lp 2 updates, non-empty -> non-empty, of which a row left)
CASES = {'nidda_H_s1': ('bench/oracle/mip_dive_rows/nidda_H_s1_1.json.gz', (20, 2377, 36, 0)),
         'gmu-35-40_default': ('bench/oracle/mip_dive_rows/gmu-35-40_default_1.json.gz', (32, 283, 46, 0)),
         'assign1-5-8_default': ('bench/oracle/mip_dive_rows/assign1-5-8_default_1.json.gz', (17, 200, 30, 0)),
         'nidda_bnb_H_s1': (CAP7, (20, 2377, 36, 0))}
SWAP_CASE, SWAP_NNZ = 'nidda_bnb_H_s1', (1840, 4955)
# the M2-03d-6 API LP and y:ws belong to this target (M2-03d-7 check L)
TARGET = {'call': 19901, 'node': 16948, 'dhash': '4051b704-7e653419'}
SRC = 'crates/mip/src/dive_rows.rs'
LISTED = 'let listed = |k: i64, ind: &[i32]| x.pos[ind[k as usize] as usize] >= 0;'
REMOVE = '''let (p, last) = (x.pos[i] as usize, *x.list.last().unwrap());
                x.list[p] = last;
                x.pos[last as usize] = p as i32;
                x.list.pop();'''
# (name, [(old, new)], rejected?): one clause broken at a time; False = not discriminated by the captures (section 1)
MUTANTS = [
    ('rows in decreasing order', [('for (i, &f) in flags.iter().enumerate() {', 'for (i, &f) in flags.iter().enumerate().rev() {')], True),
    ('react_all ignored', [('self.mark(nrows, &list, REACT)', 'let _ = list;\n        Ok(())')], True),
    ('rhs of listed rows kept', [('x.rhs[r as usize] = 0.0;', 'let _ = r;')], True),
    ('reduced length = full length', [('x.len[j] = (a - s as i64) as i32;', 'x.len[j] = len[j];')], True),
    ('already listed row appended again', [('if x.pos[i] < 0 {', 'if f & DEACT != 0 {')], False),
    ('order-preserving removal', [(REMOVE, '''let p = x.pos[i] as usize;
                x.list.remove(p);
                for q in p..x.list.len() {
                    x.pos[x.list[q] as usize] = q as i32;
                }''')], False),
    ('stable partition', [(LISTED, LISTED + '''
                {
                    let (f, g): (Vec<_>, Vec<_>) = (s..s + len[j] as usize).map(|k| (ind[k], val[k])).partition(|e| x.pos[e.0 as usize] < 0);
                    for (k, e) in f.iter().chain(&g).enumerate() {
                        (ind[s + k], val[s + k]) = *e;
                    }
                    a = (s + f.len()) as i64;
                    b = a - 1;
                }''')], False),
]


class Fail(Exception):
    pass


def need(cond, msg):
    if not cond:
        raise Fail(msg)


def sha(p):
    with open(os.path.join(ROOT, p), 'rb') as f:
        return hashlib.sha256(f.read()).hexdigest()


def load(p):
    op = gzip.open if p.endswith('.gz') else open
    with op(os.path.join(ROOT, p), 'rt') as f:
        return json.load(f)


def load_inputs():
    m8, m7 = load(MAN8)['captures'], load(MAN7)['files']
    for p in [c for c, _ in CASES.values() if c != CAP7]:
        need(m8.get(p) == sha(p), f'M: {p} sha256 != M2-03d-8 manifest')
    for p in (CAP7, API_LP, CAP6):
        need(m7.get(p) == sha(p), f'M: {p} sha256 != M2-03d-7 manifest')
    data = {c: (sha(p), load(p)['events']) for c, (p, _) in CASES.items()}
    ws = [e for e in load(CAP6)['events'] if e['event'] == 'retry_solve'][0]['events']
    return data, load(API_LP), [e for e in ws if e['event'] == 'y:ws' and e['k'] == 2][0]


def export(case, h, events, api):
    """Replay input from operation fields only: no b:xlist rows/count, no x_rows, no y:ws."""
    out, rows, tlp = [f'case {case} {h}'], {}, None
    for e in events:
        ev = e['event']
        need(ev != 'b:op_fail', f'X {case}: failed op {e}')
        if ev == 'b:lpnew':
            out.append(f'lp {e["lp"]}')
            rows[e['lp']] = e['D_rows']
        elif ev in ('b:op', 'b:xlist'):
            lp, n = e['lp'], e['D_rows']
            if n != rows[lp]:
                out.append(f'rows {lp} {n}')
                rows[lp] = n
            if ev == 'b:xlist':
                out.append(f'update {lp} {e["call"]}')
            elif e['op'] == 'deact':
                out.append(f'deact {lp} {n} ' + ' '.join(map(str, e['rows'])))
            else:
                need(e['op'] == 'react_all', f'X {case}: op {e["op"]} is not ported')
                out.append(f'react_all {lp} {n}')
        elif ev == 'b:t:cda5b':
            tlp = e['lp']
            if e['k'] == 1:
                out += [f'dlp', ' '.join(map(str, api['cbeg'])), ' '.join(map(str, api['ind'])), ' '.join(api['val']), ' '.join(api['rhs'])]
        elif ev in ('b:t:swap_in', 'b:t:swap_out'):
            out.append(f'{ev[4:]} {tlp} {e["k"]}')
    return '\n'.join(out + ['end']) + '\n'


def parse_trace(text):
    lines = text.rstrip('\n').split('\n')
    head = lines[0].split()
    need(len(head) == 3 and head[0] == 'case', f'T: bad trace head {lines[0][:80]!r}')
    lists, swaps = [], []
    for ln in lines[1:]:
        p = [x.split() for x in ln.split('|')]
        if p[0][0] == 'list':
            lists.append((int(p[0][1]), int(p[0][2]), list(map(int, p[1]))))
        else:
            swaps.append((p[0][0], int(p[0][2]), list(map(int, p[0][3:])), p[1:]))
    return head[1], head[2], lists, swaps


def compare(case, h, text, events, api, ws):
    name, th, lists, swaps = parse_trace(text)
    need((name, th) == (case, h), f'T {case}: trace is for {name} {th}, capture {h}')
    caps = [e for e in events if e['event'] == 'b:xlist']
    need(len(lists) == len(caps), f'K {case}: {len(lists)} list updates in the trace, capture {len(caps)}')
    prev, kept, left = {}, 0, 0
    for (lp, call, rows), e in zip(lists, caps):
        need((lp, call) == (e['lp'], e['call']), f'T {case}: trace update lp {lp} call {call} != capture lp {e["lp"]} call {e["call"]}')
        need(rows == e['rows'] and e['count'] == len(rows), f'P {case}: lp {lp} call {call} node {e["node"]}: rust {len(rows)} rows != '
             f'capture {e["count"]}; first diff at {next((k for k, (a, b) in enumerate(zip(rows, e["rows"])) if a != b), min(len(rows), len(e["rows"])))}: '
             f'rust {rows[:8]} capture {e["rows"][:8]}')
        if prev.get(lp) and rows:
            kept += 1
            left += not set(prev[lp]) <= set(rows)
        prev[lp] = rows
    got = (sum(lp == 1 for lp, _, _ in lists), sum(lp == 2 for lp, _, _ in lists), kept, left)
    need(got == CASES[case][1], f'K {case}: counts {got} != {CASES[case][1]}')
    if any(e['event'] == 'b:t:cda5b' for e in events):   # both nidda captures carry the M2-03d-7 target events
        check_swap(swaps, events, api, ws)
    else:
        need(not swaps and case != SWAP_CASE, f'S {case}: swaps {len(swaps)} without target events')
    return got


def check_swap(swaps, events, api, ws):
    ent = {e['k']: e for e in events if e['event'] == 'b:t:cda5b'}
    sin = {e['k']: e for e in events if e['event'] == 'b:t:swap_in'}
    sout = {e['k']: e for e in events if e['event'] == 'b:t:swap_out'}
    for k, e in ent.items():
        need((e['node'], e['dhash']['hash'], e['is_node_lp']) == (TARGET['node'], TARGET['dhash'], True), f'S k={k}: target is not the M2-03d-6 node LP')
    need([(s[0], s[1]) for s in swaps] == [('swap_in', 1), ('swap_out', 1), ('swap_in', 2), ('swap_out', 2)], f'S: swap sequence {[(s[0], s[1]) for s in swaps]}')
    api_len = [b - a for a, b in zip(api['cbeg'], api['cbeg'][1:])]
    for kind, k, nums, p in swaps:
        if kind == 'swap_in':
            built, refs = nums
            lst, ln, ind, val, rhs = list(map(int, p[0])), list(map(int, p[1])), list(map(int, p[2])), p[3], p[4]
            need(lst == ent[k]['x_rows'] and len(lst) == ent[k]['x_count'], f'S k={k}: rust list at the swap != capture x_rows')
            need(refs == sin[k]['ref_after'] == 1 and built == (k == 2), f'S k={k}: refs {refs} built before {built}')
            cbeg = [0]
            for x in ln:
                cbeg.append(cbeg[-1] + x)
            need(cbeg[-1] == sin[k]['D_nnz'] == SWAP_NNZ[0], f'S k={k}: reduced nnz {cbeg[-1]}')
            need(cbeg == ws['cbeg'], f'S k={k}: reduced column starts != y:ws, first diff col {next((j for j, (a, b) in enumerate(zip(cbeg, ws["cbeg"])) if a != b), len(cbeg))}')
            need(ind == ws['ind'], f'S k={k}: reduced ind != y:ws, first diff at {next((j for j, (a, b) in enumerate(zip(ind, ws["ind"])) if a != b), len(ind))}')
            need(val == ws['val'], f'S k={k}: reduced val != y:ws')
            need(rhs == ws['b'], f'S k={k}: rhs != y:ws b, first diff row {next((i for i, (a, b) in enumerate(zip(rhs, ws["b"])) if a != b), len(rhs))}')
            diff = [i for i, (a, b) in enumerate(zip(rhs, api['rhs'])) if a != b]
            need(diff == sin[k]['rhs_diff'] and [rhs[i] for i in diff] == sin[k]['rhs_now'], f'S k={k}: changed rhs rows != capture rhs_diff')
        else:
            refs, = nums
            ln, rhs = list(map(int, p[0])), p[1]
            need(refs == sout[k]['ref_after'] == 0, f'S k={k}: refs after swap_out {refs}')
            need(ln == api_len and sum(ln) == sout[k]['D_nnz'] == SWAP_NNZ[1], f'S k={k}: restored lengths != API')
            need(rhs == api['rhs'], f'S k={k}: restored rhs != API')


def cargo(ws, target, din, dout):
    env = dict(os.environ, DIVE_LIST_IN=din, DIVE_LIST_OUT=dout, CARGO_TARGET_DIR=target,
               PATH=os.path.expanduser('~/.cargo/bin') + os.pathsep + os.environ.get('PATH', ''))
    shutil.rmtree(dout, ignore_errors=True)       # no stale trace can pass
    r = subprocess.run(['cargo', 'test', '--release', '-p', 'rsolver-mip', '--test', 'dive_list_l1', '--', '--ignored'],
                       cwd=ws, env=env, capture_output=True, text=True)
    need(r.returncode == 0 and '1 passed' in r.stdout, f'cargo test failed ({r.returncode}):\n{r.stdout[-2000:]}{r.stderr[-2000:]}')


def traces(dout):
    out = {}
    for c in CASES:
        with open(os.path.join(dout, f'{c}.trace.txt')) as f:
            out[c] = f.read()
    return out


def compare_all(data, api, ws, tr):
    return [(c, compare(c, h, tr[c], events, api, ws)) for c, (h, events) in data.items()]


def tampers(data, api, ws, tr):
    def edit(case, f):
        t = copy.deepcopy(tr)
        lines = t[case].split('\n')
        f(lines)
        t[case] = '\n'.join(lines)
        return t

    def first(lines, cond):
        return next(k for k in range(1, len(lines)) if lines[k] and cond(lines[k].split('|')))

    def drop_row(lines):
        k = first(lines, lambda p: p[0].startswith('list') and p[1].split())
        p = lines[k].split('|')
        lines[k] = p[0] + '| ' + ' '.join(p[1].split()[1:])

    def swap(lines):
        k = first(lines, lambda p: p[0].startswith('list') and len(p[1].split()) > 1)
        p = lines[k].split('|')
        r = p[1].split()
        lines[k] = p[0] + '| ' + ' '.join([r[1], r[0]] + r[2:])

    def del_update(lines):
        del lines[first(lines, lambda p: p[0].startswith('list') and p[1].split())]

    def head(lines):
        lines[0] = lines[0][:-1] + ('0' if lines[0][-1] != '0' else '1')

    def ind(lines):
        k = first(lines, lambda p: p[0].startswith('swap_in'))
        p = lines[k].split('|')
        v = p[3].split()
        v[0] = str(int(v[0]) + 1)
        p[3] = ' ' + ' '.join(v) + ' '
        lines[k] = '|'.join(p)

    cases = [('list row dropped', edit('gmu-35-40_default', drop_row)), ('list order', edit('assign1-5-8_default', swap)),
             ('update deleted', edit('nidda_H_s1', del_update)), ('capture sha256', edit('gmu-35-40_default', head)),
             ('swapped ind', edit(SWAP_CASE, ind))]
    for name, t in cases:
        try:
            compare_all(data, api, ws, t)
        except Fail as x:
            print(f'rejected: {name}: {x}')
            continue
        raise Fail(f'tamper not rejected: {name}')


def mutants(data, api, ws, din):
    base = os.path.join(WORK, 'mutant')
    wsp = os.path.join(base, 'ws')
    shutil.rmtree(wsp, ignore_errors=True)
    os.makedirs(wsp)
    for p in ('Cargo.toml', 'Cargo.lock'):
        shutil.copy(os.path.join(ROOT, p), wsp)
    shutil.copytree(os.path.join(ROOT, 'crates'), os.path.join(wsp, 'crates'))
    orig = open(os.path.join(ROOT, SRC)).read()
    for name, subs, discriminated in MUTANTS:
        s = orig
        for a, b in subs:
            need(s.count(a) == 1, f'mutant {name}: pattern {a!r} found {s.count(a)} times')
            s = s.replace(a, b)
        with open(os.path.join(wsp, SRC), 'w') as f:
            f.write(s)
        dout = os.path.join(base, 'out')
        why = None
        try:
            cargo(wsp, os.path.join(base, 'target'), din, dout)
        except Fail as x:
            # a replay assertion (the row deletion guard) rejects the mutant; a build error fails the self-test
            t = str(x).split('\n')
            k = next((i for i, ln in enumerate(t) if 'panicked at crates/mip/src/dive_rows.rs' in ln), None)
            need(k is not None, f'mutant {name}: {x}')
            why = 'replay panicked: ' + ' '.join(t[k:k + 2]).strip()
        if why is None:
            try:
                compare_all(data, api, ws, traces(dout))
            except Fail as x:
                why = str(x)
        if why is not None:
            need(discriminated, f'mutant {name} was rejected but section 1 lists it as not discriminated: {why}')
            print(f'mutant rejected: {name}: {why}')
            continue
        need(not discriminated, f'mutant not rejected: {name}')
        print(f'mutant passes (not discriminated by the captures, section 1): {name}')
    shutil.rmtree(wsp)


def main():
    self_test = '--self-test' in sys.argv[1:]
    data, api, ws = load_inputs()
    print(f'M: captures match the M2-03d-7/03d-8 manifests ({len(data)} cases, API LP, y:ws)')
    din, dout = os.path.join(WORK, 'in'), os.path.join(WORK, 'out')
    shutil.rmtree(din, ignore_errors=True)
    os.makedirs(din)
    for c, (h, events) in data.items():
        with open(os.path.join(din, f'{c}.input.txt'), 'w') as f:
            f.write(export(c, h, events, api))
    print(f'X: replay inputs in {os.path.relpath(din, ROOT)}')
    cargo(ROOT, os.path.join(ROOT, 'target'), din, dout)
    tr = traces(dout)
    for c, got in compare_all(data, api, ws, tr):
        print(f'P/K: {c} updates lp1 {got[0]} lp2 {got[1]}, non-empty -> non-empty {got[2]} (row left {got[3]})')
    print(f'S: nidda target (call {TARGET["call"]} node {TARGET["node"]}) k=1,2 in both nidda captures: rust list = x_rows, reduced D = y:ws (cbeg/ind/val/b bits), restore = API, refs 0-1-0')
    if self_test:
        tampers(data, api, ws, tr)
        mutants(data, api, ws, din)
    print('PASS')


if __name__ == '__main__':
    try:
        main()
    except Fail as x:
        print('FAIL', x)
        sys.exit(1)
