#!/bin/bash
# M1-02w-3 copy of m1_01e/push_regress.sh: M3 push stored-oracle comparisons that go through Lu::finish (t = s3 is shared with factor_dense).
cd "$(dirname "$0")/../../.."; . ~/.cargo/env
O=bench/oracle/m1_02w3/${1:?subdir}; mkdir -p $O; : > $O/push.txt
run() { echo "== $*" >> $O/push.txt; "$@" >> $O/push.txt 2>&1; r=$?; echo "exit $r" >> $O/push.txt; echo "$r | $*"; }
run python3 tools/push_lu_rust.py --write
run python3 tools/push_lu_rust.py --generate --write
run python3 tools/push_lu_rust.py --block --write
run python3 tools/push_mode70_rust.py --write
run python3 tools/push_dense_rust.py --write
run python3 tools/push_install_rust.py --write
run git diff --exit-code --stat bench/oracle
run cargo test -p rsolver-lp --test push_lu
run cargo test --release -p rsolver-lp --test push_lu mode70_chain_trace
run cargo test --release -p rsolver-lp --test push_dense --test push_lu
run cargo test -p rsolver-lp --test push_install --test push_init_ext
run python3 tools/push_lu_rust.py --compare
run python3 tools/push_lu_rust.py --generate --compare
run python3 tools/push_lu_rust.py --block --compare
run python3 tools/push_mode70_rust.py --compare
run python3 tools/push_dense_rust.py --compare
run python3 tools/push_install_rust.py --compare
run python3 tools/push_loop_rust.py --lu --compare
run python3 tools/push_loop_rust.py --init --compare
echo PUSH_DONE
