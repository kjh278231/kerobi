#!/usr/bin/env python3
"""M2-06f-3: L1 replay of the Rust node lifetime model (crates/mip/src/node_life.rs NodeLife) on the M2-06f-2 captures.
Contract: docs/m2/mip_node_life_rust.md section 3.

(default)    M (run 1 captures = M2-06f-2 manifest), X (replay input from nn/rel/dec events only), run
             `cargo test --release -p rsolver-mip --test node_life_l1 -- --ignored`, then C/R/D/K against the capture
--self-test  also T (Rust output tampers) and V (code mutants W1-W6 on a patched copy of the workspace, per-case outcome
             fixed in section 3 before the first run)
Exit code 0 only when every requested check passes.
"""
import copy, gzip, hashlib, json, os, shutil, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = os.path.join(ROOT, 'bench/oracle/m2_06f2')
WORK = os.path.join(ROOT, 'target/mip_node_life_rust')
SRC = 'crates/mip/src/node_life.rs'
CASES = ('markshare_4_0', 'pk1', 'neos5', 'tr12-30', 'fhnw-binpack4-4', 'neos-860300')
# section 3 K: 6-case totals from docs/m2/mip_node_life.md section 6
TOTALS = {'nn': 18021, 'rel': 23939, 'lp': 19325, 'dec': 8972, 'chain': 7404, 'noI': 7238, 'nop': 166, 'noIL': 1568,
          'fr': 40496}
ALL, NONE = dict.fromkeys(CASES, True), dict.fromkeys(CASES, False)
# (name, [(old, new)], expected rejection per case); section 3 V
MUTANTS = [
    ('W1 no old < 5 stop', [('if old < 5 {', 'if false {')], ALL),
    ('W2 no parent stage < 3 stop', [('if pn.stage < 3 {', 'if false {')], NONE),
    ('W3 no child count cascade', [('if pn.children == 0 {', 'if false {')], ALL),
    ('W4 s > 4 stop as s > 3', [('if !pn.info || s > 4 {', 'if !pn.info || s > 3 {')], ALL),
    ('W5 parent I freed at stage 0', [('(x, s) = (p, 1);', '(x, s) = (p, 0);')], ALL),
    ('W6 no root exception', [('if n.root && n.info && s > 0 {', 'if false && n.info && s > 0 {')], ALL),
]


class Fail(Exception):
    pass


def need(cond, msg):
    if not cond:
        raise Fail(msg)


def sha(p):
    with open(p, 'rb') as f:
        return hashlib.sha256(f.read()).hexdigest()


def load_inputs():
    man = json.load(open(os.path.join(DATA, 'manifest.json')))
    data = {}
    for c in CASES:
        p = os.path.join(DATA, f'{c}_1.json.gz')
        h = sha(p)
        need(man['files'].get(f'{c}_1.json.gz') == h, f'M: {p} sha256 != M2-06f-2 manifest')
        data[c] = (h, json.loads(gzip.decompress(open(p, 'rb').read()))['events'])
    return data


def st(I, L):
    return 'I0' if not I else f'I1L{L}'


def export(case, h, ev):
    """Replay input (nn/rel/dec only) and the expected output lines (capture fields only). Returns (text, exp, counts)."""
    inp, exp = [f'case {case} {h}'], []
    cnt = dict.fromkeys(TOTALS, 0)
    i = 0
    while i < len(ev):
        e = ev[i]
        t = e['e']
        if t == 'nn':
            need(e['rc'] == 0, f'X {case}: event {i}: 5ff358 failed')
            inp.append(f'nn {e["par"] if e["par"] >= 0 else "-"} {e["root"]}')
            exp.append((i, f'nn {e["s"]}'))
            cnt['nn'] += 1
        elif t == 'rel':
            s = e['s']
            need(s in (0, 3, 4, 6), f'X {case}: event {i}: release stage {s} (out of scope, section 1)')
            inp.append(f'rel {e["n"]} {s}')
            old = '-' if e['old'] is None else str(e['old'])
            if s == 6:
                need(ev[i + 1]['e'] == 'rel6' and ev[i + 1]['n'] == e['n'], f'X {case}: event {i}: rel 6 without rel6')
                exp.append((i, f'rel {old} L{e["L"]}'))
                cnt['lp'] += 1
                i += 2
                continue
            exp.append((i, f'rel {old}'))
            cnt['rel'] += 1
            j = i + 1
            while j < len(ev) and ev[j]['e'] == 'fr':
                a, b = ev[j], ev[j + 1]
                need(b['e'] == 'fr_' and b['n'] == a['n'], f'X {case}: event {j}: fr without fr_')
                post = 'gone' if b.get('gone') else st(b['I'], b.get('L'))
                exp.append((j, f'fr {a["n"]} {a["s"]} {st(a["I"], a.get("L"))} {post}'))
                cnt['fr'] += 1
                j += 2
            i = j
            continue
        elif t == 'dec':
            inp.append(f'dec {e["n"]}')
            if e.get('noI'):
                exp.append((i, 'dec noI'))
                cnt['noIL'] += 1
            elif e.get('noL'):
                exp.append((i, 'dec noL'))
                cnt['noIL'] += 1
            else:
                need(e['end'] in ('noI', 'nop'), f'X {case}: event {i}: chain end {e["end"]}')
                exp.append((i, ' '.join(['dec'] + [str(a) for a in e['as']] + [e['end']])))
                cnt['chain'] += 1
                cnt[e['end']] += 1
            cnt['dec'] += 1
        elif t in ('fr', 'fr_', 'rel6'):
            raise Fail(f'X {case}: event {i}: {t} outside a release')
        i += 1
    inp.append('end')
    return '\n'.join(inp) + '\n', exp, cnt


def compare(case, head, text, exp):
    """C/R/D: first difference with case, capture event, expected and Rust lines."""
    lines = text.rstrip('\n').split('\n')
    need(lines[0] == head, f'{case}: Rust output head {lines[0]!r} vs input {head!r}')
    out = lines[1:]
    for k, (ev, want) in enumerate(exp):
        need(k < len(out), f'{case}: Rust output ends after {k} of {len(exp)} lines (capture event {ev}: {want!r})')
        kind = {'nn': 'C', 'rel': 'R', 'fr': 'R', 'dec': 'D'}[want.split()[0]]
        need(out[k] == want, f'{kind} {case}: capture event {ev}, output line {k}: original {want!r}, Rust {out[k]!r}')
    need(len(out) == len(exp), f'{case}: Rust output has {len(out)} lines, expected {len(exp)}')


def cargo(ws, target, din, dout):
    env = dict(os.environ, NODE_LIFE_IN=din, NODE_LIFE_OUT=dout, CARGO_TARGET_DIR=target,
               PATH=os.path.expanduser('~/.cargo/bin') + os.pathsep + os.environ.get('PATH', ''))
    shutil.rmtree(dout, ignore_errors=True)  # no stale trace can pass
    r = subprocess.run(['cargo', 'test', '--release', '-p', 'rsolver-mip', '--test', 'node_life_l1', '--', '--ignored'],
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
        i = next(k for k, ln in enumerate(lines) if fn(ln.split()) is not None)
        lines[i] = fn(lines[i].split())
        t[case] = '\n'.join(x for x in lines if x is not False)
        return t

    def old(p):
        return 'rel 2' if p[:2] == ['rel', '5'] and len(p) == 2 else None

    def fr_stage(p):
        return ' '.join(p[:2] + ['1'] + p[3:]) if p[:1] == ['fr'] and p[2] == '3' else None

    def fr_drop(p):
        return False if p[:1] == ['fr'] and p[2] == '1' else None

    def fr_post(p):
        return ' '.join(p[:4] + ['I1L1']) if p[:1] == ['fr'] and p[4] == 'I1L0' else None

    def anc(p):
        return ' '.join(['dec', str(int(p[1]) + 1)] + p[2:]) if p[:1] == ['dec'] and len(p) > 3 else None

    def end(p):
        return ' '.join(p[:-1] + ['nop']) if p[:1] == ['dec'] and p[-1] == 'noI' and len(p) > 2 else None

    for name, case, fn in (('previous stage', 'pk1', old), ('fr stage', 'neos5', fr_stage), ('fr deleted', 'tr12-30', fr_drop),
                           ('fr state after', 'markshare_4_0', fr_post), ('ancestor id', 'fhnw-binpack4-4', anc),
                           ('chain end', 'neos-860300', end)):
        why = outcomes(heads, exps, edit(case, fn))[case]
        need(why is not None, f'T: tamper not rejected: {name}')
        print(f'T: rejected: {name}: {why}')


def truncated(heads, exps, din):
    """Input tamper: one case without its closing `end` line must end in `panic` (caught per case) and be rejected."""
    tin, tout = os.path.join(WORK, 'trunc_in'), os.path.join(WORK, 'trunc_out')
    shutil.rmtree(tin, ignore_errors=True)
    shutil.copytree(din, tin)
    p = os.path.join(tin, 'neos-860300.input.txt')
    s = open(p).read()
    need(s.endswith('\nend\n'), 'T: input does not end with end')
    open(p, 'w').write(s[:-len('end\n')])
    cargo(ROOT, os.path.join(ROOT, 'target'), tin, tout)
    tr = traces(tout)
    res = outcomes(heads, exps, tr)
    need(tr['neos-860300'].endswith('\npanic\n') and res['neos-860300'], 'T: input without end not rejected by a caught panic')
    need(all(res[c] is None for c in CASES if c != 'neos-860300'), f'T: input without end changed other cases {res}')
    print(f'T: rejected: input without end (panic caught, other cases pass): {res["neos-860300"]}')


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
            need(bool(res[c]) == want[c], f'V: mutant {name} on {c}: {tag}, section 3 expects '
                 f'{"rejected" if want[c] else "passes"}')
    shutil.rmtree(wsp)


def main():
    self_test = '--self-test' in sys.argv[1:]
    data = load_inputs()
    print(f'M: run 1 captures match the M2-06f-2 manifest ({len(data)} cases)')
    din, dout = os.path.join(WORK, 'in'), os.path.join(WORK, 'out')
    shutil.rmtree(din, ignore_errors=True)
    os.makedirs(din)
    heads, exps, tot = {}, {}, dict.fromkeys(TOTALS, 0)
    for c, (h, ev) in data.items():
        text, exps[c], cnt = export(c, h, ev)
        heads[c] = text.split('\n', 1)[0]
        for k in tot:
            tot[k] += cnt[k]
        with open(os.path.join(din, f'{c}.input.txt'), 'w') as fh:
            fh.write(text)
        print(f'X: {c}: {json.dumps(cnt)}')
    need(tot == TOTALS, f'K: totals {tot} vs section 3 {TOTALS}')
    print(f'K: {json.dumps(tot)}')
    cargo(ROOT, os.path.join(ROOT, 'target'), din, dout)
    tr = traces(dout)
    res = outcomes(heads, exps, tr)
    for c in CASES:
        need(res[c] is None, res[c] or '')
        print(f'C/R/D: {c}: {len(exps[c])} lines: rust = original')
    if self_test:
        tampers(heads, exps, tr)
        truncated(heads, exps, din)
        mutants(heads, exps, din)
    print('PASS' + (' (+ self-test)' if self_test else ''))


if __name__ == '__main__':
    try:
        main()
    except Fail as x:
        print('FAIL', x)
        sys.exit(1)
