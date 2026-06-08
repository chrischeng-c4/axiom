# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_user_index_method"
# subject = "cpython.test_range.RangeTest.test_user_index_method"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_user_index_method
"""Auto-ported test: RangeTest::test_user_index_method (CPython 3.12 oracle)."""


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
bignum = 2 * sys.maxsize
smallnum = 42

class I:

    def __init__(self, n):
        self.n = int(n)

    def __index__(self):
        return self.n

assert list(range(I(bignum), I(bignum + 1))) == [bignum]

assert list(range(I(smallnum), I(smallnum + 1))) == [smallnum]

class IX:

    def __index__(self):
        raise RuntimeError

try:
    range(IX())
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass

class IN:

    def __index__(self):
        return 'not a number'

try:
    range(IN())
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert range(10)[:I(5)] == range(5)
try:
    range(0, 10)[:IX()]
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
try:
    range(0, 10)[:IN()]
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("RangeTest::test_user_index_method: ok")
