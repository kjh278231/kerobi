"""Presolve oracle (docs/m1_presolve.md): Gurobi's presolved model under chosen parameters, to isolate reductions.

Turning reductions off one by one (hidden GURO_PAR_PRE* parameters are settable) gives a presolved model per stage
that the port must reproduce byte for byte. Same base settings as tools/parity.py (Threads=1, WorkLimit 60).
usage: python tools/presolve_oracle.py <out_dir> [--param Name=Value ...] [name ...]
       writes <out_dir>/<name>.mps (uncompressed, so `diff` works) and <out_dir>/stats.tsv (presolve log anchors).
Gurobi: one process at a time; retry while another session holds it.
"""
import argparse, re, sys, tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from parity import instances  # noqa: E402

ANCHORS = (r"Presolve removed (\d+) rows and (\d+) columns", r"Presolve changed (\d+) inequalities",
           r"Presolve tightened (\d+) bounds and modified (\d+)", r"Presolved: (\d+) rows, (\d+) columns, (\d+)")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("--param", action="append", default=[])
    ap.add_argument("names", nargs="*")
    a = ap.parse_args()
    import gurobipy as gp
    params = {k: float(v) if "." in v or "e" in v else int(v) for k, v in (p.split("=", 1) for p in a.param)}
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    insts = instances()
    names = a.names or [n for n, p in insts.items() if p.parent.name == "netlib"]
    with tempfile.TemporaryDirectory() as tmp, open(out / "stats.tsv", "w") as st:
        st.write("instance\t" + "\t".join(["removed", "ineq2eq", "tightened", "presolved"]) + "\n")
        for name in names:
            log = Path(tmp) / f"{name}.log"
            env = gp.Env(params={"LogToConsole": 0, "LogFile": str(log), "Threads": 1, "WorkLimit": 60, **params})
            m = gp.read(str(insts[name]), env)
            q = m.presolve()
            q.write(str(out / f"{name}.mps"))
            q.dispose(); m.dispose(); env.dispose()
            text = log.read_text()
            st.write(name + "".join("\t" + (",".join(x.groups()) if (x := re.search(r, text)) else "-") for r in ANCHORS)
                     + "\n")
    print(f"{len(names)} models -> {out}")


if __name__ == "__main__":
    main()
