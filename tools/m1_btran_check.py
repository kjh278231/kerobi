"""M1-02k checker: BTRAN 221173 U^T path test (C:88-90) of the original vs rsolver.
Reads bench/oracle/m1_02k (tools/m1_btran_runs.sh): per model a plain capture and three synthetic captures.
  controls  hashes (binary, model, tool, spec), ok/DONE lines, 3 runs equal, synthetic run's natural events = plain
  original  each synthetic case's path identified (natural output+LU+0x200 equal to exactly one forced run), the path
            equals (int)(LU+0x108 + LU+0x110)*nnz <= m, every natural call's 21a941 use equals the same test (9e376 with
            a non-empty U row r: forced sparse), and each model has a case where 3*nnz <= m < D*nnz (old test differs)
  --rust    rsolver (PYTHONPATH) with RSOLVER_BTRAN_PROBE: e_r calls k <= the last spec k have the original r, D bits,
            path and output form; every synthetic case has the original path and the natural / forced sparse / forced
            dense outputs (dense flag, order, value bits)
  --self-test  also tampers with the loaded data and requires each tamper to be rejected
usage: python3 tools/m1_btran_check.py [--rust] [--self-test] [--rust-out DIR]"""
import copy, gzip, hashlib, json, os, struct, subprocess, sys, tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DIR = os.path.join(ROOT, 'bench/oracle/m1_02k')
NETLIB = os.path.join(ROOT, '../re/bench/data/netlib')
BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
MODELS = ['kb2', 'degen2', 'sc50b', 'scsd1', 'stair', 'forplan', 'fit1p']
ER = (376, 0xc51dd)  # 9e376 called back to 28c93: the dual e_r BTRAN
# Natural e_r calls whose output form (dense or not) already differed before M1-02k (same with the U-order U^T build
# 30e4d23); U^T path and D agree there. Kept as a pinned failure list: any other difference fails.
KNOWN_FORM = {'stair': [102, 118]}
# First e_r call after a refactor (same pivot on both sides) whose LU+0x108 differs: the tail U rows of rsolver's factor
# hold fewer entries than the original's (stair 1411 vs 1571 over m - t = 231); LU+0x110 and t agree. Pre-existing
# factor difference, not the U^T test: the same-state range ends there.
KNOWN_D = {'stair': 185, 'forplan': 82}


class Fail(Exception):
    pass


def sha(p):
    return hashlib.sha256(open(p, 'rb').read()).hexdigest()


def need(c, msg):
    if not c:
        raise Fail(msg)


def dval(h):
    return struct.unpack('>d', bytes.fromhex(h))[0]


def dsum(x108, x110):
    return struct.pack('>d', dval(x108) + dval(x110)).hex()


def load():
    data = {}
    for n in MODELS:
        caps = {k: json.load(gzip.open(f'{DIR}/{n}_{k}.json.gz')) for k in ('plain', 'syn_1', 'syn_2', 'syn_3')}
        spec = [l.split('\t') for l in open(f'{DIR}/{n}.spec').read().splitlines()]
        data[n] = {'caps': caps, 'spec': spec, 'spec_sha': sha(f'{DIR}/{n}.spec')}
    return data


def controls(n, d):
    caps, tool = d['caps'], sha(os.path.join(ROOT, 'tools/m1_btran_path.js'))
    model = sha(f'{NETLIB}/{n}.mps.gz')
    for k, c in caps.items():
        need(c['binary_sha256'] == BINARY, f'{n} {k}: binary sha {c["binary_sha256"]}')
        need(c['model_sha256'] == model, f'{n} {k}: model sha')
        need(c['tool_sha256'] == tool, f'{n} {k}: tool sha (capture made by another tool version)')
        need(c['ok'] and not c['errors'] and c['output'].startswith('DONE'), f'{n} {k}: run not ok: {c["output"]} {c["errors"]}')
        need(c['output'] == caps['plain']['output'], f'{n} {k}: output {c["output"]} != plain {caps["plain"]["output"]}')
        need(c['params'] == {'Threads': 1, 'Method': 1, 'Presolve': 0, 'OutputFlag': 0}, f'{n} {k}: params')
        if k != 'plain':
            need(c['spec_sha256'] == d['spec_sha'], f'{n} {k}: spec sha')
    need(caps['plain']['spec_sha256'] is None and not caps['plain']['synth'], f'{n}: plain capture has synthetic calls')
    for k in ('syn_2', 'syn_3'):
        for f in ('events', 'synth'):
            a, b = caps['syn_1'][f], caps[k][f]
            i = next((i for i, (x, y) in enumerate(zip(a, b)) if x != y), None if len(a) == len(b) else min(len(a), len(b)))
            need(i is None, f'{n}: {k} {f}[{i}] differs from syn_1: {b[i] if i is not None and i < len(b) else None} '
                            f'vs {a[i] if i is not None and i < len(a) else None}')
    a, b = caps['plain']['events'], caps['syn_1']['events']
    i = next((i for i, (x, y) in enumerate(zip(a, b)) if x != y), None if len(a) == len(b) else min(len(a), len(b)))
    need(i is None, f'{n}: synthetic run natural event {i} differs from plain (interference)')
    need(len(b) > 0, f'{n}: no natural events')
    need(len(caps['syn_1']['synth']) == len(d['spec']), f'{n}: {len(caps["syn_1"]["synth"])} synthetic cases, spec has {len(d["spec"])}')


def key(o):
    return (o['nnz'], o['idx'], o['val'], o['w200'])


def original(n, d):
    """Returns the original's path per synthetic case (True = dense U^T)."""
    for j, e in enumerate(d['caps']['plain']['events']):
        (w, ret, _), m, nnz, x108, x110, _, _, p4, _, k941, _ = e
        dense = not (nnz >= 0 and int(dval(x108) + dval(x110)) * nnz <= m)
        if w == 376:
            dense = dense and not p4  # 9e376: a non-empty U row r forces 21a941
        need(dense == (k941 == 0), f'{n}: natural call {j} {e}: 21a941 ran {k941} times, C:89 test says dense={dense}')
    paths, band = [], 0
    for c, (k, xs) in zip(d['caps']['syn_1']['synth'], d['spec']):
        need(str(c['k']) == k, f'{n}: synthetic case k {c["k"]} vs spec {k}')
        nat, sp, dn = (key(o) for o in c['o'])
        need(sp != dn, f'{n} k {c["k"]} case {c["i"]}: forced sparse and dense runs agree, path not identifiable')
        need((nat == sp) != (nat == dn), f'{n} k {c["k"]} case {c["i"]}: natural output equals '
                                         f'{"both" if nat == sp else "neither"} forced runs')
        dense, nnz, m, D = nat == dn, len(xs.split(';')), c['m'], int(dval(c['x108']) + dval(c['x110']))
        need(dense == (D * nnz > m), f'{n} k {c["k"]} case {c["i"]}: original path dense={dense}, (int)D*nnz <= m '
                                     f'with D={D} nnz={nnz} m={m} says {D * nnz > m}')
        band += 3 * nnz <= m < D * nnz
        paths.append(dense)
    need(band > 0, f'{n}: no case where the old 3*nnz test and the original differ')
    return paths, band


def rust_run(n, d, outdir):
    spec = os.path.join(outdir, f'{n}.spec')
    open(spec, 'w').write(open(f'{DIR}/{n}.spec').read())
    if os.path.exists(spec + '.out'):
        os.remove(spec + '.out')
    code = (f"import rsolver; m=rsolver.read({os.path.join(NETLIB, n + '.mps.gz')!r}); m.Params['Method']=1; "
            "m.Params['Presolve']=0; m.optimize()")
    subprocess.run([sys.executable, '-c', code], env=dict(os.environ, RSOLVER_BTRAN_PROBE=spec), check=True)
    return [json.loads(l) for l in open(spec + '.out')]


def rust(n, d, paths, out):
    er = [e for e in d['caps']['syn_1']['events'] if tuple(e[0][:2]) == ER]
    nat, syn = [r for r in out if 'r' in r], [r for r in out if 'i' in r]
    last = max(int(k) for k, _ in d['spec'])
    need(len(nat) > last and len(er) > last, f'{n}: {len(nat)} Rust / {len(er)} original e_r calls, spec needs k {last}')
    # same input state while the leaving position r and the row eta count LU+0x180 agree (a refactor only one side
    # makes, e.g. the work test of 06268b that rsolver lacks, ends it)
    matched = next((k for k, (e, r) in enumerate(zip(er, nat)) if (e[8], e[5]) != (r['r'], r['e'])), min(len(er), len(nat)))
    g = lambda v, i, f: v[i][f] if i < len(v) else None
    need(matched > last, f'{n}: e_r call {matched}: Rust r, etas {g(nat, matched, "r"), g(nat, matched, "e")} vs original '
                         f'{g(er, matched, 8), g(er, matched, 5)} '
                         f'(trajectory differs before spec k {last})')
    form = []
    if n in KNOWN_D:
        k = KNOWN_D[n]
        need(k < matched and dsum(er[k][3], er[k][4]) != nat[k]['d'], f'{n}: pinned D difference at e_r call {k} not seen')
        matched = k
    for k in range(matched):
        e, r = er[k], nat[k]
        for f, a, b in (('D', dsum(e[3], e[4]), r['d']), ('U^T dense', e[9] == 0, r['ud'])):
            need(a == b, f'{n}: e_r call {k} field {f}: original {a} Rust {b}')
        if (e[10] < 0) != r['od']:
            form.append(k)
    end = 'end' if matched == min(len(er), len(nat)) else 'D (factor)' if n in KNOWN_D else (
        f'r {nat[matched]["r"]}/{er[matched][8]}' if nat[matched]['r'] != er[matched][8] else
        f'etas {nat[matched]["e"]}/{er[matched][5]}')
    need(form == KNOWN_FORM.get(n, []), f'{n}: e_r output form differs at calls {form}, known {KNOWN_FORM.get(n, [])}')
    need(len(syn) == len(paths), f'{n}: {len(syn)} Rust synthetic cases vs {len(paths)}')
    for c, r, dense in zip(d['caps']['syn_1']['synth'], syn, paths):
        need((r['k'], r['i']) == (c['k'], c['i']), f'{n}: Rust case {r["k"]}/{r["i"]} vs {c["k"]}/{c["i"]}')
        need(r['ud'] == dense, f'{n} k {c["k"]} case {c["i"]}: Rust path dense={r["ud"]}, original dense={dense}')
        for name, o, ro in zip(('natural', 'forced sparse', 'forced dense'), c['o'], r['o']):
            a = (o['nnz'] < 0, [] if o['nnz'] < 0 else o['idx'], o['val'])
            b = (ro['dense'], [] if ro['dense'] else ro['idx'], ro['val'])
            if a != b:
                f = 'dense' if a[0] != b[0] else 'idx' if a[1] != b[1] else 'val'
                i = 0 if f == 'dense' else next((i for i, (x, y) in enumerate(zip(a[1 if f == 'idx' else 2], b[1 if f == 'idx' else 2])) if x != y), None)
                raise Fail(f'{n} k {c["k"]} case {c["i"]} {name}: field {f} index {i}: original {a[0], a[1][:8], a[2][:4]} '
                           f'Rust {b[0], b[1][:8], b[2][:4]}')
    return f'{matched}/{len(er)} ({end})' + (f' known form diff {form}' if form else '')


def check(data, routs):
    res = {}
    for n in MODELS:
        controls(n, data[n])
        paths, band = original(n, data[n])
        res[n] = [len(data[n]['caps']['plain']['events']), len(paths), sum(paths), band]
        if routs is not None:
            res[n].append(rust(n, data[n], paths, routs[n]))
    return res


def tampers(data, routs):
    def t(f):
        def g():
            d, r = copy.deepcopy(data), copy.deepcopy(routs)
            f(d, r)
            return d, r
        return g
    runs = ('syn_1', 'syn_2', 'syn_3')
    syn = lambda d, n='sc50b', k='syn_1': d[n]['caps'][k]['synth']
    band = lambda d, n='sc50b': next(i for i, c in enumerate(syn(d, n)) if c['o'][0]['nnz'] < 0 and c['o'][1]['nnz'] >= 0
                                     and 3 * len(d[n]['spec'][i][1].split(';')) <= c['m'])
    rsyn = lambda r, n='sc50b': [x for x in r[n] if 'i' in x]

    def undetermined(d, r):
        i = band(d)
        for k in runs:
            syn(d, k=k)[i]['o'][0] = copy.deepcopy(syn(d, k=k)[i]['o'][1])
            syn(d, k=k)[i]['o'][2] = copy.deepcopy(syn(d, k=k)[i]['o'][1])

    def no_band(d, r):  # drop fit1p's cases where the old test and the original differ, from the spec and every run
        n, c = 'fit1p', syn(d, 'fit1p')
        D = lambda x: int(dval(x['x108']) + dval(x['x110']))
        keep = [i for i, (x, (_, xs)) in enumerate(zip(c, d[n]['spec'])) if not 3 * len(xs.split(';')) <= x['m'] < D(x) * len(xs.split(';'))]
        d[n]['spec'] = [d[n]['spec'][i] for i in keep]
        for k in runs:
            d[n]['caps'][k]['synth'] = [syn(d, n, k)[i] for i in keep]

    def old_test(d, r):  # rsolver before M1-02k: sparse U^T when 3*nnz <= m
        for n in MODELS:
            for x, c, (_, xs) in zip(rsyn(r, n), syn(d, n), d[n]['spec']):
                x['ud'] = 3 * len(xs.split(';')) > c['m']

    out = {
        'binary sha': t(lambda d, r: d['kb2']['caps']['syn_2'].update(binary_sha256='0' * 64)),
        'spec sha': t(lambda d, r: d['degen2'].update(spec_sha='0' * 64)),
        'empty synthetic cases': t(lambda d, r: [d['sc50b']['caps'][k].update(synth=[]) for k in runs]),
        'empty natural events': t(lambda d, r: [d['kb2']['caps'][k].update(events=[]) for k in d['kb2']['caps']]),
        'run 3 value': t(lambda d, r: d['stair']['caps']['syn_3']['synth'][2]['o'][0]['val'].__setitem__(0, 'deadbeef00000000')),
        'interference': t(lambda d, r: [d['forplan']['caps'][k]['events'][5].__setitem__(9, 7) for k in runs]),
        'natural 21a941 flag': t(lambda d, r: [d['scsd1']['caps'][k]['events'][3].__setitem__(9, 0) for k in d['scsd1']['caps']]),
        'undetermined path': t(undetermined),
        'D bits (path vs C:89)': t(lambda d, r: [x.update(x110='3ff0000000000000') for k in runs for x in syn(d, k=k)]),
        'no band case': t(no_band),
    }
    if routs is not None:
        out.update({
            'Rust path (old 3*nnz)': t(old_test),
            'Rust output value': t(lambda d, r: rsyn(r, 'kb2')[3]['o'][2]['val'].__setitem__(1, 'deadbeef00000000')),
            'Rust output order': t(lambda d, r: rsyn(r)[0]['o'][1]['idx'].reverse()),
            'Rust D bits': t(lambda d, r: next(x for x in r['degen2'] if 'r' in x and x['k'] == 40).update(d='4010000000000000')),
            'Rust e_r r': t(lambda d, r: next(x for x in r['forplan'] if 'r' in x and x['k'] == 3).update(r=-1)),
            'Rust etas': t(lambda d, r: next(x for x in r['kb2'] if 'r' in x and x['k'] == 5).update(e=99)),
            'Rust new form diff': t(lambda d, r: next(x for x in r['degen2'] if 'r' in x and x['k'] == 50).update(od=not next(x for x in r['degen2'] if 'r' in x and x['k'] == 50)['od'])),
        })
    return out


def main():
    a = sys.argv[1:]
    data = load()
    routs = None
    if '--rust' in a or '--self-test' in a and '--no-rust' not in a:
        tmp = tempfile.mkdtemp(prefix='m1_02k_')
        routs = {n: rust_run(n, data[n], tmp) for n in MODELS}
    try:
        res = check(data, routs)
    except Fail as e:
        print('FAIL', e)
        sys.exit(1)
    print('model natural synthetic dense band' + (' rust_er_matched' if routs is not None else ''))
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
