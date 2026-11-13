#!/bin/bash
# M2-06h-6: VarBranch=3 (no-propagation) captures: unhooked control + three independent runs per model,
# plus one default-setting pk1 run for the non-interference check. Gurobi is always serial.
# Usage: tools/mip_sb_direct_runs.sh OUTDIR [MODEL...]
set -euo pipefail
cd "$(dirname "$0")/.."
out=${1:?OUTDIR}; shift
models=("$@"); [ ${#models[@]} -eq 0 ] && models=(markshare_4_0 pk1 neos5 neos-3627168-kasai)
params='{"NodeLimit":20,"GURO_PAR_SBMAXITERS":10,"VarBranch":3}'
for m in "${models[@]}"; do
 tools/gurun.sh node tools/mip_sb_direct.js ../re/bench/data/miplib/$m.mps.gz "$out/${m}_control.json.gz" "$params" control
 for r in 1 2 3; do
  tools/gurun.sh node tools/mip_sb_direct.js ../re/bench/data/miplib/$m.mps.gz "$out/${m}_$r.json.gz" "$params"
 done
done
[ -e "$out/pk1_default.json.gz" ] || tools/gurun.sh node tools/mip_sb_direct.js ../re/bench/data/miplib/pk1.mps.gz "$out/pk1_default.json.gz" '{"NodeLimit":300,"GURO_PAR_SBMAXITERS":10}'
