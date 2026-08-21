# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_invalid_invocation"
# subject = "cpython.test_range.RangeTest.test_invalid_invocation"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_invalid_invocation
"""Auto-ported test: RangeTest::test_invalid_invocation (CPython 3.12 oracle)."""


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
except TypeError:
    pass

try:
    range(1, 2, 3, 4)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    range(1, 2, 0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
a = int(10 * sys.maxsize)

try:
    range(a, a + 1, int(0))
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    range(1.0, 1.0, 1.0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    range(1e+100, 1e+101, 1e+101)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    range(0, 'spam')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    range(0, 42, 'spam')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    range(0.0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    range(0, 0.0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    range(0.0, 0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    range(0.0, 0.0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    range(0, 0, 1.0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    range(0, 0.0, 1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    range(0, 0.0, 1.0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    range(0.0, 0, 1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    range(0.0, 0, 1.0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    range(0.0, 0.0, 1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    range(0.0, 0.0, 1.0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("RangeTest::test_invalid_invocation: ok")
