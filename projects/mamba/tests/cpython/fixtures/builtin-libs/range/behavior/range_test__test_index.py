# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_index"
# subject = "cpython.test_range.RangeTest.test_index"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_index
"""Auto-ported test: RangeTest::test_index (CPython 3.12 oracle)."""


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
u = range(2)

assert u.index(0) == 0

assert u.index(1) == 1

try:
    u.index(2)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
u = range(-2, 3)

assert u.count(0) == 1

assert u.index(0) == 2

try:
    u.index()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class BadExc(Exception):
    pass

class BadCmp:

    def __eq__(self, other):
        if other == 2:
            raise BadExc()
        return False
a = range(4)

try:
    a.index(BadCmp())
    raise AssertionError('expected BadExc')
except BadExc:
    pass
a = range(-2, 3)

assert a.index(0) == 2

assert range(1, 10, 3).index(4) == 1

assert range(1, -10, -3).index(-5) == 2

assert range(10 ** 20).index(1) == 1

assert range(10 ** 20).index(10 ** 20 - 1) == 10 ** 20 - 1

try:
    range(1, 2 ** 100, 2).index(2 ** 87)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert range(1, 2 ** 100, 2).index(2 ** 87 + 1) == 2 ** 86

assert range(10).index(ALWAYS_EQ) == 0
print("RangeTest::test_index: ok")
