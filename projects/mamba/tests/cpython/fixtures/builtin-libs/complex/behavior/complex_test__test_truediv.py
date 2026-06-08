# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_truediv"
# subject = "cpython.test_complex.ComplexTest.test_truediv"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_complex.py::ComplexTest::test_truediv
"""Auto-ported test: ComplexTest::test_truediv (CPython 3.12 oracle)."""


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
simple_real = [float(i) for i in range(-5, 6)]
simple_complex = [complex(x, y) for x in simple_real for y in simple_real]
for x in simple_complex:
    for y in simple_complex:
        check_div(x, y)
check_div(complex(1e+200, 1e+200), 1 + 0j)
check_div(complex(1e-200, 1e-200), 1 + 0j)
for i in range(100):
    check_div(complex(random(), random()), complex(random(), random()))

assert abs(complex.__truediv__(2 + 0j, 1 + 1j) - (1 - 1j)) < 1e-07

try:
    operator.truediv(1j, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.truediv(None, 1j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
for denom_real, denom_imag in [(0, NAN), (NAN, 0), (NAN, NAN)]:
    z = complex(0, 0) / complex(denom_real, denom_imag)

    assert isnan(z.real)

    assert isnan(z.imag)
print("ComplexTest::test_truediv: ok")
