# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_range_iterators_invocation"
# subject = "cpython.test_range.RangeTest.test_range_iterators_invocation"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_range_iterators_invocation
"""Auto-ported test: RangeTest::test_range_iterators_invocation (CPython 3.12 oracle)."""


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
rangeiter_type = type(iter(range(0)))

try:
    rangeiter_type(1, 3, 1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
long_rangeiter_type = type(iter(range(1 << 1000)))

try:
    long_rangeiter_type(1, 3, 1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("RangeTest::test_range_iterators_invocation: ok")
