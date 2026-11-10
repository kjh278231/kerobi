#!/bin/bash
# Natural-path scan over MIPLIB small instances; one Gurobi process at a time.
# Usage: tools/mip_linear_child_scan.sh OUTDIR
out=${1:?OUTDIR}; mkdir -p "$out"
declare -A cfg=(
 [default]='{"TimeLimit":20,"NodeLimit":20000}'
 [stress]='{"TimeLimit":20,"NodeLimit":20000,"Presolve":0,"ScaleFlag":0,"Quad":0,"NumericFocus":0,"MarkowitzTol":0.0001}'
)
for f in ../re/bench/data/miplib/*.mps.gz; do
 for c in default stress; do
  ./tools/gurun.sh timeout 90s node tools/mip_linear_child_scan.js "$f" "$out/$c/$(basename "$f" .mps.gz).json" "${cfg[$c]}" || echo "FAIL $c $f rc=$?"
 done
done
echo SCAN_DONE
