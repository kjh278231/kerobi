#!/usr/bin/env python3
"""M2-06f: L1 replay of the Rust pseudocost table (crates/mip/src/pseudocost.rs) on the M2-06e captures.
Contract: docs/m2/mip_pseudocost_rust.md section 3.

(default)    M (run 1 captures = M2-06e manifest), X (replay input from capture input fields only), run
             `cargo test --release -p rsolver-mip --test pseudocost_l1 -- --ignored`, then G/D/S/N/F/O/K against the capture
--self-test  also T (Rust output tampers) and V (code mutants on a patched copy of the workspace, per-case outcome fixed
             in section 3.1 before the first run)
Exit code 0 only when every requested check passes.
"""
import copy, gzip, hashlib, json, os, shutil, struct, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = os.path.join(ROOT, 'bench/oracle/m2_06e')
WORK = os.path.join(ROOT, 'target/mip_pseudocost_rust')
SRC = 'crates/mip/src/pseudocost.rs'
LT, C_TYPE = 0x3c, 0x43
INF = struct.pack('>d', 1e100).hex()
CASES = ('markshare_4_0', 'pk1', 'neos5', 'tr12-30', 'fhnw-binpack4-4', 'neos-860300')
# K (section 3.1): gates, fired, decay gates, decays, new tables, injected updates, symmetry steps
K = {'markshare_4_0': (2005, 2003, 2005, 914, 42, 610, 0), 'pk1': (2285, 2273, 2285, 2128, 93, 2504, 0),
     'neos5': (1993, 1988, 1993, 1485, 195, 1838, 2), 'tr12-30': (2418, 2402, 2425, 2409, 204, 11349, 0),
     'fhnw-binpack4-4': (2859, 1652, 2859, 1196, 72, 10244, 2), 'neos-860300': (882, 839, 883, 840, 30, 10907, 0)}
ALL = set(CASES)
FOUR = {'markshare_4_0', 'pk1', 'neos5', 'fhnw-binpack4-4'}
# (name, [(old, new)], cases where the mutant must be rejected, or None = report only); section 3.1
MUTANTS = [
    ('P1 exponent 0.1 <-> 0.2', [('{ 0.1 } else { 0.2 }', '{ 0.2 } else { 0.1 }')], ALL),
    ('P2 depth + 1 -> depth', [('(depth as f64 + 1.0)', '(depth as f64)')], ALL),
    ('P3 denominator 0.001 -> 0', [('(f + 0.001)', '(f + 0.0)'), ('(0.001 - f)', '(0.0 - f)')], ALL),
    ('P6 w = 1', [('(0.8 / (depth as f64 + 1.0)).powf(e)', '1.0 + 0.0 * e')], ALL),
    ('P4 add instead of max at count 0', [('if self.cnt[d][j] == 0 { if s > g { s } else { g } } else { g + s }', 'g + s')],
     {'neos5'}),
    ('P5 no 1e-10 floor', [('let g = if delta > 1e-10 { delta } else { 1e-10 };', 'let g = delta;')], FOUR),
    ('P8 no 5e99 > cutoff for status 4/5', [('if !(5e99 > cutoff) {', 'if false {')], {'fhnw-binpack4-4'}),
    ('G1 no lp - p > 1e-10 in the decay gate', [(' && n.lp - n.p > 1e-10', '')], FOUR),
    ('D1 halve w after applying', [('w *= 0.5;', 'let w0 = w;\n            w *= 0.5;\n            let w = w0;')], ALL),
    ('D2 cnt + 1 -> cnt', [('self.cnt[d][j] as f64 + 1.0', 'self.cnt[d][j] as f64')], ALL),
    ('D3 no sign test', [('if -1e-10 > s2 * v[j] {', 'if false && -1e-10 > s2 * v[j] {')], ALL),
    ('D4 s = 1 instead of L+0x54', [('(1.0, l.s as f64)', '(1.0, 1.0)')],
     {'markshare_4_0', 'pk1', 'fhnw-binpack4-4', 'neos-860300'}),
    ('S1 later column wins symmetry ties', [('best[c] < v', 'best[c] <= v')], set()),
    ('S2 no symmetry copy', [('if r >= 0 {', 'if false && r >= 0 {')], {'neos5', 'fhnw-binpack4-4'}),
    ('Z1 seed skips negative map entries', [('if m < 0 {\n                break;', 'if m < 0 {\n                continue;')],
     set()),
    ('Z2 seed sum instead of average', [('src.sum[d][m as usize] / c as f64', 'src.sum[d][m as usize]')],
     {'pk1', 'neos5', 'tr12-30', 'fhnw-binpack4-4'}),
    # added after the first run (section 6): the vector selection is not in the 06e variants
    ('R1 decay reads the other LP vector', [('{ l.vec } else { l.alt }', '{ l.alt } else { l.vec }')], None),
]


class Fail(Exception):
    pass


def need(cond, msg):
    if not cond:
        raise Fail(msg)


def sha(p):
    with open(p, 'rb') as fh:
        return hashlib.sha256(fh.read()).hexdigest()


def load_inputs():
    man = json.load(open(os.path.join(DATA, 'manifest.json')))
    data = {}
    for c in CASES:
        p = os.path.join(DATA, f'{c}_1.json.gz')
        h = sha(p)
        need(man['files'].get(f'{c}_1.json.gz') == h, f'M: {p} sha256 != M2-06e manifest')
        d = json.loads(gzip.decompress(open(p, 'rb').read()))
        need(d.get('ok') and not d.get('errors'), f'M: {c}: capture not ok')
        data[c] = (h, d)
    return data


def ent(r):
    """[j, cd, cu, sd, su] -> Rust entry token."""
    return f'{r[0]}:{r[1]}:{r[2]}:{r[3]}:{r[4]}'


def export(case, h, d):
    """Replay input and the expected outputs, from capture fields only (no rule model). Returns (text, exp, counts)."""
    bulk = {k: v for k, v in d['counts'].items() if k.startswith('bulk')}
    need(not bulk, f'X: {case}: parallel merge/reset paths ran {bulk}')
    ev = d['events']
    ops, exp = [f'case {case} {h}'], []
    size = {}
    cnt = dict.fromkeys(('gate', 'fire', 'dgate', 'dec', 'new', 'inject', 'sym', 'free', 'seed'), 0)

    def snap(where, nz):
        exp.append(('snap', where, [ent(r) for r in sorted(nz)]))

    for i, e in enumerate(ev):
        k, where = e['e'], f'event {i} ({e["e"]})'
        nxt = ev[i + 1] if i + 1 < len(ev) else {}
        if k == 'new':
            need(e['f'] == '6ec3da' and e['rc'] == 0 and e['t'] >= 0, f'X: {case} {where}: unmodeled table creation {e}')
            n = e['ncols'] + e['nrows']
            if e['src'] >= 0 and e['map'] is not None and e['flag850'] == 0:
                need(e['src'] in size and len(e['map']) == e['ncols'], f'X: {case} {where}: seed source/map')
                ops.append(f'seed {e["t"]} {n} {e["src"]} ' + ' '.join(map(str, e['map'])))
                cnt['seed'] += 1
            else:
                ops.append(f'new {e["t"]} {n}')
            size[e['t']] = n
            snap(where, e['nz'])
            cnt['new'] += 1
        elif k == 'free':
            if e['t'] < 0:
                continue
            ops.append(f'free {e["t"]}')
            snap(where, e['nz'])
            del size[e['t']]
            cnt['free'] += 1
        elif k == 'sym':
            need(e['t'] == e['tA'], f'X: {case} {where}: symmetry on table {e["t"]}, table A {e["tA"]}')
            ops.append(f'snap {e["t"]}')
            snap(where + ' before', e['before'])
            if e['S'] and e['norb'] > 0 and e['cls'] is not None:
                ops.append(f'sym {e["t"]} ' + ' '.join(map(str, e['cls'])))
                cnt['sym'] += 1
            ops.append(f'snap {e["t"]}')
            snap(where + ' after', e['after'])
        elif k == 'upd':
            need(e['t'] in size, f'X: {case} {where}: update of unknown table {e["t"]}')
            if 'site' in e:
                need(i and ev[i - 1]['e'] == 'gate', f'X: {case} {where}: node update without its gate')
                continue  # compared at the gate
            ops.append(f'show {e["t"]} {e["j"]}')
            exp.append(('show', where + ' before', [ent([e['j']] + e['before'])]))
            ops.append(f'set {e["t"]} {e["j"]} ' + ' '.join(map(str, e['after'])))
            cnt['inject'] += 1
        elif k == 'gate':
            cut = e['cut'] if e['cut'] is not None else INF
            vt = 1 if e['vt'] == C_TYPE else 0
            ops.append(f'gate {e["tA"]} {int(bool(e["parent"]))} {e["st"]} {e["depth"]} {e["j"]} {int(e["dir"] == LT)} '
                       f'{vt} {e["x"]} {e["lp"]} {e["p"]} {cut}')
            cnt['gate'] += 1
            if nxt.get('e') == 'upd' and 'site' in nxt:
                need(nxt['t'] == e['tA'] and nxt['j'] == e['j'], f'X: {case} {where}: update {nxt["t"]}/{nxt["j"]} vs gate')
                exp.append(('gate', where, ['1', nxt['w'], ent([nxt['j']] + nxt['after'])]))
                cnt['fire'] += 1
            else:
                exp.append(('gate', where, ['0']))
        elif k == 'dgate':
            ops.append(f'dgate {int(bool(e["parent"]))} {e["st"]} 0 {e["j"]} 0 0 {"0" * 16} {e["lp"]} {e["p"]}')
            exp.append(('dgate', where, ['1' if nxt.get('e') == 'dec' else '0']))
            cnt['dgate'] += 1
        elif k == 'dec':
            need(i and ev[i - 1]['e'] == 'dgate' and e['t'] == ev[i - 1]['tA'], f'X: {case} {where}: decay without gate')
            cnt['dec'] += 1
            if e.get('noI') or e.get('noL'):
                ops.append(f'dec {e["t"]} none')
            else:
                seen, anc = {}, []
                for a in e['anc']:
                    if a is None:
                        break
                    j, dr, vh = a
                    if j >= 0 and vh is not None:
                        need(seen.setdefault(j, vh) == vh, f'X: {case} {where}: v[{j}] read as {seen[j]} and {vh}')
                    anc.append(f'{j}:{int(dr == LT)}:{vh if vh is not None else "x"}')
                ops.append(f'dec {e["t"]} {e["l38"]} {e["l58"]} {e["l54"]} {size[e["t"]]} {int(bool(e["l18"] and e["l28"]))} '
                           f'{int(bool(e["l70"]))} ' + ' '.join(anc))
            for r in e.get('after', []):
                ops.append(f'show {e["t"]} {r[0]}')
                exp.append(('show', where + ' after', [ent(r)]))
        elif k in ('ins', 'tnew'):
            continue
        else:
            raise Fail(f'X: {case} {where}: unknown event')
    need(not size, f'X: {case}: tables never freed {sorted(size)}')
    ops.append('end')
    return '\n'.join(ops) + '\n', exp, cnt


def compare(case, head, text, exp):
    """G/D/S/N/F/O: first difference with case, event, original and Rust values."""
    lines = text.rstrip('\n').split('\n')
    need(lines[0] == head, f'{case}: Rust output head {lines[0]!r} vs input {head!r}')
    out = lines[1:]
    for n, (kind, where, want) in enumerate(exp):
        need(n < len(out), f'{case}: Rust output ends after {n} of {len(exp)} lines')
        got = out[n].split()
        need(got[0] == kind, f'{case} {where}: Rust line {n} is {got[0]!r}, expected {kind}')
        if got[1:] != want:
            diff = next((i for i, (a, b) in enumerate(zip(got[1:], want)) if a != b), min(len(got) - 1, len(want)))
            a = got[1:][diff] if diff < len(got) - 1 else None
            b = want[diff] if diff < len(want) else None
            raise Fail(f'{case} {where}: {kind} differs at item {diff}: Rust {a}, original {b} '
                       f'({len(got) - 1} vs {len(want)} items)')
    need(len(out) == len(exp), f'{case}: Rust output has {len(out)} lines, expected {len(exp)}')


def cargo(ws, target, din, dout):
    env = dict(os.environ, PSEUDOCOST_IN=din, PSEUDOCOST_OUT=dout, CARGO_TARGET_DIR=target,
               PATH=os.path.expanduser('~/.cargo/bin') + os.pathsep + os.environ.get('PATH', ''))
    shutil.rmtree(dout, ignore_errors=True)  # no stale trace can pass
    r = subprocess.run(['cargo', 'test', '--release', '-p', 'rsolver-mip', '--test', 'pseudocost_l1', '--', '--ignored'],
                       cwd=ws, env=env, capture_output=True, text=True)
    need(r.returncode == 0 and '1 passed' in r.stdout, f'cargo test failed ({r.returncode}):\n{r.stdout[-2000:]}{r.stderr[-2000:]}')


def traces(dout):
    return {c: open(os.path.join(dout, f'{c}.trace.txt')).read() for c in CASES}


def outcomes(heads, exps, tr):
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

    def find(lines, ok, last=False):
        idx = [i for i, ln in enumerate(lines) if ok(ln.split())]
        return idx[-1] if last else idx[0]

    def flip_bit(tok):
        return tok[:-1] + ('0' if tok[-1] != '0' else '1')

    def sum_bits(lines):  # node update result entry: down sum
        i = find(lines, lambda p: p[:2] == ['gate', '1'])
        p = lines[i].split()
        q = p[3].split(':')
        q[3] = flip_bit(q[3])
        p[3] = ':'.join(q)
        lines[i] = ' '.join(p)

    def w_bits(lines):
        i = find(lines, lambda p: p[:2] == ['gate', '1'])
        p = lines[i].split()
        p[2] = flip_bit(p[2])
        lines[i] = ' '.join(p)

    def fire_line(lines):
        i = find(lines, lambda p: p == ['dgate', '1'])
        lines[i] = 'dgate 0'

    def drop_line(lines):
        lines.pop(find(lines, lambda p: p[:1] == ['show']))

    def free_snap(lines):  # the last snapshot is a free; change one count
        i = find(lines, lambda p: p[:1] == ['snap'] and len(p) > 1, last=True)
        p = lines[i].split()
        q = p[1].split(':')
        q[1] = str(int(q[1]) + 1)
        p[1] = ':'.join(q)
        lines[i] = ' '.join(p)

    for name, case, fn in (('update entry sum bit', 'markshare_4_0', sum_bits), ('w bit', 'pk1', w_bits),
                           ('decay gate fired -> not', 'tr12-30', fire_line), ('line deleted', 'neos5', drop_line),
                           ('free snapshot count', 'fhnw-binpack4-4', free_snap)):
        why = outcomes(heads, exps, edit(case, fn))[case]
        need(why is not None, f'T: tamper not rejected: {name}')
        print(f'T: rejected: {name}: {why[:200]}')


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
            print(f'V: {name}: {c}: {tag}' + (f': {res[c][:160]}' if res[c] else ''))
            if want is not None:
                need(bool(res[c]) == (c in want), f'V: mutant {name} on {c}: {tag}, section 3.1 expects '
                     f'{"rejected" if c in want else "passes"}')
    shutil.rmtree(wsp)


def main():
    self_test = '--self-test' in sys.argv[1:]
    data = load_inputs()
    print(f'M: run 1 captures match the M2-06e manifest ({len(data)} cases)')
    din, dout = os.path.join(WORK, 'in'), os.path.join(WORK, 'out')
    shutil.rmtree(din, ignore_errors=True)
    os.makedirs(din)
    heads, exps, tot = {}, {}, {}
    for c, (h, d) in data.items():
        text, exps[c], cnt = export(c, h, d)
        heads[c] = text.split('\n', 1)[0]
        got = (cnt['gate'], cnt['fire'], cnt['dgate'], cnt['dec'], cnt['new'], cnt['inject'], cnt['sym'])
        need(got == K[c], f'K: {c}: (gates, fired, decay gates, decays, new, injected, symmetry) {got} vs {K[c]}')
        need(cnt['free'] == cnt['new'], f'K: {c}: {cnt["free"]} frees vs {cnt["new"]} tables')
        for k, v in cnt.items():
            tot[k] = tot.get(k, 0) + v
        with open(os.path.join(din, f'{c}.input.txt'), 'w') as fh:
            fh.write(text)
    print(f'X: replay inputs in {os.path.relpath(din, ROOT)}')
    print('K: ' + ', '.join(f'{k} {v}' for k, v in tot.items()))
    cargo(ROOT, os.path.join(ROOT, 'target'), din, dout)
    tr = traces(dout)
    res = outcomes(heads, exps, tr)
    for c in CASES:
        need(res[c] is None, res[c] or '')
        print(f'G/D/S/N/F/O: {c}: {len(exps[c])} lines: rust = original')
    if self_test:
        tampers(heads, exps, tr)
        mutants(heads, exps, din)
    print('PASS' + (' (+ self-test)' if self_test else ''))


if __name__ == '__main__':
    try:
        main()
    except Fail as x:
        print('FAIL', x)
        sys.exit(1)
