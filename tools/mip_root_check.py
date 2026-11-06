"""MIP root LP check (docs/m2_root.md): presolve each miplib_small model with rsolver, solve the root LP
(crates/mip root.rs) and compare with Gurobi's "Root relaxation: objective %.6e, %.0f iterations" line in
bench/gurobi/<name>/log.txt (default parameters, Threads=1). Needs no Gurobi.
usage: PYTHONPATH=python python tools/mip_root_check.py [name ...]
"""
import csv, re, sys
from pathlib import Path

from rsolver import _core

ROOT = Path(__file__).resolve().parent.parent
DATA = ROOT.parent / "re/bench/data"


def main():
    names = sys.argv[1:] or [r["name"] for r in csv.DictReader(open(DATA / "miplib_small.csv"))]
    n_obj = n_it = n_both = 0
    for n in names:
        ref = re.search(r"Root relaxation: objective (\S+), (\d+) iterations", (ROOT / f"bench/gurobi/{n}/log.txt").read_text())
        p = _core.read_model(str(DATA / f"miplib/{n}.mps.gz")).presolve([("Threads", 1.0)])
        r = p.mip_root([("Threads", 1.0)])
        if ref is None or r is None:
            ok = ref is None and r is None
            n_obj += ok; n_it += ok; n_both += ok
            print(f"{n:22s} no root: gurobi {ref is None}, rsolver {r is None}  {'OK' if ok else 'DIFF'}")
            continue
        method, st, obj, it = r
        o, i = f"{obj:.6e}" == ref[1], int(it) == int(ref[2])
        n_obj += o; n_it += i; n_both += o and i
        print(f"{n:22s} gurobi {ref[1]:>13s} {ref[2]:>5s} | rsolver m{method} st{st} {obj:.6e} {int(it):5d}  "
              f"obj {'OK' if o else 'DIFF'} iter {'OK' if i else 'DIFF'}")
    print(f"objective {n_obj}/{len(names)}, iterations {n_it}/{len(names)}, both {n_both}/{len(names)}")


if __name__ == "__main__":
    main()
