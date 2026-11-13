#!/bin/bash
# M1-02w-4 mutations of the eta.S spike mask in Lu::update; each is built, judged by m1_diag_peel_check / m1_lt_check
# --rust, and lu.rs is restored. Output: bench/oracle/m1_02w4/mutate.txt
cd "$(dirname "$0")/../../.."; . ~/.cargo/env
export PYTHONPATH=$PWD/python; PY=../.venv/bin/python; O=bench/oracle/m1_02w4/mutate.txt; : > $O
F=crates/lp/src/lu.rs; cp $F /tmp/m1_02w4_lu.rs.orig
ORIG='let s = |i: usize| if spike[i].abs() > ZERO { spike[i] } else { 0.0 };'
mut() { local name=$1 new=$2
  grep -qF "$ORIG" $F || { echo "anchor missing" >> $O; exit 2; }
  python3 -c "import sys;p='$F';s=open(p).read();a=sys.argv[1];b=sys.argv[2];assert s.count(a)==1;open(p,'w').write(s.replace(a,b))" "$ORIG" "$new"
  tools/build_local.sh > /dev/null
  echo "== $name: $new" >> $O
  $PY tools/m1_diag_peel_check.py --rust 2>&1 | tail -1 >> $O
  $PY tools/m1_lt_check.py --rust 2>&1 | tail -1 >> $O
  cp /tmp/m1_02w4_lu.rs.orig $F
}
mut 'no mask (before the fix)' 'let s = |i: usize| spike[i];'
mut '> to >=' 'let s = |i: usize| if spike[i].abs() >= ZERO { spike[i] } else { 0.0 };'
mut 'mask at 1e-12 (wrong tolerance)' 'let s = |i: usize| if spike[i].abs() > 1e-12 { spike[i] } else { 0.0 };'
tools/build_local.sh > /dev/null
cmp -s $F /tmp/m1_02w4_lu.rs.orig && echo "lu.rs restored" >> $O
