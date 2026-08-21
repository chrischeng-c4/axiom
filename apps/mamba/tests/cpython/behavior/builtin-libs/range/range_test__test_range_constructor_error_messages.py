# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_range_constructor_error_messages"
# subject = "cpython.test_range.RangeTest.test_range_constructor_error_messages"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_range_constructor_error_messages
"""Auto-ported test: RangeTest::test_range_constructor_error_messages (CPython 3.12 oracle)."""


import unittest
import sys
import pickle
import itertools
from test.support import ALWAYS_EQ


def pyrange(start, stop, step):
    if (start - stop) // step < 0:
        stop += (start - stop) % step
        while start != stop:
            yield start
            start += step

def pyrange_reversed(start, stop, step):
    stop += (start - stop) % step
    return pyrange(stop - step, start - step, -step)


# --- test body ---
try:
    range()
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('range expected at least 1 argument, got 0', str(_aR_e))
try:
    range(1, 2, 3, 4, 5, 6)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('range expected at most 3 arguments, got 6', str(_aR_e))
print("RangeTest::test_range_constructor_error_messages: ok")
