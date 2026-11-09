"""PRELIMIT sweep (docs/m2/m2_presolve.md §3): for each model, Gurobi's presolved model after each reduction call k
(GURO_PAR_PRELIMIT = k, Threads=1, WorkLimit 60 as tools/parity.py). Keeps <out>/<name>/<k>.mps only when it differs
from k-1 and writes <out>/<name>/sweep.tsv (k, sha256 prefix, changed). Stops after the output has equaled the
default result for --tail consecutive k.
usage: python tools/prelimit_sweep.py <out_dir> [--param N=V ...] [--tail 40] [--kmax 400] [name ...]
       (default: the 38 MIPLIB models; Gurobi: one process at a time, wrap in tools/gurun.sh)
"""
import argparse, hashlib, sys, tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from parity import instances  # noqa: E402


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("--param", action="append", default=[])
    ap.add_argument("--tail", type=int, default=40)
    ap.add_argument("--kmax", type=int, default=400)
    ap.add_argument("names", nargs="*")
    a = ap.parse_args()
    import gurobipy as gp
    params = {k: float(v) if "." in v or "e" in v else int(v) for k, v in (p.split("=", 1) for p in a.param)}
    insts = instances()
    names = a.names or [n for n, p in insts.items() if p.parent.name == "miplib"]
    with tempfile.TemporaryDirectory() as tmp:
        tmpf = Path(tmp) / "q.mps"

        def run(m, k):
            if k is not None:
                m.setParam("GURO_PAR_PRELIMIT", k)
            q = m.presolve()
            q.write(str(tmpf))
            q.dispose()
            return tmpf.read_bytes()

        for name in names:
            d = Path(a.out) / name
            d.mkdir(parents=True, exist_ok=True)
            env = gp.Env(params={"LogToConsole": 0, "Threads": 1, "WorkLimit": 60, **params})
            m = gp.read(str(insts[name]), env)
            final = hashlib.sha256(run(m, None)).hexdigest()
            prev, same, rows = None, 0, []
            for k in range(a.kmax + 1):
                b = run(m, k)
                h = hashlib.sha256(b).hexdigest()
                if h != prev:
                    (d / f"{k}.mps").write_bytes(b)
                rows.append(f"{k}\t{h[:12]}\t{int(h != prev)}")
                prev = h
                same = same + 1 if h == final else 0
                if same >= a.tail:
                    break
            (d / "sweep.tsv").write_text("k\tsha\tchanged\n" + "\n".join(rows) + f"\nfinal\t{final[:12]}\t-\n")
            m.dispose(); env.dispose()
            print(name, k, sum(r.endswith("\t1") for r in rows), flush=True)


if __name__ == "__main__":
    main()
