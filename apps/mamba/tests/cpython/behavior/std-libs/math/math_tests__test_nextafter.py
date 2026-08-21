# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "math_tests__test_nextafter"
# subject = "cpython.test_math.MathTests.test_nextafter"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_math.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_math.py::MathTests::test_nextafter
"""Auto-ported test: MathTests::test_nextafter (CPython 3.12 oracle)."""


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

def testAcos():

    try:
        math.acos()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('acos(-1)', math.acos(-1), math.pi)
    ftest('acos(0)', math.acos(0), math.pi / 2)
    ftest('acos(1)', math.acos(1), 0)

    try:
        math.acos(INF)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.acos(NINF)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.acos(1 + eps)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.acos(-1 - eps)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert math.isnan(math.acos(NAN))

def testAcosh():

    try:
        math.acosh()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('acosh(1)', math.acosh(1), 0)
    ftest('acosh(2)', math.acosh(2), 1.3169578969248168)

    try:
        math.acosh(0)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.acosh(-1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert math.acosh(INF) == INF

    try:
        math.acosh(NINF)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert math.isnan(math.acosh(NAN))

def testAsin():

    try:
        math.asin()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('asin(-1)', math.asin(-1), -math.pi / 2)
    ftest('asin(0)', math.asin(0), 0)
    ftest('asin(1)', math.asin(1), math.pi / 2)

    try:
        math.asin(INF)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.asin(NINF)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.asin(1 + eps)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.asin(-1 - eps)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert math.isnan(math.asin(NAN))

def testAsinh():

    try:
        math.asinh()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('asinh(0)', math.asinh(0), 0)
    ftest('asinh(1)', math.asinh(1), 0.881373587019543)
    ftest('asinh(-1)', math.asinh(-1), -0.881373587019543)

    assert math.asinh(INF) == INF

    assert math.asinh(NINF) == NINF

    assert math.isnan(math.asinh(NAN))

def testAtan():

    try:
        math.atan()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('atan(-1)', math.atan(-1), -math.pi / 4)
    ftest('atan(0)', math.atan(0), 0)
    ftest('atan(1)', math.atan(1), math.pi / 4)
    ftest('atan(inf)', math.atan(INF), math.pi / 2)
    ftest('atan(-inf)', math.atan(NINF), -math.pi / 2)

    assert math.isnan(math.atan(NAN))

def testAtan2():

    try:
        math.atan2()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('atan2(-1, 0)', math.atan2(-1, 0), -math.pi / 2)
    ftest('atan2(-1, 1)', math.atan2(-1, 1), -math.pi / 4)
    ftest('atan2(0, 1)', math.atan2(0, 1), 0)
    ftest('atan2(1, 1)', math.atan2(1, 1), math.pi / 4)
    ftest('atan2(1, 0)', math.atan2(1, 0), math.pi / 2)
    ftest('atan2(1, -1)', math.atan2(1, -1), 3 * math.pi / 4)
    ftest('atan2(0., -inf)', math.atan2(0.0, NINF), math.pi)
    ftest('atan2(0., -2.3)', math.atan2(0.0, -2.3), math.pi)
    ftest('atan2(0., -0.)', math.atan2(0.0, -0.0), math.pi)

    assert math.atan2(0.0, 0.0) == 0.0

    assert math.atan2(0.0, 2.3) == 0.0

    assert math.atan2(0.0, INF) == 0.0

    assert math.isnan(math.atan2(0.0, NAN))
    ftest('atan2(-0., -inf)', math.atan2(-0.0, NINF), -math.pi)
    ftest('atan2(-0., -2.3)', math.atan2(-0.0, -2.3), -math.pi)
    ftest('atan2(-0., -0.)', math.atan2(-0.0, -0.0), -math.pi)

    assert math.atan2(-0.0, 0.0) == -0.0

    assert math.atan2(-0.0, 2.3) == -0.0

    assert math.atan2(-0.0, INF) == -0.0

    assert math.isnan(math.atan2(-0.0, NAN))
    ftest('atan2(inf, -inf)', math.atan2(INF, NINF), math.pi * 3 / 4)
    ftest('atan2(inf, -2.3)', math.atan2(INF, -2.3), math.pi / 2)
    ftest('atan2(inf, -0.)', math.atan2(INF, -0.0), math.pi / 2)
    ftest('atan2(inf, 0.)', math.atan2(INF, 0.0), math.pi / 2)
    ftest('atan2(inf, 2.3)', math.atan2(INF, 2.3), math.pi / 2)
    ftest('atan2(inf, inf)', math.atan2(INF, INF), math.pi / 4)

    assert math.isnan(math.atan2(INF, NAN))
    ftest('atan2(-inf, -inf)', math.atan2(NINF, NINF), -math.pi * 3 / 4)
    ftest('atan2(-inf, -2.3)', math.atan2(NINF, -2.3), -math.pi / 2)
    ftest('atan2(-inf, -0.)', math.atan2(NINF, -0.0), -math.pi / 2)
    ftest('atan2(-inf, 0.)', math.atan2(NINF, 0.0), -math.pi / 2)
    ftest('atan2(-inf, 2.3)', math.atan2(NINF, 2.3), -math.pi / 2)
    ftest('atan2(-inf, inf)', math.atan2(NINF, INF), -math.pi / 4)

    assert math.isnan(math.atan2(NINF, NAN))
    ftest('atan2(2.3, -inf)', math.atan2(2.3, NINF), math.pi)
    ftest('atan2(2.3, -0.)', math.atan2(2.3, -0.0), math.pi / 2)
    ftest('atan2(2.3, 0.)', math.atan2(2.3, 0.0), math.pi / 2)

    assert math.atan2(2.3, INF) == 0.0

    assert math.isnan(math.atan2(2.3, NAN))
    ftest('atan2(-2.3, -inf)', math.atan2(-2.3, NINF), -math.pi)
    ftest('atan2(-2.3, -0.)', math.atan2(-2.3, -0.0), -math.pi / 2)
    ftest('atan2(-2.3, 0.)', math.atan2(-2.3, 0.0), -math.pi / 2)

    assert math.atan2(-2.3, INF) == -0.0

    assert math.isnan(math.atan2(-2.3, NAN))

    assert math.isnan(math.atan2(NAN, NINF))

    assert math.isnan(math.atan2(NAN, -2.3))

    assert math.isnan(math.atan2(NAN, -0.0))

    assert math.isnan(math.atan2(NAN, 0.0))

    assert math.isnan(math.atan2(NAN, 2.3))

    assert math.isnan(math.atan2(NAN, INF))

    assert math.isnan(math.atan2(NAN, NAN))

def testAtanh():

    try:
        math.atan()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('atanh(0)', math.atanh(0), 0)
    ftest('atanh(0.5)', math.atanh(0.5), 0.5493061443340549)
    ftest('atanh(-0.5)', math.atanh(-0.5), -0.5493061443340549)

    try:
        math.atanh(1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.atanh(-1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.atanh(INF)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.atanh(NINF)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert math.isnan(math.atanh(NAN))

def testCbrt():

    try:
        math.cbrt()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('cbrt(0)', math.cbrt(0), 0)
    ftest('cbrt(1)', math.cbrt(1), 1)
    ftest('cbrt(8)', math.cbrt(8), 2)
    ftest('cbrt(0.0)', math.cbrt(0.0), 0.0)
    ftest('cbrt(-0.0)', math.cbrt(-0.0), -0.0)
    ftest('cbrt(1.2)', math.cbrt(1.2), 1.062658569182611)
    ftest('cbrt(-2.6)', math.cbrt(-2.6), -1.375068867074141)
    ftest('cbrt(27)', math.cbrt(27), 3)
    ftest('cbrt(-1)', math.cbrt(-1), -1)
    ftest('cbrt(-27)', math.cbrt(-27), -3)

    assert math.cbrt(INF) == INF

    assert math.cbrt(NINF) == NINF

    assert math.isnan(math.cbrt(NAN))

def testCeil():

    try:
        math.ceil()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    assert int == type(math.ceil(0.5))

    assert math.ceil(0.5) == 1

    assert math.ceil(1.0) == 1

    assert math.ceil(1.5) == 2

    assert math.ceil(-0.5) == 0

    assert math.ceil(-1.0) == -1

    assert math.ceil(-1.5) == -1

    assert math.ceil(0.0) == 0

    assert math.ceil(-0.0) == 0

    class TestCeil:

        def __ceil__(self):
            return 42

    class FloatCeil(float):

        def __ceil__(self):
            return 42

    class TestNoCeil:
        pass

    class TestBadCeil:
        __ceil__ = BadDescr()

    assert math.ceil(TestCeil()) == 42

    assert math.ceil(FloatCeil()) == 42

    assert math.ceil(FloatLike(42.5)) == 43

    try:
        math.ceil(TestNoCeil())
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        math.ceil(TestBadCeil())
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
    t = TestNoCeil()
    t.__ceil__ = lambda *args: args

    try:
        math.ceil(t)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        math.ceil(t, 0)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    assert math.ceil(FloatLike(+1.0)) == +1.0

    assert math.ceil(FloatLike(-1.0)) == -1.0

def testComb():
    comb = math.comb
    factorial = math.factorial
    for n in range(500):
        for k in range(n + 1) if n < 100 else range(30) if n < 200 else range(10):

            assert comb(n, k) == factorial(n) // (factorial(k) * factorial(n - k))
    for n in range(1, 100):
        for k in range(1, n):

            assert comb(n, k) == comb(n - 1, k - 1) + comb(n - 1, k)
    for n in range(100):

        assert comb(n, 0) == 1

        assert comb(n, n) == 1
    for n in range(1, 100):

        assert comb(n, 1) == n

        assert comb(n, n - 1) == n
    for n in range(100):
        for k in range(n // 2):

            assert comb(n, k) == comb(n, n - k)

    try:
        comb(10, 1.0)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        comb(10, decimal.Decimal(1.0))
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        comb(10, '1')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        comb(10.0, 1)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        comb(decimal.Decimal(10.0), 1)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        comb('10', 1)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        comb(10)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        comb(10, 1, 3)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        comb()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        comb(-1, 1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        comb(-2 ** 1000, 1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        comb(1, -1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        comb(1, -2 ** 1000)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert comb(1, 2) == 0

    assert comb(1, 2 ** 1000) == 0
    n = 2 ** 1000

    assert comb(n, 0) == 1

    assert comb(n, 1) == n

    assert comb(n, 2) == n * (n - 1) // 2

    assert comb(n, n) == 1

    assert comb(n, n - 1) == n

    assert comb(n, n - 2) == n * (n - 1) // 2
    if support.check_impl_detail(cpython=True):

        try:
            comb(n, n // 2)
            raise AssertionError('expected OverflowError')
        except OverflowError:
            pass
    for n, k in ((True, True), (True, False), (False, False)):

        assert comb(n, k) == 1

        assert type(comb(n, k)) is int

    assert comb(IntSubclass(5), IntSubclass(2)) == 10

    assert comb(MyIndexable(5), MyIndexable(2)) == 10
    for k in range(3):

        assert type(comb(IntSubclass(5), IntSubclass(k))) is int

        assert type(comb(MyIndexable(5), MyIndexable(k))) is int

def testConstants():
    ftest('pi', math.pi, 3.141592653589793)
    ftest('e', math.e, 2.718281828459045)

    assert math.tau == 2 * math.pi

def testCopysign():

    assert math.copysign(1, 42) == 1.0

    assert math.copysign(0.0, 42) == 0.0

    assert math.copysign(1.0, -42) == -1.0

    assert math.copysign(3, 0.0) == 3.0

    assert math.copysign(4.0, -0.0) == -4.0

    try:
        math.copysign()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    assert math.copysign(1.0, 0.0) == 1.0

    assert math.copysign(1.0, -0.0) == -1.0

    assert math.copysign(INF, 0.0) == INF

    assert math.copysign(INF, -0.0) == NINF

    assert math.copysign(NINF, 0.0) == INF

    assert math.copysign(NINF, -0.0) == NINF

    assert math.copysign(1.0, INF) == 1.0

    assert math.copysign(1.0, NINF) == -1.0

    assert math.copysign(INF, INF) == INF

    assert math.copysign(INF, NINF) == NINF

    assert math.copysign(NINF, INF) == INF

    assert math.copysign(NINF, NINF) == NINF

    assert math.isnan(math.copysign(NAN, 1.0))

    assert math.isnan(math.copysign(NAN, INF))

    assert math.isnan(math.copysign(NAN, NINF))

    assert math.isnan(math.copysign(NAN, NAN))

    assert math.isinf(math.copysign(INF, NAN))

    assert abs(math.copysign(2.0, NAN)) == 2.0

def testCos():

    try:
        math.cos()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('cos(-pi/2)', math.cos(-math.pi / 2), 0, abs_tol=math.ulp(1))
    ftest('cos(0)', math.cos(0), 1)
    ftest('cos(pi/2)', math.cos(math.pi / 2), 0, abs_tol=math.ulp(1))
    ftest('cos(pi)', math.cos(math.pi), -1)
    try:

        assert math.isnan(math.cos(INF))

        assert math.isnan(math.cos(NINF))
    except ValueError:

        try:
            math.cos(INF)
            raise AssertionError('expected ValueError')
        except ValueError:
            pass

        try:
            math.cos(NINF)
            raise AssertionError('expected ValueError')
        except ValueError:
            pass

    assert math.isnan(math.cos(NAN))

def testCosh():

    try:
        math.cosh()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('cosh(0)', math.cosh(0), 1)
    ftest('cosh(2)-2*cosh(1)**2', math.cosh(2) - 2 * math.cosh(1) ** 2, -1)

    assert math.cosh(INF) == INF

    assert math.cosh(NINF) == INF

    assert math.isnan(math.cosh(NAN))

def testDegrees():

    try:
        math.degrees()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('degrees(pi)', math.degrees(math.pi), 180.0)
    ftest('degrees(pi/2)', math.degrees(math.pi / 2), 90.0)
    ftest('degrees(-pi/4)', math.degrees(-math.pi / 4), -45.0)
    ftest('degrees(0)', math.degrees(0), 0)

def testDist():
    from decimal import Decimal as D
    from fractions import Fraction as F
    dist = math.dist
    sqrt = math.sqrt

    assert dist((1.0, 2.0, 3.0), (4.0, 2.0, -1.0)) == 5.0

    assert dist((1, 2, 3), (4, 2, -1)) == 5.0
    for i in range(9):
        for j in range(5):
            p = tuple((random.uniform(-5, 5) for k in range(i)))
            q = tuple((random.uniform(-5, 5) for k in range(i)))

            assert abs(dist(p, q) - sqrt(sum(((px - qx) ** 2.0 for px, qx in zip(p, q))))) < 1e-07

    assert dist([1.0, 2.0, 3.0], [4.0, 2.0, -1.0]) == 5.0

    assert dist(iter([1.0, 2.0, 3.0]), iter([4.0, 2.0, -1.0])) == 5.0

    assert dist((14.0, 1.0), (2.0, -4.0)) == 13.0

    assert dist((14, 1), (2, -4)) == 13

    assert dist((FloatLike(14.0), 1), (2, -4)) == 13

    assert dist((11, 1), (FloatLike(-1.0), -4)) == 13

    assert dist((14, FloatLike(-1.0)), (2, -6)) == 13

    assert dist((14, -1), (2, -6)) == 13

    assert dist((D(14), D(1)), (D(2), D(-4))) == D(13)

    assert dist((F(14, 32), F(1, 32)), (F(2, 32), F(-4, 32))) == F(13, 32)

    assert dist((True, True, False, False, True, True), (True, False, True, False, False, False)) == 2.0

    assert dist((13.25, 12.5, -3.25), (13.25, 12.5, -3.25)) == 0.0

    assert dist((), ()) == 0.0

    assert 1.0 == math.copysign(1.0, dist((-0.0,), (0.0,)))

    assert 1.0 == math.copysign(1.0, dist((0.0,), (-0.0,)))

    assert dist((1.5, 1.5, 0.5), (0, 0, 0)) == dist((1.5, 0.5, 1.5), (0, 0, 0))

    class T(tuple):
        pass

    assert dist(T((1, 2, 3)), (4, 2, -1)) == 5.0
    try:
        dist(p=(1, 2, 3), q=(4, 5, 6))
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        dist((1, 2, 3))
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        dist((1, 2, 3), (4, 5, 6), (7, 8, 9))
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        dist(1, 2)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        dist((1.1, 'string', 2.2), (1, 2, 3))
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        dist((1, 2, 3, 4), (5, 6, 7))
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
    try:
        dist((1, 2, 3), (4, 5, 6, 7))
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
    try:
        dist((1,) * 17 + ('spam',), (1,) * 18)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        dist('abc', 'xyz')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    int_too_big_for_float = 10 ** (sys.float_info.max_10_exp + 5)
    try:
        dist((1, int_too_big_for_float), (2, 3))
        raise AssertionError('expected (ValueError, OverflowError)')
    except (ValueError, OverflowError):
        pass
    try:
        dist((2, 3), (1, int_too_big_for_float))
        raise AssertionError('expected (ValueError, OverflowError)')
    except (ValueError, OverflowError):
        pass
    try:
        dist((1,), 2)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        dist([1], 2)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    class BadFloat:
        __float__ = BadDescr()
    try:
        dist([1], [BadFloat()])
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
    for i in range(20):
        p, q = (random.random(), random.random())

        assert dist((p,), (q,)) == abs(p - q)
    values = [NINF, -10.5, -0.0, 0.0, 10.5, INF, NAN]
    for p in itertools.product(values, repeat=3):
        for q in itertools.product(values, repeat=3):
            diffs = [px - qx for px, qx in zip(p, q)]
            if any(map(math.isinf, diffs)):

                assert dist(p, q) == INF
            elif any(map(math.isnan, diffs)):

                assert math.isnan(dist(p, q))
    fourthmax = FLOAT_MAX / 4.0
    for n in range(32):
        p = (fourthmax,) * n
        q = (0.0,) * n

        assert math.isclose(dist(p, q), fourthmax * math.sqrt(n))

        assert math.isclose(dist(q, p), fourthmax * math.sqrt(n))
    for exp in range(32):
        scale = FLOAT_MIN / 2.0 ** exp
        p = (4 * scale, 3 * scale)
        q = (0.0, 0.0)

        assert math.dist(p, q) == 5 * scale

        assert math.dist(q, p) == 5 * scale

def testExp():

    try:
        math.exp()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('exp(-1)', math.exp(-1), 1 / math.e)
    ftest('exp(0)', math.exp(0), 1)
    ftest('exp(1)', math.exp(1), math.e)

    assert math.exp(INF) == INF

    assert math.exp(NINF) == 0.0

    assert math.isnan(math.exp(NAN))

    try:
        math.exp(1000000)
        raise AssertionError('expected OverflowError')
    except OverflowError:
        pass

def testExp2():

    try:
        math.exp2()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('exp2(-1)', math.exp2(-1), 0.5)
    ftest('exp2(0)', math.exp2(0), 1)
    ftest('exp2(1)', math.exp2(1), 2)
    ftest('exp2(2.3)', math.exp2(2.3), 4.924577653379665)

    assert math.exp2(INF) == INF

    assert math.exp2(NINF) == 0.0

    assert math.isnan(math.exp2(NAN))

    try:
        math.exp2(1000000)
        raise AssertionError('expected OverflowError')
    except OverflowError:
        pass

def testFabs():

    try:
        math.fabs()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('fabs(-1)', math.fabs(-1), 1)
    ftest('fabs(0)', math.fabs(0), 0)
    ftest('fabs(1)', math.fabs(1), 1)

def testFactorial():

    assert math.factorial(0) == 1
    total = 1
    for i in range(1, 1000):
        total *= i

        assert math.factorial(i) == total

        assert math.factorial(i) == py_factorial(i)

    try:
        math.factorial(-1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.factorial(-10 ** 100)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

def testFactorialHugeInputs():

    try:
        math.factorial(10 ** 100)
        raise AssertionError('expected OverflowError')
    except OverflowError:
        pass

    try:
        math.factorial(1e+100)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

def testFactorialNonIntegers():

    try:
        math.factorial(5.0)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        math.factorial(5.2)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        math.factorial(-1.0)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        math.factorial(-1e+100)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        math.factorial(decimal.Decimal('5'))
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        math.factorial(decimal.Decimal('5.2'))
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        math.factorial('5')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

def testFloor():

    try:
        math.floor()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    assert int == type(math.floor(0.5))

    assert math.floor(0.5) == 0

    assert math.floor(1.0) == 1

    assert math.floor(1.5) == 1

    assert math.floor(-0.5) == -1

    assert math.floor(-1.0) == -1

    assert math.floor(-1.5) == -2

    class TestFloor:

        def __floor__(self):
            return 42

    class FloatFloor(float):

        def __floor__(self):
            return 42

    class TestNoFloor:
        pass

    class TestBadFloor:
        __floor__ = BadDescr()

    assert math.floor(TestFloor()) == 42

    assert math.floor(FloatFloor()) == 42

    assert math.floor(FloatLike(41.9)) == 41

    try:
        math.floor(TestNoFloor())
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        math.floor(TestBadFloor())
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
    t = TestNoFloor()
    t.__floor__ = lambda *args: args

    try:
        math.floor(t)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        math.floor(t, 0)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    assert math.floor(FloatLike(+1.0)) == +1.0

    assert math.floor(FloatLike(-1.0)) == -1.0

def testFmod():

    try:
        math.fmod()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('fmod(10, 1)', math.fmod(10, 1), 0.0)
    ftest('fmod(10, 0.5)', math.fmod(10, 0.5), 0.0)
    ftest('fmod(10, 1.5)', math.fmod(10, 1.5), 1.0)
    ftest('fmod(-10, 1)', math.fmod(-10, 1), -0.0)
    ftest('fmod(-10, 0.5)', math.fmod(-10, 0.5), -0.0)
    ftest('fmod(-10, 1.5)', math.fmod(-10, 1.5), -1.0)

    assert math.isnan(math.fmod(NAN, 1.0))

    assert math.isnan(math.fmod(1.0, NAN))

    assert math.isnan(math.fmod(NAN, NAN))

    try:
        math.fmod(1.0, 0.0)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.fmod(INF, 1.0)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.fmod(NINF, 1.0)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.fmod(INF, 0.0)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert math.fmod(3.0, INF) == 3.0

    assert math.fmod(-3.0, INF) == -3.0

    assert math.fmod(3.0, NINF) == 3.0

    assert math.fmod(-3.0, NINF) == -3.0

    assert math.fmod(0.0, 3.0) == 0.0

    assert math.fmod(0.0, NINF) == 0.0

    try:
        math.fmod(INF, INF)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

def testFrexp():

    try:
        math.frexp()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    def testfrexp(name, result, expected):
        (mant, exp), (emant, eexp) = (result, expected)
        if abs(mant - emant) > eps or exp != eexp:
            self.fail('%s returned %r, expected %r' % (name, result, expected))
    testfrexp('frexp(-1)', math.frexp(-1), (-0.5, 1))
    testfrexp('frexp(0)', math.frexp(0), (0, 0))
    testfrexp('frexp(1)', math.frexp(1), (0.5, 1))
    testfrexp('frexp(2)', math.frexp(2), (0.5, 2))

    assert math.frexp(INF)[0] == INF

    assert math.frexp(NINF)[0] == NINF

    assert math.isnan(math.frexp(NAN)[0])

def testFsum():
    from sys import float_info
    mant_dig = float_info.mant_dig
    etiny = float_info.min_exp - mant_dig

    def msum(iterable):
        """Full precision summation.  Compute sum(iterable) without any
            intermediate accumulation of error.  Based on the 'lsum' function
            at https://code.activestate.com/recipes/393090-binary-floating-point-summation-accurate-to-full-p/

            """
        tmant, texp = (0, 0)
        for x in iterable:
            mant, exp = math.frexp(x)
            mant, exp = (int(math.ldexp(mant, mant_dig)), exp - mant_dig)
            if texp > exp:
                tmant <<= texp - exp
                texp = exp
            else:
                mant <<= exp - texp
            tmant += mant
        tail = max(len(bin(abs(tmant))) - 2 - mant_dig, etiny - texp)
        if tail > 0:
            h = 1 << tail - 1
            tmant = tmant // (2 * h) + bool(tmant & h and tmant & 3 * h - 1)
            texp += tail
        return math.ldexp(tmant, texp)
    test_values = [([], 0.0), ([0.0], 0.0), ([1e+100, 1.0, -1e+100, 1e-100, 1e+50, -1.0, -1e+50], 1e-100), ([1e+100, 1.0, -1e+100, 1e-100, 1e+50, -1, -1e+50], 1e-100), ([2.0 ** 53, -0.5, -2.0 ** (-54)], 2.0 ** 53 - 1.0), ([2.0 ** 53, 1.0, 2.0 ** (-100)], 2.0 ** 53 + 2.0), ([2.0 ** 53 + 10.0, 1.0, 2.0 ** (-100)], 2.0 ** 53 + 12.0), ([2.0 ** 53 - 4.0, 0.5, 2.0 ** (-54)], 2.0 ** 53 - 3.0), ([1.0 / n for n in range(1, 1001)], float.fromhex('0x1.df11f45f4e61ap+2')), ([(-1.0) ** n / n for n in range(1, 1001)], float.fromhex('-0x1.62a2af1bd3624p-1')), ([1e+16, 1.0, 1e-16], 1.0000000000000002e+16), ([1e+16 - 2.0, 1.0 - 2.0 ** (-53), -(1e+16 - 2.0), -(1.0 - 2.0 ** (-53))], 0.0), ([2.0 ** n - 2.0 ** (n + 50) + 2.0 ** (n + 52) for n in range(-1074, 972, 2)] + [-2.0 ** 1022], float.fromhex('0x1.5555555555555p+970'))]
    terms = [1.7 ** i for i in range(1001)]
    test_values.append(([terms[i + 1] - terms[i] for i in range(1000)] + [-terms[1000]], -terms[0]))
    for i, (vals, expected) in enumerate(test_values):
        try:
            actual = math.fsum(vals)
        except OverflowError:

            raise AssertionError('test %d failed: got OverflowError, expected %r for math.fsum(%.100r)' % (i, expected, vals))
        except ValueError:

            raise AssertionError('test %d failed: got ValueError, expected %r for math.fsum(%.100r)' % (i, expected, vals))

        assert actual == expected
    from random import random, gauss, shuffle
    for j in range(1000):
        vals = [7, 1e+100, -7, -1e+100, -9e-20, 8e-20] * 10
        s = 0
        for i in range(200):
            v = gauss(0, random()) ** 7 - s
            s += v
            vals.append(v)
        shuffle(vals)
        s = msum(vals)

        assert msum(vals) == math.fsum(vals)

    assert math.fsum([1.0, math.inf]) == math.inf

    assert math.isnan(math.fsum([math.nan, 1.0]))

    assert math.fsum([1e+100, FloatLike(1.0), -1e+100, 1e-100, 1e+50, FloatLike(-1.0), -1e+50]) == 1e-100

    try:
        math.fsum([1e+308, 1e+308])
        raise AssertionError('expected OverflowError')
    except OverflowError:
        pass

    try:
        math.fsum([math.inf, -math.inf])
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.fsum(['spam'])
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        math.fsum(1)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        math.fsum([10 ** 1000])
        raise AssertionError('expected OverflowError')
    except OverflowError:
        pass

    def bad_iter():
        yield 1.0
        raise ZeroDivisionError

    try:
        math.fsum(bad_iter())
        raise AssertionError('expected ZeroDivisionError')
    except ZeroDivisionError:
        pass

def testGcd():
    gcd = math.gcd

    assert gcd(0, 0) == 0

    assert gcd(1, 0) == 1

    assert gcd(-1, 0) == 1

    assert gcd(0, 1) == 1

    assert gcd(0, -1) == 1

    assert gcd(7, 1) == 1

    assert gcd(7, -1) == 1

    assert gcd(-23, 15) == 1

    assert gcd(120, 84) == 12

    assert gcd(84, -120) == 12

    assert gcd(1216342683557601535506311712, 436522681849110124616458784) == 32
    x = 434610456570399902378880679233098819019853229470286994367836600566
    y = 1064502245825115327754847244914921553977
    for c in (652560, 576559230871654959816130551884856912003141446781646602790216406874):
        a = x * c
        b = y * c

        assert gcd(a, b) == c

        assert gcd(b, a) == c

        assert gcd(-a, b) == c

        assert gcd(b, -a) == c

        assert gcd(a, -b) == c

        assert gcd(-b, a) == c

        assert gcd(-a, -b) == c

        assert gcd(-b, -a) == c

    assert gcd() == 0

    assert gcd(120) == 120

    assert gcd(-120) == 120

    assert gcd(120, 84, 102) == 6

    assert gcd(120, 1, 84) == 1

    try:
        gcd(120.0)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        gcd(120.0, 84)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        gcd(120, 84.0)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        gcd(120, 1, 84.0)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    assert gcd(MyIndexable(120), MyIndexable(84)) == 12

def testHypot():
    from decimal import Decimal
    from fractions import Fraction
    hypot = math.hypot
    args = (math.e, math.pi, math.sqrt(2.0), math.gamma(3.5), math.sin(2.1))
    for i in range(len(args) + 1):

        assert abs(hypot(*args[:i]) - math.sqrt(sum((s ** 2 for s in args[:i])))) < 1e-07

    assert hypot(12.0, 5.0) == 13.0

    assert hypot(12, 5) == 13

    assert hypot(0.75, -1) == 1.25

    assert hypot(-1, 0.75) == 1.25

    assert hypot(0.75, FloatLike(-1.0)) == 1.25

    assert hypot(FloatLike(-1.0), 0.75) == 1.25

    assert hypot(Decimal(12), Decimal(5)) == 13

    assert hypot(Fraction(12, 32), Fraction(5, 32)) == Fraction(13, 32)

    assert hypot(True, False, True, True, True) == 2.0

    assert hypot(0.0, 0.0) == 0.0

    assert hypot(-10.5) == 10.5

    assert hypot() == 0.0

    assert 1.0 == math.copysign(1.0, hypot(-0.0))

    assert hypot(1.5, 1.5, 0.5) == hypot(1.5, 0.5, 1.5)
    try:
        hypot(x=1)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        hypot(1.1, 'string', 2.2)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    int_too_big_for_float = 10 ** (sys.float_info.max_10_exp + 5)
    try:
        hypot(1, int_too_big_for_float)
        raise AssertionError('expected (ValueError, OverflowError)')
    except (ValueError, OverflowError):
        pass

    assert hypot(INF) == INF

    assert hypot(0, INF) == INF

    assert hypot(10, INF) == INF

    assert hypot(-10, INF) == INF

    assert hypot(NAN, INF) == INF

    assert hypot(INF, NAN) == INF

    assert hypot(NINF, NAN) == INF

    assert hypot(NAN, NINF) == INF

    assert hypot(-INF, INF) == INF

    assert hypot(-INF, -INF) == INF

    assert hypot(10, -INF) == INF

    assert math.isnan(hypot(NAN))

    assert math.isnan(hypot(0, NAN))

    assert math.isnan(hypot(NAN, 10))

    assert math.isnan(hypot(10, NAN))

    assert math.isnan(hypot(NAN, NAN))

    assert math.isnan(hypot(NAN))
    fourthmax = FLOAT_MAX / 4.0
    for n in range(32):

        assert math.isclose(hypot(*[fourthmax] * n), fourthmax * math.sqrt(n))
    for exp in range(32):
        scale = FLOAT_MIN / 2.0 ** exp

        assert math.hypot(4 * scale, 3 * scale) == 5 * scale

    try:
        math.hypot(*[1.0] * 18, 'spam')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

def testHypotAccuracy():
    hypot = math.hypot
    Decimal = decimal.Decimal
    high_precision = decimal.Context(prec=500)
    for hx, hy in [('0x1.10e89518dca48p+29', '0x1.1970f7565b7efp+30'), ('0x1.10106eb4b44a2p+29', '0x1.ef0596cdc97f8p+29'), ('0x1.459c058e20bb7p+30', '0x1.993ca009b9178p+29'), ('0x1.378371ae67c0cp+30', '0x1.fbe6619854b4cp+29'), ('0x1.f4cd0574fb97ap+29', '0x1.50fe31669340ep+30'), ('0x1.494b2cdd3d446p+29', '0x1.212a5367b4c7cp+29'), ('0x1.f84e649f1e46dp+29', '0x1.1fa56bef8eec4p+30'), ('0x1.2e817edd3d6fap+30', '0x1.eb0814f1e9602p+29'), ('0x1.0d3a6e3d04245p+29', '0x1.32a62fea52352p+30'), ('0x1.888e19611bfc5p+29', '0x1.52b8e70b24353p+29'), ('0x1.538816d48a13fp+29', '0x1.7967c5ca43e16p+29'), ('0x1.57b47b7234530p+29', '0x1.74e2c7040e772p+29'), ('0x1.821b685e9b168p+30', '0x1.677dc1c1e3dc6p+29'), ('0x1.9e8247f67097bp+29', '0x1.24bd2dc4f4baep+29'), ('0x1.b73b59e0cb5f9p+29', '0x1.da899ab784a97p+28'), ('0x1.94a8d2842a7cfp+30', '0x1.326a51d4d8d8ap+30'), ('0x1.e930b9cd99035p+29', '0x1.5a1030e18dff9p+30'), ('0x1.1592bbb0e4690p+29', '0x1.a9c337b33fb9ap+29'), ('0x1.1243a50751fd4p+29', '0x1.a5a10175622d9p+29'), ('0x1.57a8596e74722p+30', '0x1.42d1af9d04da9p+30'), ('0x1.ee7dbd9565899p+29', '0x1.7ab4d6fc6e4b4p+29'), ('0x1.5c6bfbec5c4dcp+30', '0x1.02511184b4970p+30'), ('0x1.59dcebba995cap+30', '0x1.50ca7e7c38854p+29'), ('0x1.768cdd94cf5aap+29', '0x1.9cfdc5571d38ep+29'), ('0x1.dcf137d60262ep+29', '0x1.1101621990b3ep+30'), ('0x1.3a2d006e288b0p+30', '0x1.e9a240914326cp+29'), ('0x1.62a32f7f53c61p+29', '0x1.47eb6cd72684fp+29'), ('0x1.d3bcb60748ef2p+29', '0x1.3f13c4056312cp+30'), ('0x1.282bdb82f17f3p+30', '0x1.640ba4c4eed3ap+30'), ('0x1.89d8c423ea0c6p+29', '0x1.d35dcfe902bc3p+29')]:
        x = float.fromhex(hx)
        y = float.fromhex(hy)
        with decimal.localcontext(high_precision):
            z = float((Decimal(x) ** 2 + Decimal(y) ** 2).sqrt())

        assert hypot(x, y) == z

def testIsfinite():

    assert math.isfinite(0.0)

    assert math.isfinite(-0.0)

    assert math.isfinite(1.0)

    assert math.isfinite(-1.0)

    assert not math.isfinite(float('nan'))

    assert not math.isfinite(float('inf'))

    assert not math.isfinite(float('-inf'))

def testIsinf():

    assert math.isinf(float('inf'))

    assert math.isinf(float('-inf'))

    assert math.isinf(1e309)

    assert math.isinf(-1e309)

    assert not math.isinf(float('nan'))

    assert not math.isinf(0.0)

    assert not math.isinf(1.0)

def testIsnan():

    assert math.isnan(float('nan'))

    assert math.isnan(float('-nan'))

    assert math.isnan(float('inf') * 0.0)

    assert not math.isnan(float('inf'))

    assert not math.isnan(0.0)

    assert not math.isnan(1.0)

def testIsqrt():
    test_values = list(range(1000)) + list(range(10 ** 6 - 1000, 10 ** 6 + 1000)) + [2 ** e + i for e in range(60, 200) for i in range(-40, 40)] + [3 ** 9999, 10 ** 5001]
    for value in test_values:
        s = math.isqrt(value)

        assert type(s) is int

        assert s * s <= value

        assert value < (s + 1) * (s + 1)
    try:
        math.isqrt(-1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
    s = math.isqrt(True)

    assert type(s) is int

    assert s == 1
    s = math.isqrt(False)

    assert type(s) is int

    assert s == 0

    class IntegerLike(object):

        def __init__(self, value):
            self.value = value

        def __index__(self):
            return self.value
    s = math.isqrt(IntegerLike(1729))

    assert type(s) is int

    assert s == 41
    try:
        math.isqrt(IntegerLike(-3))
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
    bad_values = [3.5, 'a string', decimal.Decimal('3.5'), 3.5j, 100.0, -4.0]
    for value in bad_values:
        try:
            math.isqrt(value)
            raise AssertionError('expected TypeError')
        except TypeError:
            pass

def testLdexp():

    try:
        math.ldexp()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        math.ldexp(2.0, 1.1)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('ldexp(0,1)', math.ldexp(0, 1), 0)
    ftest('ldexp(1,1)', math.ldexp(1, 1), 2)
    ftest('ldexp(1,-1)', math.ldexp(1, -1), 0.5)
    ftest('ldexp(-1,1)', math.ldexp(-1, 1), -2)

    try:
        math.ldexp(1.0, 1000000)
        raise AssertionError('expected OverflowError')
    except OverflowError:
        pass

    try:
        math.ldexp(-1.0, 1000000)
        raise AssertionError('expected OverflowError')
    except OverflowError:
        pass

    assert math.ldexp(1.0, -1000000) == 0.0

    assert math.ldexp(-1.0, -1000000) == -0.0

    assert math.ldexp(INF, 30) == INF

    assert math.ldexp(NINF, -213) == NINF

    assert math.isnan(math.ldexp(NAN, 0))
    for n in [10 ** 5, 10 ** 10, 10 ** 20, 10 ** 40]:

        assert math.ldexp(INF, -n) == INF

        assert math.ldexp(NINF, -n) == NINF

        assert math.ldexp(1.0, -n) == 0.0

        assert math.ldexp(-1.0, -n) == -0.0

        assert math.ldexp(0.0, -n) == 0.0

        assert math.ldexp(-0.0, -n) == -0.0

        assert math.isnan(math.ldexp(NAN, -n))

        try:
            math.ldexp(1.0, n)
            raise AssertionError('expected OverflowError')
        except OverflowError:
            pass

        try:
            math.ldexp(-1.0, n)
            raise AssertionError('expected OverflowError')
        except OverflowError:
            pass

        assert math.ldexp(0.0, n) == 0.0

        assert math.ldexp(-0.0, n) == -0.0

        assert math.ldexp(INF, n) == INF

        assert math.ldexp(NINF, n) == NINF

        assert math.isnan(math.ldexp(NAN, n))

def testLog():

    try:
        math.log()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        math.log(1, 2, 3)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('log(1/e)', math.log(1 / math.e), -1)
    ftest('log(1)', math.log(1), 0)
    ftest('log(e)', math.log(math.e), 1)
    ftest('log(32,2)', math.log(32, 2), 5)
    ftest('log(10**40, 10)', math.log(10 ** 40, 10), 40)
    ftest('log(10**40, 10**20)', math.log(10 ** 40, 10 ** 20), 2)
    ftest('log(10**1000)', math.log(10 ** 1000), 2302.5850929940457)

    try:
        math.log(-1.5)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.log(-10 ** 1000)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.log(10, -10)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.log(NINF)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert math.log(INF) == INF

    assert math.isnan(math.log(NAN))

def testLog10():

    try:
        math.log10()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('log10(0.1)', math.log10(0.1), -1)
    ftest('log10(1)', math.log10(1), 0)
    ftest('log10(10)', math.log10(10), 1)
    ftest('log10(10**1000)', math.log10(10 ** 1000), 1000.0)

    try:
        math.log10(-1.5)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.log10(-10 ** 1000)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.log10(NINF)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert math.log(INF) == INF

    assert math.isnan(math.log10(NAN))

def testLog1p():

    try:
        math.log1p()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    for n in [2, 2 ** 90, 2 ** 300]:

        assert abs(math.log1p(n) - math.log1p(float(n))) < 1e-07

    try:
        math.log1p(-1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert math.log1p(INF) == INF

def testLog2():

    try:
        math.log2()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    assert math.log2(1) == 0.0

    assert math.log2(2) == 1.0

    assert math.log2(4) == 2.0

    assert math.log2(2 ** 1023) == 1023.0

    assert math.log2(2 ** 1024) == 1024.0

    assert math.log2(2 ** 2000) == 2000.0

    try:
        math.log2(-1.5)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.log2(NINF)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert math.isnan(math.log2(NAN))

def testLog2Exact():
    actual = [math.log2(math.ldexp(1.0, n)) for n in range(-1074, 1024)]
    expected = [float(n) for n in range(-1074, 1024)]

    assert actual == expected

def testModf():

    try:
        math.modf()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    def testmodf(name, result, expected):
        (v1, v2), (e1, e2) = (result, expected)
        if abs(v1 - e1) > eps or abs(v2 - e2):
            self.fail('%s returned %r, expected %r' % (name, result, expected))
    testmodf('modf(1.5)', math.modf(1.5), (0.5, 1.0))
    testmodf('modf(-1.5)', math.modf(-1.5), (-0.5, -1.0))

    assert math.modf(INF) == (0.0, INF)

    assert math.modf(NINF) == (-0.0, NINF)
    modf_nan = math.modf(NAN)

    assert math.isnan(modf_nan[0])

    assert math.isnan(modf_nan[1])

def testPerm():
    perm = math.perm
    factorial = math.factorial
    for n in range(500):
        for k in range(n + 1) if n < 100 else range(30) if n < 200 else range(10):

            assert perm(n, k) == factorial(n) // factorial(n - k)
    for n in range(1, 100):
        for k in range(1, n):

            assert perm(n, k) == perm(n - 1, k - 1) * k + perm(n - 1, k)
    for n in range(1, 100):

        assert perm(n, 0) == 1

        assert perm(n, 1) == n

        assert perm(n, n) == factorial(n)
    for n in range(20):

        assert perm(n) == factorial(n)

        assert perm(n, None) == factorial(n)

    try:
        perm(10, 1.0)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        perm(10, decimal.Decimal(1.0))
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        perm(10, '1')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        perm(10.0, 1)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        perm(decimal.Decimal(10.0), 1)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        perm('10', 1)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        perm()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        perm(10, 1, 3)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        perm()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        perm(-1, 1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        perm(-2 ** 1000, 1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        perm(1, -1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        perm(1, -2 ** 1000)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert perm(1, 2) == 0

    assert perm(1, 2 ** 1000) == 0
    n = 2 ** 1000

    assert perm(n, 0) == 1

    assert perm(n, 1) == n

    assert perm(n, 2) == n * (n - 1)
    if support.check_impl_detail(cpython=True):

        try:
            perm(n, n)
            raise AssertionError('expected OverflowError')
        except OverflowError:
            pass
    for n, k in ((True, True), (True, False), (False, False)):

        assert perm(n, k) == 1

        assert type(perm(n, k)) is int

    assert perm(IntSubclass(5), IntSubclass(2)) == 20

    assert perm(MyIndexable(5), MyIndexable(2)) == 20
    for k in range(3):

        assert type(perm(IntSubclass(5), IntSubclass(k))) is int

        assert type(perm(MyIndexable(5), MyIndexable(k))) is int

def testPow():

    try:
        math.pow()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('pow(0,1)', math.pow(0, 1), 0)
    ftest('pow(1,0)', math.pow(1, 0), 1)
    ftest('pow(2,1)', math.pow(2, 1), 2)
    ftest('pow(2,-1)', math.pow(2, -1), 0.5)

    assert math.pow(INF, 1) == INF

    assert math.pow(NINF, 1) == NINF

    assert math.pow(1, INF) == 1.0

    assert math.pow(1, NINF) == 1.0

    assert math.isnan(math.pow(NAN, 1))

    assert math.isnan(math.pow(2, NAN))

    assert math.isnan(math.pow(0, NAN))

    assert math.pow(1, NAN) == 1

    try:
        math.pow(1e+100, 1e+100)
        raise AssertionError('expected OverflowError')
    except OverflowError:
        pass

    assert math.pow(0.0, INF) == 0.0

    assert math.pow(0.0, 3.0) == 0.0

    assert math.pow(0.0, 2.3) == 0.0

    assert math.pow(0.0, 2.0) == 0.0

    assert math.pow(0.0, 0.0) == 1.0

    assert math.pow(0.0, -0.0) == 1.0

    try:
        math.pow(0.0, -2.0)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.pow(0.0, -2.3)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.pow(0.0, -3.0)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert math.pow(0.0, NINF) == INF

    assert math.isnan(math.pow(0.0, NAN))

    assert math.pow(INF, INF) == INF

    assert math.pow(INF, 3.0) == INF

    assert math.pow(INF, 2.3) == INF

    assert math.pow(INF, 2.0) == INF

    assert math.pow(INF, 0.0) == 1.0

    assert math.pow(INF, -0.0) == 1.0

    assert math.pow(INF, -2.0) == 0.0

    assert math.pow(INF, -2.3) == 0.0

    assert math.pow(INF, -3.0) == 0.0

    assert math.pow(INF, NINF) == 0.0

    assert math.isnan(math.pow(INF, NAN))

    assert math.pow(-0.0, INF) == 0.0

    assert math.pow(-0.0, 3.0) == -0.0

    assert math.pow(-0.0, 2.3) == 0.0

    assert math.pow(-0.0, 2.0) == 0.0

    assert math.pow(-0.0, 0.0) == 1.0

    assert math.pow(-0.0, -0.0) == 1.0

    try:
        math.pow(-0.0, -2.0)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.pow(-0.0, -2.3)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.pow(-0.0, -3.0)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert math.pow(-0.0, NINF) == INF

    assert math.isnan(math.pow(-0.0, NAN))

    assert math.pow(NINF, INF) == INF

    assert math.pow(NINF, 3.0) == NINF

    assert math.pow(NINF, 2.3) == INF

    assert math.pow(NINF, 2.0) == INF

    assert math.pow(NINF, 0.0) == 1.0

    assert math.pow(NINF, -0.0) == 1.0

    assert math.pow(NINF, -2.0) == 0.0

    assert math.pow(NINF, -2.3) == 0.0

    assert math.pow(NINF, -3.0) == -0.0

    assert math.pow(NINF, NINF) == 0.0

    assert math.isnan(math.pow(NINF, NAN))

    assert math.pow(-1.0, INF) == 1.0

    assert math.pow(-1.0, 3.0) == -1.0

    try:
        math.pow(-1.0, 2.3)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert math.pow(-1.0, 2.0) == 1.0

    assert math.pow(-1.0, 0.0) == 1.0

    assert math.pow(-1.0, -0.0) == 1.0

    assert math.pow(-1.0, -2.0) == 1.0

    try:
        math.pow(-1.0, -2.3)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert math.pow(-1.0, -3.0) == -1.0

    assert math.pow(-1.0, NINF) == 1.0

    assert math.isnan(math.pow(-1.0, NAN))

    assert math.pow(1.0, INF) == 1.0

    assert math.pow(1.0, 3.0) == 1.0

    assert math.pow(1.0, 2.3) == 1.0

    assert math.pow(1.0, 2.0) == 1.0

    assert math.pow(1.0, 0.0) == 1.0

    assert math.pow(1.0, -0.0) == 1.0

    assert math.pow(1.0, -2.0) == 1.0

    assert math.pow(1.0, -2.3) == 1.0

    assert math.pow(1.0, -3.0) == 1.0

    assert math.pow(1.0, NINF) == 1.0

    assert math.pow(1.0, NAN) == 1.0

    assert math.pow(2.3, 0.0) == 1.0

    assert math.pow(-2.3, 0.0) == 1.0

    assert math.pow(NAN, 0.0) == 1.0

    assert math.pow(2.3, -0.0) == 1.0

    assert math.pow(-2.3, -0.0) == 1.0

    assert math.pow(NAN, -0.0) == 1.0

    try:
        math.pow(-1.0, 2.3)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.pow(-15.0, -3.1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert math.pow(1.9, NINF) == 0.0

    assert math.pow(1.1, NINF) == 0.0

    assert math.pow(0.9, NINF) == INF

    assert math.pow(0.1, NINF) == INF

    assert math.pow(-0.1, NINF) == INF

    assert math.pow(-0.9, NINF) == INF

    assert math.pow(-1.1, NINF) == 0.0

    assert math.pow(-1.9, NINF) == 0.0

    assert math.pow(1.9, INF) == INF

    assert math.pow(1.1, INF) == INF

    assert math.pow(0.9, INF) == 0.0

    assert math.pow(0.1, INF) == 0.0

    assert math.pow(-0.1, INF) == 0.0

    assert math.pow(-0.9, INF) == 0.0

    assert math.pow(-1.1, INF) == INF

    assert math.pow(-1.9, INF) == INF
    ftest('(-2.)**3.', math.pow(-2.0, 3.0), -8.0)
    ftest('(-2.)**2.', math.pow(-2.0, 2.0), 4.0)
    ftest('(-2.)**1.', math.pow(-2.0, 1.0), -2.0)
    ftest('(-2.)**0.', math.pow(-2.0, 0.0), 1.0)
    ftest('(-2.)**-0.', math.pow(-2.0, -0.0), 1.0)
    ftest('(-2.)**-1.', math.pow(-2.0, -1.0), -0.5)
    ftest('(-2.)**-2.', math.pow(-2.0, -2.0), 0.25)
    ftest('(-2.)**-3.', math.pow(-2.0, -3.0), -0.125)

    try:
        math.pow(-2.0, -0.5)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.pow(-2.0, 0.5)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

def testRadians():

    try:
        math.radians()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('radians(180)', math.radians(180), math.pi)
    ftest('radians(90)', math.radians(90), math.pi / 2)
    ftest('radians(-45)', math.radians(-45), -math.pi / 4)
    ftest('radians(0)', math.radians(0), 0)

def testRemainder():
    from fractions import Fraction

    def validate_spec(x, y, r):
        """
            Check that r matches remainder(x, y) according to the IEEE 754
            specification. Assumes that x, y and r are finite and y is nonzero.
            """
        fx, fy, fr = (Fraction(x), Fraction(y), Fraction(r))
        self.assertLessEqual(abs(fr), abs(fy / 2))
        n = (fx - fr) / fy
        self.assertEqual(n, int(n))
        if abs(fr) == abs(fy / 2):
            self.assertEqual(n / 2, int(n / 2))
    testcases = ['-4.0 1 -0.0', '-3.8 1  0.8', '-3.0 1 -0.0', '-2.8 1 -0.8', '-2.0 1 -0.0', '-1.8 1  0.8', '-1.0 1 -0.0', '-0.8 1 -0.8', '-0.0 1 -0.0', ' 0.0 1  0.0', ' 0.8 1  0.8', ' 1.0 1  0.0', ' 1.8 1 -0.8', ' 2.0 1  0.0', ' 2.8 1  0.8', ' 3.0 1  0.0', ' 3.8 1 -0.8', ' 4.0 1  0.0', '0x0.0p+0 0x1.921fb54442d18p+2 0x0.0p+0', '0x1.921fb54442d18p+0 0x1.921fb54442d18p+2  0x1.921fb54442d18p+0', '0x1.921fb54442d17p+1 0x1.921fb54442d18p+2  0x1.921fb54442d17p+1', '0x1.921fb54442d18p+1 0x1.921fb54442d18p+2  0x1.921fb54442d18p+1', '0x1.921fb54442d19p+1 0x1.921fb54442d18p+2 -0x1.921fb54442d17p+1', '0x1.921fb54442d17p+2 0x1.921fb54442d18p+2 -0x0.0000000000001p+2', '0x1.921fb54442d18p+2 0x1.921fb54442d18p+2  0x0p0', '0x1.921fb54442d19p+2 0x1.921fb54442d18p+2  0x0.0000000000001p+2', '0x1.2d97c7f3321d1p+3 0x1.921fb54442d18p+2  0x1.921fb54442d14p+1', '0x1.2d97c7f3321d2p+3 0x1.921fb54442d18p+2 -0x1.921fb54442d18p+1', '0x1.2d97c7f3321d3p+3 0x1.921fb54442d18p+2 -0x1.921fb54442d14p+1', '0x1.921fb54442d17p+3 0x1.921fb54442d18p+2 -0x0.0000000000001p+3', '0x1.921fb54442d18p+3 0x1.921fb54442d18p+2  0x0p0', '0x1.921fb54442d19p+3 0x1.921fb54442d18p+2  0x0.0000000000001p+3', '0x1.f6a7a2955385dp+3 0x1.921fb54442d18p+2  0x1.921fb54442d14p+1', '0x1.f6a7a2955385ep+3 0x1.921fb54442d18p+2  0x1.921fb54442d18p+1', '0x1.f6a7a2955385fp+3 0x1.921fb54442d18p+2 -0x1.921fb54442d14p+1', '0x1.1475cc9eedf00p+5 0x1.921fb54442d18p+2  0x1.921fb54442d10p+1', '0x1.1475cc9eedf01p+5 0x1.921fb54442d18p+2 -0x1.921fb54442d10p+1', ' 1  0.c  0.4', '-1  0.c -0.4', ' 1 -0.c  0.4', '-1 -0.c -0.4', ' 1.4  0.c -0.4', '-1.4  0.c  0.4', ' 1.4 -0.c -0.4', '-1.4 -0.c  0.4', '0x1.dp+1023 0x1.4p+1023  0x0.9p+1023', '0x1.ep+1023 0x1.4p+1023 -0x0.ap+1023', '0x1.fp+1023 0x1.4p+1023 -0x0.9p+1023']
    for case in testcases:
        x_hex, y_hex, expected_hex = case.split()
        x = float.fromhex(x_hex)
        y = float.fromhex(y_hex)
        expected = float.fromhex(expected_hex)
        validate_spec(x, y, expected)
        actual = math.remainder(x, y)

        assert actual.hex() == expected.hex()
    tiny = float.fromhex('1p-1074')
    for n in range(-25, 25):
        if n == 0:
            continue
        y = n * tiny
        for m in range(100):
            x = m * tiny
            actual = math.remainder(x, y)
            validate_spec(x, y, actual)
            actual = math.remainder(-x, y)
            validate_spec(-x, y, actual)
    for value in [NAN, 0.0, -0.0, 2.0, -2.3, NINF, INF]:
        assertIsNaN(math.remainder(NAN, value))
        assertIsNaN(math.remainder(value, NAN))
    for value in [-2.3, -0.0, 0.0, 2.3]:

        assert math.remainder(value, INF) == value

        assert math.remainder(value, NINF) == value
    for value in [NINF, -2.3, -0.0, 0.0, 2.3, INF]:
        try:
            math.remainder(INF, value)
            raise AssertionError('expected ValueError')
        except ValueError:
            pass
        try:
            math.remainder(NINF, value)
            raise AssertionError('expected ValueError')
        except ValueError:
            pass
        try:
            math.remainder(value, 0.0)
            raise AssertionError('expected ValueError')
        except ValueError:
            pass
        try:
            math.remainder(value, -0.0)
            raise AssertionError('expected ValueError')
        except ValueError:
            pass

def testSin():

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

def testSinh():

    try:
        math.sinh()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('sinh(0)', math.sinh(0), 0)
    ftest('sinh(1)**2-cosh(1)**2', math.sinh(1) ** 2 - math.cosh(1) ** 2, -1)
    ftest('sinh(1)+sinh(-1)', math.sinh(1) + math.sinh(-1), 0)

    assert math.sinh(INF) == INF

    assert math.sinh(NINF) == NINF

    assert math.isnan(math.sinh(NAN))

def testSqrt():

    try:
        math.sqrt()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('sqrt(0)', math.sqrt(0), 0)
    ftest('sqrt(0)', math.sqrt(0.0), 0.0)
    ftest('sqrt(2.5)', math.sqrt(2.5), 1.5811388300841898)
    ftest('sqrt(0.25)', math.sqrt(0.25), 0.5)
    ftest('sqrt(25.25)', math.sqrt(25.25), 5.024937810560445)
    ftest('sqrt(1)', math.sqrt(1), 1)
    ftest('sqrt(4)', math.sqrt(4), 2)

    assert math.sqrt(INF) == INF

    try:
        math.sqrt(-1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        math.sqrt(NINF)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert math.isnan(math.sqrt(NAN))

def testSumProd():
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

def testTan():

    try:
        math.tan()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('tan(0)', math.tan(0), 0)
    ftest('tan(pi/4)', math.tan(math.pi / 4), 1)
    ftest('tan(-pi/4)', math.tan(-math.pi / 4), -1)
    try:

        assert math.isnan(math.tan(INF))

        assert math.isnan(math.tan(NINF))
    except ValueError:

        try:
            math.tan(INF)
            raise AssertionError('expected ValueError')
        except ValueError:
            pass

        try:
            math.tan(NINF)
            raise AssertionError('expected ValueError')
        except ValueError:
            pass

    assert math.isnan(math.tan(NAN))

def testTanh():

    try:
        math.tanh()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ftest('tanh(0)', math.tanh(0), 0)
    ftest('tanh(1)+tanh(-1)', math.tanh(1) + math.tanh(-1), 0, abs_tol=math.ulp(1))
    ftest('tanh(inf)', math.tanh(INF), 1)
    ftest('tanh(-inf)', math.tanh(NINF), -1)

    assert math.isnan(math.tanh(NAN))

def testTanhSign():

    assert math.tanh(-0.0) == -0.0

    assert math.copysign(1.0, math.tanh(-0.0)) == math.copysign(1.0, -0.0)

assert math.nextafter(4503599627370496.0, -INF) == 4503599627370495.5

assert math.nextafter(4503599627370496.0, INF) == 4503599627370497.0

assert math.nextafter(9.223372036854776e+18, 0.0) == 9.223372036854775e+18

assert math.nextafter(-9.223372036854776e+18, 0.0) == -9.223372036854775e+18

assert math.nextafter(1.0, -INF) == float.fromhex('0x1.fffffffffffffp-1')

assert math.nextafter(1.0, INF) == float.fromhex('0x1.0000000000001p+0')

assert math.nextafter(1.0, -INF, steps=1) == float.fromhex('0x1.fffffffffffffp-1')

assert math.nextafter(1.0, INF, steps=1) == float.fromhex('0x1.0000000000001p+0')

assert math.nextafter(1.0, -INF, steps=3) == float.fromhex('0x1.ffffffffffffdp-1')

assert math.nextafter(1.0, INF, steps=3) == float.fromhex('0x1.0000000000003p+0')
for steps in range(1, 5):

    assert math.nextafter(2.0, 2.0, steps=steps) == 2.0
    assertEqualSign(math.nextafter(-0.0, +0.0, steps=steps), +0.0)
    assertEqualSign(math.nextafter(+0.0, -0.0, steps=steps), -0.0)
smallest_subnormal = sys.float_info.min * sys.float_info.epsilon

assert math.nextafter(+0.0, INF) == smallest_subnormal

assert math.nextafter(-0.0, INF) == smallest_subnormal

assert math.nextafter(+0.0, -INF) == -smallest_subnormal

assert math.nextafter(-0.0, -INF) == -smallest_subnormal
assertEqualSign(math.nextafter(smallest_subnormal, +0.0), +0.0)
assertEqualSign(math.nextafter(-smallest_subnormal, +0.0), -0.0)
assertEqualSign(math.nextafter(smallest_subnormal, -0.0), +0.0)
assertEqualSign(math.nextafter(-smallest_subnormal, -0.0), -0.0)
largest_normal = sys.float_info.max

assert math.nextafter(INF, 0.0) == largest_normal

assert math.nextafter(-INF, 0.0) == -largest_normal

assert math.nextafter(largest_normal, INF) == INF

assert math.nextafter(-largest_normal, -INF) == -INF
assertIsNaN(math.nextafter(NAN, 1.0))
assertIsNaN(math.nextafter(1.0, NAN))
assertIsNaN(math.nextafter(NAN, NAN))

assert 1.0 == math.nextafter(1.0, INF, steps=0)
try:
    math.nextafter(1.0, INF, steps=-1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("MathTests::test_nextafter: ok")
