# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_large_operands"
# subject = "cpython.test_range.RangeTest.test_large_operands"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_large_operands
"""Auto-ported test: RangeTest::test_large_operands (CPython 3.12 oracle)."""


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
x = range(10 ** 20, 10 ** 20 + 10, 3)

assert len(x) == 4

assert len(list(x)) == 4
x = range(10 ** 20 + 10, 10 ** 20, 3)

assert len(x) == 0

assert len(list(x)) == 0

assert not x
x = range(10 ** 20, 10 ** 20 + 10, -3)

assert len(x) == 0

assert len(list(x)) == 0

assert not x
x = range(10 ** 20 + 10, 10 ** 20, -3)

assert len(x) == 4

assert len(list(x)) == 4

assert x
for x in [range(-2 ** 100), range(0, -2 ** 100), range(0, 2 ** 100, -1)]:

    assert list(x) == []

    assert not x
a = int(10 * sys.maxsize)
b = int(100 * sys.maxsize)
c = int(50 * sys.maxsize)

assert list(range(a, a + 2)) == [a, a + 1]

assert list(range(a + 2, a, -1)) == [a + 2, a + 1]

assert list(range(a + 4, a, -2)) == [a + 4, a + 2]
seq = list(range(a, b, c))

assert a in seq

assert b not in seq

assert len(seq) == 2

assert seq[0] == a

assert seq[-1] == a + c
seq = list(range(b, a, -c))

assert b in seq

assert a not in seq

assert len(seq) == 2

assert seq[0] == b

assert seq[-1] == b - c
seq = list(range(-a, -b, -c))

assert -a in seq

assert -b not in seq

assert len(seq) == 2

assert seq[0] == -a

assert seq[-1] == -a - c
print("RangeTest::test_large_operands: ok")
