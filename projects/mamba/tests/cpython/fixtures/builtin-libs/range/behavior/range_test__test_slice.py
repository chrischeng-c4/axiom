# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_slice"
# subject = "cpython.test_range.RangeTest.test_slice"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_slice
"""Auto-ported test: RangeTest::test_slice (CPython 3.12 oracle)."""


import sys


def check(r, start, stop, step=None):
    index = slice(start, stop, step)
    assert list(r[index]) == list(r)[index]
    assert len(r[index]) == len(list(r)[index])


for candidate in [
    range(10),
    range(0),
    range(1, 9, 3),
    range(8, 0, -3),
    range(sys.maxsize + 1, sys.maxsize + 10),
]:
    check(candidate, 0, 2)
    check(candidate, 0, 20)
    check(candidate, 1, 2)
    check(candidate, 20, 30)
    check(candidate, -30, -20)
    check(candidate, -1, 100, 2)
    check(candidate, 0, -1)
    check(candidate, -1, -3, -1)

print("RangeTest::test_slice: ok")
