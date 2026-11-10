#!/bin/bash
# M2-03d-6 captures: node LP of the M2-03d-5 general B&B retry (nidda H seed 1, node 16948).
# One read-only baseline (mip_retry_basis.js natural, check A) then three instrumented captures; one Gurobi process at a time, no TimeLimit.
out=${1:?OUTDIR}; M=../re/bench/data/miplib/neos-3754480-nidda.mps.gz
P='{"NodeLimit":32898,"Seed":1,"Presolve":0,"ScaleFlag":0,"Quad":0,"NumericFocus":0,"MarkowitzTol":0.0001,"Heuristics":0,"RINS":0}'
rc=0
./tools/gurun.sh timeout 3600s node tools/mip_retry_basis.js natural $M "$out/nidda_bnb_H_s1_natural.json" "$P" || { echo "FAIL baseline"; rc=1; }
for r in 1 2 3; do
 ./tools/gurun.sh timeout 3600s node tools/mip_retry_node_lp.js $M "$out/nidda_bnb_H_s1_$r.json" "$P" || { echo "FAIL run $r"; rc=1; }
done
echo RUNS_DONE rc=$rc
exit $rc
