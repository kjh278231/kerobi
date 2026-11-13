#!/bin/bash
# M1-02w-3 regression (copy of m1_01e/regress.sh) (arg: output subdir, e.g. base or final) (worktree root, after tools/build_local.sh). Outputs next to this script; prints one line each.
cd "$(dirname "$0")/../../.."
. ~/.cargo/env
O=bench/oracle/m1_02w3/${1:?subdir}; mkdir -p $O
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
run lt $PY tools/m1_lt_check.py --self-test
run diag_peel $PY tools/m1_diag_peel_check.py --self-test
for e in 126 129 130; do : > $O/urow$e.spec; RSOLVER_BTRAN_PROBE=$O/urow$e.spec RSOLVER_UROW_PROBE=$e $PY -c "import rsolver; m=rsolver.read('../re/bench/data/netlib/bnl1.mps.gz'); m.Params['Method']=1; m.Params['Presolve']=0; m.optimize()" >/dev/null 2>&1; run urow$e python3 tools/m1_urow_cmp.py bench/oracle/m1_02y/explore/bnl1_urow$e.json.gz $O/urow$e.spec.out; rm -f $O/urow$e.spec $O/urow$e.spec.out; done
for e in 185; do : > $O/urow$e.spec; RSOLVER_BTRAN_PROBE=$O/urow$e.spec RSOLVER_UROW_PROBE=$e $PY -c "import rsolver; m=rsolver.read('../re/bench/data/netlib/stair.mps.gz'); m.Params['Method']=1; m.Params['Presolve']=0; m.optimize()" >/dev/null 2>&1; run urow$e python3 tools/m1_urow_cmp.py bench/oracle/m1_02w/explore/stair_lu108_185.json.gz $O/urow$e.spec.out; rm -f $O/urow$e.spec $O/urow$e.spec.out; done
run block $PY tools/m1_block_check.py --repro --rust --replay --lu
run dense $PY tools/m1_dense_check.py --repro --replay --rust --self-test
run pathcmp $PY tools/proto/pathcmp.py all
run lp_check $PY tools/lp_check.py --methods 1,0,-1 --timeout 120 --csv $O/lp_check.csv
echo REGRESS_DONE
