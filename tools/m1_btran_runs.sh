#!/bin/bash
# M1-02k baseline: per model one plain capture (natural BTRAN events), the synthetic spec built from it, then three
# independent synthetic captures. usage: tools/m1_btran_runs.sh OUTDIR   (Gurobi one process at a time)
set -e
cd "$(dirname "$0")/.."
out=$1; [ -n "$out" ] || { echo "usage: $0 OUTDIR"; exit 2; }
mkdir -p "$out"
while read -r n ks; do
  mps=../re/bench/data/netlib/$n.mps.gz
  [ -e "$out/${n}_plain.json.gz" ] || tools/gurun.sh node tools/m1_btran_path.js "$mps" "$out/${n}_plain.json.gz" 1 | tail -1
  [ -e "$out/$n.spec" ] || python3 tools/m1_btran_spec.py "$out/${n}_plain.json.gz" "$ks" > "$out/$n.spec"
  for i in 1 2 3; do
    [ -e "$out/${n}_syn_$i.json.gz" ] || tools/gurun.sh node tools/m1_btran_path.js "$mps" "$out/${n}_syn_$i.json.gz" 1 "$out/$n.spec" | tail -1
  done
done <<'CASES'
kb2 0,17,30
degen2 0,100,300
sc50b 0,10
scsd1 10
stair 17
forplan 12
fit1p 21
CASES
echo RUNS_DONE
