"""G5 (2): the optimize header ("Optimize a model with …", fingerprint, "Model has …", "Variable types …",
coefficient statistics and warnings) equals Gurobi's log.txt in bench/gurobi for Netlib 64 + miplib_small 38.
No Gurobi is needed."""
import re
from pathlib import Path

import pytest
import rsolver

ROOT = Path(__file__).resolve().parents[1]
LOGS = sorted((ROOT / "bench/gurobi").glob("*/log.txt"))


@pytest.mark.parametrize("log", LOGS, ids=[p.parent.name for p in LOGS])
def test_header_matches_gurobi(log):
    text = log.read_text()
    path = re.search(r"^Read MPS format model from file (.*)$", text, re.M).group(1)
    want = re.search(r"^(Optimize a model with .*?\n)\n", text, re.M | re.S).group(1)
    assert rsolver.read(path)._core.header(0) == want
