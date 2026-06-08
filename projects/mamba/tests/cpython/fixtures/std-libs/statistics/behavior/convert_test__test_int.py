# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "convert_test__test_int"
# subject = "cpython.test_statistics.ConvertTest.test_int"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_statistics.py::ConvertTest::test_int
"""Auto-ported test: ConvertTest::test_int (CPython 3.12 oracle)."""


import bisect
import collections
import collections.abc
import copy
import decimal
import doctest
import itertools
import math
import pickle
import random
import sys
import unittest
from test import support
from test.support import import_helper, requires_IEEE_754
from decimal import Decimal
from fractions import Fraction
import statistics


x = 'Test suite for statistics module, including helper NumericTestCase and\napprox_equal function.\n\n'

def sign(x):
    """Return -1.0 for negatives, including -0.0, otherwise +1.0."""
    return math.copysign(1, x)

def _nan_equal(a, b):
    """Return True if a and b are both the same kind of NAN.

    >>> _nan_equal(Decimal('NAN'), Decimal('NAN'))
    True
    >>> _nan_equal(Decimal('sNAN'), Decimal('sNAN'))
    True
    >>> _nan_equal(Decimal('NAN'), Decimal('sNAN'))
    False
    >>> _nan_equal(Decimal(42), Decimal('NAN'))
    False

    >>> _nan_equal(float('NAN'), float('NAN'))
    True
    >>> _nan_equal(float('NAN'), 0.5)
    False

    >>> _nan_equal(float('NAN'), Decimal('NAN'))
    False

    NAN payloads are not compared.
    """
    if type(a) is not type(b):
        return False
    if isinstance(a, float):
        return math.isnan(a) and math.isnan(b)
    aexp = a.as_tuple()[2]
    bexp = b.as_tuple()[2]
    return aexp == bexp and aexp in ('n', 'N')

def _calc_errors(actual, expected):
    """Return the absolute and relative errors between two numbers.

    >>> _calc_errors(100, 75)
    (25, 0.25)
    >>> _calc_errors(100, 100)
    (0, 0.0)

    Returns the (absolute error, relative error) between the two arguments.
    """
    base = max(abs(actual), abs(expected))
    abs_err = abs(actual - expected)
    rel_err = abs_err / base if base else float('inf')
    return (abs_err, rel_err)

def approx_equal(x, y, tol=1e-12, rel=1e-07):
    """approx_equal(x, y [, tol [, rel]]) => True|False

    Return True if numbers x and y are approximately equal, to within some
    margin of error, otherwise return False. Numbers which compare equal
    will also compare approximately equal.

    x is approximately equal to y if the difference between them is less than
    an absolute error tol or a relative error rel, whichever is bigger.

    If given, both tol and rel must be finite, non-negative numbers. If not
    given, default values are tol=1e-12 and rel=1e-7.

    >>> approx_equal(1.2589, 1.2587, tol=0.0003, rel=0)
    True
    >>> approx_equal(1.2589, 1.2587, tol=0.0001, rel=0)
    False

    Absolute error is defined as abs(x-y); if that is less than or equal to
    tol, x and y are considered approximately equal.

    Relative error is defined as abs((x-y)/x) or abs((x-y)/y), whichever is
    smaller, provided x or y are not zero. If that figure is less than or
    equal to rel, x and y are considered approximately equal.

    Complex numbers are not directly supported. If you wish to compare to
    complex numbers, extract their real and imaginary parts and compare them
    individually.

    NANs always compare unequal, even with themselves. Infinities compare
    approximately equal if they have the same sign (both positive or both
    negative). Infinities with different signs compare unequal; so do
    comparisons of infinities with finite numbers.
    """
    if tol < 0 or rel < 0:
        raise ValueError('error tolerances must be non-negative')
    if math.isnan(x) or math.isnan(y):
        return False
    if x == y:
        return True
    if math.isinf(x) or math.isinf(y):
        return False
    actual_error = abs(x - y)
    allowed_error = max(tol, rel * max(abs(x), abs(y)))
    return actual_error <= allowed_error

class _DoNothing:
    """
    When doing numeric work, especially with floats, exact equality is often
    not what you want. Due to round-off error, it is often a bad idea to try
    to compare floats with equality. Instead the usual procedure is to test
    them with some (hopefully small!) allowance for error.

    The ``approx_equal`` function allows you to specify either an absolute
    error tolerance, or a relative error, or both.

    Absolute error tolerances are simple, but you need to know the magnitude
    of the quantities being compared:

    >>> approx_equal(12.345, 12.346, tol=1e-3)
    True
    >>> approx_equal(12.345e6, 12.346e6, tol=1e-3)  # tol is too small.
    False

    Relative errors are more suitable when the values you are comparing can
    vary in magnitude:

    >>> approx_equal(12.345, 12.346, rel=1e-4)
    True
    >>> approx_equal(12.345e6, 12.346e6, rel=1e-4)
    True

    but a naive implementation of relative error testing can run into trouble
    around zero.

    If you supply both an absolute tolerance and a relative error, the
    comparison succeeds if either individual test succeeds:

    >>> approx_equal(12.345e6, 12.346e6, tol=1e-3, rel=1e-4)
    True

    """
    pass

py_statistics = import_helper.import_fresh_module('statistics', blocked=['_statistics'])

c_statistics = import_helper.import_fresh_module('statistics', fresh=['_statistics'])

def load_tests(loader, tests, ignore):
    """Used for doctest/unittest integration."""
    tests.addTests(doctest.DocTestSuite())
    return tests


# --- test body ---
def check_exact_equal(x, y):
    """Check that x equals y, and has the same type as well."""

    assert x == y

    assert type(x) is type(y)
x = statistics._convert(Fraction(71), int)
check_exact_equal(x, 71)

class MyInt(int):
    pass
x = statistics._convert(Fraction(17), MyInt)
check_exact_equal(x, MyInt(17))
print("ConvertTest::test_int: ok")
