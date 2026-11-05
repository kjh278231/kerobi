#!/bin/bash
# Run a Gurobi-using command; retry every 5 s while another process holds the Gurobi lock (gurobi_guard.pth).
for i in $(seq 1 720); do
  out=$("$@" 2>&1); rc=$?
  if grep -q 'Gurobi is already running' <<<"$out"; then sleep 5; continue; fi
  printf '%s\n' "$out"; exit $rc
done
echo "gave up waiting for Gurobi lock"; exit 99
