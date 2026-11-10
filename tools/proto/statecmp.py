"""Per-iteration bit comparison of rsolver's dual state with Gurobi's (docs/m1/m1_notes.md §8).
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
        "m.Params['Method'] = 1; m.Params['Presolve'] = 0; m.optimize()")
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
    gl = pr.get("list") or []
    if pr.get("mode") == 1:  # mode 1: the list is P+0x10[:P+0x28] (P+0x3c is the count at the last full path)
        gl = gl[:pr.get("allow")]
        if pr.get("thr") and b.get("thr") and F(pr["thr"]) != F(b["thr"]):
            out.append(f"thr g {F(pr['thr'])!r} mine {F(b['thr'])!r}")
    la, lb = [ga.get(p) for p in gl], [gb.get(p) for p in b["list"]]
    if pr and (pr.get("mode") != b["mode"] or la != lb):
        k = next((t for t in range(min(len(la), len(lb))) if la[t] != lb[t]), min(len(la), len(lb)))
        out.append(f"pricing (vars) g mode {pr.get('mode')} len {len(la)} | mine mode {b['mode']} len {len(lb)}; first diff at {k}: "
                   f"g {la[k:k + 4]} mine {lb[k:k + 4]}")
    if pr.get("score") and b.get("score") and pr.get("mode") == 1:
        gs, ms = [F(v) for v in pr["score"]], [F(v) for v in b["score"]]
        fl = pr.get("flags") or [0] * len(gs)  # stale scores sit at feasible positions (flag ≥ 2): skip them
        k = [j for j in range(len(va)) if va[j] >= 0 and vb[j] >= 0 and fl[va[j]] < 2 and gs[va[j]] != ms[vb[j]]]
        if k:
            out.append(f"score {len(k)} diff, first var {k[0]}: g {gs[va[k[0]]]!r} mine {ms[vb[k[0]]]!r}")
    if pr.get("row_cost") and b.get("row_cost"):
        if pr.get("cost_mode") != b.get("cost_mode"):
            out.append(f"cost mode g {pr.get('cost_mode')} mine {b.get('cost_mode')}")
        gs, ms = [F(v) for v in pr["row_cost"]], [F(v) for v in b["row_cost"]]
        k = [j for j in range(len(va)) if va[j] >= 0 and vb[j] >= 0 and gs[va[j]] != ms[vb[j]]]
        if k:
            out.append(f"row_cost {len(k)} diff, first var {k[0]}: g {gs[va[k[0]]]!r} mine {ms[vb[k[0]]]!r}")
        gv, mv = [F(v) for v in pr["var_cost"]], [F(v) for v in b["var_cost"]]
        k = [j for j in range(len(gv)) if gv[j] != mv[j]]
        if k:
            out.append(f"var_cost {len(k)} diff, first {k[0]}: g {gv[k[0]]!r} mine {mv[k[0]]!r}")
    lu = a.get("lu") or {}
    if lu and (lu.get("k"), pr.get("allow")) != (b.get("k"), b.get("allow") if b.get("mode") != 1 else len(b["list"])):
        out.append(f"lu k/allow g {lu.get('k')}/{pr.get('allow')} mine {b.get('k')}/{b.get('allow') if b.get('mode') != 1 else len(b['list'])}")
    if lu and "eta" in b and (lu.get("eta"), lu.get("cols")) != (b["eta"], b["cols"]):
        out.append(f"lu eta/cols g {lu.get('eta')}/{lu.get('cols')} mine {b['eta']}/{b['cols']}")
    gp, mp = a.get("pt") or {}, b.get("pt") or {}
    if mp and any(gp.get(k) != mp[k] for k in ("rc", "deg", "deg0")) or mp and gp.get("it0") != float(mp["it0"]):
        out.append(f"pt g {[gp.get(k) for k in ('rc', 'deg', 'deg0', 'it0')]} mine {[mp[k] for k in ('rc', 'deg', 'deg0', 'it0')]}")
    print(f"iter {it}: " + ("OK" if not out else "; ".join(out)))
