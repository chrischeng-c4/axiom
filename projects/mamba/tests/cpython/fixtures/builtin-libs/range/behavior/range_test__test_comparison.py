# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_comparison"
# subject = "cpython.test_range.RangeTest.test_comparison"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_comparison
"""Auto-ported test: RangeTest::test_comparison (CPython 3.12 oracle)."""


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
test_ranges = [range(0), range(0, -1), range(1, 1, 3), range(1), range(5, 6), range(5, 6, 2), range(5, 7, 2), range(2), range(0, 4, 2), range(0, 5, 2), range(0, 6, 2)]
test_tuples = list(map(tuple, test_ranges))
ranges_eq = [a == b for a in test_ranges for b in test_ranges]
tuples_eq = [a == b for a in test_tuples for b in test_tuples]

assert ranges_eq == tuples_eq
ranges_ne = [a != b for a in test_ranges for b in test_ranges]

assert ranges_ne == [not x for x in ranges_eq]
for a in test_ranges:
    for b in test_ranges:
        if a == b:

            assert hash(a) == hash(b)

assert (range(0) == ()) is False

assert (() == range(0)) is False

assert (range(2) == [0, 1]) is False

assert range(0, 2 ** 100 - 1, 2) == range(0, 2 ** 100, 2)

assert hash(range(0, 2 ** 100 - 1, 2)) == hash(range(0, 2 ** 100, 2))

assert range(0, 2 ** 100, 2) != range(0, 2 ** 100 + 1, 2)

assert range(2 ** 200, 2 ** 201 - 2 ** 99, 2 ** 100) == range(2 ** 200, 2 ** 201, 2 ** 100)

assert hash(range(2 ** 200, 2 ** 201 - 2 ** 99, 2 ** 100)) == hash(range(2 ** 200, 2 ** 201, 2 ** 100))

assert range(2 ** 200, 2 ** 201, 2 ** 100) != range(2 ** 200, 2 ** 201 + 1, 2 ** 100)
try:
    range(0) < range(0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    range(0) > range(0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    range(0) <= range(0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    range(0) >= range(0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("RangeTest::test_comparison: ok")
