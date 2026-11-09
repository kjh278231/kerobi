"""G5 (4): optimize() with presolve (Presolve -1, Threads 1, WorkLimit 60) through the Rust driver against Gurobi
captures of tools/parity.py: status, objective (relative 1e-6) and BarIterCount / PDHGIterCount on Netlib 64.
Method 1 checks status and objective only. Also the log anchors of each path. No Gurobi is needed."""
import csv
import gzip
import json
import re
from pathlib import Path

import pytest
import rsolver

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT.parent / "re/bench/data/netlib"
PATHS = [("gurobi_bar_pre", {"Method": 2, "Crossover": 0}, "BarIterCount"),
         ("gurobi_pdhg_pre", {"Method": 6, "Crossover": 0}, "PDHGIterCount"),
         ("gurobi_dual_pre", {"Method": 1}, None)]


def solve(name, params, log=None):
    m = rsolver.read(DATA / f"{name}.mps.gz")
    for k, v in {"OutputFlag": 0, "Threads": 1, "WorkLimit": 60, **params}.items():
        m.setParam(k, v)
    if log:
        m.setParam("LogFile", str(log))
    m.optimize()
    return m


@pytest.mark.parametrize("cap,params,iters", PATHS, ids=[p[0] for p in PATHS])
def test_driver_matches_gurobi(cap, params, iters):
    bad = []
    for d in sorted((ROOT / "bench" / cap).iterdir()):
        ref = json.loads((d / "summary.json").read_text())
        m = solve(d.name, params)
        ok = m.Status == ref["Status"] and abs(m._objval - ref["ObjVal"]) <= 1e-6 * max(1.0, abs(ref["ObjVal"]))
        if iters:
            ok = ok and getattr(m, iters) == ref[iters]
        if not ok:
            bad.append((d.name, m.Status, m._objval, iters and getattr(m, iters)))
    assert not bad


def body(text, simplex):
    """The log from "Optimize a model" to the end of the solve, run-dependent fields masked (seconds, work units, the
    time column); for the simplex path its iteration rows and counts too (simplex internals are G1's)."""
    t = re.split(r"\nUser-callback calls|\nPresolve a model", text[text.index("Optimize a model"):])[0].rstrip("\n")
    out = []
    for line in t.splitlines():
        line = re.sub(r"[0-9.]+ seconds \([0-9.]+ work units\)", "T", re.sub(r"time: [0-9.]+s", "time: T", line))
        line = re.sub(r" +\d+s$", " Ts", line)
        if simplex:
            if re.match(r"^ +\d+ +[-+0-9.e]+ +[-+0-9.e]+ +[-+0-9.e]+ +Ts$", line):
                continue
            line = re.sub(r"^Solved in \d+", "Solved in N", line)
        out.append(line)
    return out


@pytest.mark.parametrize("cap,params,iters", PATHS, ids=[p[0] for p in PATHS])
def test_driver_log(cap, params, iters, tmp_path):
    """Whole log (header, presolve, barrier statistics, iteration tables, summary) line for line."""
    for name in ("afiro", "grow7"):  # grow7: presolve removes nothing (no "Presolve removed" line)
        solve(name, params, tmp_path / f"{name}.log")
        got = body((tmp_path / f"{name}.log").read_text(), iters is None)
        assert got == body((ROOT / "bench" / cap / name / "log.txt").read_text(), iters is None)


@pytest.mark.parametrize("name", ["degen3", "forplan"])
def test_barrier_common_coefficient_trace(name):
    """Common-coefficient columns need an unpeeled solve product and a two-accumulator residual product.
    Both models previously diverged immediately after iteration zero; compare every callback against Gurobi.
    """
    c = rsolver.GRB.Callback
    keys = (c.BARRIER_ITRCNT, c.BARRIER_PRIMOBJ, c.BARRIER_DUALOBJ, c.BARRIER_PRIMINF, c.BARRIER_DUALINF, c.BARRIER_COMPL)
    m = rsolver.read(DATA / f"{name}.mps.gz")
    for k, v in {"OutputFlag": 0, "Threads": 1, "Method": 2, "Crossover": 0, "Presolve": -1}.items():
        m.setParam(k, v)
    rows = []
    m.optimize(lambda model, where: rows.append([float(model.cbGet(k)).hex() for k in keys]))
    with gzip.open(ROOT / "bench/gurobi_bar_pre" / name / "trace.csv.gz", "rt") as f:
        want = [[float(v).hex() for v in row[1:]] for row in csv.reader(f) if row[0] == "barrier"]
    assert rows == want
