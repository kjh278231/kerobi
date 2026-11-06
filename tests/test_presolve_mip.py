"""M2 MIP presolve skeleton (crates/mip): state, prelude, finishing and model generation against Gurobi's
Model.presolve() (tools/presolve_oracle.py) in bench/gurobi_presolve_mip, plus the MPS writer's integer sections."""
from pathlib import Path

import pytest
import rsolver

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT.parent / "re/bench/data"
FIX = ROOT / "bench/gurobi_presolve_mip"


def test_writer_integer_sections():
    """INTORG/INTEND markers and LI bounds as Gurobi writes the original model (a4590c)."""
    got = rsolver.read(DATA / "miplib/gen-ip054.mps.gz")._core.to_mps()
    assert got == (FIX / "gen-ip054_orig.mps").read_text()


# neos859080: empty rows dropped (36d89a), [0,1] integers become binaries; timtab1: rhs -2.8e-14 rounded to 0 at
# finishing; syn_round: bound rounding, binary clipping, a near-integral fixed integer becoming continuous, free row
@pytest.mark.parametrize("src,params,want", [
    (DATA / "miplib/neos859080.mps.gz", [("Presolve", 0.0)], "neos859080_pre0.mps"),
    (DATA / "miplib/timtab1.mps.gz", [("GURO_PAR_PRELIMIT", 2.0)], "timtab1_lim2.mps"),
    (FIX / "syn_round.mps", [("Presolve", 0.0)], "syn_round_pre0.mps"),
    # syn_dfix: 57999f drops x's tiny coefficient, the final dual fixing fixes the now empty x at its lower bound
    (FIX / "syn_dfix.mps", [], "syn_dfix_pre.mps"),
])
def test_presolved_model_matches_gurobi(src, params, want):
    assert rsolver.read(src)._core.presolve(params).to_mps() == (FIX / want).read_text()


def test_infeasible_integer_fixing():
    """An integer column fixed at 2.5: Gurobi's Model.presolve() fails with errno -1."""
    with pytest.raises(rsolver.GurobiError, match="Unable to create presolved model"):
        rsolver.read(FIX / "syn_infeas.mps").presolve()
