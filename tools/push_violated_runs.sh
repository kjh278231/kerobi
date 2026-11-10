#!/bin/bash
# M3-04c-2 원본 캡처 전체(docs/m3/primal_push_violated.md §5). 기존 파일은 캡처 도구가 덮어쓰지 않는다.
set -e
cd "$(dirname "$0")/.."
D=bench/oracle/m3_push_violated
for i in 1 2 3; do tools/gurun.sh ../.venv/bin/python tools/push_infeasible_ratio_oracle.py --capture $D/ratio-$i.json; done
X="$(cat $D/violated.extra.json)"
for k in primal dual control; do for i in 1 2 3; do
  tools/gurun.sh node tools/primal_push_violated_capture.js $k $D/violated.lp $D/violated-$k-$i.json "$X"
done; done
mkdir -p $D/noninterference
for k in primal dual; do
  tools/gurun.sh node tools/primal_push_arm_capture.js $k $D/violated.lp $D/noninterference/violated-$k-armagent.json "$X"
done
R="$(cat $D/retry.extra.json)"
tools/gurun.sh node tools/primal_push_violated_capture.js primal $D/retry.lp $D/noninterference/retry-primal-new.json "$R"
tools/gurun.sh node tools/primal_push_arm_capture.js primal $D/retry.lp $D/noninterference/retry-primal-armagent.json "$R"
