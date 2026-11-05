"""Hypothesis check: Gurobi's iteration-0 CHUZR list (mode 0, 15d2d rules) = basis positions violating a FINITE bound
by more than tol, tol = 1e-6 * 1e30 * 1e-5 = 1e19 while some nonbasic sits on an artificial bound (spx+0x2cc), else
1e-6; sorted by score viol^2/w descending. usage: python tools/proto/list0.py [name ...] (bench/oracle/iter0-5p)"""
import gzip, json, sys
from pathlib import Path
D = Path(__file__).resolve().parents[2] / "bench/oracle/iter0-5p"
F = lambda v: float.fromhex(v)
names = sys.argv[1:] or sorted(p.name[:-9] for p in D.glob("*.jsonl.gz"))
ok = okset = 0
for name in names:
    s = json.loads(gzip.open(D / f"{name}.jsonl.gz", "rt").readline())
    m, n = s["m"], s["n"]
    head = [None] * m
    art = False
    l, u = [F(x) for x in s["l"]], [F(x) for x in s["u"]]
    for j, v in enumerate(s["vstat"]):
        if v >= 0:
            head[v] = j
        elif (v == -1 and l[j] <= -1e30) or (v == -2 and u[j] >= 1e30):
            art = True
    tol = 1e19 if art else 1e-6
    xb, w = [F(x) for x in s["xB"]], [F(x) for x in s["w"]]
    sc = {}
    for p in range(m):
        j = head[p]
        if l[j] - xb[p] > tol and l[j] > -1e30:
            v = l[j] - xb[p]
        elif xb[p] - u[j] > tol and u[j] < 1e30:
            v = xb[p] - u[j]
        else:
            continue
        sc[p] = v * v / w[p] if w[p] else float('inf')
    mine = sorted(sc, key=lambda p: -sc[p])
    g = s["price"]["list"] or []
    okset += set(mine) == set(g)
    # order equal up to ties: scores along g must be non-increasing
    same = set(mine) == set(g) and all(sc[a] >= sc[b] for a, b in zip(g, g[1:]))
    ok += same
    if not same:
        k = next((i for i, (a, b) in enumerate(zip(g, g[1:])) if not sc.get(a, 0) >= sc.get(b, 0)), None)
        if k is not None:
            print("   ", [(p, head[p], head[p] >= n, sc[p], w[p]) for p in g[max(0, k - 1):k + 3]])
        print(f"{name:9s} art {art} mode {s['price']['mode']} g {len(g)} mine {len(mine)} only g {sorted(set(g)-set(mine))[:5]} only mine {sorted(set(mine)-set(g))[:5]}")
print(f"set equal {okset}/{len(names)}, order equal up to ties {ok}/{len(names)}")
