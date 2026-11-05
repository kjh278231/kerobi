"""G5 (4): optimize() with presolve (Presolve -1, Threads 1, WorkLimit 60) through the Rust driver against Gurobi
captures of tools/parity.py: status, objective (relative 1e-6) and BarIterCount / PDHGIterCount on Netlib 64.
Method 1 checks status and objective only. Also the log anchors of each path. No Gurobi is needed."""
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


def anchors(text):
    """Log lines from "Optimize a model" to the end of the solve, numbers that vary run to run masked."""
    body = re.split(r"\nUser-callback calls|\nPresolve a model", text[text.index("Optimize a model"):])[0]
    keep = re.compile(r"^(Optimize|Model|Coefficient|  \w+ range|Presolve|Presolved|Start PDHG|Solved in|"
                      r"Barrier solved|PDHG solved|PDHG performed|Optimal objective|Work limit)")
    # simplex iteration counts are G1's (Method 1 checks status and objective only)
    mask = lambda s: re.sub(r"^Solved in \d+", "Solved in N",
                            re.sub(r"[0-9.]+ seconds \([0-9.]+ work units\)", "T", re.sub(r"time: [0-9.]+s", "time: T", s)))
    return [mask(s) for s in body.splitlines() if keep.match(s)]


@pytest.mark.parametrize("cap,params,iters", PATHS, ids=[p[0] for p in PATHS])
def test_driver_log_anchors(cap, params, iters, tmp_path):
    for name in ("afiro", "grow7"):  # grow7: presolve removes nothing (no "Presolve removed" line)
        solve(name, params, tmp_path / f"{name}.log")
        assert anchors((tmp_path / f"{name}.log").read_text()) == anchors((ROOT / "bench" / cap / name / "log.txt").read_text())
