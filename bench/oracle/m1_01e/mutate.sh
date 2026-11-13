#!/bin/bash
# M1-01e Rust mutations: each edits the source in place (asserted), rebuilds, must make the checker fail, then restores.
cd "$(dirname "$0")/../../.."
. ~/.cargo/env; export PYTHONPATH=$PWD/python; PY=../.venv/bin/python
mut() { local name=$1 file=$2 old=$3 new=$4 mode=$5
  python3 -c "import sys; p,a,b=sys.argv[1:]; s=open(p).read(); assert s.count(a)==1, 'missing: '+a; open(p,'w').write(s.replace(a,b))" "$file" "$old" "$new" || { echo "$name NOT APPLIED"; return; }
  [ "$mode" = --rust ] && tools/build_local.sh >/dev/null 2>&1
  if $PY tools/m1_block_check.py $mode >/tmp/m1_01e_mut.txt 2>&1; then echo "$name SURVIVED"; else echo "$name rejected | $(grep -m1 -E 'FAIL |replay .*: k|factor' /tmp/m1_01e_mut.txt | cut -c1-140)"; fi
  python3 -c "import sys; p,a,b=sys.argv[1:]; s=open(p).read(); assert s.count(b)==1; open(p,'w').write(s.replace(b,a))" "$file" "$old" "$new"; }
S=crates/lp/src/simplex.rs L=crates/lp/src/lu.rs
mut reset-each-refactor $S '        self.block.x2c0 += 1;' '        self.block = BlockState { x2c0: self.block.x2c0 + 1, x4e0: 0, x4e4: 0 };' --rust
mut no-x2c0-increment $S '        self.block.x2c0 += 1;' '' --rust
mut start-x2c0-1 $S 'block: BlockState { x2c0: 0, x4e0: 0, x4e4: 0 },' 'block: BlockState { x2c0: 1, x4e0: 0, x4e4: 0 },' --rust
mut block-off $S "            self.lu = Lu::factor_block(self.spx, &self.head, self.markowitz_tol, &mut self.block);" "            self.lu = Lu::factor_tol(self.spx, &self.head, self.markowitz_tol);" --rust
mut decide-x2c0-21 $L '            if st.x2c0 > 20 {' '            if st.x2c0 > 21 {' --replay
mut decide-le $L '{ -1 } else { 2_000_000_000 }' '{ 2_000_000_000 } else { -1 }' --replay
tools/build_local.sh >/dev/null 2>&1; echo MUTATE_DONE
