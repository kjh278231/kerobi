#!/bin/bash
# M2-06c baseline captures (docs/m2/mip_plunge_rule.md §5): 5 cases x 3 independent runs, one Gurobi process at a time.
# Usage: tools/mip_plunge_rule_runs.sh OUTDIR
set -euo pipefail
out=${1:?OUTDIR}; cd "$(dirname "$0")/.."
M=../re/bench/data/miplib
for r in 1 2 3; do
  for m in markshare_4_0 pk1 neos5 tr12-30; do
    tools/gurun.sh node tools/mip_plunge_rule.js $M/$m.mps.gz "$out/${m}_$r.json.gz" '{"NodeLimit":2000}'
  done
  tools/gurun.sh node tools/mip_plunge_rule.js $M/pk1.mps.gz "$out/pk1-pool_$r.json.gz" '{"NodeLimit":2000,"PoolSearchMode":2}'
done
echo RUNS-DONE
