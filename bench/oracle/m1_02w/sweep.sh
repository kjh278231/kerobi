#!/bin/bash
# M1-02w exploration: one m1_pool_size capture per licensed Netlib model (bench/oracle/path1), Gurobi one at a time.
cd "$(dirname "$0")/../../.."
export FRIDA_NODE=${FRIDA_NODE:-/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida}
O=bench/oracle/m1_02w/${1:-pool}; mkdir -p $O
for f in bench/oracle/path1/*.json.gz; do n=$(basename $f .json.gz)
  [ -e $O/$n.json.gz ] || tools/gurun.sh node tools/m1_pool_size.js ../re/bench/data/netlib/$n.mps.gz $O/$n.json.gz
done
echo SWEEP_DONE
