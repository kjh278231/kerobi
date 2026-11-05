#!/bin/bash
# BFRT input/output corpus (docs/m1_notes.md §8.5): first 50 calls of 1c35c5 per Netlib LP into bench/oracle/bfrt_io.
# Gurobi runs one process at a time; retries while another process holds the license guard.
cd "$(dirname "$0")/.."
for d in bench/gurobi_dual_nopre/*/; do
  name=$(basename "$d")
  [ -s bench/oracle/bfrt_io/$name.jsonl.gz ] && continue
  python3 - "$name" > /tmp/bfrt_spec_$$.json <<'PY'
import json, sys
s = json.load(open("tools/trace_specs/bfrt_io.json")); s["model"] = sys.argv[1]; s["max"] = 100
print(json.dumps(s))
PY
  for t in 1 2 3 4 5 6 7 8 9 10; do
    out=$(timeout 600 node tools/fn_trace.js /tmp/bfrt_spec_$$.json bench/oracle/bfrt_io/$name.jsonl 2>&1 | tail -1)
    echo "$name: $out"
    echo "$out" | grep -q DONE && break
    sleep 5
  done
  gzip -f bench/oracle/bfrt_io/$name.jsonl
done
rm -f /tmp/bfrt_spec_$$.json
