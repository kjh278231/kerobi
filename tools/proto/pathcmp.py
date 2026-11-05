"""Pivot path of rsolver (Method 1) vs bench/oracle/path1: first differing pivot. usage python tools/proto/pathcmp.py name[,name...]"""
import gzip, json, sys, rsolver
import os; D = os.path.join(os.path.dirname(os.path.abspath(__file__)), '../../bench/oracle/path1')
for n in sys.argv[1].split(','):
    g = json.load(gzip.open(f'{D}/{n}.json.gz'))
    gp = []
    for it, ent, lev, flips, obj in g['path'][1:]:
        if ent or lev:
            gp.append((tuple(ent), tuple(lev)))
    m = rsolver.read(f'/home/sds/workspace/gurobi-sandbox/re/bench/data/netlib/{n}.mps.gz'); m.Params['Method'] = 1; m.optimize()
    mp = [((q,), (o,)) for (_, q, o) in m._path]
    k = next((i for i, (a, b) in enumerate(zip(gp, mp)) if a != b), min(len(gp), len(mp)))
    print(f"{n:9s} gurobi {g['IterCount']:.0f} pivots {len(gp)}  mine {m.IterCount:.0f} pivots {len(mp)}  first diff at pivot {k}: g {gp[k] if k < len(gp) else None} mine {mp[k] if k < len(mp) else None}")
