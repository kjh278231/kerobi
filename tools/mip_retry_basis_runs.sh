#!/bin/bash
# M2-03c-2 captures: natural and synthetic export, three independent processes each; one Gurobi process at a time.
out=${1:?OUTDIR}; M=../re/bench/data/miplib
S='{"NodeLimit":6000,"Presolve":0,"ScaleFlag":0,"Quad":0,"NumericFocus":0,"MarkowitzTol":0.0001}'
D='{"NodeLimit":20000}'
rc=0
for r in 1 2 3; do
 for k in natural synth; do
  ./tools/gurun.sh timeout 600s node tools/mip_retry_basis.js $k $M/neos-3754480-nidda.mps.gz "$out/nidda_stress_${k}_$r.json" "$S" || { echo "FAIL nidda $k $r"; rc=1; }
  ./tools/gurun.sh timeout 600s node tools/mip_retry_basis.js $k $M/gmu-35-50.mps.gz "$out/gmu-35-50_default_${k}_$r.json" "$D" || { echo "FAIL gmu $k $r"; rc=1; }
 done
done
echo RUNS_DONE rc=$rc
exit $rc
