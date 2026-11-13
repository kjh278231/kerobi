#!/usr/bin/env python3
"""M2-06h-7: small synthetic MIP models whose SB candidate list can hold SOS continuous members.

Each group g has K continuous x[g,k] >= 0 in one SOS1 (or SOS2) and must meet demand d[g]; column k has
capacity slack*sum(d)/K*y[k] with binary y[k] (open cost). The tight variants (slack 1.03-1.05) are bin packing like,
so the LP relaxation splits groups
over several columns and violates the SOS sets (695bd7 candidate group 2, 5d4427 D:442-520).
Usage: run under tools/gurun.sh with the shared venv python: mip_sos_models.py OUTDIR
"""
import random
import sys

import gurobipy as gp


def build(env, name, groups, k, sos_type, seed, slack):
    rnd = random.Random(seed)
    m = gp.Model(name, env=env)
    d = [rnd.randint(10, 50) for _ in range(groups)]
    cap = [int(slack * sum(d) / k) for _ in range(k)]
    y = [m.addVar(vtype='B', obj=rnd.randint(200, 400), name=f'y{j}') for j in range(k)]
    x = [[m.addVar(obj=rnd.randint(1, 20), name=f'x{g}_{j}') for j in range(k)] for g in range(groups)]
    for g in range(groups):
        m.addConstr(gp.quicksum(x[g]) == d[g], name=f'd{g}')
        m.addSOS(gp.GRB.SOS_TYPE1 if sos_type == 1 else gp.GRB.SOS_TYPE2, x[g], list(range(1, k + 1)))
    for j in range(k):
        m.addConstr(gp.quicksum(x[g][j] for g in range(groups)) <= cap[j] * y[j], name=f'c{j}')
    return m


if __name__ == '__main__':
    out = sys.argv[1]
    env = gp.Env(params={'LogToConsole': 0})
    for name, groups, k, t, seed, slack in [('sos1_g30k4', 30, 4, 1, 1, 1.6), ('sos1_g60k6', 60, 6, 1, 2, 1.6), ('sos2_g30k5', 30, 5, 2, 3, 1.6),
                                               ('sos1_g40k5t', 40, 5, 1, 4, 1.05), ('sos1_g60k6t', 60, 6, 1, 5, 1.03), ('sos2_g40k5t', 40, 5, 2, 6, 1.05)]:
        m = build(env, name, groups, k, t, seed, slack)
        m.write(f'{out}/{name}.mps')
        print(name, m.NumVars, m.NumConstrs, m.NumSOS)
