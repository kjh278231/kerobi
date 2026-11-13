#!/bin/bash
# M1-01e formal captures: each model 3 independent runs (Gurobi one at a time).
cd "$(dirname "$0")/../../.."
export FRIDA_NODE=${FRIDA_NODE:-/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida}
O=bench/oracle/m1_01e/formal; mkdir -p $O
for n in bnl1 maros 25fv47 grow15 grow22 scfxm2 perold stair degen2 kb2; do for r in 1 2 3; do
  [ -e $O/$n-$r.json.gz ] || tools/gurun.sh node tools/m1_block_state.js ../re/bench/data/netlib/$n.mps.gz $O/$n-$r.json.gz
done; done
echo FORMAL_DONE
