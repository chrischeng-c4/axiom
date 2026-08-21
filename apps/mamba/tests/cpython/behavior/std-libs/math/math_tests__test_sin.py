# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "math_tests__test_sin"
# subject = "cpython.test_math.MathTests.testSin"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_math.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_math.py::MathTests::testSin
"""Auto-ported test: MathTests::testSin (CPython 3.12 oracle)."""


from test.support import verbose, requires_IEEE_754
from test import support
import unittest
import fractions
import itertools
import decimal
import math
import os
import platform
import random
import struct
import sys


eps = 1e-05

NAN = float('nan')

INF = float('inf')

NINF = float('-inf')

FLOAT_MAX = sys.float_info.max

FLOAT_MIN = sys.float_info.min

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

file = __file__

test_dir = os.path.dirname(file) or os.curdir

math_testcases = os.path.join(test_dir, 'math_testcases.txt')

test_file = os.path.join(test_dir, 'cmath_testcases.txt')

def to_ulps(x):
    """Convert a non-NaN float x to an integer, in such a way that
    adjacent floats are converted to adjacent integers.  Then
    abs(ulps(x) - ulps(y)) gives the difference in ulps between two
    floats.

    The results from this function will only make sense on platforms
    where native doubles are represented in IEEE 754 binary64 format.

    Note: 0.0 and -0.0 are converted to 0 and -1, respectively.
    """
    n = struct.unpack('<q', struct.pack('<d', x))[0]
    if n < 0:
        n = ~(n + 2 ** 63)
    return n

def count_set_bits(n):
    """Number of '1' bits in binary expansion of a nonnnegative integer."""
    return 1 + count_set_bits(n & n - 1) if n else 0

def partial_product(start, stop):
    """Product of integers in range(start, stop, 2), computed recursively.
    start and stop should both be odd, with start <= stop.

    """
    numfactors = stop - start >> 1
    if not numfactors:
        return 1
    elif numfactors == 1:
        return start
    else:
        mid = start + numfactors | 1
        return partial_product(start, mid) * partial_product(mid, stop)

def py_factorial(n):
    """Factorial of nonnegative integer n, via "Binary Split Factorial Formula"
    described at http://www.luschny.de/math/factorial/binarysplitfact.html

    """
    inner = outer = 1
    for i in reversed(range(n.bit_length())):
        inner *= partial_product((n >> i + 1) + 1 | 1, (n >> i) + 1 | 1)
        outer *= inner
    return outer << n - count_set_bits(n)

def ulp_abs_check(expected, got, ulp_tol, abs_tol):
    """Given finite floats `expected` and `got`, check that they're
    approximately equal to within the given number of ulps or the
    given absolute tolerance, whichever is bigger.

    Returns None on success and an error message on failure.
    """
    ulp_error = abs(to_ulps(expected) - to_ulps(got))
    abs_error = abs(expected - got)
    if abs_error <= abs_tol or ulp_error <= ulp_tol:
        return None
    else:
        fmt = 'error = {:.3g} ({:d} ulps); permitted error = {:.3g} or {:d} ulps'
        return fmt.format(abs_error, ulp_error, abs_tol, ulp_tol)

def parse_mtestfile(fname):
    """Parse a file with test values

    -- starts a comment
    blank lines, or lines containing only a comment, are ignored
    other lines are expected to have the form
      id fn arg -> expected [flag]*

    """
    with open(fname, encoding='utf-8') as fp:
        for line in fp:
            if '--' in line:
                line = line[:line.index('--')]
            if not line.strip():
                continue
            lhs, rhs = line.split('->')
            id, fn, arg = lhs.split()
            rhs_pieces = rhs.split()
            exp = rhs_pieces[0]
            flags = rhs_pieces[1:]
            yield (id, fn, float(arg), float(exp), flags)

def parse_testfile(fname):
    """Parse a file with test values

    Empty lines or lines starting with -- are ignored
    yields id, fn, arg_real, arg_imag, exp_real, exp_imag
    """
    with open(fname, encoding='utf-8') as fp:
        for line in fp:
            if line.startswith('--') or not line.strip():
                continue
            lhs, rhs = line.split('->')
            id, fn, arg_real, arg_imag = lhs.split()
            rhs_pieces = rhs.split()
            exp_real, exp_imag = (rhs_pieces[0], rhs_pieces[1])
            flags = rhs_pieces[2:]
            yield (id, fn, float(arg_real), float(arg_imag), float(exp_real), float(exp_imag), flags)

def result_check(expected, got, ulp_tol=5, abs_tol=0.0):
    """Compare arguments expected and got, as floats, if either
    is a float, using a tolerance expressed in multiples of
    ulp(expected) or absolutely (if given and greater).

    As a convenience, when neither argument is a float, and for
    non-finite floats, exact equality is demanded. Also, nan==nan
    as far as this function is concerned.

    Returns None on success and an error message on failure.
    """
    if got == expected:
        if not got and (not expected):
            if math.copysign(1, got) != math.copysign(1, expected):
                return f'expected {expected}, got {got} (zero has wrong sign)'
        return None
    failure = 'not equal'
    if isinstance(expected, float) and isinstance(got, int):
        got = float(got)
    elif isinstance(got, float) and isinstance(expected, int):
        expected = float(expected)
    if isinstance(expected, float) and isinstance(got, float):
        if math.isnan(expected) and math.isnan(got):
            failure = None
        elif math.isinf(expected) or math.isinf(got):
            pass
        else:
            failure = ulp_abs_check(expected, got, ulp_tol, abs_tol)
    if failure is not None:
        fail_fmt = 'expected {!r}, got {!r}'
        fail_msg = fail_fmt.format(expected, got)
        fail_msg += ' ({})'.format(failure)
        return fail_msg
    else:
        return None

class FloatLike:

    def __init__(self, value):
        self.value = value

    def __float__(self):
        return self.value

class IntSubclass(int):
    pass

class MyIndexable(object):

    def __init__(self, value):
        self.value = value

    def __index__(self):
        return self.value

class BadDescr:

    def __get__(self, obj, objtype=None):
        raise ValueError

def load_tests(loader, tests, pattern):
    from doctest import DocFileSuite
    tests.addTest(DocFileSuite('ieee754.txt'))
    return tests


# --- test body ---
def assertEqualSign(x, y):
    """Similar to assertEqual(), but compare also the sign with copysign().

        Function useful to compare signed zeros.
        """

    assert x == y

    assert math.copysign(1.0, x) == math.copysign(1.0, y)

def assertIsNaN(value):
    if not math.isnan(value):

        raise AssertionError('Expected a NaN, got {!r}.'.format(value))

def ftest(name, got, expected, ulp_tol=5, abs_tol=0.0):
    """Compare arguments expected and got, as floats, if either
        is a float, using a tolerance expressed in multiples of
        ulp(expected) or absolutely, whichever is greater.

        As a convenience, when neither argument is a float, and for
        non-finite floats, exact equality is demanded. Also, nan==nan
        in this function.
        """
    failure = result_check(expected, got, ulp_tol, abs_tol)
    if failure is not None:

        raise AssertionError('{}: {}'.format(name, failure))

try:
    math.sin()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
ftest('sin(0)', math.sin(0), 0)
ftest('sin(pi/2)', math.sin(math.pi / 2), 1)
ftest('sin(-pi/2)', math.sin(-math.pi / 2), -1)
try:

    assert math.isnan(math.sin(INF))

    assert math.isnan(math.sin(NINF))
except ValueError:

    try:
        math.sin(INF)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.sin(NINF)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

assert math.isnan(math.sin(NAN))
print("MathTests::testSin: ok")
