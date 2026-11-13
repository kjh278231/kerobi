"""M1-02x checker: BTRAN 221173 sparse L^T test `(double)LU+0x140 < 0.1*m` (C:265-268) of the original vs rsolver.
Reads bench/oracle/m1_02x (tools/m1_lt_runs.sh): three captures of tools/m1_lt_path.js per model.
  controls  hashes (binary, model, tool), ok/DONE lines, 3 runs equal, events projected to the M1-02k fields equal the
            M1-02k plain capture (bench/oracle/m1_02k/<model>_plain.json.gz: same solve, no interference)
  original  the 0x12ec77 hit (reached when nnz >= 0 and LU+0x104*nnz <= m) carries the entry LU+0x140 and m; hit with
            L^T input nnz > 0: sparse output (nnz >= 0) iff LU+0x140 < 0.1m; hit with nnz 0: output nnz 0; no hit:
            dense output. Needs a target (hit, nnz > 0, LU+0x140 >= 0.1m) and a control (hit, nnz > 0,
            0 <= LU+0x140 < 0.1m)
  --rust    rsolver (PYTHONPATH) with RSOLVER_BTRAN_PROBE: inside the M1-02k same-state range (leaving r and row eta
            count agree, ends at the pinned LU+0x108 difference) every dual e_r call has Rust `lt` = LU+0x140 and the
            same output form, idx order and value bits (pinned pre-existing differences: KNOWN_VAL, KNOWN_IDX); the
            range holds at least one target call
  --self-test  also tampers with the loaded data and requires each tamper to be rejected
usage: python3 tools/m1_lt_check.py [--rust] [--self-test]"""
import copy, gzip, hashlib, json, os, subprocess, sys, tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DIR = os.path.join(ROOT, 'bench/oracle/m1_02x')
K02 = os.path.join(ROOT, 'bench/oracle/m1_02k')
NETLIB = os.path.join(ROOT, '../re/bench/data/netlib')
BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
MODELS = ['stair', 'degen2', 'fit1p', 'bnl1']
ER = (376, 0xc51dd)
# M1-02k pinned first e_r call whose LU+0x108 differs after a refactor (M1-02w): the same-state range ends there
KNOWN_D = {'stair': 185, 'bnl1': 174}
# where the r / row eta count match ends without a pinned LU+0x108 call (a shorter range fails)
RANGE = {'degen2': 356, 'fit1p': 117}
# Pre-existing e_r output differences inside the range, the same with the LU+0x140 test removed (M1-02x §6,
# bench/oracle/m1_02x/diffs_*.txt). The stair values (from call 31) were the FT new diagonal's eta alignment, fixed in
# M1-02y; left: bnl1 level-path order (same rows; the moved row 587 differs by <= 9e-16 relative in 4 of 7), the
# refactor numbering of M1-02z. Pinned exactly: a pinned call that agrees or any other difference fails. Form and
# LU+0x140 are compared on every call.
KNOWN_VAL = {}
KNOWN_IDX = {'bnl1': [130, 134, 144, 153, 159, 162, 173]}
WRAP, M, NNZ, X180, R, OUT, X140, HIT, IDX, VAL = 0, 1, 2, 5, 8, 10, 11, 12, 13, 14


class Fail(Exception):
    pass


def need(c, msg):
    if not c:
        raise Fail(msg)


def sha(p):
    return hashlib.sha256(open(p, 'rb').read()).hexdigest()


def load():
    return {n: {'caps': [json.load(gzip.open(f'{DIR}/{n}_{i}.json.gz')) for i in (1, 2, 3)],
                'k02': json.load(gzip.open(f'{K02}/{n}_plain.json.gz'))['events']} for n in MODELS}


def first(a, b):
    return next((i for i, (x, y) in enumerate(zip(a, b)) if x != y), None if len(a) == len(b) else min(len(a), len(b)))


def controls(n, d):
    tool, model = sha(os.path.join(ROOT, 'tools/m1_lt_path.js')), sha(f'{NETLIB}/{n}.mps.gz')
    c1 = d['caps'][0]
    for i, c in enumerate(d['caps'], 1):
        need(c['binary_sha256'] == BINARY, f'{n} run {i}: binary sha {c["binary_sha256"]}')
        need(c['model_sha256'] == model, f'{n} run {i}: model sha')
        need(c['tool_sha256'] == tool, f'{n} run {i}: tool sha (capture made by another tool version)')
        need(c['ok'] and not c['errors'] and c['output'].startswith('DONE'), f'{n} run {i}: not ok: {c["output"]} {c["errors"]}')
        need(c['params'] == {'Threads': 1, 'Method': 1, 'Presolve': 0, 'OutputFlag': 0}, f'{n} run {i}: params')
        need(c['output'] == c1['output'], f'{n} run {i}: output {c["output"]} != run 1 {c1["output"]}')
        j = first(c1['events'], c['events'])
        need(j is None, f'{n} run {i}: event {j} differs from run 1')
    ev = c1['events']
    need(ev, f'{n}: no events')
    j = first([e[:11] for e in ev], d['k02'])
    need(j is None, f'{n}: event {j} projected to the M1-02k fields differs from M1-02k plain '
                    f'{ev[j][:11] if j is not None and j < len(ev) else None} vs {d["k02"][j] if j is not None and j < len(d["k02"]) else None}')


def original(n, d):
    tgt = ctl = 0
    for j, e in enumerate(d['caps'][0]['events']):
        h, m, out = e[HIT], e[M], e[OUT]
        er = tuple(e[WRAP][:2]) == ER
        need(len(e) == (15 if er else 13), f'{n}: event {j} has {len(e)} fields (e_r: 15, else 13)')
        if h is None:
            need(out < 0, f'{n}: event {j}: no 0x12ec77 hit (nnz < 0 or LU+0x104*nnz > m) but output nnz {out}')
            continue
        nnz, hm, x140 = h
        need(hm == m and x140 == e[X140], f'{n}: event {j}: hit m {hm} / LU+0x140 {x140} vs entry {m} / {e[X140]}')
        if nnz == 0:
            need(out == 0, f'{n}: event {j}: L^T input nnz 0, output nnz {out}')
            continue
        need(nnz > 0, f'{n}: event {j}: hit with nnz {nnz}')
        sparse = x140 < 0.1 * m
        need((out >= 0) == sparse, f'{n}: event {j}: LU+0x140 {x140} m {m} says sparse={sparse}, output nnz {out}')
        tgt, ctl = tgt + (not sparse), ctl + (sparse and x140 >= 0)
    return tgt, ctl


def rust_run(n, outdir):
    spec = os.path.join(outdir, f'{n}.spec')
    open(spec, 'w').close()
    code = (f"import rsolver; m=rsolver.read({os.path.join(NETLIB, n + '.mps.gz')!r}); m.Params['Method']=1; "
            "m.Params['Presolve']=0; m.optimize()")
    subprocess.run([sys.executable, '-c', code], env=dict(os.environ, RSOLVER_BTRAN_PROBE=spec), check=True)
    return [r for r in map(json.loads, open(spec + '.out')) if 'r' in r]


def rust(n, d, nat):
    er = [e for e in d['caps'][0]['events'] if tuple(e[WRAP][:2]) == ER]
    matched = next((k for k, (e, r) in enumerate(zip(er, nat)) if (e[R], e[X180]) != (r['r'], r['e'])), min(len(er), len(nat)))
    if n in KNOWN_D:
        need(KNOWN_D[n] <= matched, f'{n}: same-state range ends at e_r call {matched}, before the pinned LU+0x108 call {KNOWN_D[n]}')
        matched = KNOWN_D[n]
    else:
        need(matched == RANGE[n], f'{n}: same-state range ends at e_r call {matched}, pinned {RANGE[n]}')
    tgt = 0
    for k in range(matched):
        e, r = er[k], nat[k]
        need(r['lt'] == e[X140], f'{n}: e_r call {k}: Rust LU+0x140 {r["lt"]}, original {e[X140]}')
        dense = e[OUT] < 0
        need(r['od'] == dense, f'{n}: e_r call {k}: Rust output dense={r["od"]}, original dense={dense} '
                               f'(LU+0x140 {e[X140]}, m {e[M]}, hit {e[HIT]})')
        if k in KNOWN_IDX.get(n, []):
            need(not dense and e[IDX] != r['idx'] and sorted(e[IDX]) == sorted(r['idx']),
                 f'{n}: e_r call {k}: pinned order difference not seen (same order, or other rows)')
            continue
        if k in KNOWN_VAL.get(n, []):
            need(dense or e[IDX] == r['idx'], f'{n}: e_r call {k}: output idx differs at a value-only pinned call')
            need(e[VAL] != r['val'], f'{n}: e_r call {k}: pinned value difference not seen')
            continue
        if not dense:
            i = first(e[IDX], r['idx'])
            if i is not None:
                raise Fail(f'{n}: e_r call {k} output idx {i}: original {e[IDX][i:i + 4]} Rust {r["idx"][i:i + 4]}')
        i = first(e[VAL], r['val'])
        if i is not None:
            raise Fail(f'{n}: e_r call {k} output value {i}: original {e[VAL][i] if i < len(e[VAL]) else None} '
                       f'Rust {r["val"][i] if i < len(r["val"]) else None}')
        h = e[HIT]
        tgt += h is not None and h[0] > 0 and h[2] >= 0.1 * e[M]
    return matched, len(er), tgt


def check(data, routs):
    res = {}
    for n in MODELS:
        controls(n, data[n])
        res[n] = [len(data[n]['caps'][0]['events']), *original(n, data[n])]
        if routs is not None:
            res[n].extend(rust(n, data[n], routs[n]))
    need(sum(v[1] for v in res.values()) > 0, 'no target event (hit, nnz > 0, LU+0x140 >= 0.1m)')
    need(sum(v[2] for v in res.values()) > 0, 'no control event (hit, nnz > 0, 0 <= LU+0x140 < 0.1m)')
    if routs is not None:
        need(sum(v[5] for v in res.values()) > 0, 'no target e_r call inside the Rust same-state range')
    return res


def tampers(data, routs):
    def t(f):
        def g():
            d, r = copy.deepcopy(data), copy.deepcopy(routs)
            f(d, r)
            return d, r
        return g

    def each(d, pred, fn):
        for n in MODELS:
            for c in d[n]['caps']:
                for e in c['events']:
                    if e[HIT] is not None and pred(e):
                        fn(e)

    is_tgt = lambda e: e[HIT][0] > 0 and e[HIT][2] >= 0.1 * e[M]
    is_ctl = lambda e: e[HIT][0] > 0 and 0 <= e[HIT][2] < 0.1 * e[M]

    def zero_x140(e):
        e[X140] = 0
        e[HIT][2] = 0

    def neg_x140(e):
        e[X140] = -1
        e[HIT][2] = -1

    def no_control(d, r):  # every control's LU+0x140 set to -1, and the Rust lt of the same e_r calls with it
        each(d, is_ctl, neg_x140)
        for n in MODELS:
            er = [e for e in d[n]['caps'][0]['events'] if tuple(e[WRAP][:2]) == ER]
            for e, x in zip(er, r[n]):
                x['lt'] = e[X140]

    stair = lambda r, k: r['stair'][k]

    def pinned_agrees(d, r):  # the original's bnl1 e_r 130 output set to Rust's in every run
        for c in d['bnl1']['caps']:
            e = [x for x in c['events'] if tuple(x[WRAP][:2]) == ER][130]
            e[IDX], e[VAL] = list(r['bnl1'][130]['idx']), list(r['bnl1'][130]['val'])
    out = {
        'binary sha': t(lambda d, r: d['degen2']['caps'][1].update(binary_sha256='0' * 64)),
        'empty events': t(lambda d, r: [c.update(events=[]) for c in d['fit1p']['caps']]),
        'run 3 event': t(lambda d, r: d['bnl1']['caps'][2]['events'][7].__setitem__(OUT, 5)),
        'projection (interference)': t(lambda d, r: [c['events'][3].__setitem__(9, 99) for c in d['stair']['caps']]),
        'LU+0x140 zeroed': t(lambda d, r: each(d, lambda e: True, zero_x140)),
        'no target event': t(lambda d, r: each(d, is_tgt, lambda e: e.__setitem__(HIT, None))),
        'no control event': t(no_control),
        'hit m': t(lambda d, r: each(d, is_ctl, lambda e: e[HIT].__setitem__(1, e[M] + 1))),
    }
    if routs is not None:
        out.update({
            'Rust LU+0x140': t(lambda d, r: stair(r, 102).update(lt=40)),
            'Rust target form (old rule)': t(lambda d, r: [stair(r, k).update(od=False) for k in (102, 118)]),
            'Rust value bits': t(lambda d, r: stair(r, 118)['val'].__setitem__(5, 'deadbeef00000000')),
            'Rust idx order': t(lambda d, r: next(x for x in r['degen2'] if not x['od'] and len(x['idx']) > 1)['idx'].reverse()),
            'Rust etas': t(lambda d, r: r['fit1p'][3].update(e=99)),
            'pinned call agrees': t(pinned_agrees),
            'pinned order call: other rows': t(lambda d, r: r['bnl1'][130]['idx'].__setitem__(0, 9999)),
            'no target in range': t(lambda d, r: [[e.__setitem__(HIT, None) for e in
                                                   [x for x in c['events'] if tuple(x[WRAP][:2]) == ER][:KNOWN_D['stair']]
                                                   if e[HIT] is not None and is_tgt(e)] for c in d['stair']['caps']]),
        })
    return out


def main():
    a = sys.argv[1:]
    data = load()
    routs = None
    if '--rust' in a or '--self-test' in a:
        tmp = tempfile.mkdtemp(prefix='m1_02x_')
        routs = {n: rust_run(n, tmp) for n in MODELS}
    try:
        res = check(data, routs)
    except Fail as e:
        print('FAIL', e)
        sys.exit(1)
    print('model events target control' + (' rust_matched/er target_in_range' if routs is not None else ''))
    for n, v in res.items():
        print(n, *v)
    print('PASS', 'original' + (' + rust' if routs is not None else ''))
    if '--self-test' in a:
        bad = 0
        for name, f in tampers(data, routs).items():
            d, r = f()
            try:
                check(d, r)
                print('NOT REJECTED', name)
                bad += 1
            except Fail as e:
                print('rejected', name, '|', str(e)[:150])
        if bad:
            sys.exit(1)
        print('SELF-TEST PASS')


main()
