"""M1-02l exploratory (docs/m1/m1_02l_eta_peel.md section 6): LU pool fill LU+0x160 of the original (a
tools/m1_eta_pool_lite.js capture) against the rsolver on PYTHONPATH (RSOLVER_SNAP "pool", snapshot k = state at
pivot k) on one Netlib model, Method=1 Presolve=0. Prints the first differing pivot, the updates before --upto whose
growth differs, and the row etas alive at --upto whose offset parity (value alignment) differs. Not a gate.
usage: PYTHONPATH=$PWD/python ../.venv/bin/python tools/m1_pool_explore.py CAPTURE.json.gz MODEL.mps.gz [--upto K]"""
import gzip, json, os, subprocess, sys, tempfile

cap, model = sys.argv[1], sys.argv[2]
upto = int(sys.argv[sys.argv.index("--upto") + 1]) if "--upto" in sys.argv else None
ev = json.load(gzip.open(cap))["events"]
fd, snap = tempfile.mkstemp(suffix=".jsonl")
os.close(fd)
code = (f"import rsolver; m = rsolver.read({model!r}); m.Params['Method'] = 1; m.Params['Presolve'] = 0; "
        "m.Params['OutputFlag'] = 0; m.optimize()")
subprocess.run([sys.executable, "-c", code], env=dict(os.environ, RSOLVER_SNAP=snap), check=True)
mine = {}
for line in open(snap):
    s = json.loads(line)
    mine.setdefault(s["iter"], s)
os.unlink(snap)
pool = lambda k: ev[k]["ft"]["pool"]
n = min(len(ev), len(mine))
first = next((k for k in range(n) if mine[k]["pool"] != pool(k)["x160"]), None)
print(f"{len(ev)} captured pivots, {len(mine)} Rust snapshots; first pivot with Rust pool != LU+0x160: {first}")
K = upto if upto is not None else n - 1
grow = []
for k in range(1, K + 1):
    if pool(k)["x18c"] == 0:
        if mine[k]["pool"] != pool(k)["x160"]:
            grow.append((k, "factor", pool(k)["x160"], mine[k]["pool"]))
        continue
    do, dr = pool(k)["x160"] - pool(k - 1)["x160"], mine[k]["pool"] - mine[k - 1]["pool"]
    if do != dr:
        grow.append((k - 1, "update", do, dr, "eta", pool(k)["x184"] - pool(k - 1)["x184"], mine[k]["eta"] - mine[k - 1]["eta"]))
print(f"pool growth differs on {len(grow)} events before pivot {K}: {grow}")
start = max(j for j in range(K + 1) if pool(j)["x18c"] == 0)
ups = [j for j in range(start, K) if pool(j + 1)["x184"] > pool(j)["x184"]]
odd = [(j, q["len"], q["vo"] % 2, mine[j]["pool"] % 2) for q, j in zip(pool(K)["etas"], ups)]
print(f"pivot {K}: factor at {start}, {len(odd)} etas; parity differs on {sum(a != b for _, _, a, b in odd)}, "
      f"length >= 8 (update, len, orig, Rust): {[e for e in odd if e[2] != e[3] and e[1] >= 8]}")
