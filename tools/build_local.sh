#!/bin/bash
# Build the extension into this worktree's python/rsolver; never install into the shared .venv (its rsolver.pth
# points at the main tree). Run with PYTHONPATH=$PWD/python afterwards.
set -e
cd "$(dirname "$0")/.."
PYO3_PYTHON=../.venv/bin/python cargo build --release -p rsolver 2>&1 | grep -E '^(warning|error)|-->' | head -30 || true
cp target/release/lib_core.so python/rsolver/_core.cpython-313-x86_64-linux-gnu.so
