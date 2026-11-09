"""Compare the barrier prototype (bar_ipm.py) with Gurobi on the Netlib set (docs/m3/m3_ipm.md §5).

For each instance: LP data and the factor work estimates (F+0x160/0x168) from the iteration-0 oracle record
bench/oracle/bar/it0/<name>.jsonl.gz, reference trajectory and result from bench/gurobi_bar_nopre/<name>/.
LP from the MPS file (lp_from_model, needs PYTHONPATH=python for rsolver). Prints one row per instance (status, BarIterCount, objective error, first iteration whose pobj/dobj/compl leaves the
reference trajectory) and the match counts. usage: python tools/proto/bar_ipm_check.py [name ...] [--json out]
"""
import os

os.environ.setdefault("OMP_NUM_THREADS", "4")  # dense numpy on a shared machine
os.environ.setdefault("OPENBLAS_NUM_THREADS", "4")
import gzip
import json
import struct
import sys
import time

sys.path.insert(0, os.path.dirname(__file__))
import bar_ipm as P  # noqa: E402
import numpy as np  # noqa: E402

ROOT = os.path.join(os.path.dirname(__file__), "..", "..")
REF = os.path.join(ROOT, "bench", "gurobi_bar_nopre")


def check(nm):
    rec = P.load_oracle(os.path.join(ROOT, "bench", "oracle", "bar", "it0", nm + ".jsonl.gz"))[0]
    F = b"".join(struct.pack("<d", float.fromhex(t)) for t in rec["F"])
    F160, F168 = struct.unpack_from("<d", F, 0x160)[0], struct.unpack_from("<d", F, 0x168)[0]
    lp = P.lp_from_model(os.path.join(ROOT, "..", "re", "bench", "data", "netlib", nm + ".mps.gz"))
    st0 = P.initial_point(lp)
    h = P.hexf
    e0 = max(float(np.max(np.abs(a - h(rec[k])) / (1 + np.abs(h(rec[k])))))
             for a, k in ((st0.x, "x"), (st0.z, "z"), (st0.w, "a670"), (st0.s, "a678")) if len(a))
    ref = [ln.rstrip().split(",") for ln in gzip.open(os.path.join(REF, nm, "trace.csv.gz"), "rt") if ln.startswith("barrier")]
    summ = json.load(open(os.path.join(REF, nm, "summary.json")))
    t0 = time.time()
    status, iters, tr = P.run(lp, F160, F168)
    div = None
    for t in tr:
        it = t[0]
        if it >= len(ref):
            break
        g = [float(v) for v in ref[it][2:7]]
        bad = (abs(t[1] - g[0]) > 1e-6 * (1 + abs(g[0])) or abs(t[2] - g[1]) > 1e-6 * (1 + abs(g[1]))
               or abs(t[5] - g[4]) > 1e-4 * abs(g[4]))
        if bad:
            div = it
            break
    obj = tr[-1][1]
    objerr = abs(obj - summ["ObjVal"]) / max(1.0, abs(summ["ObjVal"]))
    return dict(name=nm, m=lp.m, n=lp.n, init_err=e0, status=status, grb_status=summ["Status"], iters=iters,
                grb_iters=summ["BarIterCount"], objerr=objerr, diverge=div, secs=round(time.time() - t0, 1))


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    out = sys.argv[sys.argv.index("--json") + 1] if "--json" in sys.argv else None
    if out in args:
        args.remove(out)
    names = args or sorted(os.listdir(REF))
    rows = []
    print("| name | m | n | iter-0 x,z,w,s err | status | iters (Gurobi) | obj rel err | first divergence | s |")
    print("|---|---|---|---|---|---|---|---|---|")
    for nm in names:
        try:
            r = check(nm)
        except Exception as e:  # keep going: one failing instance must not hide the table
            r = dict(name=nm, error=repr(e)[:120])
            print(f"| {nm} | error {r['error']} |")
            rows.append(r)
            continue
        rows.append(r)
        mark = "" if r["iters"] == r["grb_iters"] else " **"
        print(f"| {nm} | {r['m']} | {r['n']} | {r['init_err']:.0e} | {r['status']} | {r['iters']} ({r['grb_iters']}){mark} | {r['objerr']:.1e} | "
              f"{'-' if r['diverge'] is None else r['diverge']} | {r['secs']} |", flush=True)
    ok = [r for r in rows if "error" not in r]
    same = sum(r["iters"] == r["grb_iters"] for r in ok)
    st = sum(r["status"] == r["grb_status"] for r in ok)
    obj = sum(r["objerr"] <= 1e-6 for r in ok)
    near = sum(abs(r["iters"] - r["grb_iters"]) <= 0.05 * r["grb_iters"] for r in ok)
    init = sum(r["init_err"] <= 1e-10 for r in ok)
    print(f"\niteration-0 point within 1e-10 {init}/{len(rows)}, BarIterCount equal {same}/{len(rows)}, within 5% {near}/{len(rows)}, status equal {st}/{len(rows)}, "
          f"objective rel err <= 1e-6 {obj}/{len(rows)}")
    if out:
        json.dump(rows, open(out, "w"), indent=1)


if __name__ == "__main__":
    main()
