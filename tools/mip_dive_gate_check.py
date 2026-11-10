#!/usr/bin/env python3
"""M2-03d-11 checker: DynamicRows gate inputs and outcome, contract docs/m2/mip_dive_gate.md section 3.

(default)         manifest hashes, then M (inputs), B (3 runs identical), N (M2-03d-8 projection), I (identity), R (rule),
                  P (C:526 / C:1581 pairing and M2-03d-8 gate events), D (10e662 / d:sel link), C (DynamicRows=0 control),
                  V (clause discrimination report)
--self-test       also requires every in-memory tamper to be rejected
--write-manifest  record capture, tool, reference and model hashes
--sweep DIR       exploration only: apply I, R, P to every *.json.gz capture in DIR and print clause counts (single runs, no manifest)
Exit code 0 only when every requested check passes.
"""
import copy, glob, json, os, sys
from collections import Counter

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mip_dive_rows_check import BINARY, CASES, MIPLIB, RUNS, Fail, load, need, sha  # noqa: E402

DATA = 'bench/oracle/mip_dive_gate'
MANIFEST = 'bench/oracle/mip_dive_gate_manifest.json'
TOOLS = ['tools/mip_dive_gate.js', 'tools/mip_dive_gate_runs.sh']
REF8 = 'bench/oracle/mip_dive_rows'          # M2-03d-8 captures, same cases and params
REACT_RET = 0x51096e                         # 5fb8ec+0x79e, return of the C:526 10e662 call
SEQ = ['51305f', '513078', '51319d', '5131af', '5131b7', '5131c2']
VARIANTS = ['dyn', 'depth526', 'depth1581', 'state', 'a0', 'sol', 'a7f0ad', 'a7f923', 'p18_off', 'p18_1', 'p18_3']


def cap(case, r):
    return f'{DATA}/{case}_{r}.json.gz'


def load_all():
    return {c: {r: load(cap(c, r)) for r in RUNS} for c in CASES}


def gates(evs, w):
    """Gate evaluations in capture order, each with the positive-hook hits that follow it."""
    out, cur = [], None
    for i, e in enumerate(evs):
        if e['event'] in ('g:526', 'g:1581'):
            cur = (e, [])
            out.append(cur)
        elif e['event'] == 'g:hit':
            need(cur is not None and e['fb'] == cur[0]['fb'], f'{w}: event {i} hit {e["at"]} has no gate of its 5fb8ec call')
            cur[1].append(e)
    return out


def predict(g, hits, v=None):
    """Positive hooks the C rule says are reached (C:526, C:1581-1610 and 0x513198-0x5131bc); v drops or changes one clause."""
    dyn = g['dyn'] != 0 or v == 'dyn'
    if g['event'] == 'g:526':
        return ['510961'] if dyn and (g['depth'] == g['w34'] or v == 'depth526') else []
    seq = []
    if not (dyn and (g['depth'] == g['w34'] or v == 'depth1581') and (g['state'] == 2 or v == 'state')):
        return seq
    seq.append('51305f')
    if not ((not g['a0_null'] or v == 'a0') and (not g['sol_null'] or v == 'sol')):
        return seq
    seq += ['513078', '51319d']
    eax = {h['at']: h.get('eax') for h in hits}
    if eax.get('51319d') != 0 and v != 'a7f0ad':
        return seq
    seq.append('5131af')
    if eax.get('5131af') != 0 and v != 'a7f923':
        return seq
    seq.append('5131b7')
    lim = {'p18_1': 1, 'p18_3': 3}.get(v, 2)
    if g['p3_18'] < lim or v == 'p18_off':
        seq.append('5131c2')
    return seq


def first_fail(g):
    """The M2-03d-8 d:gate_counts key for a C:1581 evaluation (same clause order as tools/mip_dive_rows.js)."""
    if g['dyn'] == 0:
        return 'dyn0'
    return ('depth' if g['depth'] != g['w34'] else 'state' if g['state'] != 2 else 'a0' if g['a0_null']
            else 'sol' if g['sol_null'] else 'pass')


def check_identity(gs, w):
    for g, hits in gs:
        need(g['fb'] is not None and g['slot_is_node'] and g['rbx_is_param2'],
             f'I {w}: {g["event"]} fb {g["fb"]} k {g["k"]}: node slot {g["slot_is_node"]} rbx {g["rbx_is_param2"]}')
        for h in hits:
            if h['at'] == '5131b7':
                need(h['r15_is_param3'] and h['r15_18'] == g['p3_18'],
                     f'I {w}: fb {g["fb"]}: r15 = param_3 {h["r15_is_param3"]}, r15+0x18 {h["r15_18"]} != gate value {g["p3_18"]}')


def check_rule_one(gs, w, v=None):
    for g, hits in gs:
        got, exp = [h['at'] for h in hits], predict(g, hits, v)
        need(got == exp, f'R {w}: {g["event"]} fb {g["fb"]} k {g["k"]} node {g["node"]} depth {g["mip_depth"]}: hits {got} != rule {exp}')
    return len(gs)


def check_pairs(evs, gs, w):
    """C:526 at most once per 5fb8ec call and on the same node/dyn/depth/W+0x34 as the C:1581 evaluations of that call;
    M2-03d-8 d:gate / d:blocked / d:sel / d:gate_counts agree with the raw values."""
    by = {}
    for g, _ in gs:
        by.setdefault(g['fb'], []).append(g)
    lone = 0
    for fb, lst in by.items():
        g526 = [g for g in lst if g['event'] == 'g:526']
        need(len(g526) <= 1, f'P {w}: 5fb8ec call {fb} has {len(g526)} C:526 evaluations')
        if not g526:
            lone += 1
            continue
        for g in lst:
            if g['event'] == 'g:1581':
                for k in ('node', 'dyn', 'depth', 'w34', 'mip_depth'):
                    need(g[k] == g526[0][k], f'P {w}: 5fb8ec call {fb}: C:1581 {k} {g[k]} != C:526 {g526[0][k]}')
    passed = [g for g, hits in gs if any(h['at'] == '513078' for h in hits)]
    dg = [e for e in evs if e['event'] == 'd:gate']
    need(len(dg) == len(passed), f'P {w}: {len(dg)} d:gate events != {len(passed)} C:1581 passes')
    for a, g in zip(dg, passed):
        need((a['mip_depth'], a['call'], a['node'], a['node_depth'], a['p34'], a['dyn']) ==
             (g['mip_depth'], g['call'], g['node'], g['depth'], g['w34'], g['dyn']), f'P {w}: d:gate {a} != {g}')
    nsel = sum(1 for _, hits in gs if any(h['at'] == '5131c2' for h in hits))
    nsel8 = sum(1 for e in evs if e['event'] == 'd:sel')
    nblk8 = sum(1 for e in evs if e['event'] == 'd:blocked')
    need(nsel == nsel8, f'P {w}: {nsel} 5131c2 hits != {nsel8} d:sel')
    need(len(passed) - nsel == nblk8, f'P {w}: {len(passed) - nsel} blocked != {nblk8} d:blocked')
    gc = [e for e in evs if e['event'] == 'd:gate_counts']
    if gc:
        exp = {}
        for g, _ in gs:
            if g['event'] == 'g:1581':
                d = exp.setdefault(str(g['mip_depth']), Counter())
                d[first_fail(g)] += 1
        need({k: dict(v) for k, v in exp.items()} == gc[-1]['counts'], f'P {w}: d:gate_counts {gc[-1]["counts"]} != raw {exp}')
    return lone


def stats(gs):
    s = Counter()
    for g, hits in gs:
        t = g['event'][2:]
        s[f'{t}:eval'] += 1
        s[f'{t}:dyn={g["dyn"]}'] += 1
        if g['dyn'] != 0:
            s[f'{t}:depth_ne'] += g['depth'] != g['w34']
        if t == '1581':
            s[f'1581:first={first_fail(g)}'] += 1
            if g['dyn'] != 0 and g['depth'] == g['w34']:
                s[f'1581:state={g["state"]}'] += 1
            s['1581:a0_null'] += g['a0_null']
            s['1581:sol_null'] += g['sol_null']
            s[f'1581:p3_18={g["p3_18"]}'] += 1
        for h in hits:
            s[f'hit:{h["at"]}'] += 1
            if h.get('eax'):
                s[f'hit:{h["at"]}:eax!=0'] += 1
    return {k: v for k, v in s.items() if v}


def check_inputs(data, man):
    tool = sha('tools/mip_dive_gate.js')
    for c, spec in CASES.items():
        for r, d in data[c].items():
            w = f'M {c} run {r}'
            need(d['ok'] and not d['errors'] and d['kind'] == 'dive_gate', f'{w}: capture not ok: {d["errors"][:1]}')
            need(d['binary_sha256'] == BINARY, f'{w}: binary sha256 {d["binary_sha256"]}')
            need(d['tool_sha256'] == tool, f'{w}: tool sha256 {d["tool_sha256"]} != current {tool}')
            need(d['model_sha256'] == man['models'][spec['model']], f'{w}: model sha256 {d["model_sha256"]}')
            need(d['params'] == spec['params'], f'{w}: params {d["params"]}')
            need(d['target'] == spec['target'] and d['output'] == spec['output'], f'{w}: target/output {d["target"]} {d["output"]!r}')
    return f'M: {len(CASES)} cases x {len(RUNS)} runs, binary/model/tool/params/output'


def check_repro(data):
    out = []
    for c in CASES:
        base = data[c][RUNS[0]]['events']
        need(len(base) > 0, f'B {c}: no events')
        for r in RUNS[1:]:
            ev = data[c][r]['events']
            need(len(ev) == len(base), f'B {c} run {r}: {len(ev)} events != {len(base)}')
            for i, (a, b) in enumerate(zip(base, ev)):
                need(a == b, f'B {c} run {r}: event {i} ({a.get("event")}) differs from run {RUNS[0]}')
        out.append(f'{c} {len(base)}')
    return 'B: 3 runs identical: ' + ', '.join(out)


def check_projection(data, ref):
    n = 0
    for c in CASES:
        ev8 = ref[c]['events']
        for r in RUNS:
            proj = [e for e in data[c][r]['events'] if e['event'][:2] in ('b:', 'd:')]
            need(len(proj) == len(ev8), f'N {c} run {r}: {len(proj)} b:/d: events != M2-03d-8 {len(ev8)}')
            for i, (a, b) in enumerate(zip(proj, ev8)):
                need(a == b, f'N {c} run {r}: event {i} ({a["event"]}) differs from M2-03d-8: {json.dumps(a)[:160]} != {json.dumps(b)[:160]}')
        n += len(ev8)
    return f'N: b:/d: projection = M2-03d-8 capture ({n} events over {len(CASES)} cases, 3 runs each)'


def check_case(c, evs, st=None):
    w = f'{c}'
    gs = gates(evs, w)
    need(gs, f'R {w}: no gate evaluations')
    check_identity(gs, w)
    n = check_rule_one(gs, w)
    lone = check_pairs(evs, gs, w)
    if st is not None:
        st[c] = stats(gs)
        st[c]['5fb8ec_without_526'] = lone
    return gs, n


def check_link(c, evs, gs):
    react = sum(1 for g, hits in gs if g['event'] == 'g:526' and g['mip_depth'] == 1 and hits)
    ops = sum(1 for e in evs if e['event'] == 'b:op' and e['op'] == 'react_all' and e['ret'] == REACT_RET)
    need(react == ops, f'D {c}: {react} top-level C:526 true != {ops} b:op react_all at 5fb8ec+0x79e')
    return react


def check_control(c, gs):
    for g, hits in gs:
        need(g['dyn'] == 0 and not hits, f'C {c}: {g["event"]} fb {g["fb"]} dyn {g["dyn"]} hits {[h["at"] for h in hits]}')


def run_checks(data, ref, man, st=None):
    lines = [check_inputs(data, man), check_repro(data), check_projection(data, ref)]
    total, links = 0, []
    for c, spec in CASES.items():
        evs = data[c][RUNS[0]]['events']
        gs, n = check_case(c, evs, st)
        total += n
        if spec.get('control'):
            check_control(c, gs)
        links.append(f'{c} {check_link(c, evs, gs)}')
    lines.append(f'I/R/P: {total} gate evaluations: identity, rule = positive hooks, C:526/C:1581 pairing, M2-03d-8 d:gate/d:sel/d:blocked/gate_counts')
    lines.append('D: top-level C:526 true = 10e662 at 5fb8ec+0x79e: ' + ', '.join(links))
    lines.append('C: DynamicRows=0 x3: dyn 0 at every gate evaluation, no positive hook')
    return lines


def discrimination(data):
    rows = []
    for v in VARIANTS:
        res = 'survives (C only)'
        for c in CASES:
            gs = gates(data[c][RUNS[0]]['events'], c)
            try:
                check_rule_one(gs, c, v)
            except Fail as e:
                res = 'rejected: ' + str(e)[:150]
                break
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


ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def ref_paths():
    return {c: f'{REF8}/{c}_1.json.gz' for c in CASES}


def write_manifest():
    data = load_all()
    models = {spec['model']: data[c][RUNS[0]]['model_sha256'] for c, spec in CASES.items()}
    man = {'task': 'M2-03d-11', 'binary_sha256': BINARY, 'models': models,
           'captures': {cap(c, r): sha(cap(c, r)) for c in CASES for r in RUNS},
           'tools': {p: sha(p) for p in TOOLS}, 'refs': {p: sha(p) for p in ref_paths().values()}}
    with open(os.path.join(ROOT, MANIFEST), 'w') as f:
        json.dump(man, f, indent=1, sort_keys=True)
        f.write('\n')
    print('wrote', MANIFEST)


def first(evs, pred):
    return next(i for i, e in enumerate(evs) if pred(e))


def tampers():
    """In-memory tampers; each must make run_checks fail. Input tampers aim at evaluations whose outcome they change."""
    N, A, G = 'nidda_H_s1', 'assign1-5-8_default', 'gmu-35-40_default'

    def ev(d, c, r=1):
        return d[c][r]['events']

    def every_run(c, fn):
        def m(d, ref, man):
            for r in RUNS:
                fn(d[c][r]['events'])
        return m

    def nth(evs, pred, k=0):
        return [e for e in evs if pred(e)][k]

    passes = lambda e: e['event'] == 'g:1581' and e['dyn'] != 0 and e['depth'] == e['w34'] and e['state'] == 2
    true526 = lambda e: e['event'] == 'g:526' and e['dyn'] != 0 and e['depth'] == e['w34']
    st_fail = lambda e: e['event'] == 'g:1581' and e['dyn'] != 0 and e['depth'] == e['w34'] and e['state'] != 2
    depth_fail = lambda e: e['event'] == 'g:1581' and e['dyn'] != 0 and e['depth'] != e['w34']

    def set_field(c, pred, k, val):
        return every_run(c, lambda evs: nth(evs, pred).__setitem__(k, val))

    def drop_hit(c, at):
        def f(evs):
            del evs[first(evs, lambda e: e['event'] == 'g:hit' and e['at'] == at)]
        return every_run(c, f)

    def add_hit(c, pred, at):
        def f(evs):
            i = first(evs, pred)
            evs.insert(i + 1, {'event': 'g:hit', 'at': at, 'fb': evs[i]['fb']})
        return every_run(c, f)

    def set_hit(c, at, k, val):
        return every_run(c, lambda evs: nth(evs, lambda e: e['event'] == 'g:hit' and e['at'] == at).__setitem__(k, val))

    yield 'binary sha256', lambda d, r, m: d[N][1].__setitem__('binary_sha256', '0' * 64)
    yield 'tool sha256', lambda d, r, m: d[A][2].__setitem__('tool_sha256', '0' * 64)
    yield 'model sha256', lambda d, r, m: d[G][3].__setitem__('model_sha256', '0' * 64)
    yield 'params', lambda d, r, m: d[N][1]['params'].__setitem__('NodeLimit', 32897)
    yield 'output line', lambda d, r, m: d[A][1].__setitem__('output', 'DONE 8 2999')
    yield 'run 2 differs', lambda d, r, m: nth(ev(d, G, 2), lambda e: e['event'] == 'g:526').__setitem__('k', 99)
    yield 'no events', lambda d, r, m: [d[A][x].__setitem__('events', []) for x in RUNS]
    yield 'M2-03d-8 projection (one d:sel row)', every_run(N, lambda evs: nth(evs, lambda e: e['event'] == 'd:sel' and e['count'] > 0)['list'].pop())
    yield 'M2-03d-8 reference event', lambda d, r, m: r[G]['events'].pop()
    # raw inputs of evaluations whose outcome they change
    yield 'dyn of a passing C:1581 -> 0', set_field(N, passes, 'dyn', 0)
    yield 'depth of a passing C:1581', set_field(N, passes, 'depth', 7777)
    yield 'W+0x34 of a true C:526', set_field(G, true526, 'w34', -5)
    yield 'state of a failing C:1581 -> 2', set_field(A, st_fail, 'state', 2)
    yield 'state of a passing C:1581 -> 1', set_field(N, passes, 'state', 1)
    yield 'depth-failing C:1581 made equal', every_run(N, lambda evs: nth(evs, depth_fail).__setitem__('w34', nth(evs, depth_fail)['depth']))
    yield 'a0 null on a passing C:1581', set_field(N, passes, 'a0_null', True)
    yield 'sol null on a passing C:1581', set_field(G, passes, 'sol_null', True)
    yield 'param_3+0x18 = 2 on a passing C:1581', set_field(N, passes, 'p3_18', 2)
    yield 'a7f0ad eax 1', set_hit(N, '51319d', 'eax', 1)
    yield 'a7f923 eax 1', set_hit(A, '5131af', 'eax', 1)
    yield 'r15+0x18 differs from gate value', set_hit(G, '5131b7', 'r15_18', 0)
    yield 'r15 != param_3', set_hit(N, '5131b7', 'r15_is_param3', False)
    yield 'node slot identity', set_field(N, lambda e: e['event'] == 'g:526', 'slot_is_node', False)
    yield 'rbx identity', set_field(A, lambda e: e['event'] == 'g:1581', 'rbx_is_param2', False)
    yield 'C:1581 node != C:526 node', set_field(G, passes, 'node', -1.0)
    yield 'drop hit 510961', drop_hit(N, '510961')
    yield 'drop hit 5131c2', drop_hit(A, '5131c2')
    yield 'extra hit 51305f after a depth failure', add_hit(N, depth_fail, '51305f')
    yield 'control: positive hook added', add_hit('nidda_H_s1_dr0', lambda e: e['event'] == 'g:526', '510961')
    yield 'control: dyn -1', set_field('nidda_H_s1_dr0', lambda e: e['event'] == 'g:1581', 'dyn', -1)


def sweep(d):
    ok = True
    rows = {}
    for p in sorted(glob.glob(os.path.join(d, '*.json.gz'))):
        c = os.path.basename(p)[:-8]
        cap_ = load(os.path.abspath(p))
        st = {}
        if not any(e['event'] == 'g:526' for e in cap_['events']):
            print(f'{c:28} {cap_["output"]:16} no 5fb8ec gate evaluation')
            continue
        try:
            check_case(c, cap_['events'], st)
            res = 'MATCH'
        except Fail as e:
            res, ok = 'MISMATCH ' + str(e)[:200], False
            st[c] = stats(gates(cap_['events'], c)) if res.startswith('MISMATCH R') else {}
        rows[c] = (cap_['output'], res, st.get(c, {}))
        print(f'{c:28} {cap_["output"]:16} {res}')
        print('    ', json.dumps(st.get(c, {}), sort_keys=True))
    tot = Counter()
    for _, _, s in rows.values():
        tot.update(s)
    print('TOTAL', json.dumps(dict(tot), sort_keys=True))
    return ok


def main():
    args = sys.argv[1:]
    if '--sweep' in args:
        ok = sweep(args[args.index('--sweep') + 1])
        print('SWEEP', 'MATCH' if ok else 'MISMATCH')
        return 0 if ok else 1
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
            print(f'counts {c}:', json.dumps(st[c], sort_keys=True))
        print('V: clause variants')
        for v, res in discrimination(data):
            print(f'   {v:10} {res}')
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
