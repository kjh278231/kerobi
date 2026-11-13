#!/usr/bin/env python3
"""M1-02w-3: the M1 refactor's dense switch and pool size F+0x158 around every 036ade of a Method=1 run, original
(tools/m1_dense_state.js captures) against Rust. Contract: docs/m1/m1_02w3_dense.md section 3. Exit 0 only when every
requested check passes.
  --repro      <model>-{1,2,3}.json.gz agree exactly (events incl. diagonals, output); identity; F+0x158 life:
               first entry = 045a76 FactSpace (k 5, c 0), next entry = Grow size / x1.5 after -1, else the exit size
  --replay     every original call of run 1 into Rust Lu::factor_dense (cargo test m1_dense_replay) with its head, block
               state, tol and F+0x158: next F+0x158, block exit, LU+0xd8/0xe4/0x100, dense n, local_70, diagonal bits
  --rust       this worktree's build (PYTHONPATH=$PWD/python, RSOLVER_BLOCK_PROBE) up to the first head difference:
               the same fields except the diagonal plus the entry block state and F+0x158
  --self-test  input mutants of the replay (cap 1,000,000: dense array alignment; cap x10: no shortage) and tampers of
               the original data must be rejected
  --dir D      capture directory (default bench/oracle/m1_02w3/formal)
"""
import copy, glob, gzip, hashlib, json, os, subprocess, sys, tempfile

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
sys.path.insert(0, os.path.join(ROOT, 'tools'))
from m1_factspace import factspace  # noqa: E402

BIN = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
PY = os.path.join(ROOT, '../.venv/bin/python')
NETLIB = os.path.join(ROOT, '../re/bench/data/netlib')
args = sys.argv[1:]
D = os.path.join(ROOT, args[args.index('--dir') + 1]) if '--dir' in args else os.path.join(ROOT, 'bench/oracle/m1_02w3/formal')
MODELS = ['grow15', 'grow22', 'perold', 'pilot4', 'stair']
# Rust calls compared before the first head difference (M1-02w-3 result): fewer means the probe or build broke
NEED = {'grow15': 0, 'grow22': 0, 'perold': 0, 'pilot4': 0, 'stair': 0}
sha = lambda p: hashlib.sha256(open(p, 'rb').read()).hexdigest()


def load(path):
    return json.load(gzip.open(path))


def calls(c):
    """036ade calls in order with the diagonal read after it (None when not read or not fresh)."""
    out = []
    for e in c['events']:
        if 'diag_of' in e:
            assert e['diag_of'] == out[-1]['i'], 'diagonal not after its call'
            out[-1]['diag'] = e['diag'] if e['upd'] == 0 else None
        else:
            out.append(dict(e, diag=None))
    return out


def next_cap(e):
    """F+0x158 of the next 036ade call when this one returned -1 (05b257 C:1479-1513), else None."""
    if e['rc'] != -1:
        return None
    p, q = e['pool_leave']
    return p if p == q else int(e['pool'][0] * 1.5)


def nnz(model):
    import rsolver
    return rsolver.read(os.path.join(NETLIB, model + '.mps.gz')).NumNZs


def check_capture(c, name):
    if c.get('binary_sha256') != BIN: return f'{name}: binary hash'
    if c.get('model_sha256') != sha(os.path.join(NETLIB, c['model'])): return f'{name}: model hash'
    if not c.get('ok') or c.get('errors'): return f'{name}: run not ok'
    ev = calls(c)
    if not ev: return f'{name}: no calls'
    want = factspace(ev[0]['m'], ev[0]['n'], nnz(name.rsplit('-', 1)[0]))
    if ev[0]['pool'][0] != want: return f'{name}: first F+0x158 {ev[0]["pool"][0]}, FactSpace {want}'
    for i, e in enumerate(ev):
        if e['rc'] not in (0, -1): return f'{name}: call {i} rc {e["rc"]}'
        if e['pool'][1:] != [0, e['pool'][0] - 1]: return f'{name}: call {i} F+0x160/0x168 {e["pool"]}'
        if i + 1 < len(ev):
            nx = next_cap(e)
            want = e['pool_leave'][0] if nx is None else nx
            if ev[i + 1]['pool'][0] != want: return f'{name}: call {i + 1} F+0x158 {ev[i + 1]["pool"][0]}, rule {want}'
    return None


def captures():
    runs = {}
    for p in sorted(glob.glob(os.path.join(D, '*-[123].json.gz'))):
        n, r = os.path.basename(p)[:-8].rsplit('-', 1)
        runs.setdefault(n, {})[int(r)] = load(p)
    return runs


def repro(runs):
    bad = []
    if sorted(runs) != MODELS: bad.append(f'models {sorted(runs)}')
    for n, rr in sorted(runs.items()):
        if sorted(rr) != [1, 2, 3]: bad.append(f'{n}: runs {sorted(rr)}'); continue
        for r, c in rr.items():
            e = check_capture(c, f'{n}-{r}')
            if e: bad.append(e)
        for r in (2, 3):
            if rr[r]['events'] != rr[1]['events'] or rr[r]['output'] != rr[1]['output']:
                bad.append(f'{n}: run {r} differs from run 1')
    return bad


def original_fields(e):
    return dict(next=next_cap(e), leave=e['leave'], lu=[e['lu'][0], e['lu'][3], e['lu'][4]],
                dense=e['dense'], mode70=e['mode70'] > 0)


def replay(runs, cap=None):
    """All calls of run 1 of every model; cap(e) overrides the injected F+0x158 (self-test). Returns (failures, notes)."""
    with tempfile.TemporaryDirectory() as t:
        src, dst, order = os.path.join(t, 'in.txt'), os.path.join(t, 'out.txt'), []
        with open(src, 'w') as f:
            for n, rr in sorted(runs.items()):
                f.write(f'model {os.path.join(NETLIB, n + ".mps.gz")}\n')
                for e in calls(rr[1]):
                    c = cap(e) if cap else e['pool'][0]
                    f.write(' '.join(map(str, e['enter'] + [e['tol'], c] + e['head'])) + '\n'); order.append((n, e))
        env = dict(os.environ, DENSE_IN=src, DENSE_OUT=dst)
        subprocess.run(['cargo', 'test', '--release', '-q', '-p', 'rsolver-lp', '--test', 'm1_dense_replay', '--', '--ignored'],
                       cwd=ROOT, env=env, check=True, capture_output=True)
        res = [l.split() for l in open(dst) if not l.startswith('model ')]
    fail, notes = [], {}
    if len(res) != len(order): fail.append(f'replay: {len(res)} results for {len(order)} calls')
    for (n, e), r in zip(order, res):
        c = notes.setdefault(n, dict(calls=0, equal=0, diag=0, dense=0, short=0, mode70=0, panic=0, first=None))
        if r[0] == 'panic':
            c['calls'] += 1; c['panic'] += 1
            if c['first'] is None: c['first'] = f'call {e["i"]} (k {e["k"]}): Rust panic: {" ".join(r[1:])}'
            continue
        v = list(map(int, r[:10]))
        got = dict(next=None if v[0] == -1 else v[0], leave=v[1:3], lu=[v[3], v[6], v[7]],
                   dense=[] if v[8] == -1 else [v[8]], mode70=v[9] == 1)
        want = original_fields(e)
        c['calls'] += 1
        c['dense'] += bool(e['dense']); c['short'] += e['rc'] == -1; c['mode70'] += e['mode70'] > 0
        diff = [f'{k}: original {want[k]} Rust {got[k]}' for k in want if want[k] != got[k]]
        if not diff and e['diag'] is not None:
            c['diag'] += 1
            if r[10:] != e['diag']:
                p = next((j for j, (a, b) in enumerate(zip(e['diag'], r[10:])) if a != b), min(len(r) - 10, len(e['diag'])))
                diff.append(f'diag position {p}: original {e["diag"][p] if p < len(e["diag"]) else None} '
                            f'Rust {r[10 + p] if p < len(r) - 10 else None} ({len(e["diag"])} vs {len(r) - 10})')
        if not diff: c['equal'] += 1
        elif c['first'] is None: c['first'] = f'call {e["i"]} (k {e["k"]}, F+0x158 {e["pool"][0]}): ' + '; '.join(diff)
    fail += [f'replay {n}: {c["first"]}' for n, c in notes.items() if c['first']]
    return fail, notes


def rust_probe(model):
    with tempfile.TemporaryDirectory() as t:
        out = os.path.join(t, 'probe.jsonl')
        env = dict(os.environ, RSOLVER_BLOCK_PROBE=out)
        code = (f"import rsolver; m=rsolver.read({os.path.join(NETLIB, model + '.mps.gz')!r}); m.Params['Method']=1; "
                "m.Params['Presolve']=0; m.optimize()")
        subprocess.run([PY, '-c', code], env=env, check=True, capture_output=True)
        return [json.loads(l) for l in open(out)] if os.path.exists(out) else []


def compare(name, orig, rust):
    """Call by call up to the first head difference: entry block state and F+0x158, then the replay fields."""
    n = 0
    for i, (o, r) in enumerate(zip(orig, rust)):
        if not (o['head'] == r['head'] or (i == 0 and sorted(o['head']) == sorted(r['head']))):
            return None, n, f'{name}: head differs at call {i} (original k {o["k"]}, Rust iter {r["iter"]}); {n} compared'
        want = dict(original_fields(o), enter=o['enter'], cap=o['pool'][0])
        got = dict(next=r['short'], leave=r['leave'], lu=[r['lu'][0], r['lu'][3], r['lu'][4]],
                   dense=[] if r['dense'] is None else [r['dense']], mode70=r['mode70'], enter=r['enter'], cap=r['cap'])
        diff = [f'{k}: original {want[k]} Rust {got[k]}' for k in want if want[k] != got[k]]
        if diff:
            return f'{name}: call {i} (k {o["k"]}): ' + '; '.join(diff), n, None
        n += 1
    return None, n, f'{name}: {n}/{len(orig)} calls equal' + ('' if len(orig) == len(rust) else f' (Rust {len(rust)})')


def main():
    runs = captures()
    fail = []
    if '--repro' in args:
        b = repro(runs)
        fail += b
        for n, rr in sorted(runs.items()):
            ev = calls(rr[1])
            print(f'{n}: {len(ev)} calls, dense {sum(bool(e["dense"]) for e in ev)}, local_70 {sum(e["mode70"] > 0 for e in ev)}, '
                  f'-1 {sum(e["rc"] == -1 for e in ev)}, F+0x158 {sorted({e["pool"][0] for e in ev})}, '
                  f'diagonals {sum(e["diag"] is not None for e in ev)}')
        print('repro', 'FAIL' if b else 'PASS', *b[:5])
    if '--replay' in args:
        b, notes = replay(runs)
        fail += b
        for n, c in sorted(notes.items()):
            print(f'replay {n}: {c["equal"]}/{c["calls"]} calls equal (dense {c["dense"]}, -1 {c["short"]}, local_70 {c["mode70"]}, '
                  f'diagonals compared {c["diag"]}, Rust panics {c["panic"]})' + (f' FIRST {c["first"]}' if c['first'] else ''))
        print('replay', 'FAIL' if b else 'PASS')
    if '--rust' in args:
        for n in MODELS:
            if n not in runs: continue
            err, cnt, note = compare(n, calls(runs[n][1]), rust_probe(n))
            if err: fail.append(err); print('rust FAIL', err)
            else:
                print('rust', note)
                if cnt < NEED[n]: fail.append(f'{n}: {cnt} calls compared, need {NEED[n]}')
    if '--self-test' in args:
        for name, cap in [('cap 1000000', lambda e: 1000000), ('cap x10', lambda e: e['pool'][0] * 10)]:
            b, _ = replay(runs, cap)
            print(f'mutant {name}: {len(b)} models rejected', b[0][:200] if b else '')
            if not b: fail.append(f'mutant {name} not rejected')
        n = MODELS[0]
        for name, f in [('binary hash', lambda c: c.update(binary_sha256='0' * 64)),
                        ('first F+0x158', lambda c: c['events'][0]['pool'].__setitem__(0, c['events'][0]['pool'][0] + 1)),
                        ('event changed', lambda c: c['events'][-1].__setitem__('rc', 5))]:
            t = copy.deepcopy(runs)
            f(t[n][2])
            if not repro(t): fail.append(f'tamper {name} not rejected')
            else: print(f'tamper {name}: rejected')
    print('M1_DENSE', 'FAIL' if fail else 'PASS')
    for f in fail[:10]: print(' ', f)
    return 1 if fail else 0


if __name__ == '__main__':
    sys.exit(main())
