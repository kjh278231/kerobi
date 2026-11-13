#!/usr/bin/env python3
"""M1-01e: the 1da0b2 2x2 block-phase state on the simplex struct (S+0x2c0/0x4e0/0x4e4) around every 036ade of a
Method=1 run, original (tools/m1_block_state.js captures) against Rust (RSOLVER_BLOCK_PROBE). Contract:
docs/m1/m1_01e_block_on.md section 3. Exit 0 only when every check passes.
  --repro      the formal captures <model>-{1,2,3}.json.gz agree exactly (events, output) and pass the chain check
  --rust       run this worktree's build (PYTHONPATH=$PWD/python) and compare up to the first basis-head difference
  --replay     inject every original factor (head, entry state) into Rust Lu::factor_block (cargo test m1_block_replay)
               and compare the exit state and LU+0xd8/0xe4/0x100 for all factors, independent of the trajectory
  --lu         also compare LU+0xd8/0xe4/0x100 (t) in that range (stage 2, t = s3)
  --self-test  tampers of the original data must be rejected
  --dir D      capture directory (default bench/oracle/m1_01e/formal)
"""
import copy, glob, gzip, hashlib, json, os, subprocess, sys, tempfile

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
BIN = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
TOOL = os.path.join(ROOT, 'tools/m1_block_state.js')
PY = os.path.join(ROOT, '../.venv/bin/python')
NETLIB = os.path.join(ROOT, '../re/bench/data/netlib')
args = sys.argv[1:]
D = os.path.join(ROOT, args[args.index('--dir') + 1]) if '--dir' in args else os.path.join(ROOT, 'bench/oracle/m1_01e/formal')
# Rust factors compared before the first head difference (M1-01e result): fewer means the probe or build broke
NEED = {'25fv47': 6, 'bnl1': 6, 'degen2': 9, 'grow15': 5, 'grow22': 8, 'kb2': 5, 'maros': 11, 'perold': 7, 'scfxm2': 7, 'stair': 7}
sha = lambda p: hashlib.sha256(open(p, 'rb').read()).hexdigest()


def load(path):
    return json.load(gzip.open(path))


def check_capture(c, name):
    """Identity, success and the S chain: one S, start (1,0,0), each next entry = (x2c0 + 1, previous exit) or, for
    a 036ade retry inside one 05b257 (pool shortage -1), the same x2c0 and the previous exit."""
    if c.get('binary_sha256') != BIN: return f'{name}: binary hash'
    if c.get('model_sha256') != sha(os.path.join(NETLIB, c['model'])): return f'{name}: model hash'
    if not c.get('ok') or c.get('errors'): return f'{name}: run not ok'
    ev = c['events']
    if not ev: return f'{name}: no events'
    if {e['s'] for e in ev} != {0}: return f'{name}: more than one S'
    if ev[0]['enter'] != [1, 0, 0]: return f'{name}: start {ev[0]["enter"]}'
    for i in range(1, len(ev)):
        a, b = ev[i - 1], ev[i]
        retry = a['rc'] != 0 and b['enter'] == [a['enter'][0]] + a['leave']
        if not retry and b['enter'] != [a['enter'][0] + 1] + a['leave']:
            return f'{name}: chain at call {i}: previous {a["enter"]}->{a["leave"]} rc {a["rc"]}, entry {b["enter"]}'
    return None


def groups(ev):
    """One entry per 05b257 factorization: retries after rc != 0 fold into the first call's entry and the last exit."""
    out = []
    for e in ev:
        if out and out[-1]['_rc'] != 0 and e['enter'][0] == out[-1]['enter'][0]:
            out[-1].update(leave=e['leave'], head=e['head'], lu=e['lu'], _rc=e['rc'], retries=out[-1]['retries'] + 1)
        else:
            out.append(dict(k=e['k'], enter=e['enter'], leave=e['leave'], head=e['head'], lu=e['lu'], _rc=e['rc'], retries=0))
    return out


def captures():
    runs = {}
    for p in sorted(glob.glob(os.path.join(D, '*-[123].json.gz'))):
        n, r = os.path.basename(p)[:-8].rsplit('-', 1)
        runs.setdefault(n, {})[int(r)] = load(p)
    return runs


def repro(runs):
    bad = []
    for n, rr in sorted(runs.items()):
        if sorted(rr) != [1, 2, 3]: bad.append(f'{n}: runs {sorted(rr)}'); continue
        for r, c in rr.items():
            e = check_capture(c, f'{n}-{r}')
            if e: bad.append(e)
        for r in (2, 3):
            if rr[r]['events'] != rr[1]['events'] or rr[r]['output'] != rr[1]['output']:
                bad.append(f'{n}: run {r} differs from run 1')
    return bad


def rust_probe(model):
    with tempfile.TemporaryDirectory() as t:
        out = os.path.join(t, 'probe.jsonl')
        env = dict(os.environ, RSOLVER_BLOCK_PROBE=out)
        code = (f"import rsolver; m=rsolver.read({os.path.join(NETLIB, model + '.mps.gz')!r}); m.Params['Method']=1; "
                "m.Params['Presolve']=0; m.optimize()")
        subprocess.run([PY, '-c', code], env=env, check=True, capture_output=True)
        return [json.loads(l) for l in open(out)] if os.path.exists(out) else []


def compare(name, orig, rust, lu):
    """Up to the first head difference: entry/exit state (and with lu the LU fields) exactly equal."""
    og, n = groups(orig), 0
    for i, (o, r) in enumerate(zip(og, rust)):
        same = o['head'] == r['head'] or (i == 0 and sorted(o['head']) == sorted(r['head']))
        if not same:
            return None, n, f'{name}: head differs at factor {i} (original k {o["k"]}, Rust iter {r["iter"]}); {n} compared'
        for f in ('enter', 'leave') + (('lu',) if lu else ()):
            a, b = o[f], r[f]
            if f == 'lu': a, b = [a[0], a[3], a[4]], [b[0], b[3], b[4]]  # 0xd8, 0xe4, 0x100
            if a != b:
                return f'{name}: factor {i} (k {o["k"]}) {f}: original {a} Rust {b}', n, None
        n += 1
    note = f'{name}: {n}/{len(og)} factors equal' + ('' if len(og) == len(rust) else f' (Rust {len(rust)})')
    return None, n, note


def replay(runs):
    """All factors of run 1 of every model: exit state, LU fields; returns (failures, per-model notes)."""
    with tempfile.TemporaryDirectory() as t:
        src, dst, order = os.path.join(t, 'in.txt'), os.path.join(t, 'out.txt'), []
        with open(src, 'w') as f:
            for n, rr in sorted(runs.items()):
                f.write(f'model {os.path.join(NETLIB, n + ".mps.gz")}\n')
                for e in rr[1]['events']:
                    f.write(' '.join(map(str, e['enter'] + e['head'])) + '\n'); order.append((n, e))
        env = dict(os.environ, BLOCK_IN=src, BLOCK_OUT=dst)
        subprocess.run(['cargo', 'test', '--release', '-q', '-p', 'rsolver-lp', '--test', 'm1_block_replay', '--', '--ignored'],
                       cwd=ROOT, env=env, check=True, capture_output=True)
        res = [list(map(int, l.split())) for l in open(dst) if not l.startswith('model ')]
    fail, notes = [], {}
    for (n, e), r in zip(order, res):
        o = e['leave'] + [e['lu'][0], e['lu'][3], e['lu'][4]]
        r = r[:2] + [r[2], r[5], r[6]]
        if '--lu' not in args: o, r = o[:4], r[:4]  # t (LU+0x100) only in stage 2
        c = notes.setdefault(n, dict(factors=0, equal=0, first=None))
        c['factors'] += 1
        if o == r: c['equal'] += 1
        elif c['first'] is None: c['first'] = f'k {e["k"]} enter {e["enter"]}: original (x4e0,x4e4,d8,e4,t) {o} Rust {r}'
    if len(res) != len(order): fail.append(f'replay: {len(res)} results for {len(order)} factors')
    fail += [f'replay {n}: {c["first"]}' for n, c in notes.items() if c['first']]
    return fail, notes


def coverage(orig, upto):
    g = groups(orig)[:upto]
    return dict(run=sum(e['leave'][0] != e['enter'][1] for e in g), inc=sum(e['leave'][1] not in (e['enter'][2], -1, 2000000000) for e in g),
                off=sum(e['leave'][1] == -1 != e['enter'][2] for e in g), always=sum(e['leave'][1] == 2000000000 != e['enter'][2] for e in g),
                retry=sum(e['retries'] for e in g))


def main():
    runs, fail = captures(), []
    if not runs: print('no captures in', D); return 1
    if '--repro' in args:
        b = repro(runs); fail += b
        print('repro:', 'ok' if not b else b, f'({len(runs)} models x 3)')
    if '--self-test' in args:
        rr = next(iter(runs.values()))
        tampers = {
            'binary': lambda c: c.update(binary_sha256='0' * 64),
            'empty': lambda c: c.update(events=[]),
            'x2c0': lambda c: c['events'][1]['enter'].__setitem__(0, c['events'][1]['enter'][0] + 1),
            'x4e0': lambda c: c['events'][-1]['enter'].__setitem__(1, c['events'][-1]['enter'][1] + 1),
            'x4e4': lambda c: c['events'][-1]['enter'].__setitem__(2, 7),
            'start': lambda c: c['events'][0]['enter'].__setitem__(0, 0),
            'second S': lambda c: c['events'][-1].update(s=1),
            'not ok': lambda c: c.update(ok=False),
        }
        for t, f in tampers.items():
            c = copy.deepcopy(rr[1]); f(c)
            if not check_capture(c, t): fail.append(f'tamper {t} accepted')
        c = copy.deepcopy(rr); c[2]['events'][0]['k'] += 1
        if not repro({'x': c}): fail.append('tamper run difference accepted')
        print('self-test:', len(tampers) + 1, 'tampers', 'rejected' if not any(x.startswith('tamper') for x in fail) else 'NOT all rejected')
    if '--replay' in args:
        b, notes = replay(runs); fail += b
        for n, c in sorted(notes.items()):
            g = runs[n][1]['events']
            print(f'replay {n}: {c["equal"]}/{c["factors"]} equal', coverage(g, len(g)), c['first'] or '')
    if '--rust' in args:
        for n, rr in sorted(runs.items()):
            orig = rr[1]['events']
            e = check_capture(rr[1], n)
            if e: fail.append(e); continue
            rust = rust_probe(n)
            bad, upto, note = compare(n, orig, rust, '--lu' in args)
            if not bad and upto < NEED.get(n, 1): bad = f'{n}: only {upto} factors compared (need {NEED.get(n, 1)}, Rust lines {len(rust)})'
            if bad: fail.append(bad); print('FAIL', bad); continue
            print(note, coverage(orig, upto))
    print('FAIL' if fail else 'PASS', *fail[:20], sep='\n' if fail else ' ')
    return 1 if fail else 0


if __name__ == '__main__':
    sys.exit(main())
