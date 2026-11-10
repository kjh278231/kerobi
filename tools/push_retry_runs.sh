#!/bin/bash
# M3-04c-7 원본 캡처 전체(docs/m3/primal_push_retry.md §5). 기존 파일은 캡처 도구가 덮어쓰지 않는다.
set -e
cd "$(dirname "$0")/.."
D=bench/oracle/m3_push_retry
V=bench/oracle/m3_push_violated
for i in 1 2 3; do tools/gurun.sh ../.venv/bin/python tools/push_retry_ratio_oracle.py --capture $D/ratio-$i.json; done
X="$(cat $D/retry.extra.json)"
for k in primal dual control; do for i in 1 2 3; do
  tools/gurun.sh node tools/primal_push_retry_capture.js $k $D/retry.lp $D/retry-$k-$i.json "$X"
done; done
mkdir -p $D/noninterference
for k in primal dual; do
  tools/gurun.sh node tools/primal_push_violated_capture.js $k $D/retry.lp $D/noninterference/retry-$k-violatedagent.json "$X"
done
tools/gurun.sh node tools/primal_push_retry_capture.js primal $V/violated.lp $D/noninterference/violated-primal-retryagent.json "$(cat $V/violated.extra.json)"
