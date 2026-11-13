#!/bin/bash
# M1-02w-2: three independent m1_factspace captures per case (Gurobi one at a time via tools/gurun.sh).
# usage: bench/oracle/m1_02w2/runs.sh OUTDIR
set -euo pipefail
cd "$(dirname "$0")/../../.."
export FRIDA_NODE=${FRIDA_NODE:-/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida}
O=bench/oracle/m1_02w2/${1:?OUTDIR}; mkdir -p $O
N=../re/bench/data/netlib
cases=("afiro afiro {}" "25fv47 25fv47 {}" "agg agg {}" "fit1d fit1d {}" "stair stair {}"
       "afiro_k7 afiro {\"GURO_PAR_SIMPLEXALLOCS\":7}" "grow15_twice grow15 {} twice")
for c in "${cases[@]}"; do
  read -r name mdl params mode <<<"$c"
  for r in 1 2 3; do
    [ -s $O/${name}_$r.json.gz ] || tools/gurun.sh node tools/m1_factspace.js $N/$mdl.mps.gz $O/${name}_$r.json.gz "$params" $mode
  done
done
echo RUNS_DONE
