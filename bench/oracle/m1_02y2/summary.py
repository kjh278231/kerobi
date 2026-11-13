"""Summary of gurobi_m1.jsonl / rust_m1.jsonl (tools/m1_end_dualvio.py, Method 1, Presolve 0, Threads 1)."""
import json
from pathlib import Path

D = Path(__file__).parent
G = {r["name"]: r for r in map(json.loads, open(D / "gurobi_m1.jsonl"))}
R = {r["name"]: r for r in map(json.loads, open(D / "rust_m1.jsonl"))}
v = lambda r: max(r["dvio_col"], r["dvio_row"])
print("errors", [k for k, r in G.items() if "error" in r], [k for k, r in R.items() if "error" in r])
for name, S in (("gurobi", G), ("rust", R)):
    xs = sorted(((v(r), k) for k, r in S.items() if "dvio_col" in r), reverse=True)
    print(name, "models", len(xs), "n>1e-9", sum(x > 1e-9 for x, _ in xs), "n>1e-7", sum(x > 1e-7 for x, _ in xs))
    print("  top", ", ".join("%s %.2e" % (k, x) for x, k in xs[:6]))
both = [k for k in G if k in R and "dvio_col" in G[k] and "dvio_col" in R[k]]
same = [k for k in both if G[k]["iter"] == R[k]["iter"]]
eq = [k for k in same if G[k]["obj"] == R[k]["obj"] and v(G[k]) == v(R[k])]
print("same IterCount", len(same), "of", len(both), "; obj and dual violation bit-equal among them", len(eq))
print("  not bit-equal:", ", ".join("%s %.2e/%.2e" % (k, v(G[k]), v(R[k])) for k in same if k not in eq))
close = [k for k in same if abs(v(G[k]) - v(R[k])) <= 1e-12 and abs(G[k]["obj"] - R[k]["obj"]) <= 1e-12 * max(1, abs(G[k]["obj"]))]
print("  dual violation within 1e-12 and obj within 1e-12 rel:", len(close), "of", len(same))
