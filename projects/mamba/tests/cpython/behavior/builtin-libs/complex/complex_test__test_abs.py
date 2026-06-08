# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_abs"
# subject = "cpython.test_complex.ComplexTest.test_abs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_complex.py::ComplexTest::test_abs
"""Auto-ported test: ComplexTest::test_abs (CPython 3.12 oracle)."""


import unittest
import sys
from test import support
from test.support.testcase import ComplexesAreIdenticalMixin
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS
from random import random
from math import isnan, copysign
import operator


INF = float('inf')

NAN = float('nan')

ZERO_DIVISION = ((1 + 1j, 0 + 0j), (1 + 1j, 0.0), (1 + 1j, 0), (1.0, 0 + 0j), (1, 0 + 0j))

class WithIndex:

    def __init__(self, value):
        self.value = value

    def __index__(self):
        return self.value

class WithFloat:

    def __init__(self, value):
        self.value = value

    def __float__(self):
        return self.value

class ComplexSubclass(complex):
    pass

class WithComplex:

    def __init__(self, value):
        self.value = value

    def __complex__(self):
        return self.value


# --- test body ---
nums = [complex(x / 3.0, y / 7.0) for x in range(-9, 9) for y in range(-9, 9)]
for num in nums:

    assert abs((num.real ** 2 + num.imag ** 2) ** 0.5 - abs(num)) < 1e-07
print("ComplexTest::test_abs: ok")
