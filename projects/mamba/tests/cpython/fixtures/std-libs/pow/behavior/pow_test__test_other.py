# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pow"
# dimension = "behavior"
# case = "pow_test__test_other"
# subject = "cpython.test_pow.PowTest.test_other"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pow.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pow.py::PowTest::test_other
"""Auto-ported test: PowTest::test_other (CPython 3.12 oracle)."""


import math
import unittest


# --- test body ---

assert pow(3, 3) % 8 == pow(3, 3, 8)

assert pow(3, 3) % -8 == pow(3, 3, -8)

assert pow(3, 2) % -2 == pow(3, 2, -2)

assert pow(-3, 3) % 8 == pow(-3, 3, 8)

assert pow(-3, 3) % -8 == pow(-3, 3, -8)

assert pow(5, 2) % -8 == pow(5, 2, -8)

assert pow(3, 3) % 8 == pow(3, 3, 8)

assert pow(3, 3) % -8 == pow(3, 3, -8)

assert pow(3, 2) % -2 == pow(3, 2, -2)

assert pow(-3, 3) % 8 == pow(-3, 3, 8)

assert pow(-3, 3) % -8 == pow(-3, 3, -8)

assert pow(5, 2) % -8 == pow(5, 2, -8)
for i in range(-10, 11):
    for j in range(0, 6):
        for k in range(-7, 11):
            if j >= 0 and k != 0:

                assert pow(i, j) % k == pow(i, j, k)
            if j >= 0 and k != 0:

                assert pow(int(i), j) % k == pow(int(i), j, k)
print("PowTest::test_other: ok")
