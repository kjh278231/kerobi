"""Gurobi parameter table oracle: getParamInfo of every parameter gurobipy documents (_paramdetails) plus the names
given on stdin (one per line, e.g. the hidden GURO_PAR_* names of tools/param_table.py), as TSV
name, type, min, max, default (repr of the Python values). Names gurobipy rejects are skipped.
usage: grep -o 'name: "[^"]*"' crates/model/src/param_table.rs | cut -d'"' -f2 \\
         | tools/gurun.sh python tools/param_oracle.py > bench/gurobi_params.tsv   (Gurobi runs; one process only)
"""
import sys
import gurobipy as gp
from gurobipy._paramdetails import param_details

names = list(param_details) + [l.strip() for l in sys.stdin if l.strip() and l.strip() not in param_details]
env = gp.Env(params={"OutputFlag": 0})
m = gp.Model(env=env)
print("name\ttype\tmin\tmax\tdefault")
for n in names:
    try:
        info = m.getParamInfo(n)
    except gp.GurobiError:
        continue
    if info is None:  # unknown to gurobipy (it prints a warning instead of raising)
        continue
    name, typ, _cur, lo, hi, df = info
    print(f"{name}\t{typ.__name__}\t{lo!r}\t{hi!r}\t{df!r}")
