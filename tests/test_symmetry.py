"""M1 LP presolve, pass-0 symmetry detection (`5577ba`/`551cc0`, docs/m1_presolve_symmetry.md). sym_k5 is five copies
of one block with a dependent `=` row each (tools/symmetry_models.py); Gurobi finds the copies symmetric (local_f4 1),
so automatic PreDepRow is off and the rows R*_11 stay. With GURO_PAR_PRESYMMPASSES=0 there is no detection and
`47c445` drops them. Expected models: Model.presolve() written by tools/symmetry_oracle.py --presolved. The second
test pins local_f4 and the work `5577ba` adds (tools/symmetry_check.py's bitwise check from the oracle's work before
the call) on three paths: verified (sym_k5), rejected twice and given up after two retries (sym_cube8), node limit
(sym_cycles)."""
import struct
import sys
from pathlib import Path

import pytest
import rsolver

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
from symmetry_models import EXTRA, MODELS, block_lp  # noqa: E402


@pytest.mark.parametrize("params,want", [([], "sym_k5.mps"), ([("GURO_PAR_PRESYMMPASSES", 0.0)], "sym_k5_presymm0.mps")])
def test_symmetry_keeps_dependent_rows(tmp_path, params, want):
    src = tmp_path / "sym_k5.mps"
    src.write_text(block_lp("sym_k5", *MODELS["sym_k5"]))
    got = rsolver.read(src)._core.presolve(params).to_mps()
    assert got == (ROOT / "bench/gurobi_presolve" / want).read_text()


# name: (oracle work before the call, after, local_f4), from tools/symmetry_oracle.py
CALLS = {
    "sym_k5": ("0x1.15d5ed9b2d037p+19", "0x1.4e028d9b2d037p+19", 1),
    "sym_cube8": ("0x1.f2297c8ad9157p+18", "0x1.209b5e456c8acp+19", -2),
    "sym_cycles": ("0x1.4a96a99688266p+18", "0x1.a3d8299688266p+18", -1),
}


@pytest.mark.parametrize("name", CALLS)
def test_symmetry_result_and_work(tmp_path, capfd, monkeypatch, name):
    before, after, f4 = CALLS[name]
    src = tmp_path / f"{name}.mps"
    src.write_text(block_lp(name, *MODELS[name]) if name in MODELS else EXTRA[name]())
    monkeypatch.setenv("RSOLVER_SYM_TRACE", "1")
    monkeypatch.setenv("RSOLVER_SYM_BASE", struct.pack(">d", float.fromhex(before)).hex())
    rsolver.read(src)._core.presolve([])
    calls = [line.split() for line in capfd.readouterr().err.splitlines() if line.startswith("5577ba ")]
    assert len(calls) == 1
    assert int(calls[0][5]) == f4
    assert struct.unpack(">d", bytes.fromhex(calls[0][6]))[0].hex() == float.fromhex(after).hex()
