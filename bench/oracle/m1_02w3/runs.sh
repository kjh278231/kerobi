#!/bin/bash
# M1-02w-3 formal captures: tools/m1_dense_state.js, each model 3 independent runs (Gurobi one at a time). arg: subdir.
cd "$(dirname "$0")/../../.."
export FRIDA_NODE=${FRIDA_NODE:-/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida}
O=bench/oracle/m1_02w3/${1:-formal}; mkdir -p $O
for n in stair grow15 grow22 perold pilot4; do for r in 1 2 3; do
  until [ -e $O/$n-$r.json.gz ]; do tools/gurun.sh node tools/m1_dense_state.js ../re/bench/data/netlib/$n.mps.gz $O/$n-$r.json.gz || sleep 30; done
done; done
echo RUNS_DONE
