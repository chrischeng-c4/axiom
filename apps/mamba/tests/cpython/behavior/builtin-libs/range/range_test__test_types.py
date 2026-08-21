# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_types"
# subject = "cpython.test_range.RangeTest.test_types"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_types
"""Auto-ported test: RangeTest::test_types (CPython 3.12 oracle)."""


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

assert 1.0 in range(3)

assert True in range(3)

assert 1 + 0j in range(3)

assert ALWAYS_EQ in range(3)

class C2:

    def __int__(self):
        return 1

    def __index__(self):
        return 1

assert C2() not in range(3)

assert int(C2()) in range(3)

class C3(int):

    def __eq__(self, other):
        return True

assert C3(11) in range(10)

assert C3(11) in list(range(10))
print("RangeTest::test_types: ok")
