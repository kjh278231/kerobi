#!/bin/bash
# M2-03d-13 captures: 10f448 in-place split (docs/m2/mip_dive_split.md section 4); same cases and params as tools/mip_dive_rows_runs.sh.
# Three independent read-only captures per case, one Gurobi process at a time, no TimeLimit. Existing outputs are kept.
out=${1:?OUTDIR}; L=../re/bench/data/miplib; mkdir -p "$out"
H='"NodeLimit":32898,"Seed":1,"Presolve":0,"ScaleFlag":0,"Quad":0,"NumericFocus":0,"MarkowitzTol":0.0001,"Heuristics":0,"RINS":0'
rc=0
run() { for r in 1 2 3; do f="$out/${2}_$r.json.gz"
  [ -e "$f" ] && { echo "skip $f (exists)"; continue; }
  ./tools/gurun.sh timeout 1800s node tools/mip_dive_split.js "$L/$1.mps.gz" "$f" "$3" "$4" || { echo "FAIL $f"; rc=1; }
 done; }
run neos-3754480-nidda nidda_H_s1 "{$H}" 19901
run neos-3754480-nidda nidda_H_s1_dr0 "{$H,\"GURO_PAR_DYNAMICROWS\":0}" 19901
run gmu-35-40 gmu-35-40_default '{"NodeLimit":3000}' 0
run assign1-5-8 assign1-5-8_default '{"NodeLimit":3000}' 0
echo RUNS_DONE rc=$rc
exit $rc
