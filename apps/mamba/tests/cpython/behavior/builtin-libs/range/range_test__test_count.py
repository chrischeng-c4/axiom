# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_count"
# subject = "cpython.test_range.RangeTest.test_count"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_count
"""Auto-ported test: RangeTest::test_count (CPython 3.12 oracle)."""


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

assert range(3).count(-1) == 0

assert range(3).count(0) == 1

assert range(3).count(1) == 1

assert range(3).count(2) == 1

assert range(3).count(3) == 0

assert type(range(3).count(-1)) is int

assert type(range(3).count(1)) is int

assert range(10 ** 20).count(1) == 1

assert range(10 ** 20).count(10 ** 20) == 0

assert range(3).index(1) == 1

assert range(1, 2 ** 100, 2).count(2 ** 87) == 0

assert range(1, 2 ** 100, 2).count(2 ** 87 + 1) == 1

assert range(10).count(ALWAYS_EQ) == 10

assert len(range(sys.maxsize, sys.maxsize + 10)) == 10
print("RangeTest::test_count: ok")
