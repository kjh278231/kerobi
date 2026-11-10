#!/usr/bin/env python3
"""M2-03d-13 checker: in-place split of 10f448, contract docs/m2/mip_dive_split.md section 3.

(default)         manifest hashes, then M (inputs), B (3 runs identical), N (M2-03d-11 projection), X (entry/exit consistency),
                  R (C column loop reproduces every rebuild), T (target counts), C (DynamicRows=0 control), K (counts), V (rule variants)
--self-test       also requires every in-memory tamper to be rejected
--write-manifest  record capture, tool, reference and model hashes
Exit code 0 only when every requested check passes.
"""
import copy, json, os, sys
from collections import Counter

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mip_dive_rows_check import BINARY, CASES, MIPLIB, RUNS, Fail, load, need, sha  # noqa: E402
from mip_dive_gate_check import check_repro  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = 'bench/oracle/mip_dive_split'
MANIFEST = 'bench/oracle/mip_dive_split_manifest.json'
TOOLS = ['tools/mip_dive_split.js', 'tools/mip_dive_split_runs.sh']
REF11 = 'bench/oracle/mip_dive_gate'          # M2-03d-11 captures, same tool below the s:* additions
ZERO = '0000000000000000'
POSITIVE = [c for c in CASES if not CASES[c].get('control')]
VARIANTS = {'stable': True, 'noval': True, 'xlen_len': True, 'forward': False, 'rhs_keep': False}   # True: must be rejected


def cap(case, r):
    return f'{DATA}/{case}_{r}.json.gz'


def load_all():
    return {c: {r: load(cap(c, r)) for r in RUNS} for c in CASES}


def ref_paths():
    return {c: f'{REF11}/{c}_1.json.gz' for c in CASES}


def split(ind, val, listed, v=None):
    """10f448 column loop (section 2) on one column; returns (ind, val, reduced length, exchanges). v: a wrong variant."""
    ind, val, n = list(ind), list(val), len(ind)
    if v == 'stable':
        keep = [k for k in range(n) if ind[k] not in listed]
        move = [k for k in range(n) if ind[k] in listed]
        o = keep + move
        return [ind[k] for k in o], [val[k] for k in o], len(keep), sum(1 for i, k in enumerate(o) if i != k)
    if v == 'forward':
        x = 0
        for a in range(n):
            if ind[a] in listed:
                u = next((k for k in range(a + 1, n) if ind[k] not in listed), None)
                if u is None:
                    break
                ind[a], ind[u], val[a], val[u] = ind[u], ind[a], val[u], val[a]
                x += 1
        return ind, val, sum(1 for r in ind if r not in listed), x
    a, b, x = 0, n - 1, 0
    while a <= b:
        if ind[a] not in listed:
            a += 1
            continue
        while a <= b:
            if ind[b] not in listed:
                if a < b:
                    ind[a], ind[b] = ind[b], ind[a]
                    if v != 'noval':
                        val[a], val[b] = val[b], val[a]
                    a, b, x = a + 1, b - 1, x + 1
                break
            b -= 1
    return ind, val, (n if v == 'xlen_len' else a), x


def rebuilds(evs, w):
    """s:build events with the reconstructed entry state (beg, len, columns) and exit columns."""
    refd = {e['before_ref'] for e in evs if e['event'] == 's:build' and e['before_ref'] is not None}
    kept, prev = {}, 0
    for e in evs:
        if e['event'] != 's:build':
            continue
        s = e['serial']
        need(s == prev + 1, f'X {w}: s:build serial {s} after {prev}')
        prev = s
        if e['before_ref'] is None:
            beg, ln, ind = e['beg'], e['len'], e['ind']
            need(len(beg) == len(ln) == e['cols'] and len(ind) == len(e['val_i']) == sum(ln), f'X {w}: build {s}: full state sizes')
            val = [e['val_u'][i] for i in e['val_i']]
            cols, k = [], 0
            for n in ln:
                cols.append((tuple(ind[k:k + n]), tuple(val[k:k + n])))
                k += n
            st = (tuple(beg), tuple(ln), cols)
        else:
            need(e['before_ref'] in kept, f'X {w}: build {s}: before_ref {e["before_ref"]} is not an earlier build')
            st = kept[e['before_ref']]
            need(len(st[1]) == e['cols'], f'X {w}: build {s}: referenced state has {len(st[1])} columns, event {e["cols"]}')
        cols = list(st[2])
        for j, mv in e['perm']:
            ind, val = list(cols[j][0]), list(cols[j][1])
            bi, bv = cols[j]
            for k, src in mv:
                ind[k], val[k] = bi[src], bv[src]
            cols[j] = (tuple(ind), tuple(val))
        for j, ind, val in e['raw']:
            cols[j] = (tuple(ind), tuple(val))
        after = (st[0], st[1], cols)
        if s in refd:
            kept[s] = after
        yield e, st, after


def check_consistency(e, st, w):
    s = e['serial']
    for k in ('ref_after', 'built_after'):
        need(e[k] == 1, f'X {w}: build {s}: {k} {e[k]} != 1')
    for k in ('len_is_xlen', 'rhs_is_xrhs', 'xlen_is_len', 'xrhs_is_rhs', 'ind_ptr_same'):
        need(e[k] is True, f'X {w}: build {s}: {k} {e[k]}')
    need(e['val_moved'] == 0, f'X {w}: build {s}: {e["val_moved"]} exit val entries differ from the entry val at their source')
    need(e['raw'] == [], f'X {w}: build {s}: columns that are not a permutation of the entry column: {[r[0] for r in e["raw"]][:5]}')
    need(e['x_list_pos'] == list(range(len(e['x_list']))) and e['x_marked_count'] == len(e['x_list']),
         f'X {w}: build {s}: X positions {e["x_list_pos"][:8]} / {e["x_marked_count"]} do not match the list of {len(e["x_list"])}')
    need(e['x_marked_count'] + sum(e['x_pos_neg'].values()) == e['rows'], f'X {w}: build {s}: positions cover {e["rows"]} rows')
    need(len(set(e['x_list'])) == len(e['x_list']) and all(0 <= r < e['rows'] for r in e['x_list']), f'X {w}: build {s}: list rows')


def check_rule(e, st, after, w, v=None, stats=None):
    """R: every column of the exit state is the C split of the entry column; reduced lengths and rhs copy."""
    s, listed = e['serial'], set(e['x_list'])
    xlen = list(st[1])
    for j, x in e['xlen_diff']:
        xlen[j] = x
    ex = 0
    for j, (bc, ac) in enumerate(zip(st[2], after[2])):
        if listed.isdisjoint(bc[0]):
            pi, pv, pl, x = bc[0], bc[1], len(bc[0]), 0
        else:
            pi, pv, pl, x = split(bc[0], bc[1], listed, v)
        need(tuple(pi) == ac[0], f'R {w}: build {s} col {j}: ind {list(ac[0])} != rule {list(pi)} (entry {list(bc[0])}, listed {sorted(listed & set(bc[0]))})')
        need(tuple(pv) == ac[1], f'R {w}: build {s} col {j}: val bits differ from the rule')
        need(pl == xlen[j], f'R {w}: build {s} col {j}: reduced length {xlen[j]} != rule {pl}')
        if stats is not None and x:
            ex += x
            stats['cols_exchanged'] += 1
            stats['cols_2plus'] += x >= 2
            si, _, _, _ = split(bc[0], bc[1], listed, 'stable')
            stats['cols_not_stable'] += tuple(si) != ac[0]
    zero = [] if v == 'rhs_keep' else sorted(e['rhs_list_nonzero'])
    need(e['rhs_diff_rows'] == zero, f'R {w}: build {s}: rhs copy differs at rows {e["rhs_diff_rows"][:8]}, rule {zero[:8]}')
    need(e['rhs_diff_nonzero'] == [], f'R {w}: build {s}: rhs copy rows not +0: {e["rhs_diff_nonzero"][:3]}')
    need(set(e['rhs_list_nonzero']) <= listed, f'R {w}: build {s}: rhs rows outside the list')
    if stats is not None:
        stats['builds'] += 1
        stats['builds_exchange'] += ex > 0
        stats['exchanges'] += ex
        stats['full'] += e['before_ref'] is None
        stats['rhs_zeroed'] += len(zero)
        stats['callers'][hex(e['ret'])] += 1
        stats['depths'][e['mip_depth']] += 1


def check_case(c, evs, v=None, stats=None):
    w = c
    n = 0
    for e, st, after in rebuilds(evs, w):
        if v is None:
            check_consistency(e, st, w)
        check_rule(e, st, after, w, v, stats)
        n += 1
    counts = [e for e in evs if e['event'] == 's:counts']
    need(len(counts) == 1, f'X {w}: {len(counts)} s:counts events')
    builds = sum(k for key, k in counts[0]['counts'].items() if key.endswith(':build'))
    need(builds == n, f'X {w}: s:counts says {builds} rebuilds, {n} s:build events')
    return n, counts[0]['counts']


def check_inputs(data, man):
    tool = sha('tools/mip_dive_split.js')
    for c, spec in CASES.items():
        for r, d in data[c].items():
            w = f'M {c} run {r}'
            need(d['ok'] and not d['errors'] and d['kind'] == 'dive_split', f'{w}: capture not ok: {d["errors"][:1]}')
            need(d['binary_sha256'] == BINARY, f'{w}: binary sha256 {d["binary_sha256"]}')
            need(d['tool_sha256'] == tool, f'{w}: tool sha256 {d["tool_sha256"]} != current {tool}')
            need(d['model_sha256'] == man['models'][spec['model']], f'{w}: model sha256 {d["model_sha256"]}')
            need(d['params'] == spec['params'], f'{w}: params {d["params"]}')
            need(d['target'] == spec['target'] and d['output'] == spec['output'], f'{w}: target/output {d["target"]} {d["output"]!r}')
    return f'M: {len(CASES)} cases x {len(RUNS)} runs, binary/model/tool/params/output'


def unaddressed(evs):
    """Events without s:* and without the pointer-derived LP numbering of lpId (b:lpnew, every 'lp' field): which address an LP
    object lands on is allocator luck and differs once the hooks allocate more (docs/m2/mip_dive_split.md section 3)."""
    return [{k: v for k, v in e.items() if k != 'lp'} for e in evs if not e['event'].startswith('s:') and e['event'] != 'b:lpnew']


def check_projection(data, ref):
    n = 0
    for c in CASES:
        ev11 = unaddressed(ref[c]['events'])
        for r in RUNS:
            proj = unaddressed(data[c][r]['events'])
            need(len(proj) == len(ev11), f'N {c} run {r}: {len(proj)} non-s: events != M2-03d-11 {len(ev11)}')
            for i, (a, b) in enumerate(zip(proj, ev11)):
                need(a == b, f'N {c} run {r}: event {i} ({a["event"]}) differs from M2-03d-11: {json.dumps(a)[:160]} != {json.dumps(b)[:160]}')
        n += len(ev11)
    return f'N: projection without s:*, b:lpnew and lp = M2-03d-11 capture ({n} events, {len(CASES)} cases x 3 runs)'


def new_stats():
    return {'builds': 0, 'builds_exchange': 0, 'exchanges': 0, 'cols_exchanged': 0, 'cols_2plus': 0, 'cols_not_stable': 0, 'full': 0,
            'rhs_zeroed': 0, 'callers': Counter(), 'depths': Counter()}


def run_checks(data, ref, man, st=None):
    st = {} if st is None else st
    lines = [check_inputs(data, man), check_repro(data), check_projection(data, ref)]
    for c in CASES:
        st[c] = new_stats()
        n, counts = check_case(c, data[c][RUNS[0]]['events'], None, st[c])
        st[c]['counts'] = counts
    lines.append(f'X/R: every rebuild of run 1 consistent and reproduced by the C column loop ({sum(st[c]["builds"] for c in CASES)} rebuilds)')
    for c in POSITIVE:
        need(st[c]['builds'] >= 1, f'T {c}: no rebuild')
    for k in ('builds_exchange', 'cols_not_stable'):
        need(sum(st[c][k] for c in POSITIVE) >= 1, f'T: no {k} in the positive cases')
    lines.append('T: rebuilds in every positive case; ' + ', '.join(f'{k} {sum(st[c][k] for c in POSITIVE)}'
                                                             for k in ('builds_exchange', 'cols_not_stable', 'cols_2plus')))
    for c in CASES:
        if CASES[c].get('control'):
            need(st[c]['builds'] == 0 and all(k.endswith(':xnull') for k in st[c]['counts']),
                 f'C {c}: DynamicRows=0 has entries with an X: {st[c]["counts"]}')
            lines.append(f'C: {c}: 0 rebuilds, {sum(st[c]["counts"].values())} 10f448 calls all without X')
    return lines


def discrimination(data):
    rows = []
    for v, must in VARIANTS.items():
        try:
            for c in POSITIVE:
                check_case(c, data[c][RUNS[0]]['events'], v)
            res = 'NOT DISCRIMINATED'
        except Fail as e:
            res = 'rejected: ' + str(e)[:110]
        need(not must or res != 'NOT DISCRIMINATED', f'V: variant {v} is not rejected by the captures')
        rows.append((v, res))
    return rows


def check_manifest(man):
    for group in ('captures', 'tools', 'refs'):
        for p, h in man[group].items():
            need(sha(p) == h, f'manifest: {p} sha256 {sha(p)} != {h}')
    need(set(man['captures']) == {cap(c, r) for c in CASES for r in RUNS}, 'manifest: capture list')
    need(set(man['tools']) == set(TOOLS), 'manifest: tool list')
    for name, h in man['models'].items():
        p = f'{MIPLIB}/{name}.mps.gz'
        if os.path.exists(os.path.join(ROOT, p)):
            need(sha(p) == h, f'manifest: model {name} sha256 differs')


def write_manifest():
    data = load_all()
    models = {spec['model']: data[c][RUNS[0]]['model_sha256'] for c, spec in CASES.items()}
    man = {'task': 'M2-03d-13', 'binary_sha256': BINARY, 'models': models,
           'captures': {cap(c, r): sha(cap(c, r)) for c in CASES for r in RUNS},
           'tools': {p: sha(p) for p in TOOLS}, 'refs': {p: sha(p) for p in ref_paths().values()}}
    with open(os.path.join(ROOT, MANIFEST), 'w') as f:
        json.dump(man, f, indent=1, sort_keys=True)
        f.write('\n')
    print('wrote', MANIFEST)


def tampers():
    """In-memory tampers; each must make run_checks fail."""
    N, A, G, Z = 'nidda_H_s1', 'assign1-5-8_default', 'gmu-35-40_default', 'nidda_H_s1_dr0'

    def every_run(c, fn):
        def m(d, ref, man):
            for r in RUNS:
                fn(d[c][r]['events'])
        return m

    def moved(evs):
        """First build with a column that has at least two moves, and that column."""
        for e in evs:
            if e['event'] == 's:build':
                for p in e['perm']:
                    if len(p[1]) >= 2:
                        return e, p
        raise StopIteration

    def swap_src(evs):
        e, p = moved(evs)
        p[1][0][1], p[1][1][1] = p[1][1][1], p[1][0][1]

    def unlist(e):
        # the list of a build whose original moved entries: with no listed row the rule moves nothing
        e.update(x_list=[], x_list_pos=[], x_marked_count=0, rhs_list_nonzero=[], rhs_diff_rows=[])
        e['x_pos_neg'] = {'-1': e['rows']}

    def drop_builds(c):
        def m(d, ref, man):
            for r in RUNS:
                evs = d[c][r]['events']
                evs[:] = [e for e in evs if e['event'] != 's:build']
                cnt = next(e for e in evs if e['event'] == 's:counts')['counts']
                for k in list(cnt):
                    if k.endswith(':build'):
                        del cnt[k]
        return m

    def nth_build(c, fn, pred=lambda e: True):
        return every_run(c, lambda evs: fn(next(e for e in evs if e['event'] == 's:build' and pred(e))))

    def add_control_build(d, ref, man):
        for r in RUNS:
            e = copy.deepcopy(next(x for x in d[N][r]['events'] if x['event'] == 's:build' and x['before_ref'] is None))
            e['serial'] = 1
            evs = d[Z][r]['events']
            evs.insert(len(evs) - 1, e)
            cnt = next(x for x in evs if x['event'] == 's:counts')['counts']
            cnt['d1:153d71:build'] = 1

    def xlen_plus(e):
        if e['xlen_diff']:
            e['xlen_diff'][0][1] += 1
        else:
            e['xlen_diff'].append([0, e['len'][0] + 1] if 'len' in e else [0, 10 ** 6])

    yield 'binary sha256', lambda d, r, m: [d[A][x].__setitem__('binary_sha256', '0' * 64) for x in RUNS]
    yield 'empty events', lambda d, r, m: [d[G][x].__setitem__('events', []) for x in RUNS]
    yield 'run 2 one event', lambda d, r, m: next(e for e in d[N][2]['events'] if e['event'] == 's:build').__setitem__('call', -1)
    yield 'projection (one g: event dropped)', every_run(A, lambda evs: evs.remove(next(e for e in evs if e['event'] == 'g:526')))
    yield 'projection (one d:sel list row)', every_run(N, lambda evs: next(e for e in evs if e['event'] == 'd:sel' and e['count'] > 0)['list'].pop())
    yield 'no rebuilds in a positive case (T)', drop_builds(A)
    yield 'exit column: two moves exchanged', every_run(N, swap_src)
    yield 'val moved without ind', nth_build(G, lambda e: e.__setitem__('val_moved', 1))
    yield 'reduced length +1', nth_build(N, xlen_plus, lambda e: bool(e['perm']))
    yield 'X position mismatch', nth_build(A, lambda e: e['x_list_pos'].__setitem__(0, 5), lambda e: len(e['x_list']) > 1)
    yield 'list emptied on a build with moves', nth_build(N, unlist, lambda e: bool(e['perm']))
    def ref_other(evs):
        # the first referencing build, pointed at an earlier build of a D with another column count
        bs = [e for e in evs if e['event'] == 's:build']
        e = next(e for e in bs if e['before_ref'] is not None and any(x['cols'] != e['cols'] for x in bs if x['serial'] < e['serial']))
        e['before_ref'] = next(x['serial'] for x in bs if x['serial'] < e['serial'] and x['cols'] != e['cols'])

    yield 'before_ref to a build of another D', every_run(G, ref_other)
    yield 'control has a rebuild', add_control_build
    yield 'rhs copy row not +0', nth_build(G, lambda e: e['rhs_diff_nonzero'].append([0, '3ff0000000000000']))
    yield 'rhs zeroed row missing', nth_build(N, lambda e: e['rhs_diff_rows'].pop(), lambda e: bool(e['rhs_diff_rows']))
    yield 'non-permutation column', nth_build(A, lambda e: e['raw'].append([0, [], []]))
    yield 'ref_after 2', nth_build(A, lambda e: e.__setitem__('ref_after', 2))


def main():
    args = sys.argv[1:]
    if '--write-manifest' in args:
        write_manifest()
    try:
        man = load(MANIFEST)
        check_manifest(man)
        data = load_all()
        ref = {c: load(p) for c, p in ref_paths().items()}
        st = {}
        for line in run_checks(data, ref, man, st):
            print(line)
        for c in CASES:
            s = dict(st[c], callers=dict(st[c]['callers']), depths=dict(st[c]['depths']))
            print(f'K {c}:', json.dumps(s, sort_keys=True))
        print('V: rule variants')
        for v, res in discrimination(data):
            print(f'   {v:9} {res}')
    except Fail as e:
        print('FAIL', e)
        return 1
    if '--self-test' in args:
        failed = 0
        for name, mut in tampers():
            d, r, m = copy.deepcopy(data), copy.deepcopy(ref), copy.deepcopy(man)
            try:
                mut(d, r, m)
                run_checks(d, r, m)
                print(f'NOT REJECTED: {name}')
                failed += 1
            except Fail as e:
                print(f'rejected: {name}: {str(e)[:120]}')
            except (IndexError, KeyError, ValueError, StopIteration) as e:
                print(f'NOT REJECTED (tamper error {type(e).__name__}: {e}): {name}')
                failed += 1
        if failed:
            print('FAIL', failed, 'tampers not rejected')
            return 1
    print('PASS')
    return 0


if __name__ == '__main__':
    sys.exit(main())
