#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
out=${1:?OUTDIR}
params='{"NodeLimit":300,"GURO_PAR_SBMAXITERS":10}'
tools/gurun.sh node tools/mip_sb_snapshot.js ../re/bench/data/miplib/pk1.mps.gz "$out/control.json.gz" "$params" control
for r in 1 2 3; do
 tools/gurun.sh node tools/mip_sb_snapshot.js ../re/bench/data/miplib/pk1.mps.gz "$out/$r.json.gz" "$params"
done
