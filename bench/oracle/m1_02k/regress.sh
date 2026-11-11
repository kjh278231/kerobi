#!/bin/bash
# M1-02k regression (worktree root, after tools/build_local.sh). Outputs next to this script; prints one line each.
cd "$(dirname "$0")/../../.."
. ~/.cargo/env
O=bench/oracle/m1_02k
PY=../.venv/bin/python
run() { local name=$1; shift; "$@" > $O/$name.txt 2>&1; echo "$name rc=$? | $(tail -1 $O/$name.txt | cut -c1-150)"; }
run cargo_lp cargo test -p rsolver-lp
run cargo_workspace cargo test --release --workspace
run clippy cargo clippy --workspace --all-targets -- -D warnings
tools/build_local.sh > /dev/null
export PYTHONPATH=$PWD/python
run pytest $PY -m pytest -q
for c in aq_order beta eta eta_peel list_order tau urow work factor_work; do run $c $PY tools/m1_${c}_check.py --self-test; done
for c in lsolve update usolve; do run $c $PY tools/m1_${c}_check.py --rust --self-test; done
run bfrt $PY tools/m1_bfrt_rust.py --self-test
run btran $PY tools/m1_btran_check.py --self-test
run pathcmp $PY tools/proto/pathcmp.py all
run lp_check $PY tools/lp_check.py --methods 1,0,-1 --timeout 120 --csv $O/lp_check.csv
echo REGRESS_DONE
