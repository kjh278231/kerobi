#!/bin/bash
# Run a Gurobi-using command; retry every 5 s while another process holds the Gurobi lock (gurobi_guard.pth).
# Node runs preload tools/frida_resolve.cjs so the capture tools' original-computer Frida path resolves here.
export NODE_OPTIONS="${NODE_OPTIONS:+$NODE_OPTIONS }--require $(cd "$(dirname "$0")" && pwd)/frida_resolve.cjs"
for i in $(seq 1 720); do
  out=$("$@" 2>&1); rc=$?
  if grep -q 'Gurobi is already running' <<<"$out"; then sleep 5; continue; fi
  printf '%s\n' "$out"; exit $rc
done
echo "gave up waiting for Gurobi lock"; exit 99
