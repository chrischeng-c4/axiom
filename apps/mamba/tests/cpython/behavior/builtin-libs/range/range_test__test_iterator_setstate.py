# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_iterator_setstate"
# subject = "cpython.test_range.RangeTest.test_iterator_setstate"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_iterator_setstate
"""Auto-ported test: RangeTest::test_iterator_setstate (CPython 3.12 oracle)."""


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
it = iter(range(10, 20, 2))
it.__setstate__(2)

assert list(it) == [14, 16, 18]
it = reversed(range(10, 20, 2))
it.__setstate__(3)

assert list(it) == [12, 10]
it = iter(range(-2 ** 65, 20, 2))
it.__setstate__(2 ** 64 + 7)

assert list(it) == [14, 16, 18]
it = reversed(range(10, 2 ** 65, 2))
it.__setstate__(2 ** 64 - 7)

assert list(it) == [12, 10]
print("RangeTest::test_iterator_setstate: ok")
