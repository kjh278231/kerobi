#!/usr/bin/env python3
"""M2-03d-8 checker: DynamicRows dive row selection rule, contract docs/m2/mip_dive_rows.md section 3.

(default)         manifest hashes, then M (inputs), B (3 runs identical), N (M2-03d-7 projection), G (gate), D (10e5fc argument),
                  A (66661c call rows and results), R (rule reconstruction), C (DynamicRows=0 control), V (clause discrimination report)
--self-test       also requires every in-memory tamper to be rejected
--write-manifest  record capture, tool, reference and model hashes
--sweep DIR       exploration only: apply R and A to every *.json.gz capture in DIR (no manifest, single runs)
Exit code 0 only when every requested check passes.
"""
import copy, glob, gzip, hashlib, json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mip_dive_rows_rule import Model, bits, cutoff, select, selections  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = 'bench/oracle/mip_dive_rows'
MANIFEST = 'bench/oracle/mip_dive_rows_manifest.json'
TOOLS = ['tools/mip_dive_rows.js', 'tools/mip_dive_rows_runs.sh', 'tools/mip_dive_rows_rule.py']
REF7 = 'bench/oracle/mip_blank_rows/nidda_bnb_H_s1_1.json'      # M2-03d-7 capture (b:* events, same settings)
MIPLIB = '../re/bench/data/miplib'
BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
H = {'Threads': 1, 'Seed': 1, 'LogToConsole': 0, 'NodeLimit': 32898, 'Presolve': 0, 'ScaleFlag': 0, 'Quad': 0, 'NumericFocus': 0,
     'MarkowitzTol': 0.0001, 'Heuristics': 0, 'RINS': 0}
DEF = {'Threads': 1, 'Seed': 0, 'LogToConsole': 0, 'NodeLimit': 3000}
CASES = {
    'nidda_H_s1': {'model': 'neos-3754480-nidda', 'params': H, 'target': 19901, 'output': 'DONE 8 32898'},
    'nidda_H_s1_dr0': {'model': 'neos-3754480-nidda', 'params': dict(H, GURO_PAR_DYNAMICROWS=0), 'target': 19901,
                       'output': 'DONE 8 32898', 'control': True},
    'gmu-35-40_default': {'model': 'gmu-35-40', 'params': DEF, 'target': 0, 'output': 'DONE 8 3000'},
    'assign1-5-8_default': {'model': 'assign1-5-8', 'params': DEF, 'target': 0, 'output': 'DONE 8 3000'},
}
RUNS = (1, 2, 3)
F5FB8EC = (0x5101d0, 21510)
DEACT_RET = 0x513999          # 5fb8ec+0x37c9, the C:1856 10e5fc call
RESTART_REACT = 0x5807b1      # 6a6563+0x211, restart path 10e662 (C:123-124, no DynamicRows gate)
NIDDA_DEACT = 1145            # M2-03d-7: top-level deact 10 (before the restart) + 1135
VARIANTS = ['no_flag', 'no_b7', 'tol0', 'tol_sign', 'no_maxact', 'no_eq_min', 'eq_cut', 'no_dual', 'f666_0', 'f666_1',
            'f666_noround', 'f666_nocap', 'no_t14', 'no_40', 'slack0', 'cuts_first']


class Fail(Exception):
    pass


def need(cond, msg):
    if not cond:
        raise Fail(msg)


def sha(p):
    with open(os.path.join(ROOT, p), 'rb') as f:
        return hashlib.sha256(f.read()).hexdigest()


def load(p):
    op = gzip.open if p.endswith('.gz') else open
    with op(os.path.join(ROOT, p), 'rt') as f:
        return json.load(f)


def cap(case, r):
    return f'{DATA}/{case}_{r}.json.gz'


def load_all():
    return {c: {r: load(cap(c, r)) for r in RUNS} for c in CASES}


def sels(evs, depth=None):
    return [e for e in evs if e['event'] == 'd:sel' and (depth is None or e['mip_depth'] == depth)]


def check_inputs(data, man):
    tool = sha('tools/mip_dive_rows.js')
    for c, spec in CASES.items():
        for r, d in data[c].items():
            w = f'M {c} run {r}'
            need(d['ok'] and not d['errors'], f'{w}: capture not ok: {d["errors"][:1]}')
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
    ev7 = ref['events']
    for r in RUNS:
        proj = [e for e in data['nidda_H_s1'][r]['events'] if e['event'].startswith('b:')]
        need(len(proj) == len(ev7), f'N run {r}: {len(proj)} b:* events != M2-03d-7 {len(ev7)}')
        for i, (a, b) in enumerate(zip(proj, ev7)):
            need(a == b, f'N run {r}: b:* event {i} ({a["event"]}) differs from M2-03d-7: {json.dumps(a)[:160]} != {json.dumps(b)[:160]}')
    return f'N: nidda b:* projection = M2-03d-7 capture ({len(ev7)} events, 3 runs)'


def check_gate(data):
    out = []
    for c, spec in CASES.items():
        if spec.get('control'):
            continue
        ev = data[c][RUNS[0]]['events']
        pend, gates, blocked = {}, 0, 0
        for e in ev:
            k = e['event']
            if k == 'd:gate':
                need(e['rbx_is_param2'], f'G {c}: rbx != param_2 at gate call {e["call"]}')
                need(e['dyn'] != 0 and e['node_depth'] == e['p34'], f'G {c}: gate event with a false clause: {e}')
                need(e['mip_depth'] not in pend, f'G {c}: gate call {e["call"]} without a selection after the previous gate')
                pend[e['mip_depth']] = e
                gates += 1
            elif k in ('d:sel', 'd:blocked'):
                g = pend.pop(e['mip_depth'], None)
                need(g is not None and g['node'] == e['node'], f'G {c}: {k} call {e["call"]} node {e["node"]} without its gate ({g})')
                if k == 'd:blocked':
                    blocked += 1
                    continue
                need(e['entered'] and not e['bad_slots'], f'G {c}: sel call {e["call"]}: entered {e["entered"]}, slots {e["bad_slots"]}')
                need(e['dyn'] != 0, f'G {c}: sel call {e["call"]} with DynamicRows 0')
                if e['nrows'] > 0:
                    need(e['cutoff_stack'] == bits(cutoff(e)), f'G {c}: sel call {e["call"]}: cutoff {e["cutoff_stack"]} != 5e7204 - I+0x10')
            elif k == 'd:66661c_outside':
                raise Fail(f'G {c}: 66661c called outside a selection')
        need(not pend, f'G {c}: gates without selection: {list(pend.values())[:1]}')
        gc = [e for e in ev if e['event'] == 'd:gate_counts']
        need(gc, f'G {c}: no gate counts')
        passed = sum(v.get('pass', 0) for v in gc[-1]['counts'].values())
        need(passed == gates, f'G {c}: gate counts pass {passed} != gate events {gates}')
        need(gates == len(sels(ev)) + blocked, f'G {c}: gates {gates} != selections {len(sels(ev))} + blocked {blocked}')
        out.append(f'{c} {gates} gates, blocked {blocked}')
    return 'G: every selection follows its gate (rbx = param_2, stack slots = I/mip/D pointers, cutoff): ' + ', '.join(out)


def check_deact(data):
    out = []
    for c, spec in CASES.items():
        if spec.get('control'):
            continue
        ev = data[c][RUNS[0]]['events']
        need(not [e for e in ev if e['event'] == 'b:op_fail'], f'D {c}: failed flag op')
        top = [e for e in sels(ev, 1) if e['count'] > 0]
        ops = [e for e in ev if e['event'] == 'b:op' and e['op'] == 'deact']
        need(len(top) == len(ops), f'D {c}: top-level non-empty selections {len(top)} != 10e5fc calls {len(ops)}')
        for s, o in zip(top, ops):
            need(o['ret'] == DEACT_RET, f'D {c}: deact from {o["ret"]:#x}')
            need(o['node'] == s['node'] and o['rows'] == s['list'], f'D {c}: sel call {s["call"]} node {s["node"]} list != 10e5fc rows (node {o["node"]})')
            need(s['count'] == len(s['list']), f'D {c}: count {s["count"]} != list length')
        if c == 'nidda_H_s1':
            need(len(ops) == NIDDA_DEACT, f'D {c}: {len(ops)} top-level deact != M2-03d-7 {NIDDA_DEACT}')
        out.append(f'{c} {len(ops)}')
    return 'D: top-level selection list = 10e5fc rows (5fb8ec+0x37c9): ' + ', '.join(out)


def check_rule(data, variant=None, stats=None):
    """R and A over run 1 of every non-control case; returns (events, rows selected, 66661c calls)."""
    n = rows = calls = 0
    for c, spec in CASES.items():
        if spec.get('control'):
            continue
        for e, m, lb, ub in selections(data[c][RUNS[0]]['events']):
            got, cl = select(e, m, lb, ub, variant, stats)
            if variant is None:
                need(cl == e['calls_66661c'], f'A {c}: sel call {e["call"]} node {e["node"]}: 66661c calls differ, first '
                     f'{next(((a, b) for a, b in zip(cl, e["calls_66661c"]) if a != b), (len(cl), len(e["calls_66661c"])))} (port, capture)')
            need(got == e['list'], f'R {c}: sel call {e["call"]} node {e["node"]}: rule {len(got)} rows != capture {e["count"]}; '
                 f'extra {[x for x in got if x not in e["list"]][:5]} missing {[x for x in e["list"] if x not in got][:5]}')
            n += 1
            rows += len(got)
            calls += len(cl)
    return n, rows, calls


def check_control(data):
    c = 'nidda_H_s1_dr0'
    for r in RUNS:
        ev = data[c][r]['events']
        bad = [e for e in ev if e['event'] in ('d:gate', 'd:sel', 'd:blocked', 'd:model', 'd:66661c_outside')]
        need(not bad, f'C run {r}: selection event in the DynamicRows=0 control: {bad[:1]}')
        gc = [e for e in ev if e['event'] == 'd:gate_counts']
        need(gc and all(set(v) == {'dyn0'} for v in gc[-1]['counts'].values()), f'C run {r}: gate clauses {gc[-1:]}')
        for e in ev:
            if e['event'] == 'b:op':
                need(e['dynamic_rows'] == 0, f'C run {r}: env+0x2b48 = {e["dynamic_rows"]}')
                need(e['op'] == 'react_all' and e['ret'] == RESTART_REACT and e['x_count_before'] is None,
                     f'C run {r}: flag op {e["op"]} from {e["ret"]:#x} (only the restart react_all with no X list is allowed)')
            need(e['event'] != 'b:xlist', f'C run {r}: X list built: {e}')
    ops = [e for e in data[c][RUNS[0]]['events'] if e['event'] == 'b:op']
    return f'C: DynamicRows=0 x3: env+0x2b48 = 0, gate first clause false at every pass, 0 selections, 0 deact, ' \
           f'{len(ops)} restart react_all (6a6563+0x211, no X)'


def discrimination(data):
    rep = []
    for v in VARIANTS:
        try:
            check_rule(data, v)
            rep.append((v, 'survives (C only)'))
        except Fail as e:
            rep.append((v, 'rejected: ' + str(e)[:90]))
    return rep


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
    man = {'task': 'M2-03d-8', 'binary_sha256': BINARY, 'models': models,
           'captures': {cap(c, r): sha(cap(c, r)) for c in CASES for r in RUNS},
           'tools': {p: sha(p) for p in TOOLS}, 'refs': {REF7: sha(REF7)}}
    with open(os.path.join(ROOT, MANIFEST), 'w') as f:
        json.dump(man, f, indent=1, sort_keys=True)
        f.write('\n')
    print('wrote', MANIFEST)


def run_checks(data, ref, man, stats=None):
    lines = [check_inputs(data, man), check_repro(data), check_projection(data, ref), check_gate(data), check_deact(data)]
    n, rows, calls = check_rule(data, None, stats)
    lines.append(f'A/R: {n} selections, {rows} rows, {calls} 66661c calls reproduced exactly (list order, 66661c row and result)')
    lines.append(check_control(data))
    return lines


def tampers():
    """(name, mutation) pairs; each mutation edits all runs of a case so that B passes and the targeted check must fail."""
    def every(c, f):
        def g(d, ref, man):
            for r in RUNS:
                f(d[c][r]['events'])
        return g

    def nth(evs, kind, k, cond=lambda e: True):
        return [e for e in evs if e['event'] == kind and cond(e)][k]

    def ev_set(kind, k, key, val, cond=lambda e: True):
        return lambda evs: nth(evs, kind, k, cond).__setitem__(key, val)

    def bump_hex(h):
        return format(int(h, 16) ^ 1, '016x')

    def bound(evs):
        # first selected model row of the 6th non-empty selection: an infinite ub on one of its positive-coefficient columns
        # makes its max activity 1e100 (nidda: DualReductions 0, so the row must drop out)
        e = nth(evs, 'd:sel', 5, lambda e: any(x < e['nrows'] for x in e['list']))
        m = Model(nth(evs, 'd:model', 0, lambda x: x['model'] == e['model']))
        i = [x for x in e['list'] if x < e['nrows']][0]
        j = [c for c, a in zip(*m.rows[i]) if a > 0][0]
        e['ub_diff'].append([j, bits(1e30)])

    def rhs(evs):
        e = nth(evs, 'd:sel', 0, lambda e: any(x < e['nrows'] for x in e['list']))
        mev = nth(evs, 'd:model', 0, lambda x: x['model'] == e['model'])
        i = [x for x in e['list'] if x < e['nrows']][0]
        mev['rhs'] = mev['rhs'][:16 * i] + bits(-1e20) + mev['rhs'][16 * (i + 1):]

    def slack(evs):
        e = nth(evs, 'd:sel', 0, lambda e: any(x >= e['nrows'] for x in e['list']))
        j = [x for x in e['list'] if x >= e['nrows']][0] - e['nrows']
        e['cut_slack'] = e['cut_slack'][:16 * j] + bits(1e-5) + e['cut_slack'][16 * (j + 1):]

    def cut_type(evs):
        e = nth(evs, 'd:sel', 0, lambda e: any(x >= e['nrows'] for x in e['list']))
        j = [x for x in e['list'] if x >= e['nrows']][0] - e['nrows']
        e['cut_type'][j] = 0x14

    def list_drop(evs):
        e = nth(evs, 'd:sel', 0, lambda e: e['count'] > 1)
        e['list'] = e['list'][1:]
        e['count'] -= 1

    def list_order(evs):     # a nested-MIP selection: no 10e5fc op record, so only R can see it
        e = nth(evs, 'd:sel', 0, lambda e: e['count'] > 1 and e['mip_depth'] > 1)
        e['list'][0], e['list'][1] = e['list'][1], e['list'][0]

    def f666(evs):
        e = nth(evs, 'd:sel', 0, lambda e: e['calls_66661c'])
        e['calls_66661c'][0][1] ^= 1

    def gate_node(evs):
        nth(evs, 'd:gate', 3)['node'] += 1

    def proj(evs):
        o = nth(evs, 'b:op', 0, lambda e: e['op'] == 'deact')
        o['rows'] = o['rows'][:-1]

    def ctrl(evs):
        evs.insert(1, {'event': 'd:gate', 'mip_depth': 1, 'call': 1, 'node': 0, 'node_depth': 0, 'p34': 0, 'dyn': -1, 'rbx_is_param2': True})

    def ctrl_op(evs):
        nth(evs, 'b:op', 0)['ret'] = 0x51096e

    def dyn(evs):
        for e in evs:
            if e['event'] == 'd:sel':
                e['dyn'] = 0
                return

    def cutoff_t(evs):
        e = nth(evs, 'd:sel', 2)
        e['i10'] = bump_hex(e['i10'])

    def blocked(evs):
        e = nth(evs, 'd:sel', 4)
        evs[evs.index(e)] = {'event': 'd:blocked', 'mip_depth': e['mip_depth'], 'call': e['call'], 'node': e['node']}

    def run_diff(d, ref, man):
        d['gmu-35-40_default'][2]['events'][7] = dict(d['gmu-35-40_default'][2]['events'][7], call=-1)

    return [
        ('binary hash', lambda d, ref, man: d['nidda_H_s1'][1].__setitem__('binary_sha256', '0' * 64)),
        ('tool hash', lambda d, ref, man: d['gmu-35-40_default'][3].__setitem__('tool_sha256', '0' * 64)),
        ('params', lambda d, ref, man: d['assign1-5-8_default'][1]['params'].__setitem__('Seed', 1)),
        ('model hash', lambda d, ref, man: man['models'].__setitem__('gmu-35-40', '0' * 64)),
        ('run difference', run_diff),
        ('empty events', lambda d, ref, man: [d['assign1-5-8_default'][r].__setitem__('events', []) for r in RUNS]),
        ('M2-03d-7 projection', every('nidda_H_s1', proj)),
        ('gate node', every('gmu-35-40_default', gate_node)),
        ('blocked selection', every('assign1-5-8_default', blocked)),
        ('sel DynamicRows 0', every('nidda_H_s1', dyn)),
        ('cutoff field', every('gmu-35-40_default', cutoff_t)),
        ('list one row dropped', every('assign1-5-8_default', list_drop)),
        ('list order', every('gmu-35-40_default', list_order)),
        ('bound one entry', every('nidda_H_s1', bound)),
        ('rhs one entry', every('nidda_H_s1', rhs)),
        ('cut slack', every('nidda_H_s1', slack)),
        ('cut type 0x14', every('gmu-35-40_default', cut_type)),
        ('66661c result', every('gmu-35-40_default', f666)),
        ('control gate event', every('nidda_H_s1_dr0', ctrl)),
        ('control op site', every('nidda_H_s1_dr0', ctrl_op)),
    ]


def sweep(d):
    bad = 0
    tot = {}
    for p in sorted(glob.glob(os.path.join(d, '*.json.gz'))):
        with gzip.open(p, 'rt') as f:
            cap_ = json.load(f)
        n = rows = calls = ones = 0
        first = None
        st = {}
        for e, m, lb, ub in selections(cap_['events']):
            got, cl = select(e, m, lb, ub, None, st)
            n += 1
            rows += len(got)
            calls += len(cl)
            ones += sum(r for _, r in cl)
            if first is None and (got != e['list'] or cl != e['calls_66661c']):
                first = f'sel call {e["call"]}'
        for k, v in st.items():
            tot[k] = tot.get(k, 0) + v
        bad += first is not None
        print(f'{os.path.basename(p):40} ok={cap_["ok"]} sel={n:5} rows={rows:6} 66661c={calls:6}/{ones:4} {"MISMATCH " + first if first else "match"}')
    print('clause counts:', json.dumps(tot, sort_keys=True))
    return bad == 0


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
        data, ref = load_all(), load(REF7)
        stats = {}
        for line in run_checks(data, ref, man, stats):
            print(line)
        print('clause counts:', json.dumps(stats, sort_keys=True))
        print('V: clause variants')
        for v, res in discrimination(data):
            print(f'   {v:13} {res}')
    except Fail as e:
        print('FAIL', e)
        return 1
    if '--self-test' in args:
        failed = 0
        for name, mut in tampers():
            d, r, m = copy.deepcopy(data), ref, copy.deepcopy(man)
            try:
                mut(d, r, m)
                run_checks(d, r, m)
                print(f'NOT REJECTED: {name}')
                failed += 1
            except Fail as e:
                print(f'rejected: {name}: {str(e)[:120]}')
            except (IndexError, KeyError, ValueError) as e:
                print(f'NOT REJECTED (tamper error {type(e).__name__}: {e}): {name}')
                failed += 1
        if failed:
            print('FAIL', failed, 'tampers not rejected')
            return 1
    print('PASS')
    return 0


if __name__ == '__main__':
    sys.exit(main())
