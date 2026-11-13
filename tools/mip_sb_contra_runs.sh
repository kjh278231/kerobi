#!/bin/bash
# M2-06h-7: reference captures for the propagation contradiction (up csched007, down ic97_potential) and the
# continuous SOS candidate (synthetic sos2_g40k5t): unhooked control + three independent runs each.
# Usage: tools/mip_sb_contra_runs.sh OUTDIR
set -euo pipefail
cd "$(dirname "$0")/.."
export FRIDA_NODE=${FRIDA_NODE:-/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida}
out=${1:?OUTDIR}
cases=("csched007 ../re/bench/data/miplib/csched007.mps.gz 150"
       "ic97_potential ../re/bench/data/miplib/ic97_potential.mps.gz 1050"
       "sos2_g40k5t bench/oracle/m2_06h7/models/sos2_g40k5t.mps 5000")
for c in "${cases[@]}"; do
 read -r m p nl <<<"$c"
 params="{\"NodeLimit\":$nl,\"GURO_PAR_SBMAXITERS\":10}"
 for r in control 1 2 3; do
  [ -s "$out/${m}_$r.json.gz" ] && continue
  tools/gurun.sh node tools/mip_sb_direct.js "$p" "$out/${m}_$r.json.gz" "$params" $([ $r = control ] && echo control) | cut -c1-200
 done
done
