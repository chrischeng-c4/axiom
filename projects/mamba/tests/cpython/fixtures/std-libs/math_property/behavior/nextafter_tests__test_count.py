# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math_property"
# dimension = "behavior"
# case = "nextafter_tests__test_count"
# subject = "cpython.test_math_property.NextafterTests.test_count"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_math_property.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_math_property.py::NextafterTests::test_count
"""Auto-ported test: NextafterTests::test_count."""


import functools
from math import isnan, nextafter


def assert_equal_float(x, y):
    assert (isnan(x) and isnan(y)) or x == y


def via_reduce(x, y, steps):
    return functools.reduce(nextafter, [y] * steps, x)


for x, y, steps in [
    (0.0, 1.0, 0),
    (0.0, 1.0, 1),
    (1.0, 2.0, 4),
    (-1.0, 0.0, 3),
    (42.0, -100.0, 8),
]:
    assert_equal_float(via_reduce(x, y, steps), nextafter(x, y, steps=steps))

print("NextafterTests::test_count: ok")
