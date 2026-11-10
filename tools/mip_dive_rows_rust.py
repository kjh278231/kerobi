#!/usr/bin/env python3
"""M2-03d-9: L1 replay of the Rust dive row selection (crates/mip/src/dive_rows.rs) on the M2-03d-8 captures.
Contract: docs/m2/mip_dive_rows_rust.md section 3.

(default)    M (run 1 captures = M2-03d-8 manifest), X (write replay inputs from capture input fields only), run
             `cargo test --release -p rsolver-mip --test dive_rows_l1 -- --ignored`, then T/R/A/K against the capture
--self-test  also trace tampers and code mutants (patched copy of the workspace) must be rejected
Exit code 0 only when every requested check passes.
"""
import copy, gzip, hashlib, json, os, shutil, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = 'bench/oracle/mip_dive_rows'
MANIFEST = 'bench/oracle/mip_dive_rows_manifest.json'
WORK = os.path.join(ROOT, 'target/mip_dive_rows_rust')
# case: (selections, rows selected, 66661c calls, calls returning 1), M2-03d-8 section 6.2
CASES = {'nidda_H_s1': (1320, 157545, 0, 0), 'gmu-35-40_default': (689, 9248, 19434, 328),
         'assign1-5-8_default': (84, 5342, 6844, 0)}
SRC = 'crates/mip/src/dive_rows.rs'
# (name, [(old, new)]): one discriminated clause broken at a time (M2-03d-8 V)
MUTANTS = [
    ('0.1*FeasTol -> 0', [('if !(s.feastol * 0.1 + rhs >= maxact)', 'if !(0.0 + rhs >= maxact)')]),
    ('slack 1e-5 -> 0', [('1e-05 < s.slack[k]', '0.0 < s.slack[k]')]),
    ('66661c no integral rounding', [('if integral { (rhs + 9.999999999999999e-05).floor() + 1.0 } else { rhs }', 'rhs')]),
    ('DualReductions ignored', [('s.dual_reductions == 0 || ', '')]),
    ('cuts before model rows', [('let mut out = Vec::new();', 'let mut out = Vec::new();\n    let mut first = 0usize;'),
                                ('out.push(k as i32);', 'out.insert(first, k as i32);\n            first += 1;')]),
]


class Fail(Exception):
    pass


def need(cond, msg):
    if not cond:
        raise Fail(msg)


def sha(p):
    with open(os.path.join(ROOT, p), 'rb') as f:
        return hashlib.sha256(f.read()).hexdigest()


def cap(case):
    return f'{DATA}/{case}_1.json.gz'


def split16(s):
    return ' '.join(s[k:k + 16] for k in range(0, len(s), 16))


def flat(pairs):
    return ' '.join(f'{a} {b}' for a, b in pairs)


def export(case, h, events):
    """Replay input from the capture's input fields only (no list, calls_66661c, count, no Python rule)."""
    out = [f'case {case} {h}']
    for e in events:
        if e['event'] == 'd:model':
            out += [f'model {e["model"]} {e["nrows"]} {e["ncols"]}', e['sense'], e['vtype'], ' '.join(map(str, e['row_len'])),
                    ' '.join(map(str, e['ind'])), split16(e['val']), split16(e['rhs'])]
        elif e['event'] == 'd:sel':
            need(e['rc'] is not None, f'X {case}: sel call {e["call"]} without rc')
            b7 = e['b7889a5']
            out += [f'sel {e["call"]} {e["model"]} {e["nrows"]} {e["ncuts"]} {e["dual"]} {e["feastol"]} {e["cutoff_raw"] or "-"} {e["i10"]}',
                    flat(e['lb_diff']), flat(e['ub_diff']), flat(e['flags']),
                    '-' if b7 is None else f'{b7["count"]} {flat(b7["nonneg"])}'.strip(),
                    ' '.join(map(str, e['cut_type'])), ' '.join(map(str, e['cut_40'])), split16(e['cut_slack']), split16(e['rc'])]
    return '\n'.join(out + ['end']) + '\n'


def parse_trace(text):
    lines = text.rstrip('\n').split('\n')
    head = lines[0].split()
    need(len(head) == 3 and head[0] == 'case', f'T: bad trace head {lines[0][:80]!r}')
    sel = []
    for ln in lines[1:]:
        t, rows, calls = ln.split('|')
        c = list(map(int, calls.split()))
        sel.append((int(t.split()[1]), list(map(int, rows.split())), [[c[k], c[k + 1]] for k in range(0, len(c), 2)]))
    return head[1], head[2], sel


def compare(case, h, text, caps):
    name, th, sel = parse_trace(text)
    need((name, th) == (case, h), f'T {case}: trace is for {name} {th}, capture {h}')
    need(len(sel) == len(caps), f'K {case}: {len(sel)} selections in the trace, capture {len(caps)}')
    for (call, rows, calls), e in zip(sel, caps):
        need(call == e['call'], f'T {case}: trace call {call} != capture call {e["call"]}')
        need(calls == e['calls_66661c'], f'A {case}: sel call {call} node {e["node"]}: 66661c calls differ, first '
             f'{next(((a, b) for a, b in zip(calls, e["calls_66661c"]) if a != b), (len(calls), len(e["calls_66661c"])))} (rust, capture)')
        need(rows == e['list'], f'R {case}: sel call {call} node {e["node"]}: rust {len(rows)} rows != capture {len(e["list"])}; '
             f'first diff at {next((k for k, (a, b) in enumerate(zip(rows, e["list"])) if a != b), min(len(rows), len(e["list"])))}: '
             f'rust {rows[:8]} capture {e["list"][:8]}')
    got = (len(sel), sum(len(r) for _, r, _ in sel), sum(len(c) for _, _, c in sel), sum(v for _, _, c in sel for _, v in c))
    need(got == CASES[case], f'K {case}: counts {got} != {CASES[case]}')
    return got


def load_inputs():
    man = json.load(open(os.path.join(ROOT, MANIFEST)))
    data = {}
    for c in CASES:
        h = sha(cap(c))
        need(man['captures'].get(cap(c)) == h, f'M {c}: {cap(c)} sha256 {h} != M2-03d-8 manifest')
        with gzip.open(os.path.join(ROOT, cap(c)), 'rt') as f:
            data[c] = (h, json.load(f)['events'])
    return data


def cargo(ws, target, din, dout):
    env = dict(os.environ, DIVE_ROWS_IN=din, DIVE_ROWS_OUT=dout, CARGO_TARGET_DIR=target,
               PATH=os.path.expanduser('~/.cargo/bin') + os.pathsep + os.environ.get('PATH', ''))
    shutil.rmtree(dout, ignore_errors=True)       # no stale trace can pass
    r = subprocess.run(['cargo', 'test', '--release', '-p', 'rsolver-mip', '--test', 'dive_rows_l1', '--', '--ignored'],
                       cwd=ws, env=env, capture_output=True, text=True)
    need(r.returncode == 0 and '1 passed' in r.stdout, f'cargo test failed ({r.returncode}):\n{r.stdout[-2000:]}{r.stderr[-2000:]}')


def traces(dout):
    out = {}
    for c in CASES:
        with open(os.path.join(dout, f'{c}.trace.txt')) as f:
            out[c] = f.read()
    return out


def compare_all(data, tr):
    lines = []
    for c, (h, events) in data.items():
        got = compare(c, h, tr[c], [e for e in events if e['event'] == 'd:sel'])
        lines.append(f'{c} {got[0]} sel {got[1]} rows {got[2]} calls ({got[3]} =1)')
    return lines


def tampers(data, tr):
    def edit(case, f):
        t = copy.deepcopy(tr)
        lines = t[case].split('\n')
        f(lines)
        t[case] = '\n'.join(lines)
        return t

    def first(lines, cond):
        return next(k for k in range(1, len(lines)) if lines[k] and cond(lines[k].split('|')))

    def drop_row(lines):
        k = first(lines, lambda p: p[1].split())
        p = lines[k].split('|')
        lines[k] = '|'.join([p[0], ' ' + ' '.join(p[1].split()[1:]) + ' ', p[2]])

    def swap(lines):
        k = first(lines, lambda p: len(p[1].split()) > 1)
        p = lines[k].split('|')
        r = p[1].split()
        lines[k] = '|'.join([p[0], ' ' + ' '.join([r[1], r[0]] + r[2:]) + ' ', p[2]])

    def flip(lines):
        k = first(lines, lambda p: p[2].split())
        p = lines[k].split('|')
        c = p[2].split()
        c[1] = str(1 - int(c[1]))
        lines[k] = '|'.join([p[0], p[1], ' ' + ' '.join(c)])

    def del_sel(lines):
        del lines[len(lines) // 2]

    def head(lines):
        lines[0] = lines[0][:-1] + ('0' if lines[0][-1] != '0' else '1')

    cases = [('list row dropped', edit('nidda_H_s1', drop_row)), ('list order', edit('assign1-5-8_default', swap)),
             ('66661c result', edit('gmu-35-40_default', flip)), ('selection deleted', edit('gmu-35-40_default', del_sel)),
             ('capture sha256', edit('assign1-5-8_default', head))]
    for name, t in cases:
        try:
            compare_all(data, t)
        except Fail as x:
            print(f'rejected: {name}: {x}')
            continue
        raise Fail(f'tamper not rejected: {name}')


def mutants(data, din):
    base = os.path.join(WORK, 'mutant')
    ws = os.path.join(base, 'ws')
    shutil.rmtree(ws, ignore_errors=True)
    os.makedirs(ws)
    for p in ('Cargo.toml', 'Cargo.lock'):
        shutil.copy(os.path.join(ROOT, p), ws)
    shutil.copytree(os.path.join(ROOT, 'crates'), os.path.join(ws, 'crates'))
    orig = open(os.path.join(ROOT, SRC)).read()
    for name, subs in MUTANTS:
        s = orig
        for a, b in subs:
            need(s.count(a) == 1, f'mutant {name}: pattern {a!r} found {s.count(a)} times')
            s = s.replace(a, b)
        with open(os.path.join(ws, SRC), 'w') as f:
            f.write(s)
        dout = os.path.join(base, 'out')
        cargo(ws, os.path.join(base, 'target'), din, dout)
        try:
            compare_all(data, traces(dout))
        except Fail as x:
            print(f'mutant rejected: {name}: {x}')
            continue
        raise Fail(f'mutant not rejected: {name}')
    shutil.rmtree(ws)


def main():
    self_test = '--self-test' in sys.argv[1:]
    data = load_inputs()
    print(f'M: run 1 captures match the M2-03d-8 manifest ({len(data)} cases)')
    din, dout = os.path.join(WORK, 'in'), os.path.join(WORK, 'out')
    shutil.rmtree(din, ignore_errors=True)
    os.makedirs(din)
    for c, (h, events) in data.items():
        with open(os.path.join(din, f'{c}.input.txt'), 'w') as f:
            f.write(export(c, h, events))
    print(f'X: replay inputs in {os.path.relpath(din, ROOT)}')
    cargo(ROOT, os.path.join(ROOT, 'target'), din, dout)
    tr = traces(dout)
    for ln in compare_all(data, tr):
        print('R/A/K:', ln)
    print('T/R/A/K: every selection list (order) and 66661c call (row, result) = capture')
    if self_test:
        tampers(data, tr)
        mutants(data, din)
    print('PASS')


if __name__ == '__main__':
    try:
        main()
    except Fail as x:
        print('FAIL', x)
        sys.exit(1)
