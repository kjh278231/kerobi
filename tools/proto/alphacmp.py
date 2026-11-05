"""α-vector order and bits of rsolver's dual pivot rows vs Gurobi's BFRT inputs (bench/oracle/bfrt_io), call by call.
usage: python tools/proto/alphacmp.py name [ncalls]   (rsolver must be built with the RSOLVER_BFRT dump)"""
import gzip, json, os, subprocess, sys, tempfile
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
name, ncalls = sys.argv[1], int(sys.argv[2]) if len(sys.argv) > 2 else 10
g = [x for x in map(json.loads, gzip.open(ROOT / f"bench/oracle/bfrt_io/{name}.jsonl.gz", "rt")) if x[0] == "bfrt" and x[1] == ">"]
dump = tempfile.mktemp(suffix=".jsonl")
code = (f"import rsolver; m = rsolver.read({str(ROOT.parent / 're/bench/data/netlib' / (name + '.mps.gz'))!r}); "
        "m.Params['Method'] = 1; m.optimize()")
subprocess.run([sys.executable, "-c", code], env=dict(os.environ, RSOLVER_BFRT=dump), check=True)
mine = [json.loads(x) for x in open(dump)]
os.unlink(dump)
for k, (a, b) in enumerate(zip(g[:ncalls], mine)):
    gi, gv = a[5][1], a[5][2]
    mi, mv = b["idx"], [abs(float.fromhex(v)) for v in b["val"]]
    if a[4][1] != b["leave"]:
        print(f"call {k}: leaving differs g {a[4][1]} mine {b['leave']}"); break
    if gi != mi:
        t = next((t for t in range(min(len(gi), len(mi))) if gi[t] != mi[t]), min(len(gi), len(mi)))
        n = a[2] if a[2] <= len(gi) else 0
        print(f"  rho order (slack part) g {[j for j in gi if j >= min(gi[a[2]:] or [0])][:12]} mine {mi[a[2]:a[2] + 12]}")
        print(f"call {k}: order differs at {t} (len g {len(gi)} mine {len(mi)}): g {gi[max(0, t - 2):t + 6]} mine {mi[max(0, t - 2):t + 6]}"); break
    bad = [t for t in range(len(gi)) if abs(gv[t]) != mv[t]]
    print(f"call {k}: order equal ({len(gi)})" + (f", |value| differs at {len(bad)} e.g. j {gi[bad[0]]}: g {abs(gv[bad[0]])!r} mine {mv[bad[0]]!r}" if bad else ", values equal"))
