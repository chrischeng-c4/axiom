# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "approx_equal_inexact_test__test_approx_equal_relative_fractions"
# subject = "cpython.test_statistics.ApproxEqualInexactTest.test_approx_equal_relative_fractions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_statistics.py::ApproxEqualInexactTest::test_approx_equal_relative_fractions
"""Auto-ported test: ApproxEqualInexactTest::test_approx_equal_relative_fractions (CPython 3.12 oracle)."""


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
def do_approx_equal_abs_test(x, delta):
    template = 'Test failure for x={!r}, y={!r}'
    for y in (x + delta, x - delta):
        msg = template.format(x, y)

        assert approx_equal(x, y, tol=2 * delta, rel=0)

        assert not approx_equal(x, y, tol=delta / 2, rel=0)

def do_approx_equal_rel_test(x, delta):
    template = 'Test failure for x={!r}, y={!r}'
    for y in (x * (1 + delta), x * (1 - delta)):
        msg = template.format(x, y)

        assert approx_equal(x, y, tol=0, rel=2 * delta)

        assert not approx_equal(x, y, tol=0, rel=delta / 2)

def do_check_both(a, b, tol, rel, tol_flag, rel_flag):
    check = self_assertTrue if tol_flag else self_assertFalse
    check(approx_equal(a, b, tol=tol, rel=0))
    check = self_assertTrue if rel_flag else self_assertFalse
    check(approx_equal(a, b, tol=0, rel=rel))
    check = self_assertTrue if tol_flag or rel_flag else self_assertFalse
    check(approx_equal(a, b, tol=tol, rel=rel))
F = Fraction
delta = Fraction(3, 8)
for f in [F(3, 84), F(17, 30), F(49, 50), F(92, 85)]:
    for d in (delta, float(delta)):
        do_approx_equal_rel_test(f, d)
        do_approx_equal_rel_test(-f, d)
print("ApproxEqualInexactTest::test_approx_equal_relative_fractions: ok")
