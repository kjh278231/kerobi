"""M1 LP presolve, pass-0 symmetry detection (`5577ba`/`551cc0`, docs/m1_presolve_symmetry.md). sym_k5 is five copies
of one block with a dependent `=` row each (tools/symmetry_models.py); Gurobi finds the copies symmetric (local_f4 1),
so automatic PreDepRow is off and the rows R*_11 stay. With GURO_PAR_PRESYMMPASSES=0 there is no detection and
`47c445` drops them. Expected models: Model.presolve() written by tools/symmetry_oracle.py --presolved."""
import sys
from pathlib import Path

import pytest
import rsolver

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
from symmetry_models import MODELS, block_lp  # noqa: E402


@pytest.mark.parametrize("params,want", [([], "sym_k5.mps"), ([("GURO_PAR_PRESYMMPASSES", 0.0)], "sym_k5_presymm0.mps")])
def test_symmetry_keeps_dependent_rows(tmp_path, params, want):
    src = tmp_path / "sym_k5.mps"
    src.write_text(block_lp("sym_k5", *MODELS["sym_k5"]))
    got = rsolver.read(src)._core.presolve(params).to_mps()
    assert got == (ROOT / "bench/gurobi_presolve" / want).read_text()
