# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_large_range"
# subject = "cpython.test_range.RangeTest.test_large_range"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_large_range
"""Auto-ported test: RangeTest::test_large_range (CPython 3.12 oracle)."""


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
def _range_len(x):
    try:
        length = len(x)
    except OverflowError:
        step = x[1] - x[0]
        length = 1 + (x[-1] - x[0]) // step
    return length
a = -sys.maxsize
b = sys.maxsize
expected_len = b - a
x = range(a, b)

assert a in x

assert b not in x

try:
    len(x)
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass

assert x

assert _range_len(x) == expected_len

assert x[0] == a
idx = sys.maxsize + 1

assert x[idx] == a + idx

assert x[idx:idx + 1][0] == a + idx
try:
    x[-expected_len - 1]
    raise AssertionError('expected IndexError')
except IndexError:
    pass
try:
    x[expected_len]
    raise AssertionError('expected IndexError')
except IndexError:
    pass
a = 0
b = 2 * sys.maxsize
expected_len = b - a
x = range(a, b)

assert a in x

assert b not in x

try:
    len(x)
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass

assert x

assert _range_len(x) == expected_len

assert x[0] == a
idx = sys.maxsize + 1

assert x[idx] == a + idx

assert x[idx:idx + 1][0] == a + idx
try:
    x[-expected_len - 1]
    raise AssertionError('expected IndexError')
except IndexError:
    pass
try:
    x[expected_len]
    raise AssertionError('expected IndexError')
except IndexError:
    pass
a = 0
b = sys.maxsize ** 10
c = 2 * sys.maxsize
expected_len = 1 + (b - a) // c
x = range(a, b, c)

assert a in x

assert b not in x

try:
    len(x)
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass

assert x

assert _range_len(x) == expected_len

assert x[0] == a
idx = sys.maxsize + 1

assert x[idx] == a + idx * c

assert x[idx:idx + 1][0] == a + idx * c
try:
    x[-expected_len - 1]
    raise AssertionError('expected IndexError')
except IndexError:
    pass
try:
    x[expected_len]
    raise AssertionError('expected IndexError')
except IndexError:
    pass
a = sys.maxsize ** 10
b = 0
c = -2 * sys.maxsize
expected_len = 1 + (b - a) // c
x = range(a, b, c)

assert a in x

assert b not in x

try:
    len(x)
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass

assert x

assert _range_len(x) == expected_len

assert x[0] == a
idx = sys.maxsize + 1

assert x[idx] == a + idx * c

assert x[idx:idx + 1][0] == a + idx * c
try:
    x[-expected_len - 1]
    raise AssertionError('expected IndexError')
except IndexError:
    pass
try:
    x[expected_len]
    raise AssertionError('expected IndexError')
except IndexError:
    pass
print("RangeTest::test_large_range: ok")
