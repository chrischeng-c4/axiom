# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_pow"
# subject = "cpython.test_complex.ComplexTest.test_pow"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_complex.py::ComplexTest::test_pow
"""Auto-ported test: ComplexTest::test_pow (CPython 3.12 oracle)."""


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

assert abs(pow(1 + 1j, 0 + 0j) - 1.0) < 1e-07

assert abs(pow(0 + 0j, 2 + 0j) - 0.0) < 1e-07

assert pow(0 + 0j, 2000 + 0j) == 0.0

assert pow(0, 0 + 0j) == 1.0

assert pow(-1, 0 + 0j) == 1.0

try:
    pow(0 + 0j, 1j)
    raise AssertionError('expected ZeroDivisionError')
except ZeroDivisionError:
    pass

try:
    pow(0 + 0j, -1000)
    raise AssertionError('expected ZeroDivisionError')
except ZeroDivisionError:
    pass

assert abs(pow(1j, -1) - 1 / 1j) < 1e-07

assert abs(pow(1j, 200) - 1) < 1e-07

try:
    pow(1 + 1j, 1 + 1j, 1 + 1j)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    pow(1e+200 + 1j, 1e+200 + 1j)
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass

try:
    pow(1j, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    pow(None, 1j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert abs(pow(1j, 0.5) - (0.7071067811865476 + 0.7071067811865475j)) < 1e-07
a = 3.33 + 4.43j

assert a ** 0j == 1

assert a ** 0.0 + 0j == 1

assert 3j ** 0j == 1

assert 3j ** 0 == 1
try:
    0j ** a
except ZeroDivisionError:
    pass
else:

    raise AssertionError('should fail 0.0 to negative or complex power')
try:
    0j ** (3 - 2j)
except ZeroDivisionError:
    pass
else:

    raise AssertionError('should fail 0.0 to negative or complex power')

assert a ** 105 == a ** 105

assert a ** (-105) == a ** (-105)

assert a ** (-30) == a ** (-30)

assert 0j ** 0 == 1
b = 5.1 + 2.3j

try:
    pow(a, b, 0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
values = (sys.maxsize, sys.maxsize + 1, sys.maxsize - 1, -sys.maxsize, -sys.maxsize + 1, -sys.maxsize + 1)
for real in values:
    for imag in values:
        c = complex(real, imag)
        try:
            c ** real
        except OverflowError:
            pass
        try:
            c ** c
        except OverflowError:
            pass
x, y = (9j, 33j ** 3)
try:
    x ** y
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass
print("ComplexTest::test_pow: ok")
