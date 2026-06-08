# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_large_exhausted_iterator_pickling"
# subject = "cpython.test_range.RangeTest.test_large_exhausted_iterator_pickling"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_large_exhausted_iterator_pickling
"""Auto-ported test: RangeTest::test_large_exhausted_iterator_pickling (CPython 3.12 oracle)."""


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
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    r = range(20)
    i = iter(r)
    while True:
        r = next(i)
        if r == 19:
            break
    d = pickle.dumps(i, proto)
    i2 = pickle.loads(d)

    assert list(i) == []

    assert list(i2) == []
print("RangeTest::test_large_exhausted_iterator_pickling: ok")
