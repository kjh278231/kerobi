#!/bin/bash
# M2-03d-7 captures: row deactivation provenance up to the M2-03d-6 retry node (nidda H seed 1, 5ce67b call 19901).
# Three independent read-only captures, one Gurobi process at a time, no TimeLimit.
out=${1:?OUTDIR}; M=../re/bench/data/miplib/neos-3754480-nidda.mps.gz
P='{"NodeLimit":32898,"Seed":1,"Presolve":0,"ScaleFlag":0,"Quad":0,"NumericFocus":0,"MarkowitzTol":0.0001,"Heuristics":0,"RINS":0}'
rc=0
for r in 1 2 3; do
 [ -e "$out/nidda_bnb_H_s1_$r.json" ] && { echo "skip run $r (exists)"; continue; }
 ./tools/gurun.sh timeout 3600s node tools/mip_blank_rows.js $M "$out/nidda_bnb_H_s1_$r.json" "$P" 19901 || { echo "FAIL run $r"; rc=1; }
done
echo RUNS_DONE rc=$rc
exit $rc
