#!/bin/bash
# M1-02x baseline: three independent captures of tools/m1_lt_path.js per model (Gurobi one process at a time).
# usage: tools/m1_lt_runs.sh OUTDIR [model ...]   (default: the m >= 250 models of M1-02k)
set -e
cd "$(dirname "$0")/.."
out=$1; [ -n "$out" ] || { echo "usage: $0 OUTDIR [model ...]"; exit 2; }
shift; models=${*:-stair degen2 fit1p bnl1}
mkdir -p "$out"
for n in $models; do
  for i in 1 2 3; do
    [ -e "$out/${n}_$i.json.gz" ] || tools/gurun.sh node tools/m1_lt_path.js ../re/bench/data/netlib/$n.mps.gz "$out/${n}_$i.json.gz" 1 | tail -1
  done
done
echo RUNS_DONE
