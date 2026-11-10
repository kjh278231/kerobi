"""Solve actual reduced models with Gurobi and validate rsolver's restored incumbents."""
import math
import tempfile
from pathlib import Path

import gurobipy as gp
from rsolver import _core
from mip_progress_check import validate

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT.parent / 're/bench/data/miplib'


def main():
    for name in ('neos-860300', 'graphdraw-domain'):
        model = _core.read_model(str(DATA / f'{name}.mps.gz'))
        _, (status, reduced, _, _) = model.mip_presolve_root([('Threads', 1.0)])
        assert status == 0 and reduced is not None
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'reduced.mps'
            path.write_text(reduced.to_mps())
            native = gp.read(str(path))
            native.Params.OutputFlag = 0
            native.Params.Threads = 1
            native.Params.TimeLimit = 30
            native.optimize()
            if not native.SolCount:
                print(name, 'no reduced incumbent', 'status', native.Status, flush=True)
                continue
            assert native.NumVars == len(reduced.obj)
            x, objective = model.mip_restore_candidate([('Threads', 1.0)], native.X)
            validate(model, objective, x)
            assert math.isclose(objective, native.ObjVal, rel_tol=1e-6, abs_tol=1e-6), (objective, native.ObjVal)
            print(name, 'validated', 'objective', objective, 'gurobi_status', native.Status, flush=True)


if __name__ == '__main__':
    main()
