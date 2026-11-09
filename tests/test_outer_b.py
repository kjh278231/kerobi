"""Outer-B branch fixtures: native Gurobi states captured before/after a single reduction.

Regenerate with tools/outer_b_oracle.py --capture (serial gurun.sh); tests themselves use no Gurobi.
"""
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
from outer_b_oracle import cases, check  # noqa: E402


@pytest.mark.parametrize('case', cases(), ids=lambda c: c['name'])
def test_native_outer_b_state(case):
    got = check(case)
    # The positive cases must emit actual postsolve records, rather than just scan the candidate.
    if case['removed']:
        assert len(got['recs']) == case['removed']
    if case['name'] == 'pair_mixed_floor':
        assert got['ub'][3] == 3.0  # fractional continuous bound is floored on the retained integer
    if case['name'] == 'pair_mixed_binary':
        assert got['vtype'][3] == ord('B')
        assert got['vtype'][1] == ord('I')
