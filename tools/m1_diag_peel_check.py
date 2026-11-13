"""M1-02y checker: alignment of the eta values the FT new diagonal reads (05b257 C:321-372, LU+0x1d0) and the new
diagonal of every update, original vs rsolver. Reads bench/oracle/m1_02y (tools/m1_eta_align_runs.sh): three
captures of tools/m1_eta_align.js per model.
  controls  hashes (binary, model, tool), ok/DONE lines, params, 3 runs equal
  original  at every update after the first factor (refactor count > 0): one LU+0x1d0 allocation per run; idx - W =
            24I + 8 + 8(m+n) + 4m and val - idx = 4m (045a76 C:274-296, I = spx+0x3c8 of the first update, n =
            spx+0x68); W and the pool LU+0x150 16-byte aligned; values 8 mod 16 iff I + n even. Both alignments occur
  --rust    rsolver (PYTHONPATH) with RSOLVER_BTRAN_PROBE: Rust peel flag = original alignment, Rust n = spx+0x68; the
            original updates (minus the call before the first factor and the update the original makes right before a
            refactor that Rust skips, and the calls with r < 0) and the Rust updates agree in (U index r, new diagonal LU+0x50[r] bits) up to the
            pinned same-state range RANGE (a shorter agreement fails; a longer one is reported)
  --self-test  also tampers with the loaded data and requires each tamper to be rejected
usage: python3 tools/m1_diag_peel_check.py [--rust] [--self-test]"""
import copy, gzip, hashlib, json, os, subprocess, sys, tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DIR = os.path.join(ROOT, 'bench/oracle/m1_02y')
NETLIB = os.path.join(ROOT, '../re/bench/data/netlib')
BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
MODELS = ['stair', 'sc50a', 'scsd1', 'afiro', 'kb2', 'degen2']
# updates (after the alignment filter) where Rust and the original agree in r and diagonal bits; pinned after the
# first run of the fixed build (M1-02y §6). The ends are known state divergences: stair at the pivot-347 update (after
# the e_r 347 BTRAN value difference, M1-02w-3; was e_r 186 before the dense switch), degen2 at e_r 356 (M1-02x range end), scsd1 at e_r 43 (r differs: path)
RANGE = {'stair': 337, 'sc50a': 30, 'scsd1': 37, 'afiro': 9, 'kb2': 35, 'degen2': 348}
NNZ, VAL16, CHG, X100, IDX16, VAL_IDX, VAL_STRUCT, M, W16, IDX_W, S64, S68, S3C8, X188, R, X150, NREF, RET, DIAG = range(19)


class Fail(Exception):
    pass


def need(c, msg):
    if not c:
        raise Fail(msg)


def sha(p):
    return hashlib.sha256(open(p, 'rb').read()).hexdigest()


def load():
    return {n: [json.load(gzip.open(f'{DIR}/{n}_{i}.json.gz')) for i in (1, 2, 3)] for n in MODELS}


def controls(n, caps):
    tool, model = sha(os.path.join(ROOT, 'tools/m1_eta_align.js')), sha(f'{NETLIB}/{n}.mps.gz')
    for i, c in enumerate(caps, 1):
        need(c['binary_sha256'] == BINARY, f'{n} run {i}: binary sha {c["binary_sha256"]}')
        need(c['model_sha256'] == model, f'{n} run {i}: model sha')
        need(c['tool_sha256'] == tool, f'{n} run {i}: tool sha (capture made by another tool version)')
        need(c['ok'] and not c['errors'] and c['output'].startswith('DONE'), f'{n} run {i}: not ok: {c["output"]} {c["errors"]}')
        need(c['params'] == {'Threads': 1, 'Method': 1, 'Presolve': 0, 'OutputFlag': 0}, f'{n} run {i}: params')
        need(c['output'] == caps[0]['output'], f'{n} run {i}: output {c["output"]} != run 1')
        # VAL_STRUCT is a distance between two heap blocks: kept for the record, not part of the decision
        j = next((k for k, (a, b) in enumerate(zip(caps[0]['events'], c['events']))
                  if a[:VAL_STRUCT] + a[VAL_STRUCT + 1:] != b[:VAL_STRUCT] + b[VAL_STRUCT + 1:]), None)
        need(j is None and len(c['events']) == len(caps[0]['events']), f'{n} run {i}: event {j} differs from run 1')


def original(n, ev):
    need(ev, f'{n}: no events')
    up = [e for e in ev if e[NREF] > 0]
    need(up, f'{n}: no update after a factor')
    need(sum(e[CHG] for e in ev) == 1 and ev[0][CHG] == 1, f'{n}: LU+0x1d0 value array moved during the run')
    i, nn, m = up[0][S3C8], up[0][S68], up[0][M]
    peel = (i + nn) % 2 == 0
    for k, e in enumerate(up):
        need(e[M] == m and e[S68] == nn, f'{n}: update {k}: m/n changed')
        need(e[IDX_W] == 24 * i + 8 + 8 * (m + nn) + 4 * m, f'{n}: update {k}: idx - W {e[IDX_W]} != 24I + 8 + 8(m+n) + 4m '
                                                             f'(I {i}, n {nn}, m {m})')
        need(e[VAL_IDX] == 4 * m, f'{n}: update {k}: val - idx {e[VAL_IDX]} != 4m')
        need(e[W16] == 0 and e[X150] == 0, f'{n}: update {k}: W or LU+0x150 not 16-byte aligned ({e[W16]}, {e[X150]})')
        need(e[VAL16] == (8 if peel else 0), f'{n}: update {k}: values {e[VAL16]} mod 16, I + n = {i + nn}')
        need(e[RET] == 0, f'{n}: update {k}: 05b257 returned {e[RET]}')
    return peel, nn


def aligned(ev):
    """Original updates Rust also makes: not the call before the first factor, not r < 0 (no position), not one followed
    by a refactor."""
    return [e for k, e in enumerate(ev) if e[NREF] > 0 and e[R] >= 0 and not (k + 1 < len(ev) and ev[k + 1][NREF] > e[NREF])]


def rust_run(n, outdir):
    spec = os.path.join(outdir, f'{n}.spec')
    open(spec, 'w').close()
    code = (f"import rsolver; m=rsolver.read({os.path.join(NETLIB, n + '.mps.gz')!r}); m.Params['Method']=1; "
            "m.Params['Presolve']=0; m.optimize()")
    subprocess.run([sys.executable, '-c', code], env=dict(os.environ, RSOLVER_BTRAN_PROBE=spec), check=True)
    return [r for r in map(json.loads, open(spec + '.out')) if 'u' in r]


def rust(n, ev, peel, nn, ru):
    need(ru, f'{n}: no Rust update')
    for r in ru:
        need(r['peel'] == peel and r['n'] == nn, f'{n}: Rust peel {r["peel"]} n {r["n"]}, original peel {peel} n {nn}')
    orig = aligned(ev)
    agree = next((k for k, (o, r) in enumerate(zip(orig, ru)) if (o[R], o[DIAG]) != (r['step'], r['d'])), min(len(orig), len(ru)))
    if n in RANGE:
        need(agree >= RANGE[n], f'{n}: Rust and original updates agree for {agree}, pinned {RANGE[n]}; first difference '
                                f'original (r, diag) {(orig[agree][R], orig[agree][DIAG]) if agree < len(orig) else None}, '
                                f'Rust {(ru[agree]["step"], ru[agree]["d"]) if agree < len(ru) else None}')
    return agree, len(orig), len(ru)


def check(data, routs):
    res, peels = {}, set()
    for n in MODELS:
        controls(n, data[n])
        ev = data[n][0]['events']
        peel, nn = original(n, ev)
        peels.add(peel)
        res[n] = [len(ev), peel]
        if routs is not None:
            res[n] += rust(n, ev, peel, nn, routs[n])
    need(peels == {True, False}, f'only one alignment among the models: {peels}')
    return res


def tampers(data, routs):
    def t(f):
        def g():
            d, r = copy.deepcopy(data), copy.deepcopy(routs)
            f(d, r)
            return d, r
        return g

    def each(d, n, k, f):
        for c in d[n]:
            f(c['events'][k])

    def only_peel():  # kb2 and degen2 made self-consistent peel models (I + 1); original checks only
        d = copy.deepcopy(data)
        for n in ('kb2', 'degen2'):
            for c in d[n]:
                for e in c['events']:
                    e[S3C8], e[IDX_W], e[VAL16] = e[S3C8] + 1, e[IDX_W] + 24, 8
        return d, None

    first_diff = lambda r: next(x for x in r['stair'][40:] if x['step'] > 0)  # an update well inside the range
    out = {
        'binary sha': t(lambda d, r: d['kb2'][1].update(binary_sha256='0' * 64)),
        'empty events': t(lambda d, r: [c.update(events=[]) for c in d['sc50a']]),
        'run 2 diagonal': t(lambda d, r: d['stair'][1]['events'][20].__setitem__(DIAG, '0' * 16)),
        'alignment flipped': t(lambda d, r: each(d, 'stair', 5, lambda e: e.__setitem__(VAL16, 0))),
        'carve offset': t(lambda d, r: each(d, 'degen2', 3, lambda e: e.__setitem__(IDX_W, e[IDX_W] + 8))),
        'pool unaligned': t(lambda d, r: each(d, 'afiro', 2, lambda e: e.__setitem__(X150, 8))),
        'array moved': t(lambda d, r: each(d, 'kb2', 4, lambda e: e.__setitem__(CHG, 1))),
        'one alignment only': only_peel,
    }
    if routs is not None:
        out.update({
            'Rust peel flag': t(lambda d, r: [x.update(peel=not x['peel']) for x in r['stair']]),
            'Rust n': t(lambda d, r: [x.update(n=x['n'] + 1) for x in r['kb2']]),
            'Rust diagonal bits': t(lambda d, r: first_diff(r).update(d='bfeeb36746e614e3')),
            'Rust update order': t(lambda d, r: r['sc50a'].__setitem__(slice(2, 4), r['sc50a'][3:1:-1])),
            'original diagonal (all runs)': t(lambda d, r: [c['events'].__setitem__(30, c['events'][30][:DIAG] + ['3ff0000000000000'])
                                                            for c in d['stair']]),
        })
    return out


def main():
    a = sys.argv[1:]
    data = load()
    routs = None
    if '--rust' in a or '--self-test' in a:
        tmp = tempfile.mkdtemp(prefix='m1_02y_')
        routs = {n: rust_run(n, tmp) for n in MODELS}
    try:
        res = check(data, routs)
    except Fail as e:
        print('FAIL', e)
        sys.exit(1)
    print('model events peel' + (' agree original_updates rust_updates' if routs is not None else ''))
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
