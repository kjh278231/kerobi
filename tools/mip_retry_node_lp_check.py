#!/usr/bin/env python3
"""M2-03d-3 checker: retry node LP captures (tools/mip_retry_node_lp.js), contract docs/m2/mip_retry_node_lp.md section 3.
M2-03d-6 adds the general B&B node case nidda_bnb_H_s1, contract docs/m2/mip_retry_bnb_node_lp.md section 3 (checks F, G).

(default)         manifest hashes (this and the M2-03c-2 one), then checks A, B, C1-C4, E and prints the findings
--self-test       also requires every in-memory tamper to be rejected
--write-manifest  write the Rust-readable node LP files from run 1, then record capture, file, tool and model hashes
Exit code 0 only when every requested check passes.
"""
import copy, hashlib, json, os, struct, sys
from fractions import Fraction

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mip_retry_basis_check as B  # noqa: E402

ROOT = B.ROOT
DATA = 'bench/oracle/mip_retry_node_lp'
OLD = 'bench/oracle/mip_retry_basis'
MANIFEST = 'bench/oracle/mip_retry_node_lp_manifest.json'
TOOLS = ['tools/mip_retry_node_lp.js', 'tools/mip_retry_node_lp_runs.sh', 'tools/mip_retry_bnb_node_lp_runs.sh']
LPFILE = DATA + '/{}.node_lp.json'
# call-site labels for x:cda5b:enter.ret (file offset, size from ../re/decomp/index.tsv)
FUNCS = {'5ccea5': (0x4f92d0, 4320), '5ce67b': (0x4f0e10, 22107), '0c3d0d': (0x14acb0, 24529), '5fb8ec': (0x5101d0, 21510)}
LP_KEYS = ('cols', 'rows', 'model_sense', 'obj_con', 'obj', 'lb', 'ub', 'vtype', 'rhs', 'sense', 'cbeg', 'ind', 'val', 'env_params')
MODELS = {'nidda_stress': 're/bench/data/miplib/neos-3754480-nidda.mps.gz', 'gmu-35-50_default': 're/bench/data/miplib/gmu-35-50.mps.gz',
          'nidda_bnb_H_s1': 're/bench/data/miplib/neos-3754480-nidda.mps.gz'}
RUNS = (1, 2, 3)
# old: read-only natural capture of tools/mip_retry_basis.js (check A); ws: C2 applies (ScaleFlag 0); model: E applies (node LP keeps the model columns)
# node: M2-03d-6 check F, the M2-03d-5 hit (scan solve_serial 19900 counts 5ce67b entries from 0, this tool's call from 1)
CASES = {'nidda_stress': {'old': f'{OLD}/nidda_stress_natural_1.json', 'ws': True, 'model': True},
         'gmu-35-50_default': {'old': f'{OLD}/gmu-35-50_default_natural_1.json', 'ws': False, 'model': False},
         # model 'report': E fails as written (sense change); the sense differences are recorded, every other E rule still fails
         'nidda_bnb_H_s1': {'old': f'{DATA}/nidda_bnb_H_s1_natural.json', 'ws': True, 'model': 'report',
                            'node': {'call': 19901, 'id': 16948.0, 'parent': 16946.0, 'depth': 30, 'root': False, 'I94': 488,
                                     'where': {'depth': 1, 'restarts': 1, 'root_is_self': False}, 'frame': '5fb8ec',
                                     # ws rows emptied at 02f7e1 return, pinned from the capture: not a bound-redundancy rule (contract section 6)
                                     'blanked': [125, 147, 150, 175, 197, 200, 205, 212, 213, 214, 219, 228, 235, 237, 238, 275, 297, 300, 325, 347, 350, 355, 362, 363, 364, 369, 378, 385, 387, 388, 397, 400, 405, 407, 408, 409, 410, 411, 412, 413, 414, 415, 416, 417, 418, 419, 420, 421, 422, 423, 424, 425, 428, 430, 431, 432, 433, 434, 435, 436, 437, 438, 440, 441, 442, 443, 444, 445, 446, 447, 452, 453, 454, 455, 456, 458, 459, 460, 461, 462, 463, 464, 465, 466, 467, 468, 469, 475, 480, 481, 482, 483, 484, 485, 486, 487]}}}
BASIS_TOOL = 'tools/mip_retry_basis.js'
need, Fail = B.need, B.Fail


def f(h):
    return struct.unpack('>d', bytes.fromhex(h))[0]


def site(ret):
    for name, (o, n) in FUNCS.items():
        if o <= ret < o + n:
            return f'{name}+{ret - o:#x}'
    return f'{ret:#x}'


def strip_y(events):
    """The capture without the M2-03d-3 additions: top-level y: messages and y: events inside retry solves."""
    out = []
    for e in events:
        if e['event'].startswith('y:'):
            continue
        out.append(dict(e, events=[x for x in e['events'] if not x['event'].startswith('y:')]))
    return out


def only_y(d):
    top = [e for e in d['events'] if e['event'].startswith('y:')]
    inner = [[x for x in e['events'] if x['event'].startswith('y:')] for e in d['events'] if e['event'] == 'retry_solve']
    return {'top': top, 'retry': inner, 'model_dump': d['model_dump']}


def retry_y(d):
    rs = [e for e in d['events'] if e['event'] == 'retry_solve']
    need(len(rs) == 1, f'expected exactly one retry solve, got {len(rs)}')
    return [x for x in rs[0]['events'] if x['event'].startswith('y:')], rs[0]['events']


def one(es, name, **kw):
    hit = [e for e in es if e['event'] == name and all(e.get(k) == v for k, v in kw.items())]
    need(len(hit) == 1, f'expected one {name} {kw}, got {len(hit)}')
    return hit[0]


def rows_of(lp):
    """API CSC -> per row {col: value hex}."""
    rows = [dict() for _ in range(lp['rows'])]
    for j in range(lp['cols']):
        for k in range(lp['cbeg'][j], lp['cbeg'][j + 1]):
            need(j not in rows[lp['ind'][k]], f'duplicate entry row {lp["ind"][k]} col {j}')
            rows[lp['ind'][k]][j] = lp['val'][k]
    return rows


def neg(h):
    return format(int(h, 16) ^ (1 << 63), '016x')


def check_case(case, caps, old):
    cfg = CASES[case]
    # A: instrumentation did not change the solve; the reference is a read-only capture with the 03c-2 tool version
    need(old['ok'] and old['binary_sha256'] == B.BINARY and old['model_sha256'] == model_sha(case), f'{case}: reference capture not ok or wrong binary/model')
    need(old['tool_sha256'] == B.load(B.MANIFEST)['files'][BASIS_TOOL], f'{case}: reference captured with another {BASIS_TOOL}')
    need(old['params'] == caps[1]['params'], f'{case}: reference params differ')
    for r, d in caps.items():
        need(d['ok'] and not d['errors'], f'{case} run {r}: capture not ok')
        need(d['binary_sha256'] == B.BINARY, f'{case} run {r}: binary hash')
        need(d['model_sha256'] == model_sha(case), f'{case} run {r}: model hash')
        need(d['output'] == old['output'], f'{case} run {r}: output differs from 03c-2: {d["output"]!r} vs {old["output"]!r}')
        need(strip_y(d['events']) == old['events'], f'{case} run {r}: non-y events differ from the 03c-2 natural capture')
    # B: three runs agree on every y: event and the model dump
    ys = {r: only_y(d) for r, d in caps.items()}
    need(all(ys[r] == ys[1] for r in RUNS), f'{case}: y: events differ between runs')
    d = caps[1]
    ye, es = retry_y(d)
    lp = one(ye, 'y:lp')
    need(lp['instrumented'] is True and all(v == 0 for v in lp['api_rc'].values()), f'{case}: API dump rc')
    need(len(lp['cbeg']) == lp['cols'] + 1 and len(lp['ind']) == len(lp['val']) == lp['cbeg'][-1], f'{case}: CSC shape')
    need(len(lp['obj']) == len(lp['lb']) == len(lp['ub']) == len(lp['vtype']) == lp['cols'], f'{case}: column arrays')
    need(len(lp['rhs']) == len(lp['sense']) == lp['rows'], f'{case}: row arrays')
    # C1: the first LP and the retry see the same raw model data D of the node LP
    dh = [e for e in ye if e['event'] == 'y:dhash' and e['lp_is_node_lp']]
    need(len(dh) >= 1, f'{case}: no top-level cda5b on the node LP')
    need(all(e['hash'] == lp['dhash']['hash'] for e in dh), f'{case}: D hash changed between the LP calls and the API dump')
    need(lp['dhash']['rows'] == lp['rows'] and lp['dhash']['cols'] == lp['cols'], f'{case}: D dims vs API dims')
    # GRBXgetvars returns the stored values unchanged only without D scale arrays
    need(lp['dhash']['null_D3d8'] and lp['dhash']['null_D3e0'], f'{case}: D is scaled at the API dump')
    need(lp['k'] == min(e['k'] for e in dh if e['k'] > 1), f'{case}: API dump not at the first node-LP cda5b after the decision')
    # GRBXgetvars negates flagged columns when D+0x3c != 0; nidda is cross-checked by C2 and E, the others must have it 0
    need(cfg['ws'] or lp['dhash']['D3c'] == 0, f'{case}: D+0x3c is {lp["dhash"]["D3c"]} at the API dump and nothing cross-checks the columns')
    out = {'rows': lp['rows'], 'cols': lp['cols'], 'nnz': lp['cbeg'][-1], 'node_lp_cda5b': [e['k'] for e in dh], 'dump_k': lp['k']}
    if 'node' in cfg:
        check_node(case, cfg['node'], d, es, ye)
        out['node_processing'] = processing(es)
    env = lp['env_params']
    # C3: ws cutoff of every node-LP call comes from the LP env (02f7e1 C:247-259)
    cut, extra = env['Cutoff'], env['env+0x24b8']
    need(lp['lp_f0_null'], f'{case}: lp+0xf0 is set, cutoff would be 1e100')
    node_k = {e['k'] for e in dh}
    wsn = [e for e in ye if e['event'] == 'y:ws' and e['k'] in node_k]
    need(wsn, f'{case}: no ws of a node-LP call')
    for w in wsn:
        need(w['cutoff_3b8'] == cut, f'{case}: ws k={w["k"]} cutoff {w["cutoff_3b8"]} vs env Cutoff {cut}')
        need(w['cutoff2_3c0'] == (cut if f(cut) >= f(extra) else extra), f'{case}: ws k={w["k"]} +0x3c0')
    out['cutoff'] = f(cut)
    # C4: retry LP results as in 03c-2 (es equals the 03c-2 events by A); node-LP results consistent with the cutoff
    leaves = {e['k']: e for e in es if e['event'] == 'x:cda5b:leave'}
    wl = [e for e in ye if e['event'] == 'y:ws_leave']
    need(wl, f'{case}: no retry ws_leave')
    need(lp['model_sense'] == 1, f'{case}: cutoff rule written for minimisation')
    for e in wl:
        need(e['status_f0'] == leaves[e['k']]['lp']['lp_status'], f'{case}: ws status k={e["k"]} vs cda5b leave')
        # 03d-6 contract: other node-LP statuses are a finding (B fixes them over 3 runs); 03d-3 cases are 2 or 6
        if e['k'] in node_k and ('node' not in cfg or e['status_f0'] in (2, 6)):
            need(e['status_f0'] in (2, 6), f'{case}: node-LP retry status {e["status_f0"]}')
            need((f(e['obj_118']) >= f(cut)) == (e['status_f0'] == 6), f'{case}: k={e["k"]} status {e["status_f0"]} objective {f(e["obj_118"])} vs cutoff {f(cut)}')
    out['retry_ws_status'] = [(e['k'], e['status_f0'], f(e['obj_118'])) for e in wl]
    if cfg['ws']:
        # C2: the simplex work arrays of the retry LP equal the API dump (unscaled, '>' rows negated as in GRBXgetvars)
        need(len(dh) == 2, f'{case}: expected the first LP and the retry on the node LP, got {len(dh)}')
        ws = one(ye, 'y:ws')
        wh = one(ye, 'y:ws_hash')
        need(wh['hash'] == ws['hash'], f'{case}: ws arrays of the first LP and the retry differ')
        need(ws['null_colscale_e0'] and ws['null_rowscale_e8'] and f(ws['obj_scale_d8']) == 1.0, f'{case}: ws is scaled')
        n, m = lp['cols'], lp['rows']
        need((ws['cols'], ws['rows']) == (n, m), f'{case}: ws dims')
        flip = [s == '>' for s in lp['sense']]
        sgn = lp['model_sense']
        need(sgn in (1, -1), f'{case}: ModelSense')
        # blanked rows R: rows with no ws entry. 03d-6 contract change: the B&B node LP's ws at 02f7e1 return is the API LP with the
        # rows R emptied and b=0 (origin not identified); the 03d-3 cases must have R empty (then this is the original strict rule)
        R = blanked_rows(ws)
        need(R == cfg.get('node', {}).get('blanked', []), f'{case}: ws empty rows {len(R)}, first {R[:5]}, differ from the pinned set')
        for j in range(n):
            need(ws['c'][j] == (lp['obj'][j] if sgn == 1 else neg(lp['obj'][j])), f'{case}: ws c[{j}] {ws["c"][j]} vs obj {lp["obj"][j]}')
            need(ws['l'][j] == lp['lb'][j] and ws['u'][j] == lp['ub'][j], f'{case}: ws bound col {j}: '
                 f'l {ws["l"][j]} u {ws["u"][j]} vs lb {lp["lb"][j]} ub {lp["ub"][j]}')
            kept = [k2 for k2 in range(lp['cbeg'][j], lp['cbeg'][j + 1]) if lp['ind'][k2] not in R]
            need(ws['cbeg'][j + 1] - ws['cbeg'][j] == len(kept), f'{case}: ws column {j} length {ws["cbeg"][j + 1] - ws["cbeg"][j]} vs {len(kept)} outside R')
            for k, k2 in zip(range(ws['cbeg'][j], ws['cbeg'][j + 1]), kept):
                i = lp['ind'][k2]
                need(ws['ind'][k] == i, f'{case}: ws column {j} row index {ws["ind"][k]} vs {i}')
                need(ws['val'][k] == (neg(lp['val'][k2]) if flip[i] else lp['val'][k2]), f'{case}: ws value col {j} row {i}')
        for i in range(m):
            want = '0000000000000000' if i in R else neg(lp['rhs'][i]) if flip[i] else lp['rhs'][i]
            need(ws['b'][i] == want, f'{case}: ws b[{i}] {ws["b"][i]} vs {want} (rhs {lp["rhs"][i]}, blanked {i in R})')
            need(ws['sense'][i] == ('<' if flip[i] else lp['sense'][i]), f'{case}: ws sense[{i}]')
        if R:
            md = d['model_dump']['rows']
            out['blanked_rows'] = {'count': len(R), 'model_rows': sum(i < md for i in R), 'extra_rows': sum(i >= md for i in R),
                                   'rhs_nonzero': sum(f(lp['rhs'][i]) != 0.0 for i in R), 'senses': ''.join(sorted({lp['sense'][i] for i in R}))}
        out['slack_bounds'] = sorted({(ws['sense'][i], ws['l'][n + i], ws['u'][n + i]) for i in range(m)})
        out['slack_c_zero'] = all(f(x) == 0.0 for x in ws['c'][n:])
    if cfg['model']:
        # E: the node LP is the model plus extra rows; columns, objective and the model rows unchanged
        md = d['model_dump']
        need(md['cols'] == lp['cols'] and lp['rows'] >= md['rows'], f'{case}: model {md["cols"]}x{md["rows"]} vs LP')
        need(md['obj'] == lp['obj'] and md['model_sense'] == lp['model_sense'] and md['obj_con'] == lp['obj_con'],
             f'{case}: objective or sense differ from the model')
        # vtype: equal, except a model 'I' column with bounds [0, 1] is 'B' in the node LP (observed; contract section 6)
        for j, (a, c) in enumerate(zip(md['vtype'], lp['vtype'])):
            need(a == c or (a, c) == ('I', 'B') and f(md['lb'][j]) == 0.0 and f(md['ub'][j]) == 1.0, f'{case}: vtype col {j} {a} vs {c}')
        out['vtype_I_to_B'] = sum(a != c for a, c in zip(md['vtype'], lp['vtype']))
        # rows: same pattern; LP row = lam * model row exactly (as rationals), rhs too; lam < 0 turns '>' into '<' (contract section 6)
        lr, lams, sense_changed = rows_of(lp), [], []
        for i in range(md['rows']):
            a = dict(zip(*md['row_major'][i]))
            need(sorted(a) == sorted(lr[i]) and a, f'{case}: LP row {i} pattern differs from model row {i}')
            j0 = min(a)
            lam = Fraction(f(lr[i][j0])) / Fraction(f(a[j0]))
            for j in a:
                need(Fraction(f(lr[i][j])) == lam * Fraction(f(a[j])), f'{case}: LP row {i} col {j} not lam * model ({f(lr[i][j])} vs {f(a[j])}, lam {lam})')
            need(Fraction(f(lp['rhs'][i])) == lam * Fraction(f(md['rhs'][i])), f'{case}: LP row {i} rhs not lam * model rhs')
            ms, ls = md['sense'][i], lp['sense'][i]
            if not (ms == ls == '=' or (lam > 0 and ms == ls) or (lam < 0 and {ms, ls} == {'<', '>'})):
                need(cfg['model'] == 'report', f'{case}: LP row {i} sense {ls} vs model {ms}, lam {lam}')
                sense_changed.append((i, ms, ls, str(lam), lp['rhs'][i]))
            lams.append(lam)
        out['rows_negated'] = sum(l < 0 for l in lams)
        out['rows_rescaled'] = sum(abs(l) != 1 for l in lams)
        out['row_lams'] = sorted({str(l) for l in lams})[:12]
        out['extra_rows'] = lp['rows'] - md['rows']
        out['bound_diffs'] = sum(md['lb'][j] != lp['lb'][j] or md['ub'][j] != lp['ub'][j] for j in range(md['cols']))
        ints = [j for j in range(md['cols']) if lp['vtype'][j] != 'C']
        out['int_fixed'] = {v: sum(lp['lb'][j] == lp['ub'][j] == v for j in ints) for v in sorted({lp['lb'][j] for j in ints})}
        out['int_cols'] = len(ints)
        out['extra_row_senses'] = ''.join(sorted(set(lp['sense'][md['rows']:])))
        if cfg['model'] == 'report':
            out['E'] = 'FAIL as written: sense changed' if sense_changed else 'PASS'
            out['E_sense_changed'] = sense_changed
    return out


def blanked_rows(ws):
    filled = set(ws['ind'])
    return [i for i in range(ws['rows']) if i not in filled]


def check_node(case, want, d, es, ye):
    """F: the retry is the M2-03d-5 hit; G: the node basis written for Rust is the one passed to the LP calls."""
    enter = es[0]
    need(enter['event'] == 'solve:enter' and enter['call'] == want['call'], f'{case}: retry solve call {enter.get("call")} vs {want["call"]}')
    nd = enter['node']
    for k in ('id', 'parent', 'depth', 'root'):
        need(nd[k] == want[k], f'{case}: retry node {k} {nd[k]} vs {want[k]}')
    nb = one(es, 'x:retry_basis')['node_basis']
    need(nb['I94'] == want['I94'] and not nb['vb_null'] and not nb['cb_null'], f'{case}: node basis at the retry {nb}')
    w = one(ye, 'y:where')
    for k, v in want['where'].items():
        need(w[k] == v, f'{case}: y:where {k} {w[k]} vs {v}')
    lo, n = FUNCS[want['frame']]
    need(any(lo <= x < lo + n for x in w['bt_accurate'] + w['bt_fuzzy']), f'{case}: no {want["frame"]} frame in the retry backtrace')
    lp, basis = one(ye, 'y:lp'), node_lp(case, d)['node_basis']
    need(basis and len(basis['vbasis']) == lp['cols'] and len(basis['cbasis']) == lp['rows'],
         f'{case}: node basis lengths vs LP {lp["cols"]}x{lp["rows"]}')
    on_node = [e for e in es if e['event'] == 'x:cda5b:enter' and e['depth'] == 1 and (e['vb_rel'], e['cb_rel']) == ('I98', 'Ia0')]
    need(len(on_node) >= 2, f'{case}: expected the first LP and the retry with the node basis, got {len(on_node)}')
    for e in on_node:
        need(e['vbasis'] == basis['vbasis'] and e['cbasis'] == basis['cbasis'], f'{case}: cda5b k={e["k"]} basis differs from the decoded node basis')


def processing(es):
    """Node-processing events of the retry solve (compared through A and B, reported here)."""
    leave = [e for e in es if e['event'] == 'solve:leave'][-1]
    return {'set': [(e['value'], e['ret']) for e in es if e['event'] == 'set'],
            'finish': [e.get('ret') for e in es if e['event'] == 'finish:leave'],
            'exports': sum(e['event'] == 'x:export' for e in es),
            'saves': [e['event'] for e in es if e['event'].startswith('x:save_') and e['event'].endswith(':enter')],
            'leave': {'ret': leave['ret'], 'node_status': leave['node']['status'] if leave['node'] else None, 'holder_changed': leave['holder_changed']}}


_msha = {}


def model_sha(case):
    p = os.path.join('..', MODELS[case])
    if p not in _msha:
        _msha[p] = sha(p)
    return _msha[p]


def summary(case, d):
    ye, _ = retry_y(d)
    w = one(ye, 'y:where')
    mains = [e for e in d['events'] if e['event'] == 'y:mipmain']
    rest = [e for e in d['events'] if e['event'] == 'y:restart']
    dd = one(ye, 'y:decision_dhash')
    return {'where': {k: w[k] for k in ('depth', 'mipmain', 'root_is_self', 'self_628', 'restarts', 'lp_is_mip_lp', 'mip_lp', 'lp_env_2728')},
            'decision_D': {k: dd[k] for k in ('hash', 'D3c', 'null_D3d8', 'null_D3e0')},
            'enclosing_mipmain': enclosing(d),
            'mipmain_depth_counts': {str(k): sum(e['phase'] == 'enter' and e['depth'] == k for e in mains) for k in sorted({e['depth'] for e in mains})},
            'restarts': [(e['phase'], e['call'], e['mipmain'], e['restarts']) for e in rest]}


def enclosing(d):
    """5bbf7f calls active at the retry solve: (call, depth, model rows x cols, presolved, env+0x2728)."""
    call = [e for e in d['events'] if e['event'] == 'retry_solve'][0]['events'][0]['call']
    stack = []
    for e in d['events']:
        if e['event'] != 'y:mipmain' or e['call'] > call:
            continue
        stack.append(e) if e['phase'] == 'enter' else stack.pop()
    return [(e['call'], e['depth'], [e['model_rows'], e['model_cols']], e['presolved'], e['env_2728']) for e in stack]


def node_lp(case, d):
    """Rust-readable node LP (schema rsolver.node_lp.v1): the API dump, the LP calls on the node LP, the node basis if any."""
    ye, es = retry_y(d)
    lp = one(ye, 'y:lp')
    on_node = {e['k']: e['lp_is_node_lp'] for e in ye if e['event'] == 'y:dhash'}
    enters = {e['k']: e for e in es if e['event'] == 'x:cda5b:enter' and e['depth'] == 1}
    leaves = {e['k']: e for e in es if e['event'] == 'x:cda5b:leave'}
    kind = lambda e: 'node' if (e['vb_rel'], e['cb_rel']) == ('I98', 'Ia0') else 'none' if (e['vb_rel'], e['cb_rel']) == ('null', 'null') else 'given'
    calls = []
    for k, e in sorted(enters.items()):
        c = {'k': k, 'site': site(e['ret']), 'on_node_lp': on_node[k], 'retry': e['retry'], 'method': e['method'], 'basis': kind(e),
             'warm_lp238': not e['lp']['lp238_null'], 'status': leaves[k]['lp']['lp_status']}
        if c['basis'] == 'given':  # start basis passed by the caller (content captured, origin not traced)
            c.update(vbasis=e['vbasis'], cbasis=e['cbasis'])
        calls.append(c)
    first_retry = next(c for c in calls if c['retry'])
    branch = {'5ce67b': 'node_basis_direct', '0c3d0d': '0c3d0d'}[first_retry['site'].split('+')[0]]
    dec = [e for e in es if e['event'] == 'x:decode:leave']
    basis = {'vbasis': dec[0]['vbasis'], 'cbasis': dec[0]['cbasis']} if dec else None
    extra = {}
    if 'node' in CASES[case]:
        extra['simplex_blanked_rows'] = blanked_rows(one(ye, 'y:ws'))
        extra['simplex_note'] = 'simplex ws at 02f7e1 return = this LP with rows simplex_blanked_rows emptied and rhs 0 (check C2; origin not identified)'
    return dict({'schema': 'rsolver.node_lp.v1', 'case': case, 'doubles': 'u64 bit pattern, 16 hex digits',
                 'source': f'{DATA}/{case}_1.json (run 1 of 3, y:lp at cda5b k={lp["k"]})',
                 'env_params_note': 'raw reads at the offsets of re/notes/env_param_offsets.tsv; only Cutoff is cross-checked (C3)'},
                **{k: lp[k] for k in LP_KEYS}, branch=branch, lp_calls=calls, node_basis=basis, **extra)


def check_lpfiles(caps):
    for case in CASES:
        need(B.load(LPFILE.format(case)) == node_lp(case, caps[case][1]), f'{case}: stored node LP file differs from the capture')


def sha(p):
    return hashlib.sha256(open(os.path.join(ROOT, p), 'rb').read()).hexdigest()


def write_manifest():
    caps = [f'{DATA}/{c}_{r}.json' for c in CASES for r in RUNS] + [cfg['old'] for cfg in CASES.values() if cfg['old'].startswith(DATA)]
    for c in CASES:
        with open(os.path.join(ROOT, LPFILE.format(c)), 'w') as fh:
            json.dump(node_lp(c, B.load(f'{DATA}/{c}_1.json')), fh)
            fh.write('\n')
    caps += [LPFILE.format(c) for c in CASES]
    m = {'binary_sha256': B.BINARY, 'files': {p: sha(p) for p in caps + TOOLS},
         'external_models': {p: sha(os.path.join('..', p)) for p in MODELS.values()}}
    with open(os.path.join(ROOT, MANIFEST), 'w') as fh:
        json.dump(m, fh, indent=2)
        fh.write('\n')


def check_manifests(caps):
    m = B.load(MANIFEST)
    need(m['binary_sha256'] == B.BINARY, 'manifest binary hash')
    need({f'{DATA}/{c}_{r}.json' for c in CASES for r in RUNS} | {LPFILE.format(c) for c in CASES} | set(TOOLS)
         | {cfg['old'] for cfg in CASES.values() if cfg['old'].startswith(DATA)} <= set(m['files']),
         'manifest misses a file')
    for p, h in m['files'].items():
        need(sha(p) == h, f'hash mismatch {p}')
    for p, h in m['external_models'].items():
        need(sha(os.path.join('..', p)) == h, f'model hash mismatch {p}')
    B.check_manifest(B.load(B.MANIFEST))  # the M2-03c-2 captures used by check A
    for case, cs in caps.items():
        for r, d in cs.items():
            need(d['tool_sha256'] == m['files']['tools/mip_retry_node_lp.js'], f'{case} run {r}: captured with another tool version')
            need(d['model_sha256'] == m['external_models'][MODELS[case]], f'{case} run {r}: model hash')


def load_all():
    caps, olds = {}, {}
    for case, cfg in CASES.items():
        caps[case] = {r: B.load(f'{DATA}/{case}_{r}.json') for r in RUNS}
        olds[case] = B.load(cfg['old'])
    return caps, olds


def run(caps, olds):
    return {case: check_case(case, caps[case], olds[case]) for case in CASES}


def tampers(caps):
    def nidda(fn):
        def t(c, o):
            fn(c['nidda_stress'])
        return t

    def lp_of(cs, r=1):
        return retry_y(cs[r])[0]

    def all_runs(fn):
        def t(cs):
            for r in RUNS:
                fn(cs[r])
        return t

    def y_ev(d, name):
        return [x for x in retry_y(d)[0] if x['event'] == name][0]

    def bump(h):
        return format(int(h, 16) ^ 1, '016x')

    def set_val(d):
        e = y_ev(d, 'y:lp'); e['val'][7] = bump(e['val'][7])

    def set_ub(d):
        e = y_ev(d, 'y:lp'); e['ub'][3] = bump(e['ub'][3])

    def set_cut(d):
        e = y_ev(d, 'y:lp'); e['env_params']['Cutoff'] = bump(e['env_params']['Cutoff'])

    def drop_retry(d):
        d['events'] = [e for e in d['events'] if e['event'] != 'retry_solve']

    def dhash(d):
        e = y_ev(d, 'y:dhash'); e['hash'] = '0' * 16

    def ws_l(d):
        e = y_ev(d, 'y:ws'); e['l'][5] = bump(e['l'][5])

    def non_y(d):
        rs = [e for e in d['events'] if e['event'] == 'retry_solve'][0]
        x = [e for e in rs['events'] if e['event'] == 'x:cda5b:enter'][0]; x['method'] ^= 1

    def model_row(d):
        md = d['model_dump']; md['row_major'][4][1][0] = bump(md['row_major'][4][1][0])

    def drop_ws(d):
        rs = [e for e in d['events'] if e['event'] == 'retry_solve'][0]
        rs['events'] = [e for e in rs['events'] if e['event'] != 'y:ws']

    def gmu(fn):
        def t(c, o):
            fn(c['gmu-35-50_default'])
        return t

    def bnb(fn):
        def t(c, o):
            fn(c['nidda_bnb_H_s1'])
        return t

    def bnb_ref(fn, ref=True):
        """F/G tampers: also applied to the reference capture so that check A passes and F/G must reject."""
        def t(c, o):
            for r in RUNS:
                fn(c['nidda_bnb_H_s1'][r])
            if ref:
                fn(o['nidda_bnb_H_s1'])
        return t

    def rs(d):
        return [e for e in d['events'] if e['event'] == 'retry_solve'][0]['events']

    def ev(d, name):
        return [e for e in rs(d) if e['event'] == name][0]

    def parent(d):
        rs(d)[0]['node']['parent'] = None

    def call(d):
        rs(d)[0]['call'] += 1

    def i94(d):
        ev(d, 'x:retry_basis')['node_basis']['I94'] = 464

    def frame(d):
        w = ev(d, 'y:where'); w['bt_accurate'] = []; w['bt_fuzzy'] = [x for x in w['bt_fuzzy'] if not 0x5101d0 <= x < 0x5101d0 + 21510]

    def restarts(d):
        ev(d, 'y:where')['restarts'] = 0

    def dec_len(d):
        ev(d, 'x:decode:leave')['cbasis'].pop()

    def extra_blank(d):
        # empty one more ws row (entries removed, b=0) consistently in every run
        w = ev(d, 'y:ws'); i = w['ind'][0]; keep = [k for k in range(len(w['ind'])) if w['ind'][k] != i]
        cb = [sum(1 for k in keep if k < w['cbeg'][j]) for j in range(w['cols'] + 1)]
        w['ind'] = [w['ind'][k] for k in keep]; w['val'] = [w['val'][k] for k in keep]; w['cbeg'] = cb; w['b'][i] = '0' * 16

    def zero_bin(d):
        d['binary_sha256'] = '0' * 64

    def zero_model(d):
        d['model_sha256'] = '0' * 64

    def node_id(d):
        rs = [e for e in d['events'] if e['event'] == 'retry_solve'][0]
        rs['events'][0]['node']['id'] += 2

    def basis_entry(d):
        rs = [e for e in d['events'] if e['event'] == 'retry_solve'][0]
        x = [e for e in rs['events'] if e['event'] == 'x:cda5b:enter' and e['vbasis']][-1]; x['vbasis'][0] ^= 1

    bnb_t = [('bnb ' + n, bnb(t)) for n, t in [
        ('binary hash', all_runs(zero_bin)), ('model hash', all_runs(zero_model)), ('retry solve removed', all_runs(drop_retry)),
        ('node id', all_runs(node_id)), ('nnz value', all_runs(set_val)), ('cutoff', all_runs(set_cut)), ('dhash', all_runs(dhash)),
        ('non-y event', all_runs(non_y)), ('run differs', lambda cs: set_ub(cs[3])), ('ws array', all_runs(ws_l)),
        ('model row', all_runs(model_row)), ('cda5b basis', all_runs(basis_entry))]]
    bnb_t += [('bnb F/G ' + n, bnb_ref(fn, ref)) for n, fn, ref in [
        ('node id', node_id, True), ('parent', parent, True), ('call', call, True), ('I94', i94, True), ('5fb8ec frame', frame, False),
        ('restarts', restarts, False), ('cda5b basis', basis_entry, True), ('decoded basis length', dec_len, True), ('extra blanked row', extra_blank, False)]]
    return bnb_t + [('gmu cutoff', gmu(all_runs(set_cut))), ('gmu dhash', gmu(all_runs(dhash))), ('nnz value', nidda(all_runs(set_val))), ('bound', nidda(all_runs(set_ub))), ('cutoff', nidda(all_runs(set_cut))),
            ('retry solve removed', nidda(all_runs(drop_retry))), ('run differs', nidda(lambda cs: set_ub(cs[2]))),
            ('non-y event', nidda(all_runs(non_y))), ('dhash', nidda(all_runs(dhash))), ('ws array', nidda(all_runs(ws_l))),
            ('model row', nidda(all_runs(model_row))), ('ws missing', nidda(all_runs(drop_ws)))]


def main():
    if '--write-manifest' in sys.argv[1:]:
        write_manifest(); print('manifest written'); return 0
    selftest = '--self-test' in sys.argv[1:]
    caps, olds = load_all()
    try:
        check_manifests(caps)
        print('manifest PASS')
        res = run(caps, olds)
        check_lpfiles(caps)
        print('node LP files PASS')
    except Fail as e:
        print('FAIL', e); return 1
    for case, r in res.items():
        print('PASS', case, json.dumps(r))
        print('  ', json.dumps(summary(case, caps[case][1])))
    if selftest:
        for name, fn in tampers(caps):
            c, o = copy.deepcopy(caps), copy.deepcopy(olds)
            fn(c, o)
            try:
                run(c, o)
            except Fail as e:
                print('rejected', name, '-', e); continue
            print('FAIL tamper accepted:', name); return 1
        print('self-test: all', len(tampers(caps)), 'tampers rejected')
    return 0


if __name__ == '__main__':
    sys.exit(main())
