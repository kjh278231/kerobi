#!/usr/bin/env python3
"""M2-03d-7 checker: provenance of the empty simplex ws rows at the M2-03d-6 retry node, contract docs/m2/mip_blank_rows.md section 3.

(default)         manifest hashes, then checks M (inputs), B (3 runs identical), L (same state as M2-03d-6), X (X list = R),
                  S (cda5b 10f448/10f5ae swap), P (replay of the 10e5fc/10e62f/10e662 flag ops through edd71 gives every X list)
--self-test       also requires every in-memory tamper to be rejected
--write-manifest  record capture, tool and reference hashes
Exit code 0 only when every requested check passes.
"""
import copy, hashlib, json, os, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = 'bench/oracle/mip_blank_rows'
MANIFEST = 'bench/oracle/mip_blank_rows_manifest.json'
TOOLS = ['tools/mip_blank_rows.js', 'tools/mip_blank_rows_runs.sh']
REF_LP = 'bench/oracle/mip_retry_node_lp/nidda_bnb_H_s1.node_lp.json'   # M2-03d-6: simplex_blanked_rows, API rhs
REF_CAP = 'bench/oracle/mip_retry_node_lp/nidda_bnb_H_s1_1.json'        # M2-03d-6: y:dhash, y:ws hashes
# exploratory probe (single run, tool of commit e44ed5e): 08b2fe source and ws<->D aliases at every 02f7e1 return it recorded
PROBE = DATA + '/history/probe_e44ed5e.json'
PROBE_TOOL = '95848f0a115aa2ad'   # sha256 prefix of e44ed5e:tools/mip_blank_rows.js
BINARY = '8e87fca221fdffc922331c8619426031b3e72dfd419446422f88af0cb0e98f9c'
PARAMS = {'Threads': 1, 'Seed': 1, 'LogToConsole': 0, 'NodeLimit': 32898, 'Presolve': 0, 'ScaleFlag': 0, 'Quad': 0, 'NumericFocus': 0,
          'MarkowitzTol': 0.0001, 'Heuristics': 0, 'RINS': 0}
TARGET, NODE = 19901, 16948.0
# call sites (file offset, size from ../re/decomp/index.tsv)
FUNCS = {'5fb8ec': (0x5101d0, 21510), '6a6563': (0x5805a0, None)}
ZERO = '0000000000000000'
RUNS = (1, 2, 3)


class Fail(Exception):
    pass


def need(cond, msg):
    if not cond:
        raise Fail(msg)


def load(p):
    with open(os.path.join(ROOT, p)) as f:
        return json.load(f)


def sha(p):
    with open(os.path.join(ROOT, p), 'rb') as f:
        return hashlib.sha256(f.read()).hexdigest()


def site(ret):
    o, n = FUNCS['5fb8ec']
    if o <= ret < o + n:
        return f'5fb8ec+{ret - o:#x}'
    return f'{ret:#x}'


def refs():
    lp, cap = load(REF_LP), load(REF_CAP)
    es = [e for e in cap['events'] if e['event'] == 'retry_solve'][0]['events']
    one = lambda name, k: [e for e in es if e['event'] == name and e.get('k') == k][0]
    yl = one('y:lp', 2)
    return {'R': lp['simplex_blanked_rows'], 'api_rhs': yl['rhs'], 'api_nnz': yl['cbeg'][-1], 'model_sha256': cap['model_sha256'],
            'dhash': {k: one('y:dhash', k)['hash'] for k in (1, 2)}, 'dnnz': one('y:dhash', 1)['nnz_end'],
            'ws_hash': {1: one('y:ws_hash', 1)['hash'], 2: one('y:ws', 2)['hash']}, 'ws_nnz': one('y:ws', 2)['cbeg'][-1]}


def check_inputs(runs, ref):
    tool = sha('tools/mip_blank_rows.js')
    for r, d in runs.items():
        need(d['ok'] and not d['errors'], f'M run {r}: capture not ok: {d["errors"][:1]}')
        need(d['binary_sha256'] == BINARY, f'M run {r}: binary sha256 {d["binary_sha256"]}')
        need(d['model_sha256'] == ref['model_sha256'], f'M run {r}: model sha256 differs from M2-03d-6')
        need(d['tool_sha256'] == tool, f'M run {r}: tool sha256 {d["tool_sha256"]} != current {tool}')
        need(d['params'] == PARAMS, f'M run {r}: params {d["params"]}')
        need(d['target'] == TARGET and d['output'] == 'DONE 8 32898', f'M run {r}: target/output {d["target"]} {d["output"]!r}')


def check_repro(runs):
    base = runs[RUNS[0]]['events']
    need(len(base) > 0, 'B: no events')
    for r in RUNS[1:]:
        es = runs[r]['events']
        for i, (a, b) in enumerate(zip(base, es)):
            need(a == b, f'B run {r}: first difference at event {i}: {json.dumps(a)[:200]} vs {json.dumps(b)[:200]}')
        need(len(es) == len(base), f'B run {r}: {len(es)} events vs {len(base)}')


def target_events(es):
    t = {}
    for e in es:
        if e['event'].startswith('b:t:'):
            t.setdefault((e['event'], e['k']), []).append(e)
    for key, v in t.items():
        need(len(v) == 1, f'target: {len(v)} events {key}')
    return {k: v[0] for k, v in t.items()}


def check_state(es, ref):
    """L same node and state as M2-03d-6; X the X list is R; S the swap. Returns the target lp id."""
    t = target_events(es)
    need({k for _, k in t} == {1, 2}, f'L: target cda5b calls {sorted({k for _, k in t})}, expected k 1 and 2')
    R = ref['R']
    lp = None
    for k in (1, 2):
        c = t.get(('b:t:cda5b', k))
        need(c is not None, f'L k={k}: no b:t:cda5b')
        need(c['node'] == NODE and c['is_node_lp'], f'L k={k}: node {c["node"]} node_lp {c["is_node_lp"]}')
        need(c['dhash']['hash'] == ref['dhash'][k] and c['dhash']['nnz_end'] == ref['dnnz'], f'L k={k}: D hash {c["dhash"]} != M2-03d-6 {ref["dhash"][k]}')
        lp = c['lp'] if lp is None else lp
        need(c['lp'] == lp, f'L k={k}: lp id {c["lp"]} != {lp}')
        w = t.get(('b:t:2f7e1', k))
        need(w is not None and w['rc'] == 0 and not w['ws_null'], f'L k={k}: no 02f7e1 return with ws')
        need(w['ws_hash']['hash'] == ref['ws_hash'][k], f'L k={k}: ws hash {w["ws_hash"]["hash"]} != M2-03d-6 {ref["ws_hash"][k]}')
        need(w['ws_len_is_D_len'] and w['ws_b_is_D_rhs'] and w['ws_nnz'] == ref['ws_nnz'], f'S k={k}: ws arrays are not the swapped D arrays {w}')
        # X: list and marks
        need(not c['x_null'], f'X k={k}: no X')
        need(sorted(c['x_rows']) == sorted(R) and len(c['x_rows']) == len(R) == c['x_count'], f'X k={k}: X list {len(c["x_rows"])} rows != R {len(R)}: '
             f'extra {sorted(set(c["x_rows"]) - set(R))[:5]} missing {sorted(set(R) - set(c["x_rows"]))[:5]}')
        need(c['x_marked'] == sorted([[r, p] for p, r in enumerate(c['x_rows'])]), f'X k={k}: X+0x10 positions do not index the list')
        need(c['dynamic_rows'] != 0, f'X k={k}: env+0x2b48 (DynamicRows) is 0')
        # S: swap in / out
        si, so = t.get(('b:t:swap_in', k)), t.get(('b:t:swap_out', k))
        need(si is not None and so is not None, f'S k={k}: swap events missing')
        for name, e in (('in', si), ('out', so)):
            need(e['D_len_was_xlen'] and e['D_rhs_was_xrhs'] and e['xlen_was_D_len'] and e['xrhs_was_D_rhs'] and not e['D_len_unchanged'],
                 f'S k={k} {name}: arrays not exchanged {e}')
            need(set(e['rhs_diff']) <= set(R), f'S k={k} {name}: rhs changes outside R {sorted(set(e["rhs_diff"]) - set(R))[:5]}')
        need(si['ref_before'] == 0 and si['ref_after'] == 1 and so['ref_before'] == 1 and so['ref_after'] == 0, f'S k={k}: refcount {si} {so}')
        need(si['was_nnz'] == ref['api_nnz'] and si['D_nnz'] == ref['ws_nnz'] and so['was_nnz'] == ref['ws_nnz'] and so['D_nnz'] == ref['api_nnz'],
             f'S k={k}: nnz {si["was_nnz"]}->{si["D_nnz"]}, {so["was_nnz"]}->{so["D_nnz"]}')
        need(all(v == ZERO for v in si['rhs_now']), f'S k={k}: swapped-in rhs not +0 on R')
        # rows of R whose rhs did not change already have rhs +0 in the API LP
        need(all(ref['api_rhs'][r] == ZERO for r in set(R) - set(si['rhs_diff'])), f'S k={k}: R row with nonzero API rhs kept its rhs')
        need(sorted(si['rhs_diff']) == sorted(r for r in R if ref['api_rhs'][r] != ZERO), f'S k={k}: rhs changes != R rows with nonzero API rhs')
        need(so['rhs_diff'] == si['rhs_diff'] and [ref['api_rhs'][r] for r in so['rhs_diff']] == so['rhs_now'], f'S k={k}: swap out did not restore the API rhs')
    need(t[('b:t:swap_in', 1)]['built_before'] in (0, 1) and t[('b:t:swap_in', 2)]['built_before'] == 1, 'S: X arrays rebuilt between k=1 and k=2')
    return lp


def edd71(lst, flags):
    """edd71 C:8026-: rows in increasing order; 0x1000 appends a row not in the list, 0x2000 removes a listed row by moving the last one in"""
    lst = list(lst)
    pos = {r: p for p, r in enumerate(lst)}
    for i in sorted(flags):
        f = flags[i]
        if f == 0x1000:
            if i not in pos:
                pos[i] = len(lst)
                lst.append(i)
        elif i in pos:
            p, last = pos.pop(i), lst.pop()
            if last != i:
                lst[p] = last
                pos[last] = p
    return lst


def replay(es, lp):
    """P: fold the pending flag ops into the list at every b:xlist and compare (order included). Row deletions (D rows fall)
    renumber rows, so after one the list is unknown until an empty X list is seen; the target must be reached with a known list
    and no row deletion since the last empty list."""
    lst, flags, rows, steps, prov, ops, skipped = [], {}, None, 0, {}, {'deact': 0, 'react': 0, 'react_all': 0}, 0
    cover = {'kept': 0, 'removal': 0}   # steps from a non-empty list to a non-empty list, and those that removed a row
    origin = {}
    for e in es:
        if e.get('lp') != lp:
            continue
        need(e['event'] != 'b:op_fail', f'P: op failed {e}')
        if 'D_rows' in e:
            if rows is not None and e['D_rows'] < rows:
                lst, flags = None, {}
            rows = e['D_rows']
        if e['event'] == 'b:op':
            ops[e['op']] += 1
            if e['op'] == 'deact':
                need(site(e['ret']) == '5fb8ec+0x37c9', f'P: deact from {site(e["ret"])}')
                for r in e['rows']:
                    flags[r] = 0x1000
                    origin[r] = (e['call'], e['node'], e['node_depth'])
            elif e['op'] == 'react':
                for r in e['rows']:
                    flags[r] = 0x2000
            elif lst is not None:
                for r in lst:
                    flags[r] = 0x2000
        elif e['event'] == 'b:xlist' and lst is None:
            skipped += 1
            if e['count'] == 0:
                lst, flags = [], {}
        elif e['event'] == 'b:xlist':
            want = edd71(lst, flags)
            need(want == e['rows'] and e['count'] == len(e['rows']), f'P: X list at call {e["call"]} differs from the replay: '
                 f'got {e["rows"][:8]}.. ({e["count"]}) want {want[:8]}.. ({len(want)})')
            if lst and e['rows']:
                cover['kept'] += 1
                cover['removal'] += not set(lst) <= set(e['rows'])
            lst, flags, steps = e['rows'], {}, steps + 1
            prov = {r: origin[r] for r in lst}
        elif e['event'] == 'b:t:cda5b' and e['k'] == 1:
            need(lst is not None and e['x_rows'] == lst, 'P: target X list unknown or differs from the last replayed list')
            need(not flags, f'P: {len(flags)} flag ops pending at the target')
            return {'steps': steps, 'ops': ops, 'prov': prov, 'skipped': skipped, 'cover': cover}
    raise Fail('P: target not reached')


def check_probe(pr, ref):
    """Q1 (single exploratory run): 08b2fe returned lp+0xe8, lp+0xf8/0xf0 null and ws CSC/b == D fields at every recorded
    02f7e1 return; at the target the ws hash is the M2-03d-6 one and D == ws nnz 1840"""
    need(pr['ok'] and pr['binary_sha256'] == BINARY and pr['tool_sha256'].startswith(PROBE_TOOL) and pr['params'] == PARAMS
         and pr['target'] == TARGET and pr['model_sha256'] == ref['model_sha256'], 'Q: probe inputs')
    es = pr['events']
    need(len(es) > 0, 'Q: probe has no events')
    for e in es:
        need(e['src'] == 'lp+0xe8' and e['lpf8_null'] and e['lpf0_null'] and e.get('alias') == '11111', f'Q: call {e["call"]} k {e["k"]}: '
             f'src {e["src"]} alias {e.get("alias")}')
    t = [e for e in es if e['call'] == TARGET]
    need([e['k'] for e in t] == [1, 2], f'Q: probe target calls {[e["k"] for e in t]}')
    for e in t:
        need(e['ws_hash']['hash'] == ref['ws_hash'][e['k']] and e['D_nnz'] == e['ws_nnz'] == ref['ws_nnz'] and not e['ws_new'], f'Q: probe target k {e["k"]}')
    return len(es)


def check(runs, ref, quiet=False, probe=None):
    nq = check_probe(probe, ref) if probe is not None else None
    check_inputs(runs, ref)
    check_repro(runs)
    es = runs[RUNS[0]]['events']
    lp = check_state(es, ref)
    p = replay(es, lp)
    if not quiet:
        new = [e for e in es if e['event'] == 'b:lpnew' and e['lp'] == lp][0]
        t = target_events(es)
        print(f'Q: probe {nq} 02f7e1 returns: 08b2fe = lp+0xe8, ws CSC/b alias D (5/5); target ws hash = M2-03d-6')
        print(f'M/B: {len(RUNS)} runs, {len(es)} events each, identical')
        print(f'L: call {TARGET} node {NODE:.0f}, D hash {ref["dhash"][1]} (pre-swap) and ws hash {ref["ws_hash"][1]} = M2-03d-6; lp id {lp} '
              f'(first seen call {new["call"]}, D rows {new["D_rows"]})')
        print(f'X: X list = R ({len(ref["R"])} rows), positions consistent, DynamicRows env+0x2b48 = {t[("b:t:cda5b", 1)]["dynamic_rows"]}, '
              f'X built at k=1 entry {t[("b:t:cda5b", 1)]["x_built"]}')
        print(f'S: 10f448 swaps D len/rhs with X ({ref["api_nnz"]} -> {ref["ws_nnz"]} nnz, rhs of {len(t[("b:t:swap_in", 1)]["rhs_diff"])} R rows -> +0), 10f5ae swaps back')
        print(f'P: {p["steps"]} X-list steps replayed (order exact), {p["skipped"]} skipped after row deletions; ops up to the target {p["ops"]}; '
              f'non-empty -> non-empty {p["cover"]["kept"]}, with a removal {p["cover"]["removal"]} (swap-with-last removal not exercised if 0)')
        by = {}
        for r, o in p['prov'].items():
            by.setdefault(o, []).append(r)
        for (call, node, depth), rs in sorted(by.items()):
            print(f'   deact after 5ce67b call {call} (node {node:.0f}, depth {depth}): {len(rs)} rows of R')
    return p


TAMPERS = []


def tamper(fn):
    TAMPERS.append(fn)
    return fn


def first(es, name, k=None):
    return [e for e in es if e['event'] == name and (k is None or e.get('k') == k)][0]


def before(es, pred):
    """events of the target lp before the target, matching pred (the P tampers must hit the replay window)"""
    t = next(i for i, e in enumerate(es) if e['event'] == 'b:t:cda5b')
    lp = es[t]['lp']
    return [i for i, e in enumerate(es[:t]) if e.get('lp') == lp and pred(e)]


@tamper
def t_binary(runs, ref): runs[1]['binary_sha256'] = '0' * 64
@tamper
def t_probe_alias(runs, ref): ref['probe']['events'][3]['alias'] = '10111'
@tamper
def t_probe_src(runs, ref): ref['probe']['events'][-1]['src'] = 'lp+0xf8'
@tamper
def t_tool(runs, ref): runs[2]['tool_sha256'] = '0' * 64
@tamper
def t_params(runs, ref): runs[3]['params'] = dict(PARAMS, Seed=2)
@tamper
def t_run_diff(runs, ref): runs[2]['events'][5]['call'] = -1
@tamper
def t_no_events(runs, ref):
    for r in RUNS: runs[r]['events'] = []
@tamper
def t_dhash(runs, ref):
    for r in RUNS: first(runs[r]['events'], 'b:t:cda5b', 1)['dhash']['hash'] = 'x'
@tamper
def t_ws_hash(runs, ref):
    for r in RUNS: first(runs[r]['events'], 'b:t:2f7e1', 2)['ws_hash']['hash'] = 'x'
@tamper
def t_node(runs, ref):
    for r in RUNS: first(runs[r]['events'], 'b:t:cda5b', 2)['node'] = 1.0
@tamper
def t_xlist_drop(runs, ref):
    for r in RUNS:
        e = first(runs[r]['events'], 'b:t:cda5b', 1); e['x_rows'] = e['x_rows'][:-1]; e['x_count'] -= 1
@tamper
def t_xmark(runs, ref):
    for r in RUNS: first(runs[r]['events'], 'b:t:cda5b', 1)['x_marked'][0][1] += 1
@tamper
def t_swap_flag(runs, ref):
    for r in RUNS: first(runs[r]['events'], 'b:t:swap_in', 1)['D_len_was_xlen'] = False
@tamper
def t_swap_rhs(runs, ref):
    for r in RUNS: first(runs[r]['events'], 'b:t:swap_in', 2)['rhs_now'][0] = '3ff0000000000000'
@tamper
def t_restore(runs, ref):
    for r in RUNS: first(runs[r]['events'], 'b:t:swap_out', 1)['rhs_now'][0] = ZERO
@tamper
def t_ref_R(runs, ref): ref['R'] = ref['R'][:-1] + [0]
@tamper
def t_drop_deact(runs, ref):
    for r in RUNS:
        es = runs[r]['events']; del es[before(es, lambda e: e['event'] == 'b:op' and e['op'] == 'deact')[-1]]
@tamper
def t_extra_row(runs, ref):
    for r in RUNS:
        es = runs[r]['events']; es[before(es, lambda e: e['event'] == 'b:op' and e['op'] == 'deact')[-1]]['rows'].append(0)
@tamper
def t_drop_react_all(runs, ref):
    for r in RUNS:
        es = runs[r]['events']; del es[before(es, lambda e: e['event'] == 'b:op' and e['op'] == 'react_all')[-1]]
@tamper
def t_rows_fall(runs, ref):
    for r in RUNS:
        es = runs[r]['events']; es[before(es, lambda e: e['event'] == 'b:xlist')[-1]]['D_rows'] = 1
@tamper
def t_xlist_order(runs, ref):
    for r in RUNS:
        es = runs[r]['events']; e = es[before(es, lambda e: e['event'] == 'b:xlist' and len(e['rows']) > 1)[-2]]; e['rows'] = e['rows'][::-1]
@tamper
def t_deact_site(runs, ref):
    for r in RUNS:
        es = runs[r]['events']; es[before(es, lambda e: e['event'] == 'b:op' and e['op'] == 'deact')[-1]]['ret'] += 1


def main():
    args = sys.argv[1:]
    runs = {r: load(f'{DATA}/nidda_bnb_H_s1_{r}.json') for r in RUNS}
    ref = refs()
    ref['probe'] = load(PROBE)
    if '--write-manifest' in args:
        files = [f'{DATA}/nidda_bnb_H_s1_{r}.json' for r in RUNS] + [PROBE] + TOOLS + ['tools/mip_blank_rows_check.py', REF_LP, REF_CAP]
        with open(os.path.join(ROOT, MANIFEST), 'w') as f:
            json.dump({'binary_sha256': BINARY, 'files': {p: sha(p) for p in files}}, f, indent=1)
            f.write('\n')
    man = load(MANIFEST)
    bad = [p for p, h in man['files'].items() if p != 'tools/mip_blank_rows_check.py' and sha(p) != h]
    rc = 0
    try:
        need(not bad, f'manifest: changed {bad}')
        check(runs, ref, probe=ref['probe'])
        print('PASS')
    except Fail as e:
        print('FAIL', e)
        rc = 1
    if '--self-test' in args:
        for fn in TAMPERS:
            r2, f2 = copy.deepcopy(runs), copy.deepcopy(ref)
            fn(r2, f2)
            try:
                check(r2, f2, quiet=True, probe=f2['probe'])
                print(f'NOT REJECTED {fn.__name__}')
                rc = 1
            except Fail as e:
                print(f'rejected {fn.__name__}: {str(e)[:110]}')
        print(f'self-test: {len(TAMPERS)} tampers')
    return rc


if __name__ == '__main__':
    sys.exit(main())
