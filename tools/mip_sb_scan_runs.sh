#!/bin/bash
# M2-06h-7: one light scan per model (Gurobi serial), then a hit summary.
# Usage: tools/mip_sb_scan_runs.sh OUTDIR PARAMS_JSON [MODEL_PATH...]   (default: all of ../re/bench/data/miplib)
set -uo pipefail
cd "$(dirname "$0")/.."
export FRIDA_NODE=${FRIDA_NODE:-/home/sds/.npm/_npx/da7caeae9db9c589/node_modules/frida}
out=${1:?OUTDIR}; params=${2:?PARAMS}; shift 2
mkdir -p "$out"
models=("$@"); [ ${#models[@]} -eq 0 ] && models=(../re/bench/data/miplib/*.mps.gz)
for p in "${models[@]}"; do
 m=$(basename "$p"); m=${m%%.*}
 [ -s "$out/$m.json" ] && continue
 timeout ${SCAN_TIMEOUT:-300} tools/gurun.sh node tools/mip_sb_scan.js "$p" "$out/$m.json" "$params" | tail -1
 echo "$m rc=${PIPESTATUS[0]}"
done
python3 - "$out" <<'P'
import json, sys, glob, collections
tot = collections.Counter()
for f in sorted(glob.glob(sys.argv[1] + '/*.json')):
    d = json.load(open(f)); c = d['counts']
    hit = {k: v for k, v in c.items() if k.startswith(('contradiction', 'continuous', 'no propagation', 'stride'))}
    for k, v in c.items(): tot[k.split(':')[0] if k.startswith(('kernel', 'double', 'prop')) else k] += v
    print(d['model'], 'ok' if d['ok'] else 'FAIL', d['output'].split('\n')[-1][:60], hit)
print('TOTAL', dict(tot))
P
