#!/bin/bash
# M3-04c-8 원본 캡처 전체(docs/m3/primal_push_recall.md §5). 기존 파일은 캡처 도구가 덮어쓰지 않는다.
set -e
cd "$(dirname "$0")/.."
D=bench/oracle/m3_push_recall
R=bench/oracle/m3_push_retry
for lp in feasible violated; do
  X="$(cat $D/$lp.extra.json)"
  for k in primal dual control; do for i in 1 2 3; do
    tools/gurun.sh node tools/primal_push_recall_capture.js $k $D/$lp.lp $D/$lp-$k-$i.json "$X"
  done; done
done
mkdir -p $D/noninterference
for lp in feasible violated; do for k in primal dual; do
  tools/gurun.sh node tools/primal_push_retry_capture.js $k $D/$lp.lp $D/noninterference/$lp-$k-retryagent.json "$(cat $D/$lp.extra.json)"
done; done
tools/gurun.sh node tools/primal_push_recall_capture.js primal $R/retry.lp $D/noninterference/retry-primal-recallagent.json "$(cat $R/retry.extra.json)"
echo RECALL_RUNS_DONE
