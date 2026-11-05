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
    va, vb = a["vstat"], b["vstat"]
    k = [j for j in range(len(va)) if va[j] != vb[j] and (va[j] < 0 or vb[j] < 0)]
    if k:
        out.append(f"vstat {len(k)} diff, first {k[:5]}")
    elif va != vb:
        out.append("positions differ")
    for key in ("xB", "d", "w"):
        if a.get(key) is None:
            continue
        x, y = [F(v) for v in a[key]], [F(v) for v in b[key]]
        if key == "d":
            k = [j for j in range(min(len(x), len(y))) if x[j] != y[j]]
        else:  # by basic variable, so renumbered positions still compare
            k = [j for j in range(len(va)) if va[j] >= 0 and vb[j] >= 0 and x[va[j]] != y[vb[j]]]
            x, y = {j: x[va[j]] for j in k}, {j: y[vb[j]] for j in k}
        if k:
            j = k[0]
            out.append(f"{key} {len(k)} diff, first {'var ' if key != 'd' else ''}{j}: g {x[j]!r} mine {y[j]!r}")
    pr = a.get("price") or {}
    hv = lambda v: {s: j for j, s in enumerate(v) if s >= 0}  # position -> variable
    ga, gb = hv(va), hv(vb)
    la, lb = [ga.get(p) for p in pr.get("list") or []], [gb.get(p) for p in b["list"]]
    if pr and (pr.get("mode") != b["mode"] or la != lb):
        out.append(f"pricing (vars) g mode {pr.get('mode')} list {la[:6]} | mine mode {b['mode']} list {lb[:6]}")
    lu = a.get("lu") or {}
    if lu and (lu.get("k"), pr.get("allow")) != (b.get("k"), b.get("allow")):
        out.append(f"lu k/allow g {lu.get('k')}/{pr.get('allow')} mine {b.get('k')}/{b.get('allow')}")
    print(f"iter {it}: " + ("OK" if not out else "; ".join(out)))
