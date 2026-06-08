# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_empty"
# subject = "cpython.test_range.RangeTest.test_empty"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_empty
"""Auto-ported test: RangeTest::test_empty (CPython 3.12 oracle)."""


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
r = range(0)

assert 0 not in r

assert 1 not in r
r = range(0, -10)

assert 0 not in r

assert -1 not in r

assert 1 not in r
print("RangeTest::test_empty: ok")
