#!/bin/bash
# M2-10a (docs/m2/mip_bnb_node_cost.md): run the B&B profile harness once without instrumentation, then apply
# bench/oracle/m2_10a/profile.patch to bnb.rs, run it REPS times (default 3) and revert the patch.
#   tools/mip_bnb_profile.sh <out dir>        then: python3 tools/mip_bnb_profile.py <out dir>
set -euo pipefail
out=${1:?usage: tools/mip_bnb_profile.sh <out dir>}
cd "$(dirname "$0")/.."
patch=bench/oracle/m2_10a/profile.patch
git diff --quiet -- crates/mip/src/bnb.rs || { echo "crates/mip/src/bnb.rs has changes; refusing" >&2; exit 2; }
D=$(cd ../re/bench/data/miplib && pwd)
models="${MODELS:-gen-ip002 pk1 neos5 tr12-30 neos-3754480-nidda gmu-35-40 assign1-5-8 supportcase26 markshare_4_0 mas74 neos-860300 timtab1}"
export RSOLVER_PROFILE_NODES=${NODES:-500}
export RSOLVER_PROFILE_MODELS=$(for m in $models; do printf '%s ' "$D/$m.mps.gz"; done)
mkdir -p "$out"
. ~/.cargo/env
run() { cargo test --release -q -p rsolver-mip --test bnb_profile -- --ignored --nocapture 2>&1 | grep -E '^(RESULT|PROFILE)'; }
{ echo "head $(git rev-parse HEAD)"; echo "nodes $RSOLVER_PROFILE_NODES"; uname -srm; grep -m1 'model name' /proc/cpuinfo
  sha256sum "$patch" crates/mip/tests/bnb_profile.rs; for m in $models; do sha256sum "$D/$m.mps.gz"; done; } > "$out/env.txt"
echo "base $(cat /proc/loadavg)" > "$out/load.txt"
run > "$out/base.txt"
git apply "$patch"
trap 'git apply -R "$patch"' EXIT
for i in $(seq "${REPS:-3}"); do
  echo "prof$i $(cat /proc/loadavg)" >> "$out/load.txt"
  run > "$out/prof$i.txt"
done
echo "end $(cat /proc/loadavg)" >> "$out/load.txt"
