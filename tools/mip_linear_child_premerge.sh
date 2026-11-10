#!/bin/bash
# Pre-merge full re-run of every original capture used by M2-03c-1 into OUTDIR (one Gurobi process at a time).
# Compare with: python3 tools/mip_linear_child_rerun_compare.py OUTDIR
out=${1:?OUTDIR}; mkdir -p "$out"/{trace,experiments,scan,retry}
run() { ./tools/gurun.sh "$@" | cut -c1-140 || echo "FAIL $*"; }
for model in fractional_branch root_unbounded; do for r in 1 2 3; do
  run timeout 120s node tools/mip_linear_child_trace.js "bench/mip_linear_child/$model.lp" "$out/trace/${model}_$r.json"; done; done
for mode in natural inject_reclass inject_branch; do for r in 1 2 3; do
  run timeout 60s node tools/mip_linear_child_experiment.js "$mode" bench/mip_linear_child/continuous_candidate.lp "$out/experiments/${mode}_$r.json"; done; done
./tools/mip_linear_child_scan_repro.sh "$out/scan" | cut -c1-140
./tools/mip_linear_child_retry_runs.sh "$out/retry" | cut -c1-140
echo PREMERGE_DONE
