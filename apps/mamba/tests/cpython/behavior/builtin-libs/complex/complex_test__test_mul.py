# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_mul"
# subject = "cpython.test_complex.ComplexTest.test_mul"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_complex.py::ComplexTest::test_mul
"""Auto-ported test: ComplexTest::test_mul (CPython 3.12 oracle)."""


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

assert 1j * int(20) == complex(0, 20)

assert 1j * int(-1) == complex(0, -1)

try:
    operator.mul(1j, 10 ** 1000)
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass

try:
    operator.mul(1j, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.mul(None, 1j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ComplexTest::test_mul: ok")
