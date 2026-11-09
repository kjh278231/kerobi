"""M3 PDHG: Method=6, Presolve=0, Crossover=0 against Gurobi (bench/gurobi_pdhg_nopre, docs/m3/m3_pdhg.md). No Gurobi."""
import csv, gzip, json, os, subprocess, sys
from pathlib import Path

import rsolver

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT.parent / "re/bench/data/netlib"
C = rsolver.GRB.Callback


def test_pdhg_solves_64_lps_like_gurobi():
    """Status, objective (rel. err <= 1e-6) and PDHGIterCount on all 64, and the PDHG callback trace (every 100 PDHG
    iterations and after every power-iteration chunk) bit-identical to Gurobi's, WorkLimit 60 as in the baseline."""
    keys = (C.PDHG_ITRCNT, C.PDHG_PRIMOBJ, C.PDHG_DUALOBJ, C.PDHG_PRIMINF, C.PDHG_DUALINF, C.PDHG_COMPL)
    files = sorted((ROOT / "bench/gurobi_pdhg_nopre").glob("*/summary.json"))
    assert len(files) == 64
    for f in files:
        g = json.load(open(f))
        m = rsolver.read(DATA / f"{f.parent.name}.mps.gz")
        m.setParam("Method", 6)
        m.setParam("Presolve", 0)
        m.setParam("WorkLimit", 60)
        rows = []
        m.optimize(lambda model, where: rows.append(["pdhg", *(repr(model.cbGet(k)) for k in keys)]))
        assert m.Status == g["Status"], f.parent.name
        assert abs(m.ObjVal - g["ObjVal"]) <= 1e-6 * max(1.0, abs(g["ObjVal"])), f.parent.name
        assert m.PDHGIterCount == g["PDHGIterCount"], f.parent.name
        assert rows == list(csv.reader(gzip.open(f.parent / "trace.csv.gz", "rt"))), f.parent.name


def test_pdhg_state_matches_oracle_afiro(tmp_path):
    """Every P field and iterate vector the oracle reads (tools/pdhg_oracle.py --vec) is bit-equal at every callback."""
    dump = tmp_path / "afiro.jsonl"
    os.environ["PDHGDUMP"] = str(dump)
    try:
        m = rsolver.read(DATA / "afiro.mps.gz")
        m.setParam("Method", 6)
        m.setParam("Presolve", 0)
        m.optimize()
    finally:
        del os.environ["PDHGDUMP"]
    r = subprocess.run([sys.executable, ROOT / "tools/pdhg_cmp.py", ROOT / "bench/oracle/pdhg/afiro.jsonl.gz", dump],
                       capture_output=True, text=True)
    assert r.returncode == 0, r.stdout
    assert "7 common callbacks, 0 with differences" in r.stdout
