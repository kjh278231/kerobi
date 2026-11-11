#!/bin/bash
# M2-06h-3/06h-4: unhooked control + three independent captures per model; Gurobi is always serial.
# Usage: tools/mip_sb_loop_runs.sh OUTDIR [MODEL...]  (default: the 06h-3 models)
set -euo pipefail
cd "$(dirname "$0")/.."
out=${1:?OUTDIR}; shift
models=("$@"); [ ${#models[@]} -eq 0 ] && models=(pk1 neos5 tr12-30)
params='{"NodeLimit":300,"GURO_PAR_SBMAXITERS":10}'
for m in "${models[@]}"; do
 tools/gurun.sh node tools/mip_sb_loop.js ../re/bench/data/miplib/$m.mps.gz "$out/${m}_control.json.gz" "$params" control
 for r in 1 2 3; do
  tools/gurun.sh node tools/mip_sb_loop.js ../re/bench/data/miplib/$m.mps.gz "$out/${m}_$r.json.gz" "$params"
 done
done
