#!/bin/bash
# M1-02q reference captures: tools/m1_factor_work.js on kb2 and degen2, 3 independent runs each (one Gurobi at a time).
set -e
out=${1:-bench/oracle/m1_02q}
for m in kb2 degen2; do for i in 1 2 3; do
  tools/gurun.sh node tools/m1_factor_work.js ../re/bench/data/netlib/$m.mps.gz $out/${m}_fw_$i.json.gz
done; done
sha256sum $out/*_fw_*.json.gz
