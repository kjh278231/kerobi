"""Iteration-0 CHUZR list (mode 0) with the full 15d2d score viol^2/w * P70[j]/P68[p] (cost mode P+0x44 = 1) and the
aabbe3 sort; compares with bench/oracle/iter0-5p. usage: python tools/proto/list0b.py [--bystep] [name ...]"""
import gzip, json, sys
from pathlib import Path
import rsolver
from rsolver._core import sort_desc
ROOT = Path(__file__).resolve().parents[2]
D = ROOT / "bench/oracle/iter0-5p"
F = float.fromhex
args = [a for a in sys.argv[1:] if not a.startswith("--")]
names = args or sorted(p.name[:-9] for p in D.glob("*.jsonl.gz"))
ok = 0
for name in names:
    s = json.loads(gzip.open(D / f"{name}.jsonl.gz", "rt").readline())
    m, n = s["m"], s["n"]
    core = rsolver.read(ROOT.parent / f"re/bench/data/netlib/{name}.mps.gz")._core
    costs, steppos, usable = core.row_costs(s["vstat"])
    p68 = [0.0] * m
    if "--bystep" in sys.argv:
        p68 = costs
    else:
        for k, pos in enumerate(steppos):
            p68[pos] = costs[k]
    nnz = s["beg"][n]
    # 534f7: cost mode P+0x44 (SimplexPricing auto, not a network)
    cmode = 0 if (n < 100 or m < 100 or (20 * n < nnz and 10.0 * n * n < m * nnz)) else 1
    lens = s["len"] + [1] * m
    p70 = [nnz / n + lens[j] for j in range(n + m)]
    head = [None] * m
    l, u = [F(x) for x in s["l"]], [F(x) for x in s["u"]]
    art = False
    for j, v in enumerate(s["vstat"]):
        if v >= 0:
            head[v] = j
        elif (v == -1 and l[j] <= -1e30) or (v == -2 and u[j] >= 1e30):
            art = True
    tol = 1e-6 * 1e30 * 1e-5 if art else 1e-6
    xb, w = [F(x) for x in s["xB"]], [F(x) for x in s["w"]]
    keys, idx = [], []
    for p in range(m):
        j = head[p]
        if l[j] - xb[p] > tol and l[j] > -1e30:
            v = l[j] - xb[p]
        elif xb[p] - u[j] > tol and u[j] < 1e30:
            v = xb[p] - u[j]
        else:
            continue
        sc = v * v / w[p] if w[p] else float("inf")
        if cmode == 0:
            pass
        elif usable:
            sc = sc * (p70[j] / p68[p])
        else:
            sc = sc / p68[p]
        keys.append(1.0 * sc)
        idx.append(p)
    _, mine = sort_desc(keys, idx)
    g = s["price"]["list"] or []
    same = list(mine) == g
    ok += same
    if not same:
        k = next((i for i, (a, b) in enumerate(zip(mine, g)) if a != b), min(len(mine), len(g)))
        print(f"{name:9s} usable {usable} len {len(g)}/{len(mine)} first diff {k}: g {g[k:k+4]} mine {list(mine[k:k+4])}")
print(f"{ok}/{len(names)} equal")
