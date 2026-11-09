# M2 outer B branch fixtures

26 native Gurobi state pairs for dual aggregation, protected integer pairs, stage C pair aggregation and
small/large-row dominated columns. Each `.lp` has a matching `.json.gz` (entry and return states).

Capture serially through the shared Gurobi lock:

```sh
bash tools/build_local.sh
PYTHONPATH=$PWD/python ../.venv/bin/python tools/outer_b_oracle.py --capture
```

Offline validation (no Gurobi):

```sh
PYTHONPATH=$PWD/python ../.venv/bin/python -m pytest -q tests/test_outer_b.py
```

Parameters, expected removal counts, and injected protected-column flags live in `tools/outer_b_oracle.py`.
Direct calls occur after the initial activity recompute so earlier reductions cannot remove the patterns.
Injected flags isolate the aggregation branch; they do not represent a full SOS/semicontinuous presolve run.
See `docs/m2_red_outer_b.md` sections 3, 6–8 for branch coverage and limitations.
