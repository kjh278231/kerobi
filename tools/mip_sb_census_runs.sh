#!/bin/bash
# M2-06h-4: one exploratory capture per model (Gurobi serial), then the branch census.
# Usage: tools/mip_sb_census_runs.sh OUTDIR PARAMS_JSON [MODEL...]
set -uo pipefail
cd "$(dirname "$0")/.."
out=${1:?OUTDIR}; params=${2:?PARAMS}; shift 2
mkdir -p "$out"
models=("$@"); [ ${#models[@]} -eq 0 ] && models=($(ls ../re/bench/data/miplib | sed 's/\.mps\.gz$//'))
for m in "${models[@]}"; do
 [ -s "$out/$m.json.gz" ] && continue
 timeout 300 tools/gurun.sh node tools/mip_sb_loop.js ../re/bench/data/miplib/$m.mps.gz "$out/$m.json.gz" "$params" | tail -1
 echo "$m rc=${PIPESTATUS[0]}"
done
python3 tools/mip_sb_census.py "$out"/*.json.gz
