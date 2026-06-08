# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_range"
# subject = "cpython.test_range.RangeTest.test_range"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_range
"""Auto-ported test: RangeTest::test_range (CPython 3.12 oracle)."""


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

assert list(range(3)) == [0, 1, 2]

assert list(range(1, 5)) == [1, 2, 3, 4]

assert list(range(0)) == []

assert list(range(-3)) == []

assert list(range(1, 10, 3)) == [1, 4, 7]

assert list(range(5, -5, -3)) == [5, 2, -1, -4]
a = 10
b = 100
c = 50

assert list(range(a, a + 2)) == [a, a + 1]

assert list(range(a + 2, a, -1)) == [a + 2, a + 1]

assert list(range(a + 4, a, -2)) == [a + 4, a + 2]
seq = list(range(a, b, c))

assert a in seq

assert b not in seq

assert len(seq) == 2
seq = list(range(b, a, -c))

assert b in seq

assert a not in seq

assert len(seq) == 2
seq = list(range(-a, -b, -c))

assert -a in seq

assert -b not in seq

assert len(seq) == 2

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

try:
    range(0.0, 2, 1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    range(1, 2.0, 1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    range(1, 2, 1.0)
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

assert len(range(0, sys.maxsize, sys.maxsize - 1)) == 2
r = range(-sys.maxsize, sys.maxsize, 2)

assert len(r) == sys.maxsize
print("RangeTest::test_range: ok")
