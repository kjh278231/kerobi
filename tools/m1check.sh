#!/bin/sh
# M1 Method-1 check on the 64 licensed Netlib LPs: correctness/IterCount (bench/lp_check.csv), iteration stats, and
# first pivot-path divergence (bench/pathcmp.txt). usage: sh tools/m1check.sh [timeout]
cd "$(dirname "$0")/.." || exit 1
PY=../.venv/bin/python
$PY tools/lp_check.py --methods 1 --timeout "${1:-120}" $(ls bench/gurobi_dual_nopre) | tail -1
$PY tools/proto/itstat.py
$PY tools/proto/pathcmp.py all > bench/pathcmp.txt 2>&1; tail -1 bench/pathcmp.txt
