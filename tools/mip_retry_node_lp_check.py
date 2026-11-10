#!/usr/bin/env python3
"""M2-03d-3 checker: retry node LP captures (tools/mip_retry_node_lp.js), contract docs/m2/mip_retry_node_lp.md section 3.

(default)    checks A, B, C1-C4, E on the stored captures and prints the findings
--self-test  also requires every in-memory tamper to be rejected
Exit code 0 only when every requested check passes.
"""
import copy, json, math, os, struct, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mip_retry_basis_check as B  # noqa: E402

ROOT = B.ROOT
DATA = 'bench/oracle/mip_retry_node_lp'
OLD = 'bench/oracle/mip_retry_basis'
RUNS = (1, 2, 3)
# old: M2-03c-2 natural capture; ws: C2 applies (ScaleFlag 0); model: E applies (node LP keeps the model columns)
CASES = {'nidda_stress': {'old': 'nidda_stress_natural', 'ws': True, 'model': True},
         'gmu-35-50_default': {'old': 'gmu-35-50_default_natural', 'ws': False, 'model': False}}
need, Fail = B.need, B.Fail


def f(h):
    return struct.unpack('>d', bytes.fromhex(h))[0]


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
    # A: instrumentation did not change the solve
    for r, d in caps.items():
        need(d['ok'] and not d['errors'], f'{case} run {r}: capture not ok')
        need(d['binary_sha256'] == B.BINARY, f'{case} run {r}: binary hash')
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
    out = {'rows': lp['rows'], 'cols': lp['cols'], 'nnz': lp['cbeg'][-1], 'node_lp_cda5b': len(dh)}
    env = lp['env_params']
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
        for j in range(n):
            need(ws['c'][j] == (lp['obj'][j] if sgn == 1 else neg(lp['obj'][j])), f'{case}: ws c[{j}] {ws["c"][j]} vs obj {lp["obj"][j]}')
            need(ws['l'][j] == lp['lb'][j] and ws['u'][j] == lp['ub'][j], f'{case}: ws bound col {j}: '
                 f'l {ws["l"][j]} u {ws["u"][j]} vs lb {lp["lb"][j]} ub {lp["ub"][j]}')
            need(ws['cbeg'][j + 1] - ws['cbeg'][j] == lp['cbeg'][j + 1] - lp['cbeg'][j], f'{case}: ws column {j} length')
            for k, k2 in zip(range(ws['cbeg'][j], ws['cbeg'][j + 1]), range(lp['cbeg'][j], lp['cbeg'][j + 1])):
                i = lp['ind'][k2]
                need(ws['ind'][k] == i, f'{case}: ws column {j} row index {ws["ind"][k]} vs {i}')
                need(ws['val'][k] == (neg(lp['val'][k2]) if flip[i] else lp['val'][k2]), f'{case}: ws value col {j} row {i}')
        for i in range(m):
            need(ws['b'][i] == (neg(lp['rhs'][i]) if flip[i] else lp['rhs'][i]), f'{case}: ws b[{i}] {ws["b"][i]} vs rhs {lp["rhs"][i]}')
            need(ws['sense'][i] == ('<' if flip[i] else lp['sense'][i]), f'{case}: ws sense[{i}]')
        out['slack_bounds'] = sorted({(ws['sense'][i], ws['l'][n + i], ws['u'][n + i]) for i in range(m)})
        out['slack_c_zero'] = all(f(x) == 0.0 for x in ws['c'][n:])
        # C3: ws cutoff from the LP env (02f7e1 C:247-259)
        cut, extra = env['Cutoff'], env['env+0x24b8']
        need(lp['lp_f0_null'], f'{case}: lp+0xf0 is set, cutoff would be 1e100')
        need(ws['cutoff_3b8'] == cut, f'{case}: ws cutoff {ws["cutoff_3b8"]} vs env Cutoff {cut}')
        need(ws['cutoff2_3c0'] == (cut if f(cut) >= f(extra) else extra), f'{case}: ws+0x3c0')
        # C4: retry status as in 03c-2, cutoff consistent
        wl = [e for e in ye if e['event'] == 'y:ws_leave']
        need(len(wl) == 1, f'{case}: expected one retry ws_leave')
        old_st = [e for e in es if e['event'] == 'x:cda5b:leave' and e['k'] == 2]
        need(len(old_st) == 1 and wl[0]['status_f0'] == old_st[0]['lp']['lp_status'], f'{case}: retry status vs 03c-2')
        if wl[0]['status_f0'] == 6:
            need(f(wl[0]['obj_118']) >= f(cut), f'{case}: status 6 with objective below cutoff')
        out.update(cutoff=f(cut), retry_status=wl[0]['status_f0'], retry_obj=f(wl[0]['obj_118']))
    if cfg['model']:
        # E: the node LP is the model plus extra rows; columns, objective and the model rows unchanged
        md = d['model_dump']
        need(md['cols'] == lp['cols'] and lp['rows'] >= md['rows'], f'{case}: model {md["cols"]}x{md["rows"]} vs LP')
        need(md['obj'] == lp['obj'] and md['vtype'] == lp['vtype'] and md['model_sense'] == lp['model_sense'] and md['obj_con'] == lp['obj_con'],
             f'{case}: objective, vtype or sense differ from the model')
        lr = rows_of(lp)
        for i in range(md['rows']):
            need(dict(zip(*md['row_major'][i])) == lr[i], f'{case}: LP row {i} differs from model row {i}')
            need(md['rhs'][i] == lp['rhs'][i] and md['sense'][i] == lp['sense'][i], f'{case}: LP row {i} rhs/sense')
        out['extra_rows'] = lp['rows'] - md['rows']
        out['bound_diffs'] = sum(md['lb'][j] != lp['lb'][j] or md['ub'][j] != lp['ub'][j] for j in range(md['cols']))
        out['extra_row_senses'] = ''.join(sorted(set(lp['sense'][md['rows']:])))
    return out


def summary(case, d):
    ye, _ = retry_y(d)
    w = one(ye, 'y:where')
    mains = [e for e in d['events'] if e['event'] == 'y:mipmain']
    rest = [e for e in d['events'] if e['event'] == 'y:restart']
    return {'where': {k: w[k] for k in ('depth', 'mip', 'root_mip', 'self_628', 'restarts', 'lp_is_mip_lp', 'mip_lp')},
            'mipmain_enters': [(e['call'], e['depth'], e['model_rows'], e['model_cols']) for e in mains if e['phase'] == 'enter'],
            'restarts': [(e['phase'], e['call'], e['mip']) for e in rest]}


def load_all():
    caps, olds = {}, {}
    for case, cfg in CASES.items():
        caps[case] = {r: B.load(f'{DATA}/{case}_{r}.json') for r in RUNS}
        olds[case] = B.load(f'{OLD}/{cfg["old"]}_1.json')
    return caps, olds


def run(caps, olds):
    return {case: check_case(case, caps[case], olds[case]) for case in CASES}


def tampers(caps):
    def nidda(fn):
        def t(c):
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

    return [('nnz value', nidda(all_runs(set_val))), ('bound', nidda(all_runs(set_ub))), ('cutoff', nidda(all_runs(set_cut))),
            ('retry solve removed', nidda(all_runs(drop_retry))), ('run differs', nidda(lambda cs: set_ub(cs[2]))),
            ('non-y event', nidda(all_runs(non_y))), ('dhash', nidda(all_runs(dhash))), ('ws array', nidda(all_runs(ws_l))),
            ('model row', nidda(all_runs(model_row))), ('ws missing', nidda(all_runs(drop_ws)))]


def main():
    selftest = '--self-test' in sys.argv[1:]
    caps, olds = load_all()
    try:
        res = run(caps, olds)
    except Fail as e:
        print('FAIL', e); return 1
    for case, r in res.items():
        print('PASS', case, json.dumps(r))
        print('  ', json.dumps(summary(case, caps[case][1])))
    if selftest:
        for name, fn in tampers(caps):
            c = copy.deepcopy(caps)
            fn(c)
            try:
                run(c, olds)
            except Fail as e:
                print('rejected', name, '-', e); continue
            print('FAIL tamper accepted:', name); return 1
        print('self-test: all', len(tampers(caps)), 'tampers rejected')
    return 0


if __name__ == '__main__':
    sys.exit(main())
