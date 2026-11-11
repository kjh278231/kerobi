#!/bin/bash
# M2-06c-3 captures (docs/m2/mip_plunge_r47.md §5): 7 cases x 3 independent runs with tools/mip_plunge_score.js, plus one run of
# the pinned 06c tool for each BranchDir case (projection control). One Gurobi process at a time.
# Usage: tools/mip_plunge_score_runs.sh OUTDIR
set -euo pipefail
out=${1:?OUTDIR}; cd "$(dirname "$0")/.."
M=../re/bench/data/miplib
T=tools/mip_plunge_score.js
for r in 1 2 3; do
  for m in markshare_4_0 pk1 neos5 tr12-30; do
    tools/gurun.sh node $T $M/$m.mps.gz "$out/${m}_$r.json.gz" '{"NodeLimit":2000}'
  done
  tools/gurun.sh node $T $M/pk1.mps.gz "$out/pk1-pool_$r.json.gz" '{"NodeLimit":2000,"PoolSearchMode":2}'
  tools/gurun.sh node $T $M/pk1.mps.gz "$out/pk1-bd1_$r.json.gz" '{"NodeLimit":2000,"BranchDir":1}'
  tools/gurun.sh node $T $M/pk1.mps.gz "$out/pk1-bd-1_$r.json.gz" '{"NodeLimit":2000,"BranchDir":-1}'
done
mkdir -p "$out/control"
tools/gurun.sh node tools/mip_plunge_rule.js $M/pk1.mps.gz "$out/control/pk1-bd1_06c.json.gz" '{"NodeLimit":2000,"BranchDir":1}'
tools/gurun.sh node tools/mip_plunge_rule.js $M/pk1.mps.gz "$out/control/pk1-bd-1_06c.json.gz" '{"NodeLimit":2000,"BranchDir":-1}'
echo RUNS-DONE
