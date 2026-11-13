#!/bin/bash
# M1-02y baseline: three independent captures of tools/m1_eta_align.js per model (Gurobi one process at a time).
# usage: tools/m1_eta_align_runs.sh OUTDIR [model ...]   (default: peel stair sc50a scsd1 afiro, no peel kb2 degen2)
set -e
cd "$(dirname "$0")/.."
out=$1; [ -n "$out" ] || { echo "usage: $0 OUTDIR [model ...]"; exit 2; }
shift; models=${*:-stair sc50a scsd1 afiro kb2 degen2}
mkdir -p "$out"
for n in $models; do
  for i in 1 2 3; do
    [ -e "$out/${n}_$i.json.gz" ] || tools/gurun.sh node tools/m1_eta_align.js ../re/bench/data/netlib/$n.mps.gz "$out/${n}_$i.json.gz" 1 | tail -1
  done
done
echo RUNS_DONE
