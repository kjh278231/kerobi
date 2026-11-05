"""Differential parity harness (docs/10_porting.md §6).

capture: solve each instance with gurobipy or rsolver (same gurobipy-shaped API) under deterministic settings
         (Threads=1, WorkLimit, no TimeLimit) and store summary.json, trace.csv.gz, log.txt, presolved.mps.gz.
compare: P0 (fingerprint, presolved model) and P1 (status, iterations, nodes, first trace divergence) and P2 (work)
         between a reference and a test capture; writes one row per instance to a CSV.
usage: python tools/parity.py capture gurobipy|rsolver <out_dir> [--work-limit 60] [name ...]
       python tools/parity.py compare <ref_dir> <test_dir> [--csv bench/parity.csv]
Default instances: the 64 Netlib LPs of re/bench/results/lp.csv and the 38 MIPs of re/bench/data/miplib_small.csv.
"""
import argparse, csv, gzip, hashlib, importlib, json, re, sys, traceback
from pathlib import Path

SANDBOX = Path(__file__).resolve().parents[2]
DATA = SANDBOX / "re/bench/data"
SUMMARY = ("Fingerprint", "Status", "IterCount", "BarIterCount", "NodeCount", "Work", "Runtime", "ObjVal", "ObjBound",
           "SolCount", "IsMIP", "NumVars", "NumConstrs", "NumNZs")
MAX_ROWS = 1_000_000


def instances():
    lp = sorted({r["instance"] for r in csv.DictReader(open(SANDBOX / "re/bench/results/lp.csv"))})
    mip = [r["name"] for r in csv.DictReader(open(DATA / "miplib_small.csv"))]
    return {**{n: DATA / "netlib" / f"{n}.mps.gz" for n in lp}, **{n: DATA / "miplib" / f"{n}.mps.gz" for n in mip}}


def parse_log(text):
    """Log anchors the callbacks cannot give: presolve size, root LP (MIP fires no SIMPLEX callback), cut counts."""
    out = {}
    if m := re.search(r"Presolve removed (\d+) rows and (\d+) columns", text):
        out["presolve_removed"] = [int(m[1]), int(m[2])]
    if m := re.search(r"Root relaxation: objective ([^,]+), (\d+) iterations", text):
        out["root"] = [m[1], int(m[2])]
    if m := re.search(r"Cutting planes:\n((?:  .+\n)+)", text):
        out["cuts"] = {k.strip(): int(v) for k, v in re.findall(r"(.+): (\d+)", m[1])}
    return out


def capture_one(gp, path, d, work_limit):
    C = gp.GRB.Callback
    # Only callbacks fired by solver events are recorded; PRESOLVE and MIP polling fire on a timer and would make
    # traces differ run to run. Presolve is covered by presolved.mps.gz (P0).
    fields = {C.SIMPLEX: ("simplex", (C.SPX_ITRCNT, C.SPX_OBJVAL, C.SPX_PRIMINF, C.SPX_DUALINF, C.SPX_ISPERT)),
              C.BARRIER: ("barrier", (C.BARRIER_ITRCNT, C.BARRIER_PRIMOBJ, C.BARRIER_DUALOBJ, C.BARRIER_PRIMINF,
                                      C.BARRIER_DUALINF, C.BARRIER_COMPL)),
              C.MIPSOL: ("mipsol", (C.MIPSOL_NODCNT, C.MIPSOL_OBJ, C.MIPSOL_OBJBST, C.MIPSOL_OBJBND)),
              C.MIPNODE: ("mipnode", (C.MIPNODE_NODCNT, C.MIPNODE_STATUS, C.MIPNODE_OBJBST, C.MIPNODE_OBJBND,
                                      C.MIPNODE_SOLCNT))}
    rows, in_tree = [], [False]

    def cb(model, where):
        if where not in fields or len(rows) >= MAX_ROWS:
            return
        tag, what = fields[where]
        if tag == "mipnode":
            n = model.cbGet(C.MIPNODE_NODCNT)
            in_tree[0] = n >= 1
            if n >= 1000 and n % 1000:  # every node below 1000, then every 1000th
                return
        elif tag == "simplex" and in_tree[0]:
            return  # root LP only; node LPs would be millions of rows
        rows.append([tag, *(repr(model.cbGet(w)) for w in what)])

    env = gp.Env(params={"LogToConsole": 0, "LogFile": str(d / "log.txt"), "Threads": 1, "WorkLimit": work_limit})
    m = gp.read(str(path), env)
    m.optimize(cb)
    summary = {}
    for a in SUMMARY:
        try:
            summary[a] = getattr(m, a)
        except Exception:  # attribute not available for this model/status (e.g. ObjVal without a solution)
            summary[a] = None
    if summary["Fingerprint"] is not None:
        summary["Fingerprint"] &= 0xFFFFFFFF
    m.dispose()
    summary["log"] = parse_log((d / "log.txt").read_text())  # read before presolve() appends to the same log
    p = gp.read(str(path), env)
    q = p.presolve()
    q.write(str(d / "presolved.mps.gz"))
    summary["presolved"] = [q.NumConstrs, q.NumVars, q.NumNZs]
    summary["trace_rows"] = len(rows)
    q.dispose(); p.dispose(); env.dispose()
    with gzip.open(d / "trace.csv.gz", "wt") as f:
        csv.writer(f).writerows(rows)
    json.dump(summary, open(d / "summary.json", "w"), indent=1)


def capture(backend, out, work_limit, names):
    gp = importlib.import_module(backend)
    insts = instances()
    for name in names or insts:
        d = out / name
        d.mkdir(parents=True, exist_ok=True)
        try:
            capture_one(gp, insts[name], d, work_limit)
            s = json.load(open(d / "summary.json"))
            print(f"{name:24s} status={s['Status']} it={s['IterCount']} nodes={s['NodeCount']} work={s['Work']:.3f}", flush=True)
        except Exception:
            (d / "error.txt").write_text(traceback.format_exc())
            print(f"{name:24s} ERROR {traceback.format_exc().splitlines()[-1]}", flush=True)


def presolved_digest(d):
    p = d / "presolved.mps.gz"
    return hashlib.sha256(gzip.open(p).read()).hexdigest() if p.exists() else None


def first_divergence(ref, test):
    for i, (a, b) in enumerate(zip(ref, test)):
        if a != b:
            return f"row {i} {a[0]}: {','.join(a[1:3])} vs {','.join(b[1:3])}"
    return "" if len(ref) == len(test) else f"row {min(len(ref), len(test))}: length {len(ref)} vs {len(test)}"


def compare(ref_dir, test_dir, out_csv):
    cols = ["instance", "p0_fingerprint", "p0_presolved", "status", "iters", "bar_iters", "nodes", "root", "cuts",
            "work_ratio", "first_divergence"]
    out, tally = [], {"p0": 0, "p1": 0, "n": 0}
    for rd in sorted(p for p in ref_dir.iterdir() if (p / "summary.json").exists()):
        td = test_dir / rd.name
        r = json.load(open(rd / "summary.json"))
        if not (td / "summary.json").exists():
            out.append({"instance": rd.name, "first_divergence": "missing in test"})
            continue
        t = json.load(open(td / "summary.json"))
        load = lambda d: [row for row in csv.reader(gzip.open(d / "trace.csv.gz", "rt"))]
        same = lambda k: "ok" if r.get(k) == t.get(k) else f"{r.get(k)} vs {t.get(k)}"
        same_log = lambda k: "ok" if r["log"].get(k) == t.get("log", {}).get(k) else f"{r['log'].get(k)} vs {t.get('log', {}).get(k)}"
        row = {"instance": rd.name,
               "p0_fingerprint": same("Fingerprint"),
               "p0_presolved": "ok" if presolved_digest(rd) == presolved_digest(td) else f"{r['presolved']} vs {t.get('presolved')}",
               "status": same("Status"), "iters": same("IterCount"), "bar_iters": same("BarIterCount"), "nodes": same("NodeCount"),
               "root": same_log("root"), "cuts": same_log("cuts"),
               "work_ratio": f"{t['Work'] / r['Work']:.4f}" if r.get("Work") and t.get("Work") is not None else "",
               "first_divergence": first_divergence(load(rd), load(td))}
        tally["n"] += 1
        tally["p0"] += row["p0_fingerprint"] == row["p0_presolved"] == "ok"
        tally["p1"] += all(row[k] == "ok" for k in ("status", "iters", "bar_iters", "nodes", "root", "cuts")) and not row["first_divergence"]
        out.append(row)
    out_csv.parent.mkdir(parents=True, exist_ok=True)
    with open(out_csv, "w", newline="") as f:
        w = csv.DictWriter(f, cols)
        w.writeheader()
        w.writerows(out)
    print(f"instances {tally['n']}: P0 match {tally['p0']}, P1 match {tally['p1']} -> {out_csv}")


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("capture")
    c.add_argument("backend", choices=["gurobipy", "rsolver"])
    c.add_argument("out", type=Path)
    c.add_argument("--work-limit", type=float, default=60.0)
    c.add_argument("names", nargs="*")
    k = sub.add_parser("compare")
    k.add_argument("ref", type=Path)
    k.add_argument("test", type=Path)
    k.add_argument("--csv", type=Path, default=Path(__file__).resolve().parents[1] / "bench/parity.csv")
    a = ap.parse_args()
    if a.cmd == "capture":
        capture(a.backend, a.out, a.work_limit, a.names)
    else:
        compare(a.ref, a.test, a.csv)
    sys.exit(0)
