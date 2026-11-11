#!/usr/bin/env python3
"""M2-06d/06d-3: L1 replay of the Rust plunge selection (crates/mip/src/plunge.rs) on the M2-06c-3 captures.
Contracts: docs/m2/mip_plunge_rust.md section 3, changed by docs/m2/mip_plunge_r47_rust.md section 3.

(default)    M (run 1 captures = M2-06c-3 manifest), X (replay input from input fields only), run
             `cargo test --release -p rsolver-mip --test plunge_l1 -- --ignored`, then R/P/T against the capture and K
--self-test  also T (Rust output tampers, one input tamper) and V (code mutants D1-D10, E1-E7 on a patched copy of the
             workspace, per-case outcome fixed in section 3 before the first run)
Exit code 0 only when every requested check passes.
"""
import copy, json, os, shutil, subprocess, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mip_plunge_rule_check as R  # noqa: E402
import mip_plunge_r47_check as Q  # noqa: E402
from mip_plunge_rule_check import Fail, need, one, calls, sha, load  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = Q.DATA
WORK = os.path.join(ROOT, 'target/mip_plunge_rust')
SRC = 'crates/mip/src/plunge.rs'
TEST = 'crates/mip/tests/plunge_l1.rs'
CASES = tuple(Q.CASES)                 # the 5 M2-06c cases + pk1 BranchDir +-1
CASES5 = tuple(R.CASES)
# section 3 K (docs/m2/mip_plunge_r47_rust.md): 06c-3 section 6 table and the capture tail counts, fixed before the first run
TOTALS = {'calls': 13830, 'pick1256': 7499, 'pick312': 1980, 'pick479': 4351, 'ranc': 28245, 'x': 1023, 'r5': 239,
          'draws': 963, 'kept': 41985, 'heap': 8834, 'drop': 5, 'thr_set': 1964, 'null_anc': 89}
ALL, NONE = dict.fromkeys(CASES, True), dict.fromkeys(CASES, False)
BD = ('pk1-bd1', 'pk1-bd-1')


def but(*xs):
    return {c: c not in xs for c in CASES}


def only(*xs):
    return {c: c in xs for c in CASES}


W8_OLD = """                if sc > best && anc(it.node, it.depth, d) == top_anc {
                    idx = Some(j);
                    best = sc;
                }"""
W8_NEW = """                if sc > best {
                    if anc(it.node, it.depth, d) == top_anc {
                        idx = Some(j);
                    }
                    best = sc;
                }"""
ALL5 = only(*CASES5)
R4 = "if it.dir == b'<' as i8 { it.val.floor() } else { it.val.ceil() }"
R7 = '(it.dir as i32 + 10 * it.depth)'
R9 = 'if branch_dir == -1 { base - it.dir as f64 } else { base + it.dir as f64 }'
L2 = 'if sc > best && anc(it.node, it.depth, d) == top_anc {'
# (name, file, [(old, new)], expected rejection per case); section 3 V. D/E: 06d table on the 5 cases (E3 changed, BranchDir
# columns added; both from Python replays before the first run), F: 06c-3 variants X1-X15
MUTANTS = [
    ('D1 rule 3/12 sign swapped', SRC, [('if rule == 12 { dep + q } else { dep - q }', 'if rule == 12 { dep - q } else { dep + q }')], ALL5),
    ('D2 seed from post-pop k', TEST, [('seed(k_entry, w28)', 'seed(k, w28)')], ALL),
    ('D3 draw cutoff 0.5', SRC, [('rng.next_f64() < 0.9', 'rng.next_f64() < 0.5')], ALL5),
    ('D4 delta=2 clause off', SRC, [('if p.k <= p.m2c {', 'if false {')], ALL),
    ('D5 delta=6 clause off', SRC, [('} else if p.k % 23 == 7 {', '} else if false {')], ALL),
    ('D6 log branch rounds', SRC, [('(x as i32).clamp(3, 10)', '(x.round() as i32).clamp(3, 10)')], only('tr12-30')),
    ('D7 keep ignores ancestor', SRC, [('if same && !(it.bound > lim) {', 'if !(it.bound > lim) {')], ALL),
    ('D8 keep ignores bound', SRC, [('if same && !(it.bound > lim) {', 'if same {')], only('pk1-pool')),
    ('D9 removal always to heap', SRC, [('removed(it.node, t.cut > it.bound)', 'removed(it.node, true)')], only('neos5', 'tr12-30')),
    ('D10 rule 3/12 tie goes to last', SRC, [('if sc > best {\n', 'if sc >= best {\n')], NONE),
    ('E1 rule 1/6 ancestor comparison off', SRC, [(' && anc(it.node, it.depth, d) == top_anc {', ' {')], ALL5),
    ('E2 ancestor depth maxd-1', SRC, [('let d = maxd - 2;', 'let d = maxd - 1;')], ALL5),
    ('E3 null ancestor never matches', SRC, [('== top_anc {', '== top_anc && top_anc.is_some() {')], ALL5),
    ('E4 rule 1 depth sign +', SRC, [('(-it.depth) as f64 - (it.est - m) / 1e8', 'it.depth as f64 - (it.est - m) / 1e8')],
     only('pk1', 'neos5', 'tr12-30', 'pk1-pool')),
    ('E5 rule 1/6 best also updated on a mismatch', SRC, [(W8_OLD, W8_NEW)], ALL5),
    ('E6 rule 2 tie goes to last', SRC, [('if v > best {', 'if v >= best {')], only('neos5')),
    ('E7 rule 5 start = trunc(u*n)', SRC, [('let s = rng.next_int() % n as i32;', 'let s = (rng.next_f64() * n as f64) as i32;')],
     only('markshare_4_0', 'pk1', 'neos5', 'pk1-pool')),
    ('F1 rule 4 floor/ceil swapped', SRC, [(R4, "if it.dir == b'<' as i8 { it.val.ceil() } else { it.val.floor() }")], ALL5),
    ('F2 rule 4 no rounding', SRC, [(R4, "if it.dir == b'<' as i8 { it.val } else { it.val }")], ALL5),
    ('F3 rule 4 divisor 10', SRC, [(' / 100.0', ' / 10.0')], NONE),
    ('F4 rule 4 no depth term', SRC, [('it.depth as f64 - (x(', '0.0 - (x(')], only('markshare_4_0', 'pk1', 'neos5', 'tr12-30')),
    ('F5 rules 4/7/9 tie goes to last', SRC, [('if s > best {', 'if s >= best {')], NONE),
    ('F6 rule 4 root entries not skipped', SRC, [('(it.var >= 0).then(', '(it.var >= -1).then(')], NONE),
    ('F7 rule 7 direction byte ignored', SRC, [(R7, '(10 * it.depth)')], only('markshare_4_0')),
    ('F8 rule 7 direction byte sign -', SRC, [(R7, '(-(it.dir as i32) + 10 * it.depth)')], only('markshare_4_0')),
    ('F9 rule 7 depth factor 1', SRC, [(R7, '(it.dir as i32 + it.depth)')], NONE),
    ('F10 rule 9 sign flipped', SRC, [(R9, 'if branch_dir == -1 { base + it.dir as f64 } else { base - it.dir as f64 }')], only(*BD)),
    ('F11 rule 9 direction byte ignored', SRC, [(R9, 'if branch_dir == -1 { base } else { base }')], only(*BD)),
    ('F12 rule 1/6 first loop: last deepest entry', SRC, [('if it.depth > maxd {', 'if it.depth > maxd || (it.depth == maxd && maxd > 0) {')], ALL5),
    ('F13 rule 1/6 second loop tie >=', SRC, [(L2, 'if sc >= best && anc(it.node, it.depth, d) == top_anc {')], ALL5),
    ('F14 rule 1/6 second loop queries every entry', SRC, [(L2, 'if anc(it.node, it.depth, d) == top_anc && sc > best {')], ALL5),
    ('F15 rule 5 slot not advanced', SRC, [('let s = rng.next_int() % n as i32;', 'let s = { *rng }.next_int() % n as i32;')],
     only('markshare_4_0', 'pk1', 'neos5', 'pk1-pool')),
]


def load_inputs():
    man = json.load(open(os.path.join(DATA, 'manifest.json')))
    need(man['binary_sha256'] == R.BINARY, 'M: manifest binary hash')
    data = {}
    for c in CASES:
        p = os.path.join(DATA, f'{c}_1.json.gz')
        h = sha(p)
        need(man['files'].get(f'{c}_1.json.gz') == h, f'M: {p} sha256 != M2-06c-3 manifest')
        data[c] = (h, load(p))
    return data


def parents(c):
    """Parent table from the call's 5fb13c chains (section 2.2): id -> parent id or None (null); absent = unknown."""
    par = {}

    def put(x, p):
        need(par.get(x, p) == p, f'X: node {x} has parents {par.get(x)} and {p}')
        par[x] = p

    for a in (e for e in c['ev'] if e['e'] == 'anc'):
        ch, d, dep = a['chain'], a['d'], a['depth']
        steps = dep - d
        full = 1 if (d | steps) < 0 else steps + 1
        for x, p in zip(ch, ch[1:]):
            put(x, p)
        if len(ch) < full and len(ch) < 64:
            put(ch[-1], None)
    return par


def export(case, h, d):
    """Replay input (input fields only) and expected lines (capture fields only). Returns (text, exp, counts)."""
    inp, exp = [f'case {case} {h}'], []
    cnt = dict.fromkeys(TOTALS, 0)
    for i, c in enumerate(calls(d)):
        r = one(c, 'rule')
        if r is None:
            need(c['ev'][-1]['node'] is None, f'X {case} call {i}: empty call returned a node')
            continue
        p, rp, pk, t, te = c['pin'], one(c, 'rpre'), one(c, 'pick'), one(c, 'tail'), one(c, 'tend')
        drw, thr, en, x4, r5 = one(c, 'drw'), one(c, 'thr'), one(c, 'ents'), one(c, 'x4'), one(c, 'r5')
        ranc = [e for e in c['ev'] if e['e'] == 'ranc']
        where = f'X {case} call {i}'
        need(r['r10d'] == p['bdir'], f'{where}: rule r10d differs from pin BranchDir')
        need(en is not None and len(en['ents']) == len(pk['list']), f'{where}: ents missing or wrong size')
        need((x4 is not None) == (r['ecx'] == 4) and (x4 is None or x4['null'] == 0), f'{where}: x4 outside rule 4 or null')
        need((r5 is not None) == (r['ecx'] == 5) and (not ranc or r['ecx'] in (1, 6)), f'{where}: r5/ranc outside their rules')
        need(t['k'] == t['esi'] == r['esi'] == rp['esi'], f'{where}: tail k/esi differ from the rule-step k')
        need(t['m2c'] == p['m2c'] and t['focus'] == p['focus'], f'{where}: tail m2c/focus differ from pin')
        need(p['focus'] != 4 and rp['ecx'] == r['ecx'], f'{where}: MIPFocus 4 or rule changed by the override')
        need(rp['work'] is not None and r['work'] is not None, f'{where}: null work pointer')
        s70 = [e for e in c['ev'] if e['e'] == 's70']
        need(s70 and s70[0]['f'] == '5e70c1' and len(s70) <= 2 and all(e['f'] == '5e6efd' for e in s70[1:]),
             f'{where}: helper calls {s70}')
        rule = r['ecx']
        # ---- input
        inp.append(f'call {i}')
        inp.append(f'pin {p["k"]} {p["w28"]} {p["ign_hint"]} {p["hint"]} {p["m848"]} {p["mx808"]} {p["mx40"]} '
                   f'{p["ncols"]} {p["nint"]} {p["bdir"]} {p["focus"]}')
        inp.append(f'rk {r["esi"]} {t["m2c"]} {t["t138"]} {t["t120"]} {s70[0]["r"]} '
                   f'{s70[1]["r"] if len(s70) > 1 else "-"}')
        inp.append(f'work {rp["work"]}')
        for e, b in zip(pk['list'], en['ents']):
            inp.append(f'it {e[0]} {e[1]} {e[2]} {e[3]} {e[4]} {b[0]} {b[1]} {b[2]}')
        xs = {}
        for _, j, hx in (x4['x'] if x4 else []):     # entries sharing a variable must read the same x
            need(xs.setdefault(j, hx) == hx, f'{where}: x4 gives x[{j}] two values')
        inp += [f'x {j} {hx}' for j, hx in xs.items()]
        for a in ranc:
            inp.append(f'q {a["node"]} {a["d"]} {"-" if a["res"] is None else a["res"]}')
        if thr is not None:
            inp.append(f'thr {thr["f9a41"]} {thr["t00"]} {thr["v8239"]} {thr["t28"]}')
        for x, q in parents(c).items():
            inp.append(f'par {x} {"-" if q is None else q}')
        inp.append('.')
        # ---- expected
        exp.append((i, f'call {i}'))
        exp.append((i, f'seed {drw["st"] if drw else r["st"]}'))
        exp.append((i, f'rule {rule} {r["r8b"]} {1 if drw else 0} {r["st"]} {len(s70) - 1}'))
        for a in ranc:
            exp.append((i, f'anc {a["node"]} {a["depth"]} {a["d"]} {"-" if a["res"] is None else a["res"]}'))
        exp.append((i, f'pick {pk["idx"]} {r5["st"] if r5 else r["st"]}'))
        if rule == 4:
            exp.append((i, f'xr {len(x4["x"])}'))
        cnt['pick312' if rule in (3, 12) else 'pick479' if rule in (4, 7, 9) else 'pick1256'] += 1
        cnt['ranc'] += len(ranc)
        cnt['x'] += len(x4['x']) if x4 else 0
        cnt['r5'] += r5 is not None
        exp.append((i, f'work {pk["work"]}'))
        a0 = next(e for e in c['ev'] if e['e'] == 'anc')
        need(a0['w'] == 'sel', f'{where}: first 5fb13c is not the selected node')
        sel = c['ev'][-1]['node'][0]
        exp.append((i, f'sel {sel} {a0["d"]} {"-" if a0["res"] is None else a0["res"]}'))
        if thr is not None:
            exp.append((i, f'thr {thr["t20"]}'))
            cnt['thr_set'] += R.f(thr['t20']) < R.E100
        else:
            cnt['null_anc'] += 1
        ev = [e for e in c['ev'] if e['e'] in ('rm', 'ins')]
        k = 0
        while k < len(ev):
            e = ev[k]
            need(e['e'] == 'rm', f'{where}: insert without a release')
            if e['stage'] == 4:
                need(k + 1 < len(ev) and ev[k + 1]['e'] == 'ins' and ev[k + 1]['from'] == 'raw:5b47b7'
                     and ev[k + 1]['node'][0] == e['node'], f'{where}: stage-4 release without its heap insert')
                exp.append((i, f'rm {e["node"]} heap'))
                cnt['heap'] += 1
                k += 2
            else:
                need(e['stage'] == 0, f'{where}: release stage {e["stage"]}')
                exp.append((i, f'rm {e["node"]} drop'))
                cnt['drop'] += 1
                k += 1
        need(te['n'] == len(te['list']), f'{where}: tend n')
        exp.append((i, ' '.join(['tend', te['work']] + [str(x) for x in te['list']])))
        cnt['kept'] += len(te['list']) if thr is not None else 0
        cnt['draws'] += drw is not None
        cnt['calls'] += 1
    inp.append('end')
    return '\n'.join(inp) + '\n', exp, cnt


def compare(case, head, text, exp):
    """R/P/T: first difference with case, call, expected and Rust lines."""
    lines = text.rstrip('\n').split('\n')
    need(lines[0] == head, f'{case}: Rust output head {lines[0]!r} vs input {head!r}')
    out = lines[1:]
    for k, (call, want) in enumerate(exp):
        need(k < len(out), f'{case}: Rust output ends after {k} of {len(exp)} lines (call {call}: {want!r})')
        kind = {'call': 'X', 'seed': 'R', 'rule': 'R', 'anc': 'A', 'pick': 'P', 'xr': 'P', 'work': 'P'}.get(
            want.split()[0], 'T')
        need(out[k] == want, f'{kind} {case}: call {call}, output line {k}: original {want!r}, Rust {out[k]!r}')
    need(len(out) == len(exp), f'{case}: Rust output has {len(out)} lines, expected {len(exp)}')


def cargo(ws, target, din, dout):
    env = dict(os.environ, PLUNGE_IN=din, PLUNGE_OUT=dout, CARGO_TARGET_DIR=target,
               PATH=os.path.expanduser('~/.cargo/bin') + os.pathsep + os.environ.get('PATH', ''))
    shutil.rmtree(dout, ignore_errors=True)  # no stale trace can pass
    r = subprocess.run(['cargo', 'test', '--release', '-p', 'rsolver-mip', '--test', 'plunge_l1', '--', '--ignored'],
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

    def flip(hx):                        # last hex digit +1 (mod 16)
        return hx[:-1] + '%x' % ((int(hx[-1], 16) + 1) % 16)

    def rule_no(p):
        return ' '.join(['rule', '2'] + p[2:]) if p[:2] == ['rule', '3'] else None

    def rng(p):
        return ' '.join(p[:4] + [flip(p[4])] + p[5:]) if p[:1] == ['rule'] and p[3] == '1' else None

    def seed_(p):
        return f'seed {flip(p[1])}' if p[:1] == ['seed'] else None

    def pick_(p):
        return f'pick {int(p[1]) + 1} {p[2]}' if p[:1] == ['pick'] else None

    def pick_st(p):
        return f'pick {p[1]} {flip(p[2])}' if p[:1] == ['pick'] else None

    def anc_res(p):
        return ' '.join(p[:4] + [str(int(p[4]) + 1)]) if p[:1] == ['anc'] and p[4] != '-' else None

    def anc_del(p):
        return False if p[:1] == ['anc'] else None

    def xr(p):
        return f'xr {int(p[1]) + 1}' if p[:1] == ['xr'] else None

    def work(p):
        return f'work {flip(p[1])}' if p[:1] == ['work'] else None

    def dprime(p):
        return ' '.join(['sel', p[1], str(int(p[2]) + 1), p[3]]) if p[:1] == ['sel'] else None

    def anc(p):
        return ' '.join(['sel', p[1], p[2], str(int(p[3]) + 1)]) if p[:1] == ['sel'] and p[3] != '-' else None

    def thr(p):
        return f'thr {flip(p[1])}' if p[:1] == ['thr'] else None

    def rm_del(p):
        return False if p[:1] == ['rm'] else None

    def rm_swap(p):
        return ' '.join(p[:2] + ['drop']) if p[:1] == ['rm'] and p[2] == 'heap' else None

    def order(p):
        return ' '.join(p[:2] + [p[3], p[2]] + p[4:]) if p[:1] == ['tend'] and len(p) > 3 else None

    for name, case, fn in (('rule number', 'markshare_4_0', rule_no), ('RNG state after the draw', 'pk1', rng),
                           ('seed', 'neos5', seed_), ('pick index', 'tr12-30', pick_), ('pick work', 'pk1-pool', work),
                           ("d'", 'pk1', dprime), ('ancestor id', 'neos5', anc), ('threshold', 'pk1-pool', thr),
                           ('rm line deleted', 'markshare_4_0', rm_del), ('heap/drop swapped', 'tr12-30', rm_swap),
                           ('kept order', 'pk1', order), ('rule 9 pick index', 'pk1-bd1', pick_),
                           ('RNG state after the pick', 'markshare_4_0', pick_st), ('body ancestor result', 'pk1', anc_res),
                           ('body ancestor query deleted', 'neos5', anc_del), ('rule 4 x reads', 'tr12-30', xr)):
        why = outcomes(heads, exps, edit(case, fn))[case]
        need(why is not None, f'T: tamper not rejected: {name}')
        print(f'T: rejected: {name}: {why[:200]}')


def truncated(heads, exps, din):
    """Input tamper: one case without its closing `end` line must end in `panic` (caught per case) and be rejected."""
    tin, tout = os.path.join(WORK, 'trunc_in'), os.path.join(WORK, 'trunc_out')
    shutil.rmtree(tin, ignore_errors=True)
    shutil.copytree(din, tin)
    p = os.path.join(tin, 'neos5.input.txt')
    s = open(p).read()
    need(s.endswith('\nend\n'), 'T: input does not end with end')
    open(p, 'w').write(s[:-len('end\n')])
    cargo(ROOT, os.path.join(ROOT, 'target'), tin, tout)
    tr = traces(tout)
    res = outcomes(heads, exps, tr)
    need(tr['neos5'].endswith('\npanic\n') and res['neos5'], 'T: input without end not rejected by a caught panic')
    need(all(res[c] is None for c in CASES if c != 'neos5'), f'T: input without end changed other cases {res}')
    print(f'T: rejected: input without end (panic caught, other cases pass): {res["neos5"][:200]}')


def mutants(heads, exps, din):
    base = os.path.join(WORK, 'mutant')
    wsp = os.path.join(base, 'ws')
    shutil.rmtree(wsp, ignore_errors=True)
    os.makedirs(wsp)
    for p in ('Cargo.toml', 'Cargo.lock'):
        shutil.copy(os.path.join(ROOT, p), wsp)
    shutil.copytree(os.path.join(ROOT, 'crates'), os.path.join(wsp, 'crates'))
    orig = {f: open(os.path.join(ROOT, f)).read() for f in (SRC, TEST)}
    for name, fn, subs, want in MUTANTS:
        s = orig[fn]
        for a, b in subs:
            need(s.count(a) == 1, f'V: mutant {name}: pattern {a!r} found {s.count(a)} times')
            s = s.replace(a, b)
        for f in (SRC, TEST):
            with open(os.path.join(wsp, f), 'w') as fh:
                fh.write(s if f == fn else orig[f])
        dout = os.path.join(base, 'out')
        cargo(wsp, os.path.join(base, 'target'), din, dout)
        res = outcomes(heads, exps, traces(dout))
        for c in CASES:
            tag = 'rejected' if res[c] else 'passes'
            print(f'V: {name}: {c}: {tag}' + (f': {res[c][:160]}' if res[c] else ''))
            need(bool(res[c]) == want[c], f'V: mutant {name} on {c}: {tag}, section 3 expects '
                 f'{"rejected" if want[c] else "passes"}')
    shutil.rmtree(wsp)


def main():
    self_test = '--self-test' in sys.argv[1:]
    data = load_inputs()
    print(f'M: run 1 captures match the M2-06c-3 manifest ({len(data)} cases)')
    din, dout = os.path.join(WORK, 'in'), os.path.join(WORK, 'out')
    shutil.rmtree(din, ignore_errors=True)
    os.makedirs(din)
    heads, exps, tot = {}, {}, dict.fromkeys(TOTALS, 0)
    for c, (h, d) in data.items():
        text, exps[c], cnt = export(c, h, d)
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
        print(f'R/A/P/T: {c}: {len(exps[c])} lines: rust = original')
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
