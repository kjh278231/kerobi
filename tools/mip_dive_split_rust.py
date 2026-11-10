#!/usr/bin/env python3
"""M2-03d-14: L1 replay of Rust RowList::swap_in/swap_out (crates/mip/src/dive_rows.rs) on every 10f448 rebuild of the
M2-03d-13 captures. Contract: docs/m2/mip_dive_split_rust.md section 3.

(default)    M (capture hashes = M2-03d-13 manifest), X (inputs from entry fields only), run
             `cargo test --release -p rsolver-mip --test dive_split_l1 -- --ignored`, then P/L/H/S/K against the captures
--self-test  also trace tampers and code mutants (patched copy of the workspace)
Exit code 0 only when every requested check passes.
"""
import copy, os, shutil, subprocess, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mip_dive_rows_check import Fail, load, need, sha  # noqa: E402
from mip_dive_split_check import MANIFEST, POSITIVE, ZERO, cap, rebuilds, split  # noqa: E402
from mip_dive_list_rust import LISTED, MUTANTS as LIST_MUTANTS  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WORK = os.path.join(ROOT, 'target/mip_dive_split_rust')
SRC = 'crates/mip/src/dive_rows.rs'
# M2-03d-13 section 6 K: rebuilds, rebuilds with an exchange, exchanged columns that differ from a stable partition
PINNED = {'nidda_H_s1': (2743, 1354, 119019), 'gmu-35-40_default': (1318, 354, 22800), 'assign1-5-8_default': (245, 111, 4453)}
INNER = '''                    while a <= b {
                        if !listed(b, ind) {
                            if a < b {
                                ind.swap(a as usize, b as usize);
                                val.swap(a as usize, b as usize);
                                a += 1;
                                b -= 1;
                            }
                            break;
                        }
                        b -= 1;
                    }'''
FORWARD = '''                    let mut u = a + 1;
                    while u <= b && listed(u, ind) {
                        u += 1;
                    }
                    if u > b {
                        break;
                    }
                    ind.swap(a as usize, u as usize);
                    val.swap(a as usize, u as usize);
                    a += 1;'''
_lm = {name: subs for name, subs, _ in LIST_MUTANTS}
# (name, [(old, new)], must be rejected?); None = report the result (contract section 3)
MUTANTS = [('stable partition', _lm['stable partition'], True),
           ('val not exchanged', [('val.swap(a as usize, b as usize);', '')], True),
           ('reduced length = full length', _lm['reduced length = full length'], True),
           ('rhs of listed rows kept', _lm['rhs of listed rows kept'], True),
           ('listed = position > 0', [(LISTED, LISTED.replace('>= 0;', '> 0;'))], True),
           ('forward search', [(INNER, FORWARD)], None)]


def entry_val(e):
    return [e['val_u'][i] for i in e['val_i']]


def export(case, h, evs):
    """Replay input from entry fields only (no perm, raw, xlen_diff, rhs_diff_rows)."""
    bs = [e for e in evs if e['event'] == 's:build']
    refd = {e['before_ref'] for e in bs if e['before_ref'] is not None}
    out = [f'case {case} {h}']
    for e in bs:
        s = e['serial']
        if e['before_ref'] is None:
            out += [f'build {s} {e["rows"]} {int(s in refd)} full', ' '.join(map(str, e['beg'])), ' '.join(map(str, e['len'])),
                    ' '.join(map(str, e['ind'])), ' '.join(entry_val(e))]
        else:
            out.append(f'build {s} {e["rows"]} {int(s in refd)} ref {e["before_ref"]}')
        out += [' '.join(map(str, e['x_list'])), ' '.join(map(str, e['rhs_list_nonzero']))]
    return '\n'.join(out + ['end']) + '\n'


def parse_trace(text):
    lines = text.rstrip('\n').split('\n')
    head = lines[0].split()
    need(len(head) == 3 and head[0] == 'case', f'T: bad trace head {lines[0][:80]!r}')
    builds = []
    for ln in lines[1:]:
        p = [x.split() for x in ln.split('|')]
        if p[0][0] == 'build':
            xl, rh = p[1], p[2]
            builds.append({'nums': list(map(int, p[0][1:])), 'xlen': [[int(xl[k]), int(xl[k + 1])] for k in range(0, len(xl), 2)],
                           'rhs': [(int(rh[k]), rh[k + 1]) for k in range(0, len(rh), 2)], 'cols': {}})
        else:
            need(p[0][0] == 'col' and builds, f'T: bad trace line {ln[:80]!r}')
            j = int(p[0][1])
            need(j not in builds[-1]['cols'], f'T: column {j} twice')
            builds[-1]['cols'][j] = (tuple(map(int, p[1])), tuple(p[2]))
    return head[1], head[2], builds


def compare(case, h, text, evs):
    name, th, tb = parse_trace(text)
    need((name, th) == (case, h), f'T {case}: trace is for {name} {th}, capture {h}')
    n = ex = ns = 0
    for (e, st, after), b in zip(rebuilds(evs, case), tb):
        s = e['serial']
        w = f'{case} build {s}'
        need(b['nums'][0] == s, f'K {w}: trace build {b["nums"][0]}')
        built, refs, refs_out, restored = b['nums'][1:]
        need((built, refs) == (e['built_after'], e['ref_after']) == (1, 1), f'S {w}: rust state ({built}, {refs}), capture '
             f'({e["built_after"]}, {e["ref_after"]})')
        need(refs_out == 0 and restored == 1, f'S {w}: after swap_out refs {refs_out}, lengths/rhs restored {restored}')
        need(b['xlen'] == e['xlen_diff'], f'L {w}: reduced lengths {b["xlen"][:4]} != capture {e["xlen_diff"][:4]}')
        need([r for r, _ in b['rhs']] == e['rhs_diff_rows'] and all(v == ZERO for _, v in b['rhs']),
             f'H {w}: rhs changed rows {b["rhs"][:4]} != capture {e["rhs_diff_rows"][:4]} (all +0)')
        want = {j: ac for j, (bc, ac) in enumerate(zip(st[2], after[2])) if bc != ac}
        need(set(b['cols']) == set(want), f'P {w}: changed columns {sorted(b["cols"])[:8]} != capture {sorted(want)[:8]}')
        listed = set(e['x_list'])
        for j, col in b['cols'].items():
            need(col[0] == want[j][0], f'P {w} col {j}: ind {list(col[0])} != capture {list(want[j][0])}')
            need(col[1] == want[j][1], f'P {w} col {j}: val bits differ from the capture')
            ns += tuple(split(st[2][j][0], st[2][j][1], listed, 'stable')[0]) != col[0]
        ex += bool(b['cols'])
        n += 1
    nb = sum(e['event'] == 's:build' for e in evs)
    need(n == nb == len(tb), f'K {case}: compared {n}, trace {len(tb)}, capture {nb} rebuilds')
    need((n, ex, ns) == PINNED[case], f'K {case}: (rebuilds, with exchange, not stable) {(n, ex, ns)} != M2-03d-13 {PINNED[case]}')
    return n, ex, ns


def cargo(ws, target, din, dout):
    env = dict(os.environ, DIVE_SPLIT_IN=din, DIVE_SPLIT_OUT=dout, CARGO_TARGET_DIR=target,
               PATH=os.path.expanduser('~/.cargo/bin') + os.pathsep + os.environ.get('PATH', ''))
    shutil.rmtree(dout, ignore_errors=True)       # no stale trace can pass
    r = subprocess.run(['cargo', 'test', '--release', '-p', 'rsolver-mip', '--test', 'dive_split_l1', '--', '--ignored'],
                       cwd=ws, env=env, capture_output=True, text=True)
    need(r.returncode == 0 and '1 passed' in r.stdout, f'cargo test failed ({r.returncode}):\n{r.stdout[-2000:]}{r.stderr[-2000:]}')


def traces(dout):
    out = {}
    for c in POSITIVE:
        with open(os.path.join(dout, f'{c}.trace.txt')) as f:
            out[c] = f.read()
    return out


def compare_all(data, tr):
    return [(c, compare(c, h, tr[c], evs)) for c, (h, evs) in data.items()]


def tampers(data, tr):
    def edit(case, f):
        t = copy.deepcopy(tr)
        lines = t[case].split('\n')
        f(lines)
        t[case] = '\n'.join(lines)
        return t

    def first(lines, pred):
        return next(k for k in range(1, len(lines)) if lines[k] and pred(lines[k]))

    def col_ind(lines):
        k = first(lines, lambda ln: ln.startswith('col'))
        p = lines[k].split('|')
        v = p[1].split()
        v[0], v[1] = v[1], v[0]
        p[1] = ' ' + ' '.join(v) + ' '
        lines[k] = '|'.join(p)

    def col_val(lines):
        k = first(lines, lambda ln: ln.startswith('col'))
        p = lines[k].split('|')
        v = p[2].split()
        v[0] = f'{int(v[0], 16) ^ 1:016x}'
        p[2] = ' ' + ' '.join(v)
        lines[k] = '|'.join(p)

    def xlen(lines):
        k = first(lines, lambda ln: ln.startswith('build') and ln.split('|')[1].split())
        p = lines[k].split('|')
        v = p[1].split()
        v[1] = str(int(v[1]) + 1)
        p[1] = ' ' + ' '.join(v) + ' '
        lines[k] = '|'.join(p)

    def rhs_row(lines):
        k = first(lines, lambda ln: ln.startswith('build'))
        lines[k] += f' 1000000 {ZERO}'

    def drop_build(lines):
        k = first(lines, lambda ln: ln.startswith('build') and ln.split()[1] == '2')
        e = next(i for i in range(k + 1, len(lines)) if lines[i].startswith('build'))
        del lines[k:e]

    def head(lines):
        lines[0] = lines[0][:-1] + ('0' if lines[0][-1] != '0' else '1')

    def built(lines):
        k = first(lines, lambda ln: ln.startswith('build'))
        p = lines[k].split()
        p[2] = '0'
        lines[k] = ' '.join(p)

    N, G, A = 'nidda_H_s1', 'gmu-35-40_default', 'assign1-5-8_default'
    cases = [('column ind exchanged', edit(N, col_ind)), ('column val one bit', edit(G, col_val)), ('reduced length +1', edit(A, xlen)),
             ('rhs row added', edit(N, rhs_row)), ('build 2 dropped', edit(G, drop_build)), ('capture sha256', edit(A, head)),
             ('built flag 0', edit(N, built))]
    for name, t in cases:
        try:
            compare_all(data, t)
        except Fail as x:
            print(f'rejected: {name}: {str(x)[:140]}')
            continue
        raise Fail(f'tamper not rejected: {name}')


def mutants(data, din):
    base = os.path.join(WORK, 'mutant')
    wsp = os.path.join(base, 'ws')
    shutil.rmtree(wsp, ignore_errors=True)
    os.makedirs(wsp)
    for p in ('Cargo.toml', 'Cargo.lock'):
        shutil.copy(os.path.join(ROOT, p), wsp)
    shutil.copytree(os.path.join(ROOT, 'crates'), os.path.join(wsp, 'crates'))
    orig = open(os.path.join(ROOT, SRC)).read()
    rows = []
    for name, subs, must in MUTANTS:
        s = orig
        for a, b in subs:
            need(s.count(a) == 1, f'mutant {name}: pattern {a!r} found {s.count(a)} times')
            s = s.replace(a, b)
        with open(os.path.join(wsp, SRC), 'w') as f:
            f.write(s)
        dout = os.path.join(base, 'out')
        cargo(wsp, os.path.join(base, 'target'), din, dout)   # a build error or replay panic fails the self-test
        try:
            compare_all(data, traces(dout))
            why = None
        except Fail as x:
            why = str(x)
        need(why is not None or not must, f'mutant not rejected: {name}')
        rows.append((name, why))
        print(f'mutant {"rejected" if why else "NOT DISCRIMINATED"}: {name}' + (f': {why[:140]}' if why else ''))
    shutil.rmtree(wsp)
    return rows


def main():
    self_test = '--self-test' in sys.argv[1:]
    man = load(MANIFEST)['captures']
    data = {}
    for c in POSITIVE:
        p = cap(c, 1)
        h = sha(p)
        need(man.get(p) == h, f'M: {p} sha256 != M2-03d-13 manifest')
        data[c] = (h, load(p)['events'])
    print(f'M: {len(data)} run 1 captures match the M2-03d-13 manifest')
    din, dout = os.path.join(WORK, 'in'), os.path.join(WORK, 'out')
    shutil.rmtree(din, ignore_errors=True)
    os.makedirs(din)
    for c, (h, evs) in data.items():
        with open(os.path.join(din, f'{c}.input.txt'), 'w') as f:
            f.write(export(c, h, evs))
    print(f'X: replay inputs in {os.path.relpath(din, ROOT)} (entry fields only)')
    cargo(ROOT, os.path.join(ROOT, 'target'), din, dout)
    tr = traces(dout)
    tot = [0, 0, 0]
    for c, got in compare_all(data, tr):
        print(f'P/L/H/S/K: {c}: {got[0]} rebuilds equal (with exchange {got[1]}, columns not a stable partition {got[2]})')
        tot = [a + b for a, b in zip(tot, got)]
    print(f'K: total {tot[0]} rebuilds, {tot[1]} with exchange, {tot[2]} columns not a stable partition')
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
