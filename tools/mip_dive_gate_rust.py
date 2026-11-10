#!/usr/bin/env python3
"""M2-03d-12: L1 replay of the Rust DynamicRows gates (crates/mip/src/dive_rows.rs Gate) on the M2-03d-11 captures.
Contract: docs/m2/mip_dive_gate_rust.md section 3.

(default)    M (captures = M2-03d-11 manifest), X (replay inputs from raw gate fields and called-function eax only),
             run `cargo test --release -p rsolver-mip --test dive_gate_l1 -- --ignored`, then H/T/K against the g:hit events
--self-test  also trace tampers and code mutants (patched copy of the workspace): discriminated ones must be rejected,
             the ones the captures cannot discriminate (section 3 V) must pass
Exit code 0 only when every requested check passes.
"""
import copy, os, shutil, subprocess, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mip_dive_list_rust import ROOT, Fail, load, need, sha  # noqa: E402

MANIFEST = 'bench/oracle/mip_dive_gate_manifest.json'
WORK = os.path.join(ROOT, 'target/mip_dive_gate_rust')
SRC = 'crates/mip/src/dive_rows.rs'
SEQ = ['51305f', '513078', '51319d', '5131af', '5131b7', '5131c2']
# case: (C:1581 evaluations, select true), from docs/m2/mip_dive_gate.md section 6.2
CASES = {'nidda_H_s1': (33118, 1320), 'gmu-35-40_default': (10787, 689), 'assign1-5-8_default': (3088, 84),
         'nidda_H_s1_dr0': (33182, 0)}
TOTAL = 160353
REACT = 'self.dyn_rows != 0 && self.depth == self.dive_depth\n    }'
# (name, [(old, new)], rejected?): one clause changed at a time; False = not discriminated by the captures (section 3 V)
MUTANTS = [
    ('react: dyn ignored', [(REACT, 'self.depth == self.dive_depth\n    }')], True),
    ('react: depth ignored', [(REACT, 'self.dyn_rows != 0\n    }')], True),
    ('select: dyn ignored', [('self.dyn_rows != 0\n            && self.depth', 'self.depth')], True),
    ('select: depth ignored', [('\n            && self.depth == self.dive_depth\n            && self.lp_status', '\n            && self.lp_status')], True),
    ('select: state ignored', [('\n            && self.lp_status == 2', '')], True),
    ('select: +0x18 limit 1', [('self.p3_18 < 2', 'self.p3_18 < 1')], True),
    ('select: a7f0ad call dropped', [('&& !qp()', '&& { let _ = &qp; true }')], True),
    ('select: a7f923 call dropped', [('&& !qcp()', '&& { let _ = &qcp; true }')], True),
    ('select: I+0xa0 null ignored', [('\n            && self.has_flags', '')], False),
    ('select: I+8 null ignored', [('\n            && self.has_sol', '')], False),
    ('select: a7f0ad result ignored', [('&& !qp()', '&& { qp(); true }')], False),
    ('select: a7f923 result ignored', [('&& !qcp()', '&& { qcp(); true }')], False),
    ('select: +0x18 ignored', [('\n            && self.p3_18 < 2', '')], False),
    ('select: +0x18 limit 3', [('self.p3_18 < 2', 'self.p3_18 < 3')], False),
]


def load_inputs():
    man = dict(load(MANIFEST)['captures'])
    data = {}
    for c in CASES:
        p = f'bench/oracle/mip_dive_gate/{c}_1.json.gz'
        h = sha(p)
        need(man.get(p) == h, f'M: {p} sha256 != M2-03d-11 manifest')
        data[c] = (h, load(p)['events'])
    return data


def gates(case, events):
    """(gate event, hit 'at' list) in capture order; H: hits are a prefix of the hook sequence of their gate."""
    out = []
    for e in events:
        if e['event'] in ('g:526', 'g:1581'):
            out.append((e, []))
        elif e['event'] == 'g:hit':
            need(out and out[-1][0]['fb'] == e['fb'], f'H {case}: hit {e["at"]} has no gate of its 5fb8ec call')
            out[-1][1].append(e)
    for g, hits in out:
        at = [h['at'] for h in hits]
        ok = at in ([], ['510961']) if g['event'] == 'g:526' else at == SEQ[:len(at)] and len(at) != 2
        need(ok, f'H {case}: {g["event"]} fb {g["fb"]} k {g["k"]}: hits {at}')
    return out


def export(case, h, gs):
    """Replay input: raw gate fields and the eax of the calls the original made; never the hit sequence itself."""
    out = [f'case {case} {h}']
    for g, hits in gs:
        f = [g['dyn'], g['depth'], g['w34'], g['state'], int(g['a0_null']), int(g['sol_null']), g['p3_18']]
        if g['event'] == 'g:1581':
            eax = {x['at']: x['eax'] for x in hits if 'eax' in x}
            f += [eax.get('51319d', '-'), eax.get('5131af', '-')]
        out.append(' '.join(map(str, [g['event'][2:]] + f)))
    return '\n'.join(out + ['end']) + '\n'


def compare(case, h, text, gs):
    lines = text.rstrip('\n').split('\n')
    need(lines[0] == f'case {case} {h}', f'T {case}: trace head {lines[0][:90]!r}')
    need(len(lines) - 1 == len(gs), f'T {case}: {len(lines) - 1} trace lines != {len(gs)} gate evaluations')
    n1581 = nsel = 0
    for k, (ln, (g, hits)) in enumerate(zip(lines[1:], gs)):
        at = {x['at'] for x in hits}
        if g['event'] == 'g:526':
            exp = f'526 {int("510961" in at)}'
        else:
            exp = f'1581 {int("51319d" in at)} {int("5131af" in at)} {int("5131c2" in at)}'
            n1581 += 1
            nsel += '5131c2' in at
        need(ln == exp, f'T {case}: evaluation {k} ({g["event"]} fb {g["fb"]} k {g["k"]} node {g["node"]} mip_depth {g["mip_depth"]} '
             f'dyn {g["dyn"]} depth {g["depth"]} w34 {g["w34"]} state {g["state"]}): rust {ln!r} != capture {exp!r}')
    need((n1581, nsel) == CASES[case], f'K {case}: C:1581 evaluations, select true {(n1581, nsel)} != {CASES[case]}')
    return len(gs), n1581, nsel


def compare_all(data, gs, tr):
    res = {c: compare(c, h, tr[c], gs[c]) for c, (h, _) in data.items()}
    total = sum(r[0] for r in res.values())
    need(total == TOTAL, f'K: {total} gate evaluations != {TOTAL}')
    return res


def cargo(ws, target, din, dout):
    env = dict(os.environ, DIVE_GATE_IN=din, DIVE_GATE_OUT=dout, CARGO_TARGET_DIR=target,
               PATH=os.path.expanduser('~/.cargo/bin') + os.pathsep + os.environ.get('PATH', ''))
    shutil.rmtree(dout, ignore_errors=True)       # no stale trace can pass
    r = subprocess.run(['cargo', 'test', '--release', '-p', 'rsolver-mip', '--test', 'dive_gate_l1', '--', '--ignored'],
                       cwd=ws, env=env, capture_output=True, text=True)
    need(r.returncode == 0 and '1 passed' in r.stdout, f'cargo test failed ({r.returncode}):\n{r.stdout[-2000:]}{r.stderr[-2000:]}')


def traces(dout):
    out = {}
    for c in CASES:
        with open(os.path.join(dout, f'{c}.trace.txt')) as f:
            out[c] = f.read()
    return out


def tampers(data, gs, tr):
    def edit(case, f):
        t = copy.deepcopy(tr)
        lines = t[case].split('\n')
        f(lines)
        t[case] = '\n'.join(lines)
        return t

    def flip(lines, kind, pos):
        # first line of this kind whose field at pos is 1 (a decision the capture makes true)
        k = next(i for i in range(1, len(lines)) if lines[i].split()[:1] == [kind] and lines[i].split()[pos] == '1')
        p = lines[k].split()
        p[pos] = '0'
        lines[k] = ' '.join(p)

    def head(lines):
        lines[0] = lines[0][:-1] + ('0' if lines[0][-1] != '0' else '1')

    cases = [('react flipped', edit('gmu-35-40_default', lambda ls: flip(ls, '526', 1))),
             ('select flipped', edit('assign1-5-8_default', lambda ls: flip(ls, '1581', 3))),
             ('a7f0ad call flipped', edit('nidda_H_s1', lambda ls: flip(ls, '1581', 1))),
             ('line deleted', edit('nidda_H_s1_dr0', lambda ls: ls.pop(1))),
             ('capture sha256', edit('gmu-35-40_default', head))]
    for name, t in cases:
        try:
            compare_all(data, gs, t)
        except Fail as x:
            print(f'rejected: {name}: {x}')
            continue
        raise Fail(f'tamper not rejected: {name}')


def mutants(data, gs, din):
    base = os.path.join(WORK, 'mutant')
    wsp = os.path.join(base, 'ws')
    shutil.rmtree(wsp, ignore_errors=True)
    os.makedirs(wsp)
    for p in ('Cargo.toml', 'Cargo.lock'):
        shutil.copy(os.path.join(ROOT, p), wsp)
    shutil.copytree(os.path.join(ROOT, 'crates'), os.path.join(wsp, 'crates'))
    with open(os.path.join(ROOT, SRC)) as f:
        orig = f.read()
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
            # the replay panics when Rust calls a7f0ad/a7f923 where the original did not; a build error fails the self-test
            t = str(x).split('\n')
            k = next((i for i, ln in enumerate(t) if 'panicked at crates/mip/tests/dive_gate_l1.rs' in ln), None)
            need(k is not None, f'mutant {name}: {x}')
            why = 'replay panicked: ' + ' '.join(t[k:k + 2]).strip()
        if why is None:
            try:
                compare_all(data, gs, traces(dout))
            except Fail as x:
                why = str(x)
        if why is not None:
            need(discriminated, f'mutant {name} was rejected but section 3 V lists it as not discriminated: {why}')
            print(f'mutant rejected: {name}: {why}')
            continue
        need(not discriminated, f'mutant not rejected: {name}')
        print(f'mutant passes (not discriminated by the captures, section 3 V): {name}')
    shutil.rmtree(wsp)


def main():
    self_test = '--self-test' in sys.argv[1:]
    data = load_inputs()
    print(f'M: run 1 captures match the M2-03d-11 manifest ({len(data)} cases)')
    gs = {c: gates(c, events) for c, (_, events) in data.items()}
    print('H: every hit sequence is a prefix of its gate hook sequence')
    din, dout = os.path.join(WORK, 'in'), os.path.join(WORK, 'out')
    shutil.rmtree(din, ignore_errors=True)
    os.makedirs(din)
    for c, (h, _) in data.items():
        with open(os.path.join(din, f'{c}.input.txt'), 'w') as f:
            f.write(export(c, h, gs[c]))
    print(f'X: replay inputs in {os.path.relpath(din, ROOT)}')
    cargo(ROOT, os.path.join(ROOT, 'target'), din, dout)
    tr = traces(dout)
    for c, (n, n1581, nsel) in compare_all(data, gs, tr).items():
        print(f'T/K: {c}: {n} evaluations (C:526 {n - n1581}, C:1581 {n1581}), select {nsel}: rust = capture')
    print(f'K: {TOTAL} gate evaluations')
    if self_test:
        tampers(data, gs, tr)
        mutants(data, gs, din)
    print('PASS')


if __name__ == '__main__':
    try:
        main()
    except Fail as x:
        print('FAIL', x)
        sys.exit(1)
