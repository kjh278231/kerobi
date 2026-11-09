"""G5 (3): status rules and the log of a presolve that proves infeasibility or unboundedness (`390989` end, `0c3d0d`,
`36578f`), Method 1/2/6 (Crossover 0), against Gurobi logs in bench/gurobi_status (small hand-made LPs). No Gurobi."""
import re
from pathlib import Path

import pytest
import rsolver

FIX = Path(__file__).resolve().parents[1] / "bench/gurobi_status"
CASES = [(f, m) for f in ("inf_pre", "inf_eng", "unb_pre", "unb_eng") for m in (1, 2, 6)]
STATUS = {"Infeasible model": 3, "Model is infeasible": 3, "Infeasible or unbounded model": 4,
          "Model is infeasible or unbounded": 4, "Unbounded model": 5, "Model is unbounded": 5}


def tail(text):
    """Lines from the presolve result to the status line, times masked."""
    body = re.split(r"\nUser-callback calls", text[text.index("Optimize a model"):])[0]
    lines = [l for l in body.splitlines() if l.startswith(("Presolve", "Solved in", "Barrier solved", "PDHG solved"))
             or l in STATUS]
    return [re.sub(r"[0-9.]+ seconds \([0-9.]+ work units\)", "T", re.sub(r"time: [0-9.]+s", "time: T", l)) for l in lines]


@pytest.mark.parametrize("name,method", CASES)
def test_proved_status(name, method, tmp_path):
    want = (FIX / f"{name}.m{method}.log").read_text()
    m = rsolver.read(FIX / f"{name}.mps")
    for k, v in {"Method": method, "Crossover": 0, "Threads": 1, "OutputFlag": 1, "LogToConsole": 0,
                 "LogFile": str(tmp_path / "log")}.items():
        m.setParam(k, v)
    m.optimize()
    assert m.Status == STATUS[next(l for l in tail(want) if l in STATUS)]
    assert tail((tmp_path / "log").read_text()) == tail(want)
