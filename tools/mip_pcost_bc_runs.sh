#!/bin/bash
# M2-06e-2 captures (docs/m2/mip_pseudocost_bc.md §5): the 06e cases x 3 independent runs of tools/mip_pcost_bc.js.
# One Gurobi process at a time. Usage: tools/mip_pcost_bc_runs.sh OUTDIR
set -euo pipefail
out=${1:?OUTDIR}; cd "$(dirname "$0")/.."
for r in 1 2 3; do
  for m in markshare_4_0 pk1 neos5 tr12-30 fhnw-binpack4-4 neos-860300; do
    tools/gurun.sh node tools/mip_pcost_bc.js ../re/bench/data/miplib/$m.mps.gz "$out/${m}_$r.json.gz" '{"NodeLimit":2000}'
  done
done
echo RUNS-DONE
