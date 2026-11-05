"""M1 LP presolve: rsolver's presolved model is Gurobi's, line for line (Model.presolve() written as MPS by
tools/presolve_oracle.py into bench/gurobi_presolve, default parameters). Covers the inner loop, aggregation
(519a52) and the finish stage's implied bounds (50de61)."""
from pathlib import Path

import pytest
import rsolver

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT.parent / "re/bench/data"


# degen2: 47c445 (PreDepRow) drops the dependent `=` rows CR2007A and CR2027B
@pytest.mark.parametrize("name", ["afiro", "sc50a", "beaconfd", "vtpbase", "kb2", "degen2"])
def test_presolved_model_matches_gurobi(name):
    got = rsolver.read(DATA / "netlib" / f"{name}.mps.gz")._core.presolve([]).to_mps()
    assert got == (ROOT / "bench/gurobi_presolve" / f"{name}.mps").read_text()


def test_derived_bounds_checkpoint():
    """`5b488d` right after it first acts on finnis (call 39): `=` row 3CPTRT8 and column 3RT8INV leave."""
    got = rsolver.read(DATA / "netlib" / "finnis.mps.gz")._core.presolve([("GURO_PAR_PRELIMIT", 39.0)]).to_mps()
    assert got == (ROOT / "bench/gurobi_presolve" / "finnis_lim39.mps").read_text()
