#!/usr/bin/env python3
"""M2-03d-1: project the M2-03c-2 nidda retry captures onto the Rust retry-control trace schema (v1).

(default)       re-derive the fixture from the manifest-checked captures (3 runs must agree) and require the
                stored fixture to match it exactly
--write         write the fixture instead of checking it
--compare TRACE compare a Rust trace in the same schema with the stored fixture; print the first difference
--self-test     also require every tamper of the captures, the fixture and a copied trace to be rejected
Exit code 0 only when every requested check passes. Schema and rules: docs/m2/mip_retry_rust_boundary.md.
"""
import copy, json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mip_retry_basis_check as B  # noqa: E402

FIXTURE = 'bench/oracle/mip_retry_rust/nidda_stress.json'
SCHEMA = 'rsolver.mip_retry.v1'
CASE = 'nidda_stress'
need, Fail = B.need, B.Fail


def project(d):
    """One natural capture -> fixture. Only fields the Rust retry control can emit; LP outcomes become inputs."""
    es = B.ev(d)
    _, enter = B.first(es, 'solve:enter')
    _, dec = B.first(es, 'x:decode:leave')
    _, c1 = B.first(es, 'x:cda5b:enter', k=1)
    _, l1 = B.first(es, 'x:cda5b:leave', k=1)
    _, r = B.first(es, 'retry')
    _, c2 = B.first(es, 'x:cda5b:enter', retry=True)
    _, l2 = B.first(es, 'x:cda5b:leave', k=c2['k'])
    _, leave = B.first(es, 'solve:leave')
    vb, cb = dec['vbasis'], dec['cbasis']
    need(vb and cb and len(vb) == c1['cols'] and len(cb) == c1['rows'], 'node basis missing or wrong length')
    need(not c1['retry'] and c2['k'] == 2 and len([e for e in es if e['event'] == 'x:cda5b:enter']) == 2,
         'expected exactly one first LP and one retry cda5b')
    for c in (c1, c2):  # both LPs take the node arrays I+0x98/0xa0 themselves (5ce67b C:361, C:929)
        need((c['vb_rel'], c['cb_rel']) == ('I98', 'Ia0'), f'cda5b {c["k"]} basis is not the node basis')
        need(c['vbasis'] == vb and c['cbasis'] == cb, f'cda5b {c["k"]} basis content differs from the node basis')
    saves = [e for e in es[es.index(c2):] if e['event'] in ('x:export', 'x:save_lp2:enter') and not e.get('synthetic')]
    return {
        'schema': SCHEMA, 'case': CASE, 'branch': 'node_basis_direct',
        'input': {'cols': c1['cols'], 'rows': c1['rows'], 'root': enter['node']['root'], 'cache': r['node']['cache'],
                  # param_3[3]: not captured; equals the first LP method because 8e4682 passes param_7 = 0 (no
                  # adaptive switch, 5ce67b C:362) and the node basis selects the param_3[3] branch (C:353-361)
                  'warm_method': c1['method'], 'node_basis': {'vbasis': vb, 'cbasis': cb}},
        'injected': {'first_lp_status': l1['lp']['lp_status'], 'retry_lp_status': l2['lp']['lp_status']},
        'events': [
            {'ev': 'node_lp', 'method': c1['method'], 'vbasis': c1['vbasis'], 'cbasis': c1['cbasis']},
            {'ev': 'node_lp_done', 'status': l1['lp']['lp_status']},
            {'ev': 'retry', 'method': c2['method'], 'vbasis': c2['vbasis'], 'cbasis': c2['cbasis']},
            {'ev': 'retry_done', 'status': l2['lp']['lp_status']},
            {'ev': 'node_done', 'node_status': leave['node']['status'], 'lp_status': leave['lp']['status'],
             'basis_saved': bool(saves)},
        ],
    }


def derive(caps):
    runs = [project(caps[(CASE, 'natural', r)]) for r in B.RUNS]
    need(all(p == runs[0] for p in runs), 'natural runs project differently')
    f = runs[0]
    need(f['injected']['first_lp_status'] == 12, 'first LP did not end in status 12')
    need(f['events'][2]['method'] == int(f['input']['warm_method'] == 0), 'retry method breaks param_3[3]==0 rule')
    return f


def compare(exp, got):
    """Exact comparison; returns None or the first difference."""
    for k in ('schema', 'case', 'branch', 'input', 'injected'):
        if got.get(k) != exp[k]:
            return f'{k}: expected {json.dumps(exp[k])[:120]} got {json.dumps(got.get(k))[:120]}'
    ge = got.get('events') or []
    for i, e in enumerate(exp['events']):
        if i >= len(ge):
            return f'event {i} ({e["ev"]}): missing'
        for f, v in e.items():
            if ge[i].get(f) != v:
                return f'event {i} ({e["ev"]}) field {f}: expected {json.dumps(v)[:80]} got {json.dumps(ge[i].get(f))[:80]}'
        if set(ge[i]) != set(e):
            return f'event {i} ({e["ev"]}): fields {sorted(ge[i])} != {sorted(e)}'
    if len(ge) != len(exp['events']):
        return f'event {len(exp["events"])}: unexpected {ge[len(exp["events"])]}'
    return None


def check(caps, stored):
    B.check_manifest(B.load(B.MANIFEST))
    diff = compare(derive(caps), stored)
    need(diff is None, f'stored fixture differs from captures: {diff}')
    return len(stored['events'])


def tampers():
    """(name, target, mutation). target 'caps' feeds check(); 'trace' feeds compare() against the fixture."""
    nat = lambda r: (CASE, 'natural', r)
    def ev_of(c, r, name, **kw):
        return B.first(B.ev(c[nat(r)]), name, **kw)[1]
    def drop_retry(c):
        es = B.ev(c[nat(1)])
        es[:] = [e for e in es if not (e['event'] == 'x:cda5b:enter' and e.get('retry'))]
    return [
        ('capture retry method', 'caps', lambda c: [ev_of(c, r, 'x:cda5b:enter', retry=True).update(method=1) for r in B.RUNS]),
        ('capture retry basis alias', 'caps', lambda c: ev_of(c, 1, 'x:cda5b:enter', retry=True).update(vb_rel='other')),
        ('capture retry basis content', 'caps', lambda c: ev_of(c, 2, 'x:cda5b:enter', retry=True)['vbasis'].__setitem__(0, -1)),
        ('capture node basis null', 'caps', lambda c: ev_of(c, 1, 'x:decode:leave').update(vbasis=None)),
        ('capture retry removed', 'caps', drop_retry),
        ('capture first status', 'caps', lambda c: [ev_of(c, r, 'x:cda5b:leave', k=1)['lp'].update(lp_status=2) for r in B.RUNS]),
        ('capture run differs', 'caps', lambda c: ev_of(c, 3, 'solve:leave')['node'].update(status=2)),
        ('capture natural export', 'caps', lambda c: B.ev(c[nat(2)]).insert(-3, {'event': 'x:export', 'synthetic': False})),
        ('trace retry method', 'trace', lambda t: t['events'][2].update(method=1)),
        ('trace retry from failed basis', 'trace', lambda t: t['events'][2]['vbasis'].__setitem__(3, 0)),
        ('trace first LP cold', 'trace', lambda t: t['events'][0].update(vbasis=None, cbasis=None)),
        ('trace basis saved', 'trace', lambda t: t['events'][4].update(basis_saved=True)),
        ('trace node status', 'trace', lambda t: t['events'][4].update(node_status=2)),
        ('trace extra field', 'trace', lambda t: t['events'][1].update(iter=7)),
        ('trace extra event', 'trace', lambda t: t['events'].insert(4, {'ev': 'basis_save'})),
        ('trace events empty', 'trace', lambda t: t.update(events=[])),
        ('trace injected', 'trace', lambda t: t['injected'].update(retry_lp_status=2)),
        ('trace schema', 'trace', lambda t: t.update(schema='v0')),
    ]


def main(argv):
    caps = {k: B.load(p) for k, p in B.files().items() if k[1] == 'natural'}
    if '--write' in argv:
        B.check_manifest(B.load(B.MANIFEST))
        with open(os.path.join(B.ROOT, FIXTURE), 'w') as f:
            json.dump(derive(caps), f, indent=1)
            f.write('\n')
        print('wrote', FIXTURE)
        return 0
    stored = B.load(FIXTURE)
    rc = 0
    try:
        print('fixture PASS', check(caps, stored), 'events')
    except Fail as e:
        print('fixture FAIL', e)
        rc = 1
    if '--compare' in argv:
        diff = compare(stored, B.load(argv[argv.index('--compare') + 1]))
        print('compare', 'PASS' if diff is None else 'FAIL ' + diff)
        rc |= diff is not None
    if '--self-test' in argv:
        if compare(stored, copy.deepcopy(stored)) is not None:
            print('self-test identical trace NOT ACCEPTED')
            rc = 1
        for name, target, t in tampers():
            if target == 'caps':
                c = copy.deepcopy(caps)
                t(c)
                try:
                    check(c, stored)
                    bad = None
                except (Fail, KeyError, TypeError, IndexError, ValueError) as e:
                    bad = f'{type(e).__name__}: {e}'
            else:
                tr = copy.deepcopy(stored)
                t(tr)
                bad = compare(stored, tr)
            print('self-test', name, 'rejected:' if bad else 'NOT REJECTED', bad or '')
            rc |= bad is None
    return rc


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
