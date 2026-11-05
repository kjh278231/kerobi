"""M0: MPS reading and fingerprint parity with Gurobi (values from bench/gurobi)."""
import json
from pathlib import Path

import rsolver

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT.parent / "re/bench/data"


def test_fingerprint_matches_gurobi_baseline():
    for name, sub in (("afiro", "netlib"), ("forplan", "netlib"), ("boeing1", "netlib"), ("neos17", "miplib")):
        m = rsolver.read(DATA / sub / f"{name}.mps.gz")
        assert m.Fingerprint == json.load(open(ROOT / "bench/gurobi" / name / "summary.json"))["Fingerprint"], name


def test_ranged_rows_become_slack_columns():
    m = rsolver.read(DATA / "netlib/boeing1.mps.gz")
    assert (m.NumVars, m.NumConstrs) == (473, 351)
    assert m._core.var_names[384] == "MPS_RgDMBOSHNL"
