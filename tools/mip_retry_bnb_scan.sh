#!/bin/bash
# M2-03d-5 natural retry scan; one Gurobi process at a time. Existing outputs are kept (resume after interruption).
# Usage: tools/mip_retry_bnb_scan.sh controls|matrix OUTDIR
#        tools/mip_retry_bnb_scan.sh repro OUTDIR MODEL CFG SEED NODELIMIT   (3 runs, no TimeLimit)
set -u
mode=${1:?mode}; out=${2:?OUTDIR}; mkdir -p "$out"
stress='"Presolve":0,"ScaleFlag":0,"Quad":0,"NumericFocus":0,"MarkowitzTol":0.0001'
declare -A cfg=(
 [H]="$stress,\"Heuristics\":0,\"RINS\":0"
 [P]="$stress,\"Heuristics\":0,\"RINS\":0,\"Cuts\":0"
)
run() { [ -s "$2" ] && { echo "skip $2"; return; }
 ./tools/gurun.sh timeout "$4" node tools/mip_retry_bnb_scan.js "$1" "$2" "$3" | cut -c1-300
 [ "${PIPESTATUS[0]}" = 0 ] || echo "FAIL $2"; }
lib=../re/bench/data/miplib
case $mode in
controls)
 run $lib/neos-3754480-nidda.mps.gz "$out/nidda_stress.json" "{\"TimeLimit\":20,\"NodeLimit\":20000,$stress}" 120s
 run $lib/gmu-35-50.mps.gz "$out/gmu_default.json" '{"TimeLimit":20,"NodeLimit":20000}' 120s ;;
matrix)
 for c in H P; do for s in 0 1 2; do for f in $lib/*.mps.gz; do
  run "$f" "$out/$c/s$s/$(basename "$f" .mps.gz).json" "{\"TimeLimit\":20,\"NodeLimit\":50000,\"Seed\":$s,${cfg[$c]}}" 120s
 done; done; done ;;
repro)
 m=${3:?MODEL}; c=${4:?CFG}; s=${5:?SEED}; n=${6:?NODELIMIT}
 for r in 1 2 3; do
  run $lib/$m.mps.gz "$out/repro/${m}_${c}_s${s}_$r.json" "{\"NodeLimit\":$n,\"Seed\":$s,${cfg[$c]}}" 900s
 done ;;
*) echo "unknown mode $mode"; exit 2 ;;
esac
echo SCAN_DONE
