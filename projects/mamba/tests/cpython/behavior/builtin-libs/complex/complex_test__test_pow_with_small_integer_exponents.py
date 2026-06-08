# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_pow_with_small_integer_exponents"
# subject = "cpython.test_complex.ComplexTest.test_pow_with_small_integer_exponents"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_complex.py::ComplexTest::test_pow_with_small_integer_exponents
"""Auto-ported test: ComplexTest::test_pow_with_small_integer_exponents (CPython 3.12 oracle)."""


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
def assertAlmostEqual(a, b):
    if isinstance(a, complex):
        if isinstance(b, complex):
            unittest.TestCase.assertAlmostEqual(self, a.real, b.real)
            unittest.TestCase.assertAlmostEqual(self, a.imag, b.imag)
        else:
            unittest.TestCase.assertAlmostEqual(self, a.real, b)
            unittest.TestCase.assertAlmostEqual(self, a.imag, 0.0)
    elif isinstance(b, complex):
        unittest.TestCase.assertAlmostEqual(self, a, b.real)
        unittest.TestCase.assertAlmostEqual(self, 0.0, b.imag)
    else:
        unittest.TestCase.assertAlmostEqual(self, a, b)

def assertClose(x, y, eps=1e-09):
    """Return true iff complexes x and y "are close"."""
    assertCloseAbs(x.real, y.real, eps)
    assertCloseAbs(x.imag, y.imag, eps)

def assertCloseAbs(x, y, eps=1e-09):
    """Return true iff floats x and y "are close"."""
    if abs(x) > abs(y):
        x, y = (y, x)
    if y == 0:
        return abs(x) < eps
    if x == 0:
        return abs(y) < eps

    assert abs((x - y) / y) < eps

def check_div(x, y):
    """Compute complex z=x*y, and check that z/x==y and z/y==x."""
    z = x * y
    if x != 0:
        q = z / x
        assertClose(q, y)
        q = z.__truediv__(x)
        assertClose(q, y)
    if y != 0:
        q = z / y
        assertClose(q, x)
        q = z.__truediv__(y)
        assertClose(q, x)
values = [complex(5.0, 12.0), complex(5e+100, 1.2e+101), complex(-4.0, INF), complex(INF, 0.0)]
exponents = [-19, -5, -3, -2, -1, 0, 1, 2, 3, 5, 19]
for value in values:
    for exponent in exponents:
        try:
            int_pow = value ** exponent
        except OverflowError:
            int_pow = 'overflow'
        try:
            float_pow = value ** float(exponent)
        except OverflowError:
            float_pow = 'overflow'
        try:
            complex_pow = value ** complex(exponent)
        except OverflowError:
            complex_pow = 'overflow'

        assert str(float_pow) == str(int_pow)

        assert str(complex_pow) == str(int_pow)
print("ComplexTest::test_pow_with_small_integer_exponents: ok")
