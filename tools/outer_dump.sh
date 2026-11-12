#!/bin/bash
# outer_dump.sh <out dir> [model ...]: dump every call of the outer-pass group A reductions (docs/m2/m2_red_outer_a.md)
# with tools/mip_state_dump.js into <out dir>/<model>_<fn>.json, for tools/mip_dump_check.py. Default models: the
# 38 MIPLIB ones (tools/parity.py). Existing non-empty files are kept. FNS overrides the function list.
set -u
out=$1; shift
cd "$(dirname "$0")/.."
mkdir -p "$out"
ms="$*"
[ -z "$ms" ] && ms=$(.venv/bin/python -I -c "import sys; sys.path.insert(0, 'tools'); from parity import instances; print(' '.join(n for n, p in instances().items() if p.parent.name == 'miplib'))" 2>/dev/null \
  || ../.venv/bin/python -I -c "import sys; sys.path.insert(0, 'tools'); from parity import instances; print(' '.join(n for n, p in instances().items() if p.parent.name == 'miplib'))")
for m in $ms; do
  for f in ${FNS:-3e5363 3ff744 57627e 578fd2 374521 5431ee}; do
    [ -s "$out/${m}_$f.json" ] && continue
    tools/gurun.sh node tools/mip_state_dump.js "$m" "$f" all "$out/${m}_$f.json" | tail -1
  done
done
