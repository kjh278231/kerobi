#!/bin/bash
# M2-06g-2: unhooked controls and independent captures; Gurobi is always serial.
set -euo pipefail
out=${1:?OUTDIR}; cd "$(dirname "$0")/.."
for kind in default iters10; do
  params='{"NodeLimit":300}'
  if [ "$kind" = iters10 ]; then params='{"NodeLimit":300,"GURO_PAR_SBMAXITERS":10}'; fi
  for m in pk1 neos5 tr12-30; do
    tools/gurun.sh node tools/mip_sb_restore.js ../re/bench/data/miplib/$m.mps.gz "$out/${m}_${kind}_control.json.gz" "$params" control
    for r in 1 2 3; do
      tools/gurun.sh node tools/mip_sb_restore.js ../re/bench/data/miplib/$m.mps.gz "$out/${m}_${kind}_$r.json.gz" "$params"
    done
  done
done
