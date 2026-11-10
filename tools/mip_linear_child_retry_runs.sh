#!/bin/bash
# Natural retry and status-3 caller captures, three independent processes each; one Gurobi process at a time.
out=${1:?OUTDIR}; M=../re/bench/data/miplib
S='{"NodeLimit":6000,"Presolve":0,"ScaleFlag":0,"Quad":0,"NumericFocus":0,"MarkowitzTol":0.0001}'
D='{"NodeLimit":20000}'
T='{"NodeLimit":20000}'
for r in 1 2 3; do
 ./tools/gurun.sh timeout 300s node tools/mip_linear_child_retry.js retry $M/neos-3754480-nidda.mps.gz "$out/nidda_stress_retry_$r.json" "$S" || echo "FAIL nidda $r"
 ./tools/gurun.sh timeout 300s node tools/mip_linear_child_retry.js retry $M/gmu-35-50.mps.gz "$out/gmu-35-50_default_retry_$r.json" "$D" || echo "FAIL gmu $r"
 ./tools/gurun.sh timeout 900s node tools/mip_linear_child_retry.js set3 $M/neos-4338804-snowy.mps.gz "$out/snowy_default_set3_$r.json" "$T" || echo "FAIL snowy $r"
done
echo RUNS_DONE
