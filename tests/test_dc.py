import importlib.util
from pathlib import Path

import pytest

spec = importlib.util.spec_from_file_location('dc', Path(__file__).resolve().parents[1] / 'tools/dc.py')
dc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(dc)

ROWS = [(0x100, 'GRBoptimize', 0x40, 0, 1), (0x200, 'PRIVATE00000000003a484d', 0x80, 1, 2), (0x300, 'GRBoptimizebatch', 0x10, 0, 0)]


def test_address_inside_function():
    assert dc.find(ROWS, '0x27f', False) == ([ROWS[1]], 0x27f)
    assert dc.find(ROWS, str(0x200), True)[0] == [ROWS[1]]


def test_address_in_gap_or_before_index_exits():
    for key in ('0x150', '0x10'):
        with pytest.raises(SystemExit):
            dc.find(ROWS, key, False)


def test_name_exact_then_suffix_then_substring():
    assert dc.find(ROWS, 'GRBoptimize', False)[0] == [ROWS[0]]
    assert dc.find(ROWS, '3a484d', False)[0] == [ROWS[1]]
    assert dc.find(ROWS, 'optimizeb', False)[0] == [ROWS[2]]
    assert dc.short(ROWS[1][1]) == '3a484d'
