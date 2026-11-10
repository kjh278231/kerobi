#!/usr/bin/env python3
"""M2-03c-2 checker: retry input-basis source, post-retry export, basis load and factor calls.

controls    stored captures match the 03c-1 projection and each other (3 runs), manifest hashes hold
acceptance  the frozen decision rules of docs/m2/mip_retry_basis.md section 4
--self-test also requires every in-memory tamper to be rejected
Exit code 0 only when every requested check passes.
"""
import copy, hashlib, json, os, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = 'bench/oracle/mip_retry_basis'
MANIFEST = 'bench/oracle/mip_retry_basis_manifest.json'
OLD = 'bench/oracle/mip_linear_child_retry'
BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
CASES = {'nidda_stress': 'nidda_stress_retry_1.json', 'gmu-35-50_default': 'gmu-35-50_default_retry_1.json'}
RUNS = (1, 2, 3)


class Fail(Exception):
    pass


def need(cond, msg):
    if not cond:
        raise Fail(msg)


def fnv(v):
    h = 0x811c9dc5
    for x in v:
        x &= 0xffffffff
        for sh in (0, 8, 16, 24):
            h ^= (x >> sh) & 0xff
            h = (h * 0x1000193) & 0xffffffff
    return format(h, 'x')


def decode(words, cols, srows, rows):
    """69d7ea: columns 2 bits (0,-1,-2,-3), then stored rows 1 bit (1 -> -1), MSB first in 32-bit words; extra rows 0."""
    bits = ''.join(format(w & 0xffffffff, '032b') for w in words)
    need(len(bits) >= 2 * cols + srows, 'packed basis too short')
    vb = [-int(bits[2 * j:2 * j + 2], 2) for j in range(cols)]
    cb = [-int(bits[2 * cols + i]) for i in range(srows)] + [0] * (rows - srows)
    return vb, cb


def load(path):
    with open(os.path.join(ROOT, path)) as f:
        return json.load(f)


def files():
    out = {}
    for case in CASES:
        for kind in ('natural', 'synth'):
            for r in RUNS:
                out[(case, kind, r)] = f'{DATA}/{case}_{kind}_{r}.json'
    return out


def project(d):
    """03c-1 view: drop x: events."""
    return [{'event': s['event'], 'events': [e for e in s['events'] if not e['event'].startswith('x:')]} for s in d['events']]


def check_manifest(m):
    need(m['binary_sha256'] == BINARY, 'manifest binary hash')
    for p, h in m['files'].items():
        need(hashlib.sha256(open(os.path.join(ROOT, p), 'rb').read()).hexdigest() == h, f'hash mismatch {p}')
    for p, h in m['external_models'].items():
        full = os.path.join(ROOT, '..', p)
        need(hashlib.sha256(open(full, 'rb').read()).hexdigest() == h, f'model hash mismatch {p}')
    need(set(files().values()) <= set(m['files']), 'manifest misses a capture')


def controls(caps, tool_sha):
    n = 0
    for case, old in CASES.items():
        ref = load(f'{OLD}/{old}')
        for kind in ('natural', 'synth'):
            runs = [caps[(case, kind, r)] for r in RUNS]
            for d in runs:
                need(d['binary_sha256'] == BINARY, f'{case} {kind}: binary hash')
                need(d['ok'] and not d['errors'], f'{case} {kind}: run not ok')
                need(d['tool_sha256'] == tool_sha, f'{case} {kind}: tool hash')
                need(d['model_sha256'] == ref['model_sha256'] and d['params'] == ref['params'], f'{case} {kind}: input differs from 03c-1')
                need(d['output'] == ref['output'], f'{case} {kind}: final output differs from 03c-1')
                need(len(d['events']) == 1 and d['events'][0]['event'] == 'retry_solve', f'{case} {kind}: expected one retry solve')
                need(project(d) == ref['events'], f'{case} {kind}: 03c-1 projection differs (hooks changed the run)')
                n += 1
            need(all(d['events'] == runs[0]['events'] for d in runs), f'{case} {kind}: runs differ')
    return n


def ev(d):
    return d['events'][0]['events']


def first(es, name, **kw):
    for i, e in enumerate(es):
        if e['event'] == name and all(e.get(k) == v for k, v in kw.items()):
            return i, e
    raise Fail(f'missing {name} {kw}')


def cda_block(es, k):
    """events emitted while cda5b call k ran, keyed by kind"""
    out = {'2f7e1': [], 'load_enter': [], 'load_leave': [], 'factor': 0}
    for e in es:
        if e.get('k') != k:
            continue
        if e['event'] == 'x:2f7e1:leave':
            out['2f7e1'].append(e)
        elif e['event'] == 'x:load:enter':
            out['load_enter'].append(e)
        elif e['event'] == 'x:load:leave':
            out['load_leave'].append(e)
        elif e['event'] == 'x:factor':
            out['factor'] += 1
    return out


def load_leaves(es, k):
    """x:load:leave carries no k; pair it with the preceding x:load:enter"""
    res = []
    for i, e in enumerate(es):
        if e['event'] == 'x:load:enter' and e['k'] == k:
            need(i + 1 < len(es) and es[i + 1]['event'] == 'x:load:leave', f'load {k} not followed by leave')
            res.append(es[i + 1])
    return res


def nidda(d):
    es = ev(d)
    ri, _ = first(es, 'retry')
    di, dec_in = first(es, 'x:decode:enter')
    _, dec = first(es, 'x:decode:leave')
    need(di < ri, 'decode after retry')
    need(dec_in['packed'] is not None and dec['vbasis'] is not None and dec['cbasis'] is not None, 'decode arrays null')
    vb, cb = decode(dec_in['packed'], dec_in['cols'], dec_in['node_basis']['I94'], dec_in['rows'])
    need((vb, cb) == (dec['vbasis'], dec['cbasis']), 'python decode differs from original 69d7ea output')
    h = fnv(dec['vbasis'] + dec['cbasis'])
    # Q1: provenance chain export -> copy (69e299) -> pack (69d237) -> decode
    pv = dec_in['provenance']
    need(pv and pv['fn'] == 'pack' and pv['hash'] == dec_in['packed_hash'], 'packed basis not written by the recorded pack')
    need(pv['src'] and pv['src']['hash'] == h, 'pack source differs from decoded basis')
    cp = pv['from']
    need(cp and cp['fn'] == 'copy' and cp['node'] == pv['node'] and cp['parent_unpacked'], 'pack not preceded by an unpacked copy into the same node')
    need(cp['child_unpacked'] and cp['child_unpacked']['hash'] == h, 'copied basis differs from decoded basis')
    ps = cp['parent_src']
    need(ps and ps['export'] and ps['export']['rc'] == 0 and ps['export']['hash'] == ps['hash'], 'parent basis not the last 113d92 export into it')
    # Q1: both cda5b calls of the solve receive I+0x98/I+0xa0 holding the decoded basis
    cdas = [e for e in es if e['event'] == 'x:cda5b:enter']
    need([c['retry'] for c in cdas] == [False, True], 'expected first LP and one retry cda5b')
    for c in cdas:
        need((c['vb_rel'], c['cb_rel']) == ('I98', 'Ia0'), f'cda5b {c["k"]} basis not I+0x98/I+0xa0')
        need(c['vbasis'] is not None and fnv(c['vbasis'] + c['cbasis']) == h, f'cda5b {c["k"]} basis differs from decoded')
        need(c['starts_null'] == [True] * 4 and c['node_basis']['dual_null'], f'cda5b {c["k"]} has start or dual-norm input')
    retry = cdas[1]
    need(retry['lp']['ws'] and retry['lp']['ws_f0'] == 12, 'retry not entered on a status-12 workspace')
    # Q3: 2f7e1 resets the status to 1, 325b4 loads the node basis again, factor calls recorded
    blk = cda_block(es, retry['k'])
    need([e['lp']['ws_f0'] for e in blk['2f7e1']] == [1], 'retry 2f7e1 did not reset ws+0xf0 to 1')
    need([(e['vb_rel'], e['cb_rel']) for e in blk['load_enter']] == [('I98', 'Ia0')], 'retry did not load I+0x98/I+0xa0')
    need([e['header_hash'] for e in load_leaves(es, retry['k'])] == [h], 'loaded header differs from node basis')
    need(blk['factor'] > 0, 'no factor call in retry')
    # Q2: no natural export after the retry; predicates recorded
    after = es[ri:]
    need(not any(e['event'] == 'x:export' and not e['synthetic'] for e in after), 'unexpected natural export')
    need(not any(e['event'].startswith('x:save_') for e in after), 'unexpected node basis save')
    _, ar = first(after, 'x:after_retry_call')
    need(ar['lp']['lp_status'] == 6 and ar['lp']['ws_1d8'] & 1, 'post-retry predicates differ')
    return {'q2': 'original produces no VBasis', 'factor_first_lp': cda_block(es, cdas[0]['k'])['factor'], 'factor_retry': blk['factor'],
            'source': (pv['ret'], cp['ret'], ps['export']['ret'], cp['parent'], pv['node'])}


def gmu(d):
    es = ev(d)
    ri, _ = first(es, 'retry')
    cdas = [e for e in es if e['event'] == 'x:cda5b:enter']
    need([c['retry'] for c in cdas] == [False, True, True, True], 'expected first LP and three retry cda5b')
    c1, c2, c3, c4 = cdas
    # Q1: no input basis anywhere for the first LP and the first retry simplex (cold start)
    for c in (c1, c2):
        need((c['vb_rel'], c['cb_rel']) == ('null', 'null') and c['lp']['lp238_null'] and not c['lp']['ws'], f'cda5b {c["k"]} is not a cold start')
        need(not cda_block(es, c['k'])['load_enter'], f'cda5b {c["k"]} loaded a basis')
    need(c3['vb_rel'] == 'other' and c3['vbasis'] is not None and c3['starts_null'] == [False, False, True, True], 'second simplex of 0c3d0d lacks its basis/start')
    need([e['header_hash'] for e in load_leaves(es, c3['k'])] == [fnv(c3['vbasis'] + c3['cbasis'])], 'cda5b 3 loaded header differs from its arguments')
    # Q2: natural export after the first retry LP driver
    xi, x = first(es, 'x:export', synthetic=False)
    need(xi > ri and x['rc'] == 0 and x['vbasis'] is not None and x['cbasis'] is not None, 'natural export missing or empty')
    si, _ = first(es, 'x:save_lp2:enter')
    li, sl = first(es, 'x:save_lp2:leave')
    need(si < xi < li and sl['rc'] == 0 and not sl['node_basis']['vb_null'], 'export not inside 5d4427')
    hx = fnv(x['vbasis'] + x['cbasis'])
    # second LP driver: warm basis set through 10acb2, loaded from lp+0x238, equal to the export
    need(not c4['lp']['lp238_null'] and c4['vb_rel'] == 'null', 'second retry simplex not started from lp+0x238')
    sets = [e for e in es[li:es.index(c4)] if e['event'] == 'x:setbasis']
    need(sets and sets[-1]['hash'] == hx and (sets[-1]['vb_rel'], sets[-1]['cb_rel']) == ('I98', 'Ia0'), 'lp+0x238 basis not the exported node basis')
    gi, _ = first(es, 'x:degen:enter')
    gl, _ = first(es, 'x:degen:leave')
    need(li < gi < es.index(sets[-1]) < es.index(c4) < gl, 'second retry LP not inside 68341b after the export')
    need([e['header_hash'] for e in load_leaves(es, c4['k'])] == [hx], 'second retry simplex loaded a different basis')
    for c in (c2, c3, c4):
        need([e['lp']['ws_f0'] for e in cda_block(es, c['k'])['2f7e1']] == [1], f'cda5b {c["k"]} 2f7e1 status')
    return {'q2': 'natural export', 'factors': [cda_block(es, c['k'])['factor'] for c in cdas], 'export_ret': x['ret'], 'setbasis_ret': sets[-1]['ret']}


def synth(d):
    xs = [e for e in ev(d) if e['event'] == 'x:synth_export']
    need(xs and all(e['synthetic'] for e in xs), 'synthetic export missing')
    return [(e['rc'], None if e['vbasis'] is None else fnv(e['vbasis'] + e['cbasis'])) for e in xs]


def acceptance(caps):
    res = {}
    for case, fn in (('nidda_stress', nidda), ('gmu-35-50_default', gmu)):
        out = [fn(caps[(case, 'natural', r)]) for r in RUNS]
        need(all(o == out[0] for o in out), f'{case}: acceptance summary differs across runs')
        sy = [synth(caps[(case, 'synth', r)]) for r in RUNS]
        need(all(s == sy[0] for s in sy), f'{case}: synthetic export differs across runs')
        res[case] = dict(out[0], synthetic=sy[0])
    return res


def run(which, caps, tool_sha):
    if which == 'controls':
        return controls(caps, tool_sha)
    return acceptance(caps)


def tampers(caps):
    def nid(c):
        return ev(c[('nidda_stress', 'natural', 1)])

    def gm(c):
        return ev(c[('gmu-35-50_default', 'natural', 1)])

    def find(es, name, **kw):
        return first(es, name, **kw)[1]

    def t_binary(c): c[('nidda_stress', 'natural', 2)]['binary_sha256'] = '0' * 64
    def t_empty(c): c[('gmu-35-50_default', 'natural', 1)]['events'] = []
    def t_projection(c): find(nid(c), 'retry')['lp_status'] = 5
    def t_run_diff(c): find(gm(c), 'x:factor')['k'] = -1
    def t_null_null(c):
        for e in nid(c):
            if e['event'] in ('x:decode:leave', 'x:cda5b:enter'):
                e['vbasis'] = e['cbasis'] = None
    def t_alias(c): [e for e in nid(c) if e['event'] == 'x:cda5b:enter'][1]['vb_rel'] = 'other'
    def t_decode(c): find(nid(c), 'x:decode:leave')['vbasis'][0] ^= 1
    def t_chain(c): find(nid(c), 'x:decode:enter')['provenance']['from']['parent_src']['export'] = None
    def t_reset(c): [e for e in nid(c) if e['event'] == 'x:2f7e1:leave'][-1]['lp']['ws_f0'] = 12
    def t_load(c):
        es = nid(c)
        es[:] = [e for i, e in enumerate(es) if not (e['event'].startswith('x:load') and i > first(es, 'retry')[0])]
    def t_nat_export(c): find(gm(c), 'x:export', synthetic=False)['synthetic'] = True
    def t_export_null(c): find(gm(c), 'x:export', synthetic=False)['vbasis'] = None
    def t_setbasis(c):
        for e in gm(c):
            if e['event'] == 'x:setbasis':
                e['hash'] = '0'
    def t_degen(c):
        es = gm(c)
        es[:] = [e for e in es if e['event'] != 'x:degen:enter']
    def t_synth_only(c):
        for r in RUNS:
            for e in ev(c[('gmu-35-50_default', 'natural', r)]):
                if e['event'] == 'x:export':
                    e['synthetic'] = True
    def t_synth_missing(c):
        es = ev(c[('nidda_stress', 'synth', 1)])
        es[:] = [e for e in es if e['event'] != 'x:synth_export']
    def t_spurious_export(c):
        es = nid(c)
        es.insert(len(es) - 1, {'event': 'x:export', 'synthetic': False, 'rc': 0, 'vbasis': [], 'cbasis': []})
    return [('binary', 'controls', t_binary), ('empty', 'controls', t_empty), ('projection', 'controls', t_projection),
            ('run_diff', 'controls', t_run_diff), ('null_null', 'acceptance', t_null_null), ('alias', 'acceptance', t_alias),
            ('decode', 'acceptance', t_decode), ('chain', 'acceptance', t_chain), ('reset', 'acceptance', t_reset), ('load', 'acceptance', t_load),
            ('natural_export', 'acceptance', t_nat_export), ('export_null', 'acceptance', t_export_null), ('setbasis', 'acceptance', t_setbasis), ('degen', 'acceptance', t_degen),
            ('synth_only', 'acceptance', t_synth_only), ('synth_missing', 'acceptance', t_synth_missing), ('spurious_export', 'acceptance', t_spurious_export)]


def write_manifest():
    sha = lambda p: hashlib.sha256(open(os.path.join(ROOT, p), 'rb').read()).hexdigest()
    paths = sorted(files().values()) + ['tools/mip_retry_basis.js', 'tools/mip_retry_basis_runs.sh']
    models = ['re/bench/data/miplib/neos-3754480-nidda.mps.gz', 're/bench/data/miplib/gmu-35-50.mps.gz']
    m = {'binary_sha256': BINARY, 'files': {p: sha(p) for p in paths}, 'external_models': {p: sha(os.path.join('..', p)) for p in models}}
    with open(os.path.join(ROOT, MANIFEST), 'w') as f:
        json.dump(m, f, indent=2)
        f.write('\n')


def main(argv):
    if '--write-manifest' in argv:
        write_manifest()
        return 0
    self_test = '--self-test' in argv
    which = [a for a in argv if not a.startswith('--')] or ['controls', 'acceptance']
    m = load(MANIFEST)
    caps = {k: load(p) for k, p in files().items()}
    tool_sha = m['files']['tools/mip_retry_basis.js']
    rc = 0
    try:
        check_manifest(m)
        print('manifest PASS', len(m['files']), 'files')
    except Fail as e:
        print('manifest FAIL', e)
        rc = 1
    for w in which:
        try:
            print(w, 'PASS', json.dumps(run(w, caps, tool_sha), default=str))
        except Fail as e:
            print(w, 'FAIL', e)
            rc = 1
    if self_test:
        for name, w, t in tampers(caps):
            c = copy.deepcopy(caps)
            t(c)
            try:
                run(w, c, tool_sha)
                print('self-test', name, 'NOT REJECTED')
                rc = 1
            except (Fail, KeyError, TypeError, IndexError, ValueError) as e:
                print('self-test', name, 'rejected:', type(e).__name__, e)
    return rc


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
