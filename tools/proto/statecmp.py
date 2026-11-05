"""Per-iteration bit comparison of rsolver's dual state with Gurobi's (docs/m1_notes.md §8).
usage: python tools/proto/statecmp.py <name> <gurobi.jsonl from tools/spx_oracle.py> [--iters 0-30]
Runs rsolver with RSOLVER_SNAP and reports, per iteration, which arrays differ (vstat, xB, d, w) and where."""
import json, os, subprocess, sys, tempfile
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
F = float.fromhex
name, gpath = sys.argv[1], sys.argv[2]
g = {s["iter"]: s for s in map(json.loads, open(gpath))}
snap = tempfile.mktemp(suffix=".jsonl")
code = (f"import rsolver; m = rsolver.read({str(ROOT.parent / 're/bench/data/netlib' / (name + '.mps.gz'))!r}); "
        "m.Params['Method'] = 1; m.optimize()")
subprocess.run([sys.executable, "-c", code], env=dict(os.environ, RSOLVER_SNAP=snap), check=True)
mine = {}
for line in open(snap):
    s = json.loads(line)
    mine.setdefault(s["iter"], s)  # first snapshot of an iteration count (a refactor can repeat one)
os.unlink(snap)
for it in sorted(set(g) & set(mine)):
    a, b = g[it], mine[it]
    out = []
    if a["vstat"] != b["vstat"]:
        k = [j for j in range(len(a["vstat"])) if a["vstat"][j] != b["vstat"][j]]
        out.append(f"vstat {len(k)} diff, first {k[:5]}")
    for key in ("xB", "d", "w"):
        if a.get(key) is None:
            continue
        x, y = [F(v) for v in a[key]], [F(v) for v in b[key]]
        k = [j for j in range(min(len(x), len(y))) if x[j] != y[j]]
        if k:
            j = k[0]
            out.append(f"{key} {len(k)} diff, first {j}: g {x[j]!r} mine {y[j]!r}")
    pr = a.get("price") or {}
    if pr and (pr.get("mode") != b["mode"] or (pr.get("list") or []) != b["list"]):
        out.append(f"pricing g mode {pr.get('mode')} list {(pr.get('list') or [])[:6]} | mine mode {b['mode']} list {b['list'][:6]}")
    print(f"iter {it}: " + ("OK" if not out else "; ".join(out)))
