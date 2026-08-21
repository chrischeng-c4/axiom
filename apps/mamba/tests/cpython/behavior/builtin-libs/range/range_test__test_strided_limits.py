# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_strided_limits"
# subject = "cpython.test_range.RangeTest.test_strided_limits"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_strided_limits
"""Auto-ported test: RangeTest::test_strided_limits (CPython 3.12 oracle)."""


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
r = range(0, 101, 2)

assert 0 in r

assert 1 not in r

assert 2 in r

assert 99 not in r

assert 100 in r

assert 101 not in r
r = range(0, -20, -1)

assert 0 in r

assert -1 in r

assert -19 in r

assert -20 not in r
r = range(0, -20, -2)

assert -18 in r

assert -19 not in r

assert -20 not in r
print("RangeTest::test_strided_limits: ok")
