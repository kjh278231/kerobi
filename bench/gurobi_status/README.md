# LP status oracle fixtures

The existing `*.mps` / `*.mN.log` files cover presolve-proved infeasibility and unboundedness.

`uncrush_violation.tsv` records controlled fault injection into Gurobi 13.0.3, not ordinary solver solutions.
For Netlib `afiro`, Threads=1, Presolve=-1, Crossover=0, `tools/uncrush_oracle.py` adds `Delta` to original
column 0 immediately after `365159` returns. Gurobi then performs its own violation check and status change.
The six rows use Method 2/6 and Delta 0.005/0.02/3.0. Reproduce one row with:

```bash
tools/gurun.sh ../.venv/bin/python tools/uncrush_oracle.py \
  ../re/bench/data/netlib/afiro.mps.gz /tmp/afiro-uncrush.json \
  Method=2 Crossover=0 --uncrush-delta 0=0.02
```

Each JSON records the original X, the injected delta, final X/status/MaxVio, the presolved result's violation
and its scale arrays. The TSV columns are Method, Delta, final Status, PresolvedViolation (`result+0x58`),
MinScale (minimum of the presolved column and row scales), and Violation (Gurobi `MaxVio`). FeasibilityTol is
1e-6. `crates/driver` tests its status-13 predicate against these results without loading Gurobi.

Method 2 yields statuses 2/13/13. Method 6 yields 2/2/13: its presolved violation is about 2.36e-4, so a
violation near 0.0212 still stays below its status-change threshold. Natural status-13 examples have not been
found in the Netlib baseline.
