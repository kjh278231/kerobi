"""Pivot path of rsolver (Method 1) vs bench/oracle/path1: first differing pivot.
usage: python tools/proto/pathcmp.py name[,name...] | all     (all: one line each, then the median first-diff pivot)"""
import gzip, json, os, statistics, sys, rsolver
D = os.path.join(os.path.dirname(os.path.abspath(__file__)), '../../bench/oracle/path1')
names = sorted(f[:-8] for f in os.listdir(D)) if sys.argv[1] == 'all' else sys.argv[1].split(',')
firsts = []
for n in names:
    g = json.load(gzip.open(f'{D}/{n}.json.gz'))
    gp = [(tuple(ent), tuple(lev)) for it, ent, lev, flips, obj in g['path'][1:] if ent or lev]
    m = rsolver.read(f'/home/sds/workspace/gurobi-sandbox/re/bench/data/netlib/{n}.mps.gz'); m.Params['Method'] = 1
    m.Params['IterationLimit'] = 3 * g['IterCount'] + 1000
    m.optimize()
    mp = [((q,), (o,)) for (_, q, o) in m._path]
    k = next((i for i, (a, b) in enumerate(zip(gp, mp)) if a != b), min(len(gp), len(mp)))
    firsts.append(k / max(len(gp), 1))
    print(f"{n:9s} gurobi {g['IterCount']:.0f} pivots {len(gp)}  mine {m.IterCount:.0f} pivots {len(mp)}  first diff at pivot {k}: "
          f"g {gp[k] if k < len(gp) else None} mine {mp[k] if k < len(mp) else None}", flush=True)
if len(firsts) > 1:
    print(f"median first-diff fraction {statistics.median(firsts):.3f}, full match {sum(f >= 1 for f in firsts)}/{len(firsts)}")
