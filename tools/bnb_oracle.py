#!/usr/bin/env python3
"""Compare deterministic small mixed-integer models with Gurobi (requires gurobipy)."""
import random
import tempfile
from pathlib import Path

import gurobipy as gp
from rsolver import _core


def main():
    rng = random.Random(112026)
    mismatches = []
    with tempfile.TemporaryDirectory() as tmp:
        path = str(Path(tmp) / 'case.mps')
        for case in range(200):
            with gp.Model() as m:
                m.Params.OutputFlag = 0
                m.Params.Presolve = 0
                m.Params.Heuristics = 0
                m.Params.MIPGap = 0
                m.Params.MIPGapAbs = 0
                kinds = ['B', 'B', 'I', 'C']
                xs = [m.addVar(lb=0, ub=(1 if t == 'B' else 3), vtype=t) for t in kinds]
                coeff = [rng.randrange(-4, 5) for _ in xs]
                m.setObjective(gp.quicksum(c * x for c, x in zip(coeff, xs)), 1 if case % 2 == 0 else -1)
                for _ in range(rng.randrange(1, 6)):
                    a = [rng.randrange(-3, 4) for _ in xs]
                    rhs = rng.randrange(-4, 9)
                    m.addConstr(gp.quicksum(c * x for c, x in zip(a, xs)) <= rhs)
                m.update()
                m.write(path)
                m.optimize()
                result = _core.read_model(path).mip_solve([('Heuristics', 0), ('MIPGap', 0), ('MIPGapAbs', 0)])
                expected = m.ObjVal if m.SolCount else None
                if result[0] != m.Status or (expected is not None and (result[2] is None or abs(result[2] - expected) > 1e-5)):
                    mismatches.append((case, m.Status, expected, result[:4], Path(path).read_text()))
        if mismatches:
            for mismatch in mismatches[:5]:
                print(mismatch)
            raise SystemExit(f'{len(mismatches)} mismatches')
        for cutoff in (-2.0, -1.0, 0.0):
            with gp.Model() as m:
                m.Params.OutputFlag = 0
                m.Params.Heuristics = 0
                m.Params.Cutoff = cutoff
                x = m.addVars(3, vtype='B')
                m.setObjective(-gp.quicksum(x.values()))
                for i, j in ((0, 1), (1, 2), (0, 2)):
                    m.addConstr(x[i] + x[j] <= 1)
                m.optimize()
                rows = [([(i, 1.0), (j, 1.0)], -1e30, 1.0) for i, j in ((0, 1), (1, 2), (0, 2))]
                core = _core.build_model([-1.0] * 3, rows, [0.0] * 3, [1.0] * 3, 'BBB')
                result = core.mip_solve([('Heuristics', 0), ('GURO_PAR_OBJGCD', 0), ('Cutoff', cutoff)])
                assert result[0] == m.Status and result[2] == (m.ObjVal if m.SolCount else None), (cutoff, result, m.Status)
        print('200 mixed-integer models and 3 cutoff cases matched Gurobi status and objective')


if __name__ == '__main__':
    main()
