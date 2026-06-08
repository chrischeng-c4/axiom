# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "math_tests__test_sum_prod"
# subject = "cpython.test_math.MathTests.testSumProd"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_math.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_math.py::MathTests::testSumProd
"""Auto-ported test: MathTests::testSumProd (CPython 3.12 oracle)."""


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
sumprod = math.sumprod
Decimal = decimal.Decimal
Fraction = fractions.Fraction

assert sumprod(iter([10, 20, 30]), (1, 2, 3)) == 140

assert sumprod([1.5, 2.5], [3.5, 4.5]) == 16.5

assert sumprod([], []) == 0

assert sumprod([-1], [1.0]) == -1

assert sumprod([1.0], [-1]) == -1
for v in [(10, 20, 30), (1.5, -2.5), (Fraction(3, 5), Fraction(4, 5)), (Decimal(3.5), Decimal(4.5)), (2.5, 10), (2.5, Fraction(3, 5)), (25, Fraction(3, 5)), (25, Decimal(4.5))]:
    for p, q in [(v, v), (v, v[::-1])]:
        expected = sum((p_i * q_i for p_i, q_i in zip(p, q, strict=True)))
        actual = sumprod(p, q)

        assert expected == actual

        assert type(expected) == type(actual)

try:
    sumprod()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    sumprod([])
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    sumprod([], [], [])
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    sumprod(None, [10])
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    sumprod([10], None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    sumprod(['x'], [1.0])
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    sumprod([10, 20], [30])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    sumprod([10], [20, 30])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert sumprod([10 ** 20], [1]) == 10 ** 20

assert sumprod([1], [10 ** 20]) == 10 ** 20

assert sumprod([10 ** 10], [10 ** 10]) == 10 ** 20

assert sumprod([10 ** 7] * 10 ** 5, [10 ** 7] * 10 ** 5) == 10 ** 19

try:
    sumprod([10 ** 1000], [1.0])
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass

try:
    sumprod([1.0], [10 ** 1000])
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass

def raise_after(n):
    for i in range(n):
        yield i
    raise RuntimeError
try:
    sumprod(range(10), raise_after(5))
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
try:
    sumprod(raise_after(5), range(10))
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
from test.test_iter import BasicIterClass

assert sumprod(BasicIterClass(1), [1]) == 0

assert sumprod([1], BasicIterClass(1)) == 0

class BadMultiply:

    def __mul__(self, other):
        raise RuntimeError

    def __rmul__(self, other):
        raise RuntimeError
try:
    sumprod([10, BadMultiply(), 30], [1, 2, 3])
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
try:
    sumprod([1, 2, 3], [10, BadMultiply(), 30])
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
try:
    sumprod(['abc', 3], [5, 10])
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    sumprod([5, 10], ['abc', 3])
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert sumprod([10.1, math.inf], [20.2, 30.3]) == math.inf

assert sumprod([10.1, math.inf], [math.inf, 30.3]) == math.inf

assert sumprod([10.1, math.inf], [math.inf, math.inf]) == math.inf

assert sumprod([10.1, -math.inf], [20.2, 30.3]) == -math.inf

assert math.isnan(sumprod([10.1, math.inf], [-math.inf, math.inf]))

assert math.isnan(sumprod([10.1, math.nan], [20.2, 30.3]))

assert math.isnan(sumprod([10.1, math.inf], [math.nan, 30.3]))

assert math.isnan(sumprod([10.1, math.inf], [20.3, math.nan]))
args = ((-5, -5, 10), (1.5, 4611686018427387904, 2305843009213693952))

assert sumprod(*args) == 0.0
print("MathTests::testSumProd: ok")
