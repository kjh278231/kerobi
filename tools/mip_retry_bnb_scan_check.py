#!/usr/bin/env python3
"""M2-03d-5: classify natural retries from tools/mip_retry_bnb_scan.js outputs and check controls / repro.

  python3 tools/mip_retry_bnb_scan_check.py [DIR]       # DIR default bench/oracle/mip_retry_bnb_scan
  python3 tools/mip_retry_bnb_scan_check.py --self-test [DIR]
  python3 tools/mip_retry_bnb_scan_check.py --write-manifest [DIR]   # after a full capture
Exit 0 only if the manifest matches, controls match, the matrix is complete with the contract parameters and current tool,
every matrix run is ok, and every repro group has 3 identical non-empty retry lists equal to its matrix hit.
"""
import copy, glob, hashlib, json, os, sys
from collections import Counter

BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
# expected control classification (M2-03d-3 §6: nidda node 8287 via 8e4682 after one restart; gmu depth-2 sub-MIP root)
CONTROLS = {
    'nidda_stress': [{'caller': '8e4682', 'depth': 1, 'node_id': 8287, 'parent': None, 'root': False, 'restarts': 1, 'tier': 'C'}],
    'gmu_default': [{'caller': '654502', 'depth': 2, 'node_id': 0, 'parent': None, 'root': True, 'restarts': 0, 'tier': 'C'}],
}


MODELS = '../re/bench/data/miplib'
TOOL = 'tools/mip_retry_bnb_scan.js'
STRESS = {'Presolve': 0, 'ScaleFlag': 0, 'Quad': 0, 'NumericFocus': 0, 'MarkowitzTol': 0.0001}
CFG = {'H': dict(STRESS, Heuristics=0, RINS=0), 'P': dict(STRESS, Heuristics=0, RINS=0, Cuts=0)}
SEEDS = (0, 1, 2)


def sha(p):
    with open(p, 'rb') as f:
        return hashlib.sha256(f.read()).hexdigest()


def expected_matrix():
    """contract §4: name -> params written by tools/mip_retry_bnb_scan.sh"""
    models = sorted(os.path.basename(p)[:-7] for p in glob.glob(f'{MODELS}/*.mps.gz'))
    return {f'{c}/s{s}/{m}': dict({'Threads': 1, 'Seed': s, 'LogToConsole': 0, 'TimeLimit': 20, 'NodeLimit': 50000}, **CFG[c])
            for c in CFG for s in SEEDS for m in models}


def tier(e):
    n = e['node']
    if e['caller'] == '5fb8ec' and e['depth'] == 1 and n['parent'] is not None:
        return 'A' if e['restarts'] == 0 and e['root_is_self'] else 'B'
    return 'C'


def retries(d):
    return [e for e in d['events'] if e.get('event') == 'retry']


def summary(e):
    n = e['node']
    return {'caller': e['caller'], 'depth': e['depth'], 'node_id': n['id'], 'parent': n['parent'], 'root': n['root'],
            'restarts': e['restarts'], 'tier': tier(e)}


def bnb_nodes(d):
    """normalize-before events of non-root nodes with a parent under 5fb8ec in the top-level MIP (tier A/B population)"""
    t = 0
    for k, v in d['counts'].items():
        p = k.split(':')
        if p[:4] == ['n', 'before', '5fb8ec', '1'] and p[4] == '0' and p[5] == '1':
            t += v
    return t


def load(p):
    with open(p) as f:
        return json.load(f)


def check_run(d, name, errs):
    if not d.get('ok') or d.get('binary_sha256') != BINARY or d.get('errors'):
        errs.append(f'{name}: run not ok ({d.get("detach_reason")}, errors {d.get("errors")})')


def check_controls(ctl, errs):
    for name, want in CONTROLS.items():
        d = ctl.get(name)
        if d is None:
            errs.append(f'control {name}: missing'); continue
        check_run(d, name, errs)
        got = [summary(e) for e in retries(d)]
        if got != want:
            errs.append(f'control {name}: retry classification {got} != {want}')
        if bnb_nodes(d) < 1:
            errs.append(f'control {name}: no 5fb8ec depth-1 non-root node LP observed (positive control)')


def check_repro(groups, mat, errs):
    for key, runs in sorted(groups.items()):
        m, c, s = key.rsplit('_', 2)
        hit = mat.get(f'{c}/{s}/{m}')
        if hit is None or [e for e in retries(hit)] != retries(runs[0]):
            errs.append(f'repro {key}: retry events differ from matrix run {c}/{s}/{m}')
        lists = [[{k: v for k, v in e.items()} for e in retries(d)] for d in runs]
        for i, d in enumerate(runs):
            check_run(d, f'repro {key} #{i + 1}', errs)
        if len(runs) != 3:
            errs.append(f'repro {key}: {len(runs)} runs, need 3')
        elif not lists[0]:
            errs.append(f'repro {key}: no retry event')
        else:
            for i in (1, 2):
                if lists[i] != lists[0]:
                    k = next((j for j in range(max(len(lists[0]), len(lists[i])))
                              if j >= len(lists[0]) or j >= len(lists[i]) or lists[0][j] != lists[i][j]), None)
                    errs.append(f'repro {key}: run {i + 1} retry #{k} differs: {lists[i][k] if k is not None and k < len(lists[i]) else None}'
                                f' != {lists[0][k] if k is not None and k < len(lists[0]) else None}')


def check_manifest(root, errs):
    p = f'{root}/manifest.json'
    if not os.path.exists(p):
        errs.append('manifest.json missing'); return
    want = load(p)
    have = {os.path.relpath(q, root): sha(q) for q in glob.glob(f'{root}/**/*.json', recursive=True) if not q.endswith('manifest.json')}
    have.update({t: sha(t) for t in want.get('tools', {})})
    files = dict(want['files'], **want.get('tools', {}))
    for k in sorted(set(files) | set(have)):
        if files.get(k) != have.get(k):
            errs.append(f'manifest: {k} {files.get(k)} != {have.get(k)}')


def write_manifest(root):
    files = {os.path.relpath(q, root): sha(q) for q in sorted(glob.glob(f'{root}/**/*.json', recursive=True)) if not q.endswith('manifest.json')}
    tools = {t: sha(t) for t in (TOOL, 'tools/mip_retry_bnb_scan.sh')}
    with open(f'{root}/manifest.json', 'w') as f:
        json.dump({'task': 'M2-03d-5', 'binary_sha256': BINARY, 'files': files, 'tools': tools}, f, indent=1)
        f.write('\n')
    print(f'manifest: {len(files)} files, {len(tools)} tools')


def collect(root):
    ctl = {os.path.basename(p)[:-5]: load(p) for p in glob.glob(f'{root}/controls/*.json')}
    mat = {os.path.relpath(p, f'{root}/matrix')[:-5]: load(p) for p in sorted(glob.glob(f'{root}/matrix/*/*/*.json'))}
    groups = {}
    for p in sorted(glob.glob(f'{root}/repro/*.json')):
        groups.setdefault(os.path.basename(p)[:-5].rsplit('_', 1)[0], []).append(load(p))
    return ctl, mat, groups


def evaluate(ctl, mat, groups, out=print):
    errs = []
    check_controls(ctl, errs)
    want, tool = expected_matrix(), sha(TOOL)
    if set(mat) != set(want):
        errs.append(f'matrix runs missing {sorted(set(want) - set(mat))[:5]} extra {sorted(set(mat) - set(want))[:5]}')
    for name, d in list(mat.items()) + list(ctl.items()) + [(k, d) for k, g in groups.items() for d in g]:
        if d.get('tool_sha256') != tool:
            errs.append(f'{name}: tool hash {d.get("tool_sha256")} != current {tool}')
        if name in want and d.get('params') != want[name]:
            errs.append(f'{name}: params {d.get("params")} != contract {want[name]}')
    tiers, pop, hits = Counter(), Counter(), []
    for name, d in mat.items():
        check_run(d, name, errs)
        cfg = name.split('/')[0]
        pop[cfg] += bnb_nodes(d)
        for e in retries(d):
            t = tier(e); tiers[(cfg, t)] += 1
            hits.append((name, t, summary(e), e['solve_serial'], e['lp_status'], e['node_basis'], d['output']))
    check_repro(groups, mat, errs)
    out(f'matrix runs {len(mat)} (ok {sum(1 for d in mat.values() if d.get("ok"))}); '
        f'tier A/B population (5fb8ec, depth 1, non-root, parent) per config {dict(pop)}')
    out(f'retries by (config, tier) {dict(sorted(tiers.items()))}')
    for h in hits:
        out('  retry', *h)
    out(f'repro groups {len(groups)}: {sorted(groups)}')
    for e in errs:
        out('ERROR', e)
    return errs


def self_test(root):
    ctl, mat, groups = collect(root)
    errs = []
    check_manifest(root, errs)
    if errs or evaluate(ctl, mat, groups, out=lambda *a: None):
        print('self-test: saved data does not pass', errs[:3]); return 1
    # the saved repro group must pass; mutations of it and of the controls/matrix must fail
    base = copy.deepcopy(groups)
    rk = sorted(base)[0]
    def m_mat(f):
        mm = copy.deepcopy(mat); f(mm); return (ctl, mm, base)
    first = sorted(mat)[0]
    muts = []
    def m_ctl(f):
        c = copy.deepcopy(ctl); f(c); return (c, mat, base)
    def m_rep(f):
        g = copy.deepcopy(base); f(g); return (ctl, mat, g)
    def ev(name, j):
        if name == 'r':
            name = rk
        return lambda c: [e for e in c[name]['events'] if e.get('event') == 'retry'][0]
    muts += [
        ('nidda caller', m_ctl(lambda c: ev('nidda_stress', 0)(c).__setitem__('caller', '5fb8ec'))),
        ('nidda depth', m_ctl(lambda c: ev('nidda_stress', 0)(c).__setitem__('depth', 2))),
        ('nidda parent', m_ctl(lambda c: ev('nidda_stress', 0)(c)['node'].__setitem__('parent', 1.0))),
        ('nidda node id', m_ctl(lambda c: ev('nidda_stress', 0)(c)['node'].__setitem__('id', 8288.0))),
        ('gmu restarts', m_ctl(lambda c: ev('gmu_default', 0)(c).__setitem__('restarts', 1))),
        ('gmu extra retry', m_ctl(lambda c: c['gmu_default']['events'].append(copy.deepcopy(ev('gmu_default', 0)(c))))),
        ('control positive population gone', m_ctl(lambda c: c['nidda_stress'].__setitem__('counts', {}))),
        ('control not ok', m_ctl(lambda c: c['gmu_default'].__setitem__('ok', False))),
        ('repro solve serial', m_rep(lambda g: ev('r', 0)({rk: g[rk][2]}).__setitem__('solve_serial', 1))),
        ('repro basis', m_rep(lambda g: ev('r', 0)({rk: g[rk][1]})['node_basis'].__setitem__('vb_null', True))),
        ('repro two runs', m_rep(lambda g: g[rk].pop())),
        ('repro no retry', m_rep(lambda g: [d.__setitem__('events', []) for d in g[rk]])),
        ('repro all three differ from matrix hit', m_rep(lambda g: [ev('r', 0)({rk: d})['node'].__setitem__('depth', 31) for d in g[rk]])),
        ('matrix run dropped', m_mat(lambda mm: mm.pop(first))),
        ('matrix params seed', m_mat(lambda mm: mm[first]['params'].__setitem__('Seed', 9))),
        ('matrix params cuts', m_mat(lambda mm: mm[first]['params'].pop('Cuts', None) if first.startswith('P') else mm[first]['params'].__setitem__('Cuts', 0))),
        ('matrix tool hash', m_mat(lambda mm: mm[first].__setitem__('tool_sha256', '0' * 64))),
        ('matrix run not ok', m_mat(lambda mm: mm[first].__setitem__('ok', False))),
    ]
    if evaluate(ctl, mat, base, out=lambda *a: None):
        print('self-test: synthetic repro base fails'); return 1
    bad = [n for n, args in muts if not evaluate(*args, out=lambda *a: None)]
    # tier function on hand-made events
    mk = lambda c, dp, par, rs, me: {'caller': c, 'depth': dp, 'node': {'parent': par}, 'restarts': rs, 'root_is_self': me}
    cases = [(mk('5fb8ec', 1, 3.0, 0, True), 'A'), (mk('5fb8ec', 1, 3.0, 1, False), 'B'), (mk('5fb8ec', 2, 3.0, 0, True), 'C'),
             (mk('5fb8ec', 1, None, 0, True), 'C'), (mk('8e4682', 1, 3.0, 0, True), 'C')]
    bad += [f'tier {c}' for c, want in cases if tier(c) != want]
    print(f'self-test: {len(muts)} mutations + {len(cases)} tier cases, not rejected: {bad}')
    return 1 if bad else 0


if __name__ == '__main__':
    a = sys.argv[1:]
    st = '--self-test' in a
    wm = '--write-manifest' in a
    a = [x for x in a if x not in ('--self-test', '--write-manifest')]
    root = a[0] if a else 'bench/oracle/mip_retry_bnb_scan'
    if wm:
        write_manifest(root); sys.exit(0)
    if st:
        sys.exit(self_test(root))
    e = []
    check_manifest(root, e)
    for x in e:
        print('ERROR', x)
    sys.exit(1 if evaluate(*collect(root)) or e else 0)
