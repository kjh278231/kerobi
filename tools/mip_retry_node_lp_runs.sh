#!/bin/bash
# M2-03d-3 captures: retry node LP, three independent processes per case; one Gurobi process at a time, no TimeLimit.
out=${1:?OUTDIR}; M=../re/bench/data/miplib
S='{"NodeLimit":6000,"Presolve":0,"ScaleFlag":0,"Quad":0,"NumericFocus":0,"MarkowitzTol":0.0001}'
D='{"NodeLimit":20000}'
rc=0
for r in 1 2 3; do
 ./tools/gurun.sh timeout 900s node tools/mip_retry_node_lp.js $M/neos-3754480-nidda.mps.gz "$out/nidda_stress_$r.json" "$S" || { echo "FAIL nidda $r"; rc=1; }
 ./tools/gurun.sh timeout 900s node tools/mip_retry_node_lp.js $M/gmu-35-50.mps.gz "$out/gmu-35-50_default_$r.json" "$D" || { echo "FAIL gmu $r"; rc=1; }
done
echo RUNS_DONE rc=$rc
exit $rc
