# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_contains"
# subject = "cpython.test_range.RangeTest.test_contains"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_contains
"""Auto-ported test: RangeTest::test_contains (CPython 3.12 oracle)."""


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
r = range(10)

assert 0 in r

assert 1 in r

assert 5.0 in r

assert 5.1 not in r

assert -1 not in r

assert 10 not in r

assert '' not in r
r = range(9, -1, -1)

assert 0 in r

assert 1 in r

assert 5.0 in r

assert 5.1 not in r

assert -1 not in r

assert 10 not in r

assert '' not in r
r = range(0, 10, 2)

assert 0 in r

assert 1 not in r

assert 5.0 not in r

assert 5.1 not in r

assert -1 not in r

assert 10 not in r

assert '' not in r
r = range(9, -1, -2)

assert 0 not in r

assert 1 in r

assert 5.0 in r

assert 5.1 not in r

assert -1 not in r

assert 10 not in r

assert '' not in r
print("RangeTest::test_contains: ok")
