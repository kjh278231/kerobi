"""Compare one barrier step of the Rust dump (BARDUMP "step" records) with the oracle (--vec): theta of iteration k
against record k, final direction and step lengths against record k+1 (dx0.., alpha_p/alpha_d).
usage: python tools/proto/bar_cmp_step.py <oracle.jsonl.gz> <dump.jsonl> [k ...]"""
import gzip, json, struct, sys

h = lambda v: [float.fromhex(x) for x in v]
b = lambda v: [struct.unpack('<d', bytes.fromhex(x)[::-1])[0] for x in v]
orc = {r['iter']: r for r in map(json.loads, gzip.open(sys.argv[1], 'rt'))}
steps = {r['step']: r for r in map(json.loads, open(sys.argv[2])) if 'step' in r}
ks = [int(k) for k in sys.argv[3:]] or sorted(steps)
for k in ks:
    if k not in steps or k + 1 not in orc or orc[k + 1].get('dx0') is None:
        continue
    st, o1 = steps[k], orc[k + 1]
    out = []
    if orc.get(k, {}).get('theta'):
        g, a = h(orc[k]['theta']), b(st['theta'])
        out.append('theta %d/%d' % (sum(x == y for x, y in zip(a, g)), len(g)))
    for nm, on in (('dx', 'dx0'), ('dy', 'dy0'), ('dz', 'dz0'), ('dw', 'dw0'), ('ds', 'ds0')):
        g, a = h(o1[on]), b(st[nm])
        L = min(len(a), len(g))
        out.append('%s %d/%d' % (nm, sum(a[i] == g[i] for i in range(L)), L))
    if st.get('d1x'):
        for nm, on in (('d1x', 'dx1'), ('d1y', 'dy1')):
            g, a = h(o1[on]), b(st[nm])
            L = min(len(a), len(g))
            out.append('%s %d/%d' % (nm, sum(a[i] == g[i] for i in range(L)), L))
        out.append('oms ' + ','.join('%g' % v for v in b(st['oms'])))
    a = b(st['a'])
    out.append('alpha %s %s' % (a[0] == float.fromhex(o1['alpha_p']), a[1] == float.fromhex(o1['alpha_d'])))
    out.append('K %d/%d' % (st['ncorr'], o1['ncorr']))
    print(k, ' | '.join(out))
