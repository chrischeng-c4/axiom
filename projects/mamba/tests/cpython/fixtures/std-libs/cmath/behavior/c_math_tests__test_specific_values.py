# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "c_math_tests__test_specific_values"
# subject = "cpython.test_cmath.CMathTests.test_specific_values"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmath.py::CMathTests::test_specific_values
"""Auto-ported test: CMathTests::test_specific_values (CPython 3.12 oracle)."""


from test.support import requires_IEEE_754, cpython_only, import_helper
from test.support.testcase import ComplexesAreIdenticalMixin
from test.test_math import parse_testfile, test_file
import test.test_math as test_math
import unittest
import cmath, math
from cmath import phase, polar, rect, pi
import platform
import sys


INF = float('inf')

NAN = float('nan')

complex_zeros = [complex(x, y) for x in [0.0, -0.0] for y in [0.0, -0.0]]

complex_infinities = [complex(x, y) for x, y in [(INF, 0.0), (INF, 2.3), (INF, INF), (2.3, INF), (0.0, INF), (-0.0, INF), (-2.3, INF), (-INF, INF), (-INF, 2.3), (-INF, 0.0), (-INF, -0.0), (-INF, -2.3), (-INF, -INF), (-2.3, -INF), (-0.0, -INF), (0.0, -INF), (2.3, -INF), (INF, -INF), (INF, -2.3), (INF, -0.0)]]

complex_nans = [complex(x, y) for x, y in [(NAN, -INF), (NAN, -2.3), (NAN, -0.0), (NAN, 0.0), (NAN, 2.3), (NAN, INF), (-INF, NAN), (-2.3, NAN), (-0.0, NAN), (0.0, NAN), (2.3, NAN), (INF, NAN)]]


# --- test body ---
test_functions = [getattr(cmath, fname) for fname in ['acos', 'acosh', 'asin', 'asinh', 'atan', 'atanh', 'cos', 'cosh', 'exp', 'log', 'log10', 'sin', 'sinh', 'sqrt', 'tan', 'tanh']]

def assertCEqual(a, b):
    eps = 1e-07
    if abs(a.real - b[0]) > eps or abs(a.imag - b[1]) > eps:

        raise AssertionError((a, b))

def check_polar(func):

    def check(arg, expected):
        got = func(arg)
        for e, g in zip(expected, got):
            self.rAssertAlmostEqual(e, g)
    check(0, (0.0, 0.0))
    check(1, (1.0, 0.0))
    check(-1, (1.0, pi))
    check(1j, (1.0, pi / 2))
    check(-3j, (3.0, -pi / 2))
    inf = float('inf')
    check(complex(inf, 0), (inf, 0.0))
    check(complex(-inf, 0), (inf, pi))
    check(complex(3, inf), (inf, pi / 2))
    check(complex(5, -inf), (inf, -pi / 2))
    check(complex(inf, inf), (inf, pi / 4))
    check(complex(inf, -inf), (inf, -pi / 4))
    check(complex(-inf, inf), (inf, 3 * pi / 4))
    check(complex(-inf, -inf), (inf, -3 * pi / 4))
    nan = float('nan')
    check(complex(nan, 0), (nan, nan))
    check(complex(0, nan), (nan, nan))
    check(complex(nan, nan), (nan, nan))
    check(complex(inf, nan), (inf, nan))
    check(complex(-inf, nan), (inf, nan))
    check(complex(nan, inf), (inf, nan))
    check(complex(nan, -inf), (inf, nan))

def rAssertAlmostEqual(a, b, rel_err=2e-15, abs_err=5e-323, msg=None):
    """Fail if the two floating-point numbers are not almost equal.

        Determine whether floating-point values a and b are equal to within
        a (small) rounding error.  The default values for rel_err and
        abs_err are chosen to be suitable for platforms where a float is
        represented by an IEEE 754 double.  They allow an error of between
        9 and 19 ulps.
        """
    if math.isnan(a):
        if math.isnan(b):
            return

        raise AssertionError(msg or '{!r} should be nan'.format(b))
    if math.isinf(a):
        if a == b:
            return

        raise AssertionError(msg or 'finite result where infinity expected: expected {!r}, got {!r}'.format(a, b))
    if not a and (not b):
        if math.copysign(1.0, a) != math.copysign(1.0, b):

            raise AssertionError(msg or 'zero has wrong sign: expected {!r}, got {!r}'.format(a, b))
    try:
        absolute_error = abs(b - a)
    except OverflowError:
        pass
    else:
        if absolute_error <= max(abs_err, rel_err * abs(a)):
            return

    raise AssertionError(msg or '{!r} and {!r} are not sufficiently close'.format(a, b))
self_test_values = open(test_file, encoding='utf-8')
SKIP_ON_TIGER = {'tan0064'}
osx_version = None
if sys.platform == 'darwin':
    version_txt = platform.mac_ver()[0]
    try:
        osx_version = tuple(map(int, version_txt.split('.')))
    except ValueError:
        pass

def rect_complex(z):
    """Wrapped version of rect that accepts a complex number instead of
            two float arguments."""
    return cmath.rect(z.real, z.imag)

def polar_complex(z):
    """Wrapped version of polar that returns a complex number instead of
            two floats."""
    return complex(*polar(z))
for id, fn, ar, ai, er, ei, flags in parse_testfile(test_file):
    arg = complex(ar, ai)
    expected = complex(er, ei)
    if osx_version is not None and osx_version < (10, 5):
        if id in SKIP_ON_TIGER:
            continue
    if fn == 'rect':
        function = rect_complex
    elif fn == 'polar':
        function = polar_complex
    else:
        function = getattr(cmath, fn)
    if 'divide-by-zero' in flags or 'invalid' in flags:
        try:
            actual = function(arg)
        except ValueError:
            continue
        else:

            raise AssertionError('ValueError not raised in test {}: {}(complex({!r}, {!r}))'.format(id, fn, ar, ai))
    if 'overflow' in flags:
        try:
            actual = function(arg)
        except OverflowError:
            continue
        else:

            raise AssertionError('OverflowError not raised in test {}: {}(complex({!r}, {!r}))'.format(id, fn, ar, ai))
    actual = function(arg)
    if 'ignore-real-sign' in flags:
        actual = complex(abs(actual.real), actual.imag)
        expected = complex(abs(expected.real), expected.imag)
    if 'ignore-imag-sign' in flags:
        actual = complex(actual.real, abs(actual.imag))
        expected = complex(expected.real, abs(expected.imag))
    if fn in ('log', 'log10'):
        real_abs_err = 2e-15
    else:
        real_abs_err = 5e-323
    error_message = '{}: {}(complex({!r}, {!r}))\nExpected: complex({!r}, {!r})\nReceived: complex({!r}, {!r})\nReceived value insufficiently close to expected value.'.format(id, fn, ar, ai, expected.real, expected.imag, actual.real, actual.imag)
    rAssertAlmostEqual(expected.real, actual.real, abs_err=real_abs_err, msg=error_message)
    rAssertAlmostEqual(expected.imag, actual.imag, msg=error_message)
print("CMathTests::test_specific_values: ok")
