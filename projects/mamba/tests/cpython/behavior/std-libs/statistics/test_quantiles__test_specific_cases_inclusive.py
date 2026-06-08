# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_quantiles__test_specific_cases_inclusive"
# subject = "cpython.test_statistics.TestQuantiles.test_specific_cases_inclusive"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_statistics.py::TestQuantiles::test_specific_cases_inclusive
"""Auto-ported test: TestQuantiles::test_specific_cases_inclusive (CPython 3.12 oracle)."""


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

class NumericTestCase(unittest.TestCase):
    """Unit test class for numeric work.

    This subclasses TestCase. In addition to the standard method
    ``TestCase.assertAlmostEqual``,  ``assertApproxEqual`` is provided.
    """
    tol = rel = 0

    def assertApproxEqual(self, first, second, tol=None, rel=None, msg=None):
        """Test passes if ``first`` and ``second`` are approximately equal.

        This test passes if ``first`` and ``second`` are equal to
        within ``tol``, an absolute error, or ``rel``, a relative error.

        If either ``tol`` or ``rel`` are None or not given, they default to
        test attributes of the same name (by default, 0).

        The objects may be either numbers, or sequences of numbers. Sequences
        are tested element-by-element.

        >>> class MyTest(NumericTestCase):
        ...     def test_number(self):
        ...         x = 1.0/6
        ...         y = sum([x]*6)
        ...         self.assertApproxEqual(y, 1.0, tol=1e-15)
        ...     def test_sequence(self):
        ...         a = [1.001, 1.001e-10, 1.001e10]
        ...         b = [1.0, 1e-10, 1e10]
        ...         self.assertApproxEqual(a, b, rel=1e-3)
        ...
        >>> import unittest
        >>> from io import StringIO  # Suppress test runner output.
        >>> suite = unittest.TestLoader().loadTestsFromTestCase(MyTest)
        >>> unittest.TextTestRunner(stream=StringIO()).run(suite)
        <unittest.runner.TextTestResult run=2 errors=0 failures=0>

        """
        if tol is None:
            tol = self.tol
        if rel is None:
            rel = self.rel
        if isinstance(first, collections.abc.Sequence) and isinstance(second, collections.abc.Sequence):
            check = self._check_approx_seq
        else:
            check = self._check_approx_num
        check(first, second, tol, rel, msg)

    def _check_approx_seq(self, first, second, tol, rel, msg):
        if len(first) != len(second):
            standardMsg = 'sequences differ in length: %d items != %d items' % (len(first), len(second))
            msg = self._formatMessage(msg, standardMsg)
            raise self.failureException(msg)
        for i, (a, e) in enumerate(zip(first, second)):
            self._check_approx_num(a, e, tol, rel, msg, i)

    def _check_approx_num(self, first, second, tol, rel, msg, idx=None):
        if approx_equal(first, second, tol, rel):
            return None
        standardMsg = self._make_std_err_msg(first, second, tol, rel, idx)
        msg = self._formatMessage(msg, standardMsg)
        raise self.failureException(msg)

    @staticmethod
    def _make_std_err_msg(first, second, tol, rel, idx):
        assert first != second
        template = '  %r != %r\n  values differ by more than tol=%r and rel=%r\n  -> absolute error = %r\n  -> relative error = %r'
        if idx is not None:
            header = 'numeric sequences first differ at index %d.\n' % idx
            template = header + template
        abs_err, rel_err = _calc_errors(first, second)
        return template % (first, second, tol, rel, abs_err, rel_err)

def load_tests(loader, tests, ignore):
    """Used for doctest/unittest integration."""
    tests.addTests(doctest.DocTestSuite())
    return tests


# --- test body ---
quantiles = statistics.quantiles
data = [100, 200, 400, 800]
random.shuffle(data)
for n, expected in [(1, []), (2, [300.0]), (3, [200.0, 400.0]), (4, [175.0, 300.0, 500.0]), (5, [160.0, 240.0, 360.0, 560.0]), (6, [150.0, 200.0, 300.0, 400.0, 600.0]), (8, [137.5, 175, 225.0, 300.0, 375.0, 500.0, 650.0]), (10, [130.0, 160.0, 190.0, 240.0, 300.0, 360.0, 440.0, 560.0, 680.0]), (12, [125.0, 150.0, 175.0, 200.0, 250.0, 300.0, 350.0, 400.0, 500.0, 600.0, 700.0]), (15, [120.0, 140.0, 160.0, 180.0, 200.0, 240.0, 280.0, 320.0, 360.0, 400.0, 480.0, 560.0, 640.0, 720.0])]:

    assert expected == quantiles(data, n=n, method='inclusive')

    assert len(quantiles(data, n=n, method='inclusive')) == n - 1
    for datatype in (float, Decimal, Fraction):
        result = quantiles(map(datatype, data), n=n, method='inclusive')

        assert (all(type(x) == datatype) for x in result)

        assert result == list(map(datatype, expected))

    def f(x):
        return 3.5 * x - 1234.675
    exp = list(map(f, expected))
    act = quantiles(map(f, data), n=n, method='inclusive')

    assert all((math.isclose(e, a) for e, a in zip(exp, act)))

assert quantiles([0, 100], n=10, method='inclusive') == [10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]

assert quantiles(range(0, 101), n=10, method='inclusive') == [10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]
data = [random.randrange(10000) for i in range(501)]
actual = quantiles(data, n=32, method='inclusive')
data.remove(min(data))
data.remove(max(data))
expected = quantiles(data, n=32)

assert expected == actual
for k in range(2, 60):
    data = random.choices(range(100), k=k)
    q1, q2, q3 = quantiles(data, method='inclusive')

    assert q2 == statistics.median(data)
print("TestQuantiles::test_specific_cases_inclusive: ok")
