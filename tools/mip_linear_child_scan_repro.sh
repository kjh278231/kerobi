#!/bin/bash
# Re-run natural scan hits three times each in independent processes; one Gurobi process at a time.
out=${1:?OUTDIR}; D='{"TimeLimit":20,"NodeLimit":20000}'
S='{"TimeLimit":20,"NodeLimit":20000,"Presolve":0,"ScaleFlag":0,"Quad":0,"NumericFocus":0,"MarkowitzTol":0.0001}'
for r in 1 2 3; do
 for spec in "neos-3754480-nidda stress $S" "gmu-35-50 default $D" "neos-4338804-snowy default $D"; do
  set -- $spec
  ./tools/gurun.sh timeout 90s node tools/mip_linear_child_scan.js ../re/bench/data/miplib/$1.mps.gz "$out/$1_$2_$r.json" "$3" || echo "FAIL $1 $2 $r"
 done
done
echo REPRO_DONE
