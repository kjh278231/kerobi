"""Compare the Rust barrier iterates (BARDUMP=file dump in ipm.rs) bit for bit with the oracle (bar_oracle.py --vec).
usage: python tools/proto/bar_cmp.py <oracle.jsonl.gz> <dump.jsonl> [iters]"""
import gzip, json, struct, sys

h = lambda v: [float.fromhex(x) for x in v]
b = lambda v: [struct.unpack('<d', bytes.fromhex(x)[::-1])[0] for x in v]
orc = {r['iter']: r for r in map(json.loads, gzip.open(sys.argv[1], 'rt'))}
mine = {r['iter']: r for r in map(json.loads, open(sys.argv[2])) if 'iter' in r}
SC = {'compl': 0x1f0, 'mu': 0x1f8, 'rp2': 0x198, 'rd2': 0x1a8, 'ru2': 0x1b8, 'pobj': 0x130, 'dobj': 0x140,
      'rp_rel': 0x1d8, 'rd_rel': 0x1e0, 'ctx': 0x138}
for it in sorted(set(orc) & set(mine))[: int(sys.argv[3]) if len(sys.argv) > 3 else None]:
    o, r = orc[it], mine[it]
    m, n = o['m'], o['n']
    out = []
    for k, ok in (('x', 'x'), ('y', 'y'), ('z', 'z'), ('w', 'a670'), ('s', 'a678'), ('rd', 'rd'), ('rp', 'rp')):
        if o.get(ok) is None:
            continue
        a, g = b(r[k]), h(o[ok])
        L = min(len(a), len(g))
        bad = [i for i in range(L) if a[i] != g[i] and not (a[i] != a[i] and g[i] != g[i])]
        mx = max((abs(a[i] - g[i]) / max(abs(g[i]), 1e-300) for i in bad), default=0)
        out.append(f"{k} {L - len(bad)}/{L} rel{mx:.0e}")
    Bv = o.get('B')
    if Bv:
        sc = b(r['sc'])
        bads = [nm for i, (nm, off) in enumerate(SC.items()) if sc[i] != float.fromhex(Bv[off // 8])]
        out.append('scalars bad: ' + ','.join(bads))
    print(it, ' | '.join(out))
