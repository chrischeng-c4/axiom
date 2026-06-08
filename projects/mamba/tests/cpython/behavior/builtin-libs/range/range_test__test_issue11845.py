# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_issue11845"
# subject = "cpython.test_range.RangeTest.test_issue11845"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_issue11845
"""Auto-ported test: RangeTest::test_issue11845 (CPython 3.12 oracle)."""


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
r = range(*slice(1, 18, 2).indices(20))
values = {None, 0, 1, -1, 2, -2, 5, -5, 19, -19, 20, -20, 21, -21, 30, -30, 99, -99}
for i in values:
    for j in values:
        for k in values - {0}:
            r[i:j:k]
print("RangeTest::test_issue11845: ok")
