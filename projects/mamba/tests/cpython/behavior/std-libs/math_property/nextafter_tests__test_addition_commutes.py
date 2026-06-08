# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math_property"
# dimension = "behavior"
# case = "nextafter_tests__test_addition_commutes"
# subject = "cpython.test_math_property.NextafterTests.test_addition_commutes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_math_property.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_math_property.py::NextafterTests::test_addition_commutes
"""Auto-ported test: NextafterTests::test_addition_commutes."""


from math import isnan, nextafter


def assert_equal_float(x, y):
    assert (isnan(x) and isnan(y)) or x == y


for x, y, a, b in [
    (0.0, 1.0, 0, 0),
    (0.0, 1.0, 1, 2),
    (1.0, 2.0, 4, 5),
    (-1.0, 0.0, 3, 4),
    (42.0, -100.0, 8, 13),
]:
    first = nextafter(x, y, steps=a)
    second = nextafter(first, y, steps=b)
    combined = nextafter(x, y, steps=a + b)
    assert_equal_float(second, combined)

print("NextafterTests::test_addition_commutes: ok")
