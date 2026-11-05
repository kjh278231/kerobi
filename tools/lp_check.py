"""M1 LP check (docs/09 M1, Presolve=0, Threads=1): status and objective of rsolver against Gurobi on the 64 Netlib
LPs inside the license (bench/gurobi_{primal,dual,default}_nopre) and against the published optimum on the other
Netlib files (re/bench/data/netlib_readme.txt; the CPLEX/MINOS alternatives listed there also count). Also reports
IterCount equality (P1) and time ratio.
usage: python tools/lp_check.py [--methods 1,0,-1] [--timeout 600] [--csv bench/lp_check.csv] [name ...]
"""
import argparse, csv, json, math, re, subprocess, sys, time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT.parent / "re/bench/data"
BASE = {1: "gurobi_dual_nopre", 0: "gurobi_primal_nopre", -1: "gurobi_default_nopre"}


def published():
    text = (DATA / "netlib_readme.txt").read_text()
    a = text.index("Name       Rows   Cols")
    table = text[a:text.index("BOUND-TYPE TABLE", a)]
    vals = {}
    for line in table.splitlines():
        t = line.split()
        if len(t) >= 5 and re.fullmatch(r"-?\d\.\d+E[+-]\d+", t[-1].replace("**", "")):
            vals.setdefault(t[0].lower().replace(".", ""), []).append(float(t[-1]))
    alt = text[text.index("Problem        CPLEX(Sparc)"):text.index("The above CPLEX and MINOS")]
    for line in alt.splitlines()[2:]:
        t = line.split()
        if t:
            vals.setdefault(t[0].lower().replace(".", ""), []).extend(float(x) for x in t[1:] if re.fullmatch(r"-?\d\.\d+E[+-]\d+", x))
    return vals


def run_one(name, method, timeout):
    code = (f"import rsolver, json, time\nm = rsolver.read({str(DATA / 'netlib' / (name + '.mps.gz'))!r})\n"
            f"m.Params['Method'] = {method}\nm.optimize()\n"
            f"print(json.dumps({{'Status': m.Status, 'ObjVal': m._objval, 'IterCount': m.IterCount, 'Runtime': m.Runtime}}))")
    t = time.time()
    try:
        p = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True, timeout=timeout)
        return json.loads(p.stdout.strip().splitlines()[-1]) if p.returncode == 0 else {"error": p.stderr.strip().splitlines()[-1:]}
    except subprocess.TimeoutExpired:
        return {"error": f"timeout {time.time() - t:.0f}s"}
    except Exception as e:
        return {"error": repr(e)}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--methods", default="1,0,-1")
    ap.add_argument("--timeout", type=float, default=600)
    ap.add_argument("--csv", type=Path, default=ROOT / "bench/lp_check.csv")
    ap.add_argument("names", nargs="*")
    a = ap.parse_args()
    pub = published()
    names = a.names or sorted(f.name[:-7] for f in (DATA / "netlib").glob("*.mps.gz"))
    rows, tally = [], {}
    for name in names:
        for method in (int(x) for x in a.methods.split(",")):
            r = run_one(name, method, a.timeout)
            ref = ROOT / "bench" / BASE[method] / name / "summary.json"
            g = json.load(open(ref)) if ref.exists() else None
            row = {"instance": name, "method": method, "status": r.get("Status"), "obj": r.get("ObjVal"),
                   "iters": r.get("IterCount"), "time": r.get("Runtime"), "error": r.get("error", "")}
            ok = r.get("Status") == 2
            if g:
                row.update(ref_status=g["Status"], ref_obj=g["ObjVal"], ref_iters=g["IterCount"], ref_time=g["Runtime"])
                ok = r.get("Status") == g["Status"] and (g["Status"] != 2 or abs(r["ObjVal"] - g["ObjVal"]) <= 1e-6 * max(1.0, abs(g["ObjVal"])))
                row["iters_equal"] = r.get("IterCount") == g["IterCount"]
            elif name in pub and ok:
                row["ref_obj"] = pub[name][0]
                ok = any(abs(r["ObjVal"] - v) <= 1e-6 * max(1.0, abs(v)) for v in pub[name])
            row["ok"] = ok
            rows.append(row)
            k = tally.setdefault(method, [0, 0, 0, 0])
            k[0] += 1; k[1] += ok; k[2] += bool(row.get("iters_equal")); k[3] += g is not None
            print(f"{name:10s} M{method:2d} st {row['status']} obj {row['obj']} ref {row.get('ref_obj')} it {row['iters']}/{row.get('ref_iters')} "
                  f"t {row['time'] if row['time'] is None else round(row['time'], 3)}/{row.get('ref_time') and round(row['ref_time'], 3)} {'OK' if ok else 'FAIL'} {row['error']}", flush=True)
    a.csv.parent.mkdir(parents=True, exist_ok=True)
    keys = ["instance", "method", "ok", "status", "ref_status", "obj", "ref_obj", "iters", "ref_iters", "iters_equal", "time", "ref_time", "error"]
    with open(a.csv, "w", newline="") as f:
        w = csv.DictWriter(f, keys, extrasaction="ignore")
        w.writeheader()
        w.writerows(rows)
    for method, (n, ok, eq, nref) in sorted(tally.items()):
        print(f"Method {method}: correct {ok}/{n}, IterCount equal {eq}/{nref}")


if __name__ == "__main__":
    main()
