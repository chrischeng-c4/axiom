use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/statistics/approx_equal_exact_test__test_exactly_equal_absolute.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_approx_equal_exact_test__test_exactly_equal_absolute() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "approx_equal_exact_test__test_exactly_equal_absolute"
# subject = "cpython.test_statistics.ApproxEqualExactTest.test_exactly_equal_absolute"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::ApproxEqualExactTest::test_exactly_equal_absolute
"""Auto-ported test: ApproxEqualExactTest::test_exactly_equal_absolute (CPython 3.12 oracle)."""


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
def do_exactly_equal_test(x, tol, rel):
    result = approx_equal(x, x, tol=tol, rel=rel)

    assert result
    result = approx_equal(-x, -x, tol=tol, rel=rel)

    assert result
for n in [16, 1013, 1372, 1198, 971, 4]:
    do_exactly_equal_test(n, 0.01, 0)
    do_exactly_equal_test(n / 10, 0.01, 0)
    f = Fraction(n, 1234)
    do_exactly_equal_test(f, 0.01, 0)
print("ApproxEqualExactTest::test_exactly_equal_absolute: ok")
"###);
    assert_output(&out, r###"ApproxEqualExactTest::test_exactly_equal_absolute: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/approx_equal_exact_test__test_exactly_equal_absolute_decimals.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_approx_equal_exact_test__test_exactly_equal_absolute_decimals() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "approx_equal_exact_test__test_exactly_equal_absolute_decimals"
# subject = "cpython.test_statistics.ApproxEqualExactTest.test_exactly_equal_absolute_decimals"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::ApproxEqualExactTest::test_exactly_equal_absolute_decimals
"""Auto-ported test: ApproxEqualExactTest::test_exactly_equal_absolute_decimals (CPython 3.12 oracle)."""


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
def do_exactly_equal_test(x, tol, rel):
    result = approx_equal(x, x, tol=tol, rel=rel)

    assert result
    result = approx_equal(-x, -x, tol=tol, rel=rel)

    assert result
do_exactly_equal_test(Decimal('3.571'), Decimal('0.01'), 0)
do_exactly_equal_test(-Decimal('81.3971'), Decimal('0.01'), 0)
print("ApproxEqualExactTest::test_exactly_equal_absolute_decimals: ok")
"###);
    assert_output(&out, r###"ApproxEqualExactTest::test_exactly_equal_absolute_decimals: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/approx_equal_exact_test__test_exactly_equal_both.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_approx_equal_exact_test__test_exactly_equal_both() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "approx_equal_exact_test__test_exactly_equal_both"
# subject = "cpython.test_statistics.ApproxEqualExactTest.test_exactly_equal_both"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::ApproxEqualExactTest::test_exactly_equal_both
"""Auto-ported test: ApproxEqualExactTest::test_exactly_equal_both (CPython 3.12 oracle)."""


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
def do_exactly_equal_test(x, tol, rel):
    result = approx_equal(x, x, tol=tol, rel=rel)

    assert result
    result = approx_equal(-x, -x, tol=tol, rel=rel)

    assert result
for x in [41017, 16.742, -813.02, Fraction(3, 8)]:
    do_exactly_equal_test(x, 0.1, 0.01)
D = Decimal
do_exactly_equal_test(D('7.2'), D('0.1'), D('0.01'))
print("ApproxEqualExactTest::test_exactly_equal_both: ok")
"###);
    assert_output(&out, r###"ApproxEqualExactTest::test_exactly_equal_both: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/approx_equal_exact_test__test_exactly_equal_decimals.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_approx_equal_exact_test__test_exactly_equal_decimals() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "approx_equal_exact_test__test_exactly_equal_decimals"
# subject = "cpython.test_statistics.ApproxEqualExactTest.test_exactly_equal_decimals"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::ApproxEqualExactTest::test_exactly_equal_decimals
"""Auto-ported test: ApproxEqualExactTest::test_exactly_equal_decimals (CPython 3.12 oracle)."""


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
def do_exactly_equal_test(x, tol, rel):
    result = approx_equal(x, x, tol=tol, rel=rel)

    assert result
    result = approx_equal(-x, -x, tol=tol, rel=rel)

    assert result
D = Decimal
for d in map(D, '8.2 31.274 912.04 16.745 1.2047'.split()):
    do_exactly_equal_test(d, 0, 0)
print("ApproxEqualExactTest::test_exactly_equal_decimals: ok")
"###);
    assert_output(&out, r###"ApproxEqualExactTest::test_exactly_equal_decimals: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/approx_equal_exact_test__test_exactly_equal_floats.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_approx_equal_exact_test__test_exactly_equal_floats() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "approx_equal_exact_test__test_exactly_equal_floats"
# subject = "cpython.test_statistics.ApproxEqualExactTest.test_exactly_equal_floats"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::ApproxEqualExactTest::test_exactly_equal_floats
"""Auto-ported test: ApproxEqualExactTest::test_exactly_equal_floats (CPython 3.12 oracle)."""


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
def do_exactly_equal_test(x, tol, rel):
    result = approx_equal(x, x, tol=tol, rel=rel)

    assert result
    result = approx_equal(-x, -x, tol=tol, rel=rel)

    assert result
for x in [0.42, 1.974, 1497.4, 23.0, 179.5, 70.0245, 36.587]:
    do_exactly_equal_test(x, 0, 0)
print("ApproxEqualExactTest::test_exactly_equal_floats: ok")
"###);
    assert_output(&out, r###"ApproxEqualExactTest::test_exactly_equal_floats: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/approx_equal_exact_test__test_exactly_equal_fractions.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_approx_equal_exact_test__test_exactly_equal_fractions() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "approx_equal_exact_test__test_exactly_equal_fractions"
# subject = "cpython.test_statistics.ApproxEqualExactTest.test_exactly_equal_fractions"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::ApproxEqualExactTest::test_exactly_equal_fractions
"""Auto-ported test: ApproxEqualExactTest::test_exactly_equal_fractions (CPython 3.12 oracle)."""


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
def do_exactly_equal_test(x, tol, rel):
    result = approx_equal(x, x, tol=tol, rel=rel)

    assert result
    result = approx_equal(-x, -x, tol=tol, rel=rel)

    assert result
F = Fraction
for f in [F(1, 2), F(0), F(5, 3), F(9, 7), F(35, 36), F(3, 7)]:
    do_exactly_equal_test(f, 0, 0)
print("ApproxEqualExactTest::test_exactly_equal_fractions: ok")
"###);
    assert_output(&out, r###"ApproxEqualExactTest::test_exactly_equal_fractions: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/approx_equal_exact_test__test_exactly_equal_ints.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_approx_equal_exact_test__test_exactly_equal_ints() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "approx_equal_exact_test__test_exactly_equal_ints"
# subject = "cpython.test_statistics.ApproxEqualExactTest.test_exactly_equal_ints"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::ApproxEqualExactTest::test_exactly_equal_ints
"""Auto-ported test: ApproxEqualExactTest::test_exactly_equal_ints (CPython 3.12 oracle)."""


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
def do_exactly_equal_test(x, tol, rel):
    result = approx_equal(x, x, tol=tol, rel=rel)

    assert result
    result = approx_equal(-x, -x, tol=tol, rel=rel)

    assert result
for n in [42, 19740, 14974, 230, 1795, 700245, 36587]:
    do_exactly_equal_test(n, 0, 0)
print("ApproxEqualExactTest::test_exactly_equal_ints: ok")
"###);
    assert_output(&out, r###"ApproxEqualExactTest::test_exactly_equal_ints: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/approx_equal_exact_test__test_exactly_equal_relative.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_approx_equal_exact_test__test_exactly_equal_relative() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "approx_equal_exact_test__test_exactly_equal_relative"
# subject = "cpython.test_statistics.ApproxEqualExactTest.test_exactly_equal_relative"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::ApproxEqualExactTest::test_exactly_equal_relative
"""Auto-ported test: ApproxEqualExactTest::test_exactly_equal_relative (CPython 3.12 oracle)."""


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
def do_exactly_equal_test(x, tol, rel):
    result = approx_equal(x, x, tol=tol, rel=rel)

    assert result
    result = approx_equal(-x, -x, tol=tol, rel=rel)

    assert result
for x in [8347, 101.3, -7910.28, Fraction(5, 21)]:
    do_exactly_equal_test(x, 0, 0.01)
do_exactly_equal_test(Decimal('11.68'), 0, Decimal('0.01'))
print("ApproxEqualExactTest::test_exactly_equal_relative: ok")
"###);
    assert_output(&out, r###"ApproxEqualExactTest::test_exactly_equal_relative: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/correlation_and_covariance.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_correlation_and_covariance() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "correlation_and_covariance"
# subject = "statistics.correlation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.correlation: correlation/covariance track the sign of a linear relationship (+1/-1/0/0.5), correlation is scale-invariant while covariance is not, and the 'ranked' method gives Spearman correlation"""
from statistics import correlation, covariance

# correlation and covariance track the sign/strength of a linear relationship.
for x, y, r in [([1, 2, 3], [1, 2, 3], 1),
                ([1, 2, 3], [3, 2, 1], -1),
                ([1, 2, 3], [1, 2, 1], 0),
                ([1, 2, 3], [1, 3, 2], 0.5)]:
    assert abs(correlation(x, y) - r) < 1e-7, (x, y)
    assert abs(covariance(x, y) - r) < 1e-7, (x, y)
# correlation is scale-invariant; covariance is not.
assert abs(correlation([1, 2, 3], [10, 30, 20]) - 0.5) < 1e-7
assert abs(covariance([1, 2, 3], [10, 30, 20]) - 5) < 1e-7
# The 'ranked' method gives Spearman correlation on monotonic, tie-bearing data.
reading = [56, 75, 45, 71, 61, 64, 58, 80, 76, 61]
maths = [66, 70, 40, 60, 65, 56, 59, 77, 67, 63]
assert abs(correlation(reading, maths, method="ranked") - 0.6686960980480712) < 1e-7

print("correlation_and_covariance OK")
"###);
    assert_output(&out, r###"correlation_and_covariance OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/fmean_returns_float.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_fmean_returns_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "fmean_returns_float"
# subject = "statistics.fmean"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.fmean: fmean always returns a float regardless of input numeric type (float/Decimal/Fraction/bool, tuple, iterator) and its weights act as repetition counts so uniform weights equal the unweighted mean"""
from decimal import Decimal
from fractions import Fraction
from statistics import fmean

# fmean always returns a float, whatever the input numeric type or container.
for data, expected in [([3.5, 4.0, 5.25], 4.25),
                       ([Decimal("3.5"), Decimal("4.0"), Decimal("5.25")], 4.25),
                       ([Fraction(7, 2), Fraction(4, 1), Fraction(21, 4)], 4.25),
                       ([True, False, True, True, False], 0.6),
                       ((3.5, 4.0, 5.25), 4.25),
                       (iter([3.5, 4.0, 5.25]), 4.25)]:
    got = fmean(data)
    assert type(got) is float and got == expected, (got, expected)
# Weights act as repetition counts; uniform weights == the unweighted mean.
assert fmean([10, 10, 20], [0.25, 0.25, 0.5]) == fmean([10, 10, 20, 20])

print("fmean_returns_float OK")
"###);
    assert_output(&out, r###"fmean_returns_float OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/geometric_mean_nth_root.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_geometric_mean_nth_root() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "geometric_mean_nth_root"
# subject = "statistics.geometric_mean"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.geometric_mean: geometric_mean is the nth root of the product (geometric_mean([54,24,36]) ~ 36.0, [4.0,9.0] ~ 6.0)"""
from statistics import geometric_mean

# geometric_mean is the nth root of the product of the data.
assert abs(geometric_mean([54, 24, 36]) - 36.0) < 1e-7, geometric_mean([54, 24, 36])
assert abs(geometric_mean([4.0, 9.0]) - 6.0) < 1e-7, geometric_mean([4.0, 9.0])
assert abs(geometric_mean([1, 2, 4, 8]) - 2.8284271247461903) < 1e-9

print("geometric_mean_nth_root OK")
"###);
    assert_output(&out, r###"geometric_mean_nth_root OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/globals_test__test_check_all.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_globals_test__test_check_all() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "globals_test__test_check_all"
# subject = "cpython.test_statistics.GlobalsTest.test_check_all"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::GlobalsTest::test_check_all
"""Auto-ported test: GlobalsTest::test_check_all (CPython 3.12 oracle)."""


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
module = statistics
expected_metadata = ['__doc__', '__all__']
module = module
for name in module.__all__:

    assert not name.startswith('_')

    assert hasattr(module, name)
print("GlobalsTest::test_check_all: ok")
"###);
    assert_output(&out, r###"GlobalsTest::test_check_all: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/globals_test__test_meta.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_globals_test__test_meta() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "globals_test__test_meta"
# subject = "cpython.test_statistics.GlobalsTest.test_meta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::GlobalsTest::test_meta
"""Auto-ported test: GlobalsTest::test_meta (CPython 3.12 oracle)."""


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
module = statistics
expected_metadata = ['__doc__', '__all__']
for meta in expected_metadata:

    assert hasattr(module, meta)
print("GlobalsTest::test_meta: ok")
"###);
    assert_output(&out, r###"GlobalsTest::test_meta: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/linear_regression_slope_intercept.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_linear_regression_slope_intercept() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "linear_regression_slope_intercept"
# subject = "statistics.linear_regression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.linear_regression: linear_regression returns (slope, intercept) recovering known lines, and proportional=True forces a zero intercept (line through the origin)"""
from statistics import linear_regression

# linear_regression returns (slope, intercept), recovering known lines.
for x, y, intercept, slope in [([1, 2, 3], [12, 14, 16], 10, 2),
                               ([1, 2, 3], [1, 2, 3], 0, 1),
                               ([1, 2, 3], [100, 100, 100], 100, 0)]:
    s, b = linear_regression(x, y)
    assert abs(b - intercept) < 1e-7 and abs(s - slope) < 1e-7, (x, y)
# proportional=True forces a zero intercept (the line passes through the origin).
s, b = linear_regression([10, 20, 30, 40], [180, 398, 610, 799], proportional=True)
assert b == 0.0 and abs(s - (20 + 1 / 150)) < 1e-7

print("linear_regression_slope_intercept OK")
"###);
    assert_output(&out, r###"linear_regression_slope_intercept OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/mean_ieee_special_values.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_mean_ieee_special_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "mean_ieee_special_values"
# subject = "statistics.mean"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.mean: a single nan poisons the mean (float and Decimal alike), a single signed infinity dominates and is preserved, and +inf with -inf yields nan rather than an error"""
import math
import statistics
from decimal import Decimal

# A single nan poisons the mean, for both float and Decimal inputs.
for kind in (float, Decimal):
    _r = statistics.mean([1, 3, 5, 7, 9, kind("nan")])
    assert math.isnan(_r), repr(_r)
# A single infinity dominates and is preserved with its sign.
for kind in (float, Decimal):
    for sign in (1, -1):
        _inf = kind("inf") * sign
        _r = statistics.mean([1, 3, 5, 7, 9, _inf])
        assert math.isinf(_r) and _r == _inf, repr(_r)
# +inf and -inf together yield nan, not an error.
assert math.isnan(statistics.mean([2, 4, 6, float("inf"), 1, float("-inf")]))

print("mean_ieee_special_values OK")
"###);
    assert_output(&out, r###"mean_ieee_special_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/mean_type_preservation.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_mean_type_preservation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "mean_type_preservation"
# subject = "statistics.mean"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.mean: mean of an all-int list reduces to the exact value (3 for 1..5, 2.5 for 1..4) and mean of floats returns a float"""
import statistics

# An all-int list whose average is a whole number reduces to that value.
assert statistics.mean([1, 2, 3, 4, 5]) == 3, statistics.mean([1, 2, 3, 4, 5])
# An all-int list whose average is fractional yields the exact 2.5.
assert statistics.mean([1, 2, 3, 4]) == 2.5, statistics.mean([1, 2, 3, 4])
# Mean of floats is a float.
_mf = statistics.mean([1.0, 2.0, 3.0])
assert _mf == 2.0, _mf
assert statistics.mean([1.5, 2.5]) == 2.0, statistics.mean([1.5, 2.5])

print("mean_type_preservation OK")
"###);
    assert_output(&out, r###"mean_type_preservation OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/median_even_and_odd_lengths.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_median_even_and_odd_lengths() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "median_even_and_odd_lengths"
# subject = "statistics.median"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.median: median averages the middle two on even-length input ([1,3,5,7] -> 4) and returns the exact middle on odd-length ([1,3,5] -> 3), keeping int return for odd int input"""
import statistics

# Even-length: average of the two middle values.
assert statistics.median([1, 3, 5, 7]) == 4, statistics.median([1, 3, 5, 7])
assert statistics.median([1, 2, 3, 4]) == 2.5, statistics.median([1, 2, 3, 4])
# Odd-length int input returns the exact middle int.
assert statistics.median([1, 3, 5]) == 3, statistics.median([1, 3, 5])
assert statistics.median([1, 2, 3]) == 2, statistics.median([1, 2, 3])
# Float middle for a float odd-length list.
assert statistics.median([1.5, 2.5, 3.5]) == 2.5, statistics.median([1.5, 2.5, 3.5])

print("median_even_and_odd_lengths OK")
"###);
    assert_output(&out, r###"median_even_and_odd_lengths OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/median_low_and_high.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_median_low_and_high() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "median_low_and_high"
# subject = "statistics.median_low"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.median_low: median_low picks the lower of the two middle values and median_high the upper on even-length data ([1,3,5,7] -> 3 and 5)"""
import statistics

# On even-length data the two middle values are 3 and 5.
assert statistics.median_low([1, 3, 5, 7]) == 3, statistics.median_low([1, 3, 5, 7])
assert statistics.median_high([1, 3, 5, 7]) == 5, statistics.median_high([1, 3, 5, 7])
# On odd-length data both collapse to the exact middle.
assert statistics.median_low([1, 3, 5]) == 3, statistics.median_low([1, 3, 5])
assert statistics.median_high([1, 3, 5]) == 3, statistics.median_high([1, 3, 5])

print("median_low_and_high OK")
"###);
    assert_output(&out, r###"median_low_and_high OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/mode_single_most_common.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_mode_single_most_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "mode_single_most_common"
# subject = "statistics.mode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.mode: mode returns the single most common value over any hashable iterable (ints, characters, words) and on a tie returns the first-seen value, never erroring"""
import collections
from statistics import mode

# The single most common value, over any hashable iterable.
assert mode([1, 2, 2, 3]) == 2, mode([1, 2, 2, 3])
assert mode("abcbdb") == "b", mode("abcbdb")
assert mode("fe fi fo fum fi fi".split()) == "fi"
# Since Python 3.8 a tie returns the first-seen value rather than erroring.
assert mode([1, 1, 2, 2, 3]) in (1, 2), mode([1, 1, 2, 2, 3])
# A Counter is iterated by key, so the first key wins on a count tie.
assert mode(collections.Counter(a=1, b=2)) == "a"

print("mode_single_most_common OK")
"###);
    assert_output(&out, r###"mode_single_most_common OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/multimode_first_seen_order.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_multimode_first_seen_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "multimode_first_seen_order"
# subject = "statistics.multimode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.multimode: multimode returns every most-common value preserving first-seen order ('aabbbbccddddeeffffgg' -> ['b','d','f']) and returns [] for empty input"""
from statistics import multimode

# A single dominant value gives a one-element list.
assert multimode("aabbbbbbbbcc") == ["b"], multimode("aabbbbbbbbcc")
# Multiple tied modes are returned in first-seen order.
assert multimode("aabbbbccddddeeffffgg") == ["b", "d", "f"]
# A bimodal int list keeps first-seen order.
assert sorted(multimode([1, 1, 2, 2, 3])) == [1, 2]
# Empty input -> empty list (never an error).
assert multimode("") == []

print("multimode_first_seen_order OK")
"###);
    assert_output(&out, r###"multimode_first_seen_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/normaldist_arithmetic.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_normaldist_arithmetic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "normaldist_arithmetic"
# subject = "statistics.NormalDist"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.NormalDist: translating/scaling a NormalDist shifts/scales it (X+10, 10*X, X/10), adding two NormalDists combines variances (sigma=sqrt(12**2+5**2)=13), and unary negation flips the mean and copies"""
from statistics import NormalDist

X = NormalDist(100, 15)
# Translation shifts the mean; scaling scales both mean and sigma.
assert X + 10 == NormalDist(110, 15) and 10 + X == NormalDist(110, 15)
assert X - 10 == NormalDist(90, 15)
assert X * 10 == NormalDist(1000, 150) and X / 10 == NormalDist(10, 1.5)
# Same-type add/sub combine variances: sigma = sqrt(12**2 + 5**2) = 13.
assert NormalDist(100, 12) + NormalDist(40, 5) == NormalDist(140, 13)
assert NormalDist(100, 12) - NormalDist(40, 5) == NormalDist(60, 13)
# Unary negation flips the mean, keeps sigma, and produces a fresh object.
neg = -X
assert neg is not X and neg.mean == -100 and neg.stdev == 15

print("normaldist_arithmetic OK")
"###);
    assert_output(&out, r###"normaldist_arithmetic OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/normaldist_properties.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_normaldist_properties() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "normaldist_properties"
# subject = "statistics.NormalDist"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.NormalDist: a NormalDist derives mean/median/mode == mu, stdev == sigma, and variance == sigma**2 (NormalDist(100,15) -> variance 225)"""
from statistics import NormalDist

# mean / median / mode all equal mu; stdev equals sigma; variance == sigma**2.
X = NormalDist(100, 15)
assert X.mean == 100 and X.median == 100 and X.mode == 100
assert X.stdev == 15
assert X.variance == 225, X.variance

print("normaldist_properties OK")
"###);
    assert_output(&out, r###"normaldist_properties OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/normaldist_value_dunders.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_normaldist_value_dunders() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "normaldist_value_dunders"
# subject = "statistics.NormalDist"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.NormalDist: NormalDist is a value type: repr round-trips, equality is by (mu,sigma), it is hashable (set dedups equal instances), copy/deepcopy preserve equality, and __slots__ means no instance __dict__"""
import copy
from statistics import NormalDist

# repr round-trips the constructor arguments.
assert repr(NormalDist(37.5, 5.625)) == "NormalDist(mu=37.5, sigma=5.625)"
# Equality is by (mu, sigma).
assert NormalDist() == NormalDist() and NormalDist(2, 4) != NormalDist(2, 8)
# Hashable: a set dedups equal instances.
assert len({NormalDist(100, 15), NormalDist(100.0, 15.0), NormalDist(95, 15)}) == 2
# copy / deepcopy preserve equality.
nd = NormalDist(37.5, 5.625)
assert copy.copy(nd) == nd and copy.deepcopy(nd) == nd
# __slots__ means there is no instance __dict__.
_raised = False
try:
    vars(nd)
except TypeError:
    _raised = True
assert _raised, "NormalDist should have no __dict__"

print("normaldist_value_dunders OK")
"###);
    assert_output(&out, r###"normaldist_value_dunders OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/quantiles_exclusive_vs_inclusive.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_quantiles_exclusive_vs_inclusive() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "quantiles_exclusive_vs_inclusive"
# subject = "statistics.quantiles"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.quantiles: n cut points produce n-1 boundaries (n=1 -> []), the exclusive default and the 'inclusive' method give the documented CPython values, and the result element type follows the input numeric type"""
from decimal import Decimal
from fractions import Fraction
from statistics import quantiles, median

# n cut points produce n-1 boundaries (exclusive, the default); n=1 -> [].
data = [120, 200, 250, 320, 350]
for n, expected in [(1, []),
                    (2, [250.0]),
                    (4, [160.0, 250.0, 335.0]),
                    (5, [136.0, 220.0, 292.0, 344.0])]:
    got = quantiles(data, n=n)
    assert got == expected, (n, got)
    assert len(got) == n - 1
# The middle quartile equals the median.
q1, q2, q3 = quantiles(data)
assert q2 == median(data)

# The 'inclusive' method uses the data bounds as the extremes.
inc = [100, 200, 400, 800]
assert quantiles(inc, n=4, method="inclusive") == [175.0, 300.0, 500.0]
assert quantiles([0, 100], n=10, method="inclusive") == [
    10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]
# The result element type follows the input numeric type.
for kind in (float, Decimal, Fraction):
    res = quantiles(list(map(kind, inc)), n=4, method="inclusive")
    assert res == list(map(kind, [175, 300, 500])), kind

print("quantiles_exclusive_vs_inclusive OK")
"###);
    assert_output(&out, r###"quantiles_exclusive_vs_inclusive OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/quantiles_quartiles.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_quantiles_quartiles() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "quantiles_quartiles"
# subject = "statistics.quantiles"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.quantiles: quantiles(n=4) returns the three quartiles, with the middle quartile equal to the median over 1..100 (~50.5)"""
import statistics

# quantiles(n=4) gives the three quartiles Q1, Q2, Q3.
_data = list(range(1, 101))  # 1..100
_q = statistics.quantiles(_data, n=4)
assert len(_q) == 3, len(_q)
# The middle quartile is the median, ~50.5 over 1..100.
assert abs(_q[1] - 50.5) < 1, _q[1]
assert _q[1] == statistics.median(_data)

print("quantiles_quartiles OK")
"###);
    assert_output(&out, r###"quantiles_quartiles OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/sample_vs_population_stdev.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_sample_vs_population_stdev() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "sample_vs_population_stdev"
# subject = "statistics.stdev"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.stdev: sample stdev (ddof=1) exceeds population pstdev (ddof=0) on the same data, with exact CPython values for [2,4,4,4,5,5,7,9]"""
import statistics

_data = [2, 4, 4, 4, 5, 5, 7, 9]
_s = statistics.stdev(_data)   # sample stdev (ddof=1)
_p = statistics.pstdev(_data)  # population stdev (ddof=0)
# Dividing by n-1 (sample) yields a larger value than dividing by n.
assert _s > _p, (_s, _p)
assert abs(_s - 2.138089935299395) < 1e-10, _s
assert abs(_p - 2.0) < 1e-10, _p

print("sample_vs_population_stdev OK")
"###);
    assert_output(&out, r###"sample_vs_population_stdev OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_approx_equal_errors__test_bad_rel.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_approx_equal_errors__test_bad_rel() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_approx_equal_errors__test_bad_rel"
# subject = "cpython.test_statistics.TestApproxEqualErrors.test_bad_rel"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestApproxEqualErrors::test_bad_rel
"""Auto-ported test: TestApproxEqualErrors::test_bad_rel (CPython 3.12 oracle)."""


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

try:
    approx_equal(100, 100, 1, -0.1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("TestApproxEqualErrors::test_bad_rel: ok")
"###);
    assert_output(&out, r###"TestApproxEqualErrors::test_bad_rel: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_approx_equal_errors__test_bad_tol.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_approx_equal_errors__test_bad_tol() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_approx_equal_errors__test_bad_tol"
# subject = "cpython.test_statistics.TestApproxEqualErrors.test_bad_tol"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestApproxEqualErrors::test_bad_tol
"""Auto-ported test: TestApproxEqualErrors::test_bad_tol (CPython 3.12 oracle)."""


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

try:
    approx_equal(100, 100, -1, 0.1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("TestApproxEqualErrors::test_bad_tol: ok")
"###);
    assert_output(&out, r###"TestApproxEqualErrors::test_bad_tol: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_bivariate_statistics__test_small_sample_error.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_bivariate_statistics__test_small_sample_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_bivariate_statistics__test_small_sample_error"
# subject = "cpython.test_statistics.TestBivariateStatistics.test_small_sample_error"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestBivariateStatistics::test_small_sample_error
"""Auto-ported test: TestBivariateStatistics::test_small_sample_error (CPython 3.12 oracle)."""


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
for x, y in [([], []), ([], [1, 2]), ([1, 2], []), ([1], [1]), ([1], [1, 2]), ([1, 2], [1])]:
    try:
        statistics.covariance(x, y)
        raise AssertionError('expected statistics.StatisticsError')
    except statistics.StatisticsError:
        pass
    try:
        statistics.correlation(x, y)
        raise AssertionError('expected statistics.StatisticsError')
    except statistics.StatisticsError:
        pass
    try:
        statistics.linear_regression(x, y)
        raise AssertionError('expected statistics.StatisticsError')
    except statistics.StatisticsError:
        pass
print("TestBivariateStatistics::test_small_sample_error: ok")
"###);
    assert_output(&out, r###"TestBivariateStatistics::test_small_sample_error: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_bivariate_statistics__test_unequal_size_error.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_bivariate_statistics__test_unequal_size_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_bivariate_statistics__test_unequal_size_error"
# subject = "cpython.test_statistics.TestBivariateStatistics.test_unequal_size_error"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestBivariateStatistics::test_unequal_size_error
"""Auto-ported test: TestBivariateStatistics::test_unequal_size_error (CPython 3.12 oracle)."""


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
for x, y in [([1, 2, 3], [1, 2]), ([1, 2], [1, 2, 3])]:
    try:
        statistics.covariance(x, y)
        raise AssertionError('expected statistics.StatisticsError')
    except statistics.StatisticsError:
        pass
    try:
        statistics.correlation(x, y)
        raise AssertionError('expected statistics.StatisticsError')
    except statistics.StatisticsError:
        pass
    try:
        statistics.linear_regression(x, y)
        raise AssertionError('expected statistics.StatisticsError')
    except statistics.StatisticsError:
        pass
print("TestBivariateStatistics::test_unequal_size_error: ok")
"###);
    assert_output(&out, r###"TestBivariateStatistics::test_unequal_size_error: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_correlation_and_covariance__test_different_scales.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_correlation_and_covariance__test_different_scales() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_correlation_and_covariance__test_different_scales"
# subject = "cpython.test_statistics.TestCorrelationAndCovariance.test_different_scales"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestCorrelationAndCovariance::test_different_scales
"""Auto-ported test: TestCorrelationAndCovariance::test_different_scales (CPython 3.12 oracle)."""


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
x = [1, 2, 3]
y = [10, 30, 20]

assert abs(statistics.correlation(x, y) - 0.5) < 1e-07

assert abs(statistics.covariance(x, y) - 5) < 1e-07
y = [0.1, 0.2, 0.3]

assert abs(statistics.correlation(x, y) - 1) < 1e-07

assert abs(statistics.covariance(x, y) - 0.1) < 1e-07
print("TestCorrelationAndCovariance::test_different_scales: ok")
"###);
    assert_output(&out, r###"TestCorrelationAndCovariance::test_different_scales: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_correlation_and_covariance__test_results.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_correlation_and_covariance__test_results() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_correlation_and_covariance__test_results"
# subject = "cpython.test_statistics.TestCorrelationAndCovariance.test_results"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestCorrelationAndCovariance::test_results
"""Auto-ported test: TestCorrelationAndCovariance::test_results (CPython 3.12 oracle)."""


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
for x, y, result in [([1, 2, 3], [1, 2, 3], 1), ([1, 2, 3], [-1, -2, -3], -1), ([1, 2, 3], [3, 2, 1], -1), ([1, 2, 3], [1, 2, 1], 0), ([1, 2, 3], [1, 3, 2], 0.5)]:

    assert abs(statistics.correlation(x, y) - result) < 1e-07

    assert abs(statistics.covariance(x, y) - result) < 1e-07
print("TestCorrelationAndCovariance::test_results: ok")
"###);
    assert_output(&out, r###"TestCorrelationAndCovariance::test_results: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_f_mean__test_special_values.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_f_mean__test_special_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_f_mean__test_special_values"
# subject = "cpython.test_statistics.TestFMean.test_special_values"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestFMean::test_special_values
"""Auto-ported test: TestFMean::test_special_values (CPython 3.12 oracle)."""


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
fmean = statistics.fmean
NaN = float('Nan')
Inf = float('Inf')

assert math.isnan(fmean([10, NaN]))

assert math.isnan(fmean([NaN, Inf]))

assert math.isinf(fmean([10, Inf]))
try:
    fmean([Inf, -Inf])
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("TestFMean::test_special_values: ok")
"###);
    assert_output(&out, r###"TestFMean::test_special_values: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_f_mean__test_weights.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_f_mean__test_weights() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_f_mean__test_weights"
# subject = "cpython.test_statistics.TestFMean.test_weights"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestFMean::test_weights
"""Auto-ported test: TestFMean::test_weights (CPython 3.12 oracle)."""


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
fmean = statistics.fmean
StatisticsError = statistics.StatisticsError

assert fmean([10, 10, 10, 50], [0.25] * 4) == fmean([10, 10, 10, 50])

assert fmean([10, 10, 20], [0.25, 0.25, 0.5]) == fmean([10, 10, 20, 20])

assert fmean(iter([10, 10, 20]), iter([0.25, 0.25, 0.5])) == fmean([10, 10, 20, 20])
try:
    fmean([10, 20, 30], [1, 2])
    raise AssertionError('expected StatisticsError')
except StatisticsError:
    pass
try:
    fmean(iter([10, 20, 30]), iter([1, 2]))
    raise AssertionError('expected StatisticsError')
except StatisticsError:
    pass
try:
    fmean([10, 20], [-1, 1])
    raise AssertionError('expected StatisticsError')
except StatisticsError:
    pass
try:
    fmean(iter([10, 20]), iter([-1, 1]))
    raise AssertionError('expected StatisticsError')
except StatisticsError:
    pass
print("TestFMean::test_weights: ok")
"###);
    assert_output(&out, r###"TestFMean::test_weights: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_geometric_mean__test_big_and_small.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_geometric_mean__test_big_and_small() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_geometric_mean__test_big_and_small"
# subject = "cpython.test_statistics.TestGeometricMean.test_big_and_small"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestGeometricMean::test_big_and_small
"""Auto-ported test: TestGeometricMean::test_big_and_small (CPython 3.12 oracle)."""


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
geometric_mean = statistics.geometric_mean
large = 2.0 ** 1000
big_gm = geometric_mean([54.0 * large, 24.0 * large, 36.0 * large])

assert math.isclose(big_gm, 36.0 * large)

assert not math.isinf(big_gm)
small = 2.0 ** (-1000)
small_gm = geometric_mean([54.0 * small, 24.0 * small, 36.0 * small])

assert math.isclose(small_gm, 36.0 * small)

assert small_gm != 0.0
print("TestGeometricMean::test_big_and_small: ok")
"###);
    assert_output(&out, r###"TestGeometricMean::test_big_and_small: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_geometric_mean__test_mixed_int_and_float.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_geometric_mean__test_mixed_int_and_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_geometric_mean__test_mixed_int_and_float"
# subject = "cpython.test_statistics.TestGeometricMean.test_mixed_int_and_float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestGeometricMean::test_mixed_int_and_float
"""Auto-ported test: TestGeometricMean::test_mixed_int_and_float (CPython 3.12 oracle)."""


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
geometric_mean = statistics.geometric_mean
expected_mean = 3.80675409583932
values = [[2, 3, 5, 7], [2, 3, 5, 7.0], [2, 3, 5.0, 7.0], [2, 3.0, 5.0, 7.0], [2.0, 3.0, 5.0, 7.0]]
for v in values:
    actual_mean = geometric_mean(v)

    assert abs(actual_mean - expected_mean) < 1e-07
print("TestGeometricMean::test_mixed_int_and_float: ok")
"###);
    assert_output(&out, r###"TestGeometricMean::test_mixed_int_and_float: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_geometric_mean__test_special_values.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_geometric_mean__test_special_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_geometric_mean__test_special_values"
# subject = "cpython.test_statistics.TestGeometricMean.test_special_values"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestGeometricMean::test_special_values
"""Auto-ported test: TestGeometricMean::test_special_values (CPython 3.12 oracle)."""


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
geometric_mean = statistics.geometric_mean
NaN = float('Nan')
Inf = float('Inf')

assert math.isnan(geometric_mean([10, NaN]))

assert math.isnan(geometric_mean([NaN, Inf]))

assert math.isinf(geometric_mean([10, Inf]))
try:
    geometric_mean([Inf, -Inf])
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("TestGeometricMean::test_special_values: ok")
"###);
    assert_output(&out, r###"TestGeometricMean::test_special_values: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_harmonic_mean__test_empty_data.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_harmonic_mean__test_empty_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_harmonic_mean__test_empty_data"
# subject = "cpython.test_statistics.TestHarmonicMean.test_empty_data"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestHarmonicMean::test_empty_data
"""Auto-ported test: TestHarmonicMean::test_empty_data (CPython 3.12 oracle)."""


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
self_func = statistics.harmonic_mean
for empty in ([], (), iter([])):

    try:
        self_func(empty)
        raise AssertionError('expected statistics.StatisticsError')
    except statistics.StatisticsError:
        pass
print("TestHarmonicMean::test_empty_data: ok")
"###);
    assert_output(&out, r###"TestHarmonicMean::test_empty_data: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_linear_regression__test_constant_input_error.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_linear_regression__test_constant_input_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_linear_regression__test_constant_input_error"
# subject = "cpython.test_statistics.TestLinearRegression.test_constant_input_error"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestLinearRegression::test_constant_input_error
"""Auto-ported test: TestLinearRegression::test_constant_input_error (CPython 3.12 oracle)."""


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
x = [1, 1, 1]
y = [1, 2, 3]
try:
    statistics.linear_regression(x, y)
    raise AssertionError('expected statistics.StatisticsError')
except statistics.StatisticsError:
    pass
print("TestLinearRegression::test_constant_input_error: ok")
"###);
    assert_output(&out, r###"TestLinearRegression::test_constant_input_error: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_linear_regression__test_float_output.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_linear_regression__test_float_output() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_linear_regression__test_float_output"
# subject = "cpython.test_statistics.TestLinearRegression.test_float_output"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestLinearRegression::test_float_output
"""Auto-ported test: TestLinearRegression::test_float_output (CPython 3.12 oracle)."""


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
x = [Fraction(2, 3), Fraction(3, 4)]
y = [Fraction(4, 5), Fraction(5, 6)]
slope, intercept = statistics.linear_regression(x, y)

assert isinstance(slope, float)

assert isinstance(intercept, float)
slope, intercept = statistics.linear_regression(x, y, proportional=True)

assert isinstance(slope, float)

assert isinstance(intercept, float)
print("TestLinearRegression::test_float_output: ok")
"###);
    assert_output(&out, r###"TestLinearRegression::test_float_output: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_linear_regression__test_proportional.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_linear_regression__test_proportional() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_linear_regression__test_proportional"
# subject = "cpython.test_statistics.TestLinearRegression.test_proportional"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestLinearRegression::test_proportional
"""Auto-ported test: TestLinearRegression::test_proportional (CPython 3.12 oracle)."""


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
x = [10, 20, 30, 40]
y = [180, 398, 610, 799]
slope, intercept = statistics.linear_regression(x, y, proportional=True)

assert abs(slope - (20 + 1 / 150)) < 1e-07

assert intercept == 0.0
print("TestLinearRegression::test_proportional: ok")
"###);
    assert_output(&out, r###"TestLinearRegression::test_proportional: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_linear_regression__test_results.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_linear_regression__test_results() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_linear_regression__test_results"
# subject = "cpython.test_statistics.TestLinearRegression.test_results"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestLinearRegression::test_results
"""Auto-ported test: TestLinearRegression::test_results (CPython 3.12 oracle)."""


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
for x, y, true_intercept, true_slope in [([1, 2, 3], [0, 0, 0], 0, 0), ([1, 2, 3], [1, 2, 3], 0, 1), ([1, 2, 3], [100, 100, 100], 100, 0), ([1, 2, 3], [12, 14, 16], 10, 2), ([1, 2, 3], [-1, -2, -3], 0, -1), ([1, 2, 3], [21, 22, 23], 20, 1), ([1, 2, 3], [5.1, 5.2, 5.3], 5, 0.1)]:
    slope, intercept = statistics.linear_regression(x, y)

    assert abs(intercept - true_intercept) < 1e-07

    assert abs(slope - true_slope) < 1e-07
print("TestLinearRegression::test_results: ok")
"###);
    assert_output(&out, r###"TestLinearRegression::test_results: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_mean__test_empty_data.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_mean__test_empty_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_mean__test_empty_data"
# subject = "cpython.test_statistics.TestMean.test_empty_data"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestMean::test_empty_data
"""Auto-ported test: TestMean::test_empty_data (CPython 3.12 oracle)."""


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
self_func = statistics.mean
for empty in ([], (), iter([])):

    try:
        self_func(empty)
        raise AssertionError('expected statistics.StatisticsError')
    except statistics.StatisticsError:
        pass
print("TestMean::test_empty_data: ok")
"###);
    assert_output(&out, r###"TestMean::test_empty_data: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_mean__test_regression_20561.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_mean__test_regression_20561() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_mean__test_regression_20561"
# subject = "cpython.test_statistics.TestMean.test_regression_20561"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestMean::test_regression_20561
"""Auto-ported test: TestMean::test_regression_20561 (CPython 3.12 oracle)."""


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
self_func = statistics.mean
d = Decimal('1e4')

assert statistics.mean([d]) == d
print("TestMean::test_regression_20561: ok")
"###);
    assert_output(&out, r###"TestMean::test_regression_20561: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_mean__test_regression_25177.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_mean__test_regression_25177() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_mean__test_regression_25177"
# subject = "cpython.test_statistics.TestMean.test_regression_25177"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestMean::test_regression_25177
"""Auto-ported test: TestMean::test_regression_25177 (CPython 3.12 oracle)."""


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
self_func = statistics.mean

assert statistics.mean([8.988465674311579e+307, 8.98846567431158e+307]) == 8.98846567431158e+307
big = 8.98846567431158e+307
tiny = 5e-324
for n in (2, 3, 5, 200):

    assert statistics.mean([big] * n) == big

    assert statistics.mean([tiny] * n) == tiny
print("TestMean::test_regression_25177: ok")
"###);
    assert_output(&out, r###"TestMean::test_regression_25177: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_median__test_empty_data.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_median__test_empty_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_median__test_empty_data"
# subject = "cpython.test_statistics.TestMedian.test_empty_data"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestMedian::test_empty_data
"""Auto-ported test: TestMedian::test_empty_data (CPython 3.12 oracle)."""


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
self_func = statistics.median
for empty in ([], (), iter([])):

    try:
        self_func(empty)
        raise AssertionError('expected statistics.StatisticsError')
    except statistics.StatisticsError:
        pass
print("TestMedian::test_empty_data: ok")
"###);
    assert_output(&out, r###"TestMedian::test_empty_data: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_median_grouped__test_data_type_error.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_median_grouped__test_data_type_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_median_grouped__test_data_type_error"
# subject = "cpython.test_statistics.TestMedianGrouped.test_data_type_error"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestMedianGrouped::test_data_type_error
"""Auto-ported test: TestMedianGrouped::test_data_type_error (CPython 3.12 oracle)."""


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
self_func = statistics.median
self_func = statistics.median_grouped
data = ['', '', '']

try:
    self_func(data)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
data = [b'', b'', b'']

try:
    self_func(data)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
data = [1, 2, 3]
interval = ''

try:
    self_func(data, interval)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
data = [1, 2, 3]
interval = b''

try:
    self_func(data, interval)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestMedianGrouped::test_data_type_error: ok")
"###);
    assert_output(&out, r###"TestMedianGrouped::test_data_type_error: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_median_grouped__test_empty_data.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_median_grouped__test_empty_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_median_grouped__test_empty_data"
# subject = "cpython.test_statistics.TestMedianGrouped.test_empty_data"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestMedianGrouped::test_empty_data
"""Auto-ported test: TestMedianGrouped::test_empty_data (CPython 3.12 oracle)."""


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
self_func = statistics.median
self_func = statistics.median_grouped
for empty in ([], (), iter([])):

    try:
        self_func(empty)
        raise AssertionError('expected statistics.StatisticsError')
    except statistics.StatisticsError:
        pass
print("TestMedianGrouped::test_empty_data: ok")
"###);
    assert_output(&out, r###"TestMedianGrouped::test_empty_data: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_median_grouped__test_no_args.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_median_grouped__test_no_args() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_median_grouped__test_no_args"
# subject = "cpython.test_statistics.TestMedianGrouped.test_no_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestMedianGrouped::test_no_args
"""Auto-ported test: TestMedianGrouped::test_no_args (CPython 3.12 oracle)."""


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
self_func = statistics.median
self_func = statistics.median_grouped

try:
    self_func()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestMedianGrouped::test_no_args: ok")
"###);
    assert_output(&out, r###"TestMedianGrouped::test_no_args: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_median_high__test_empty_data.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_median_high__test_empty_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_median_high__test_empty_data"
# subject = "cpython.test_statistics.TestMedianHigh.test_empty_data"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestMedianHigh::test_empty_data
"""Auto-ported test: TestMedianHigh::test_empty_data (CPython 3.12 oracle)."""


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
self_func = statistics.median
self_func = statistics.median_high
for empty in ([], (), iter([])):

    try:
        self_func(empty)
        raise AssertionError('expected statistics.StatisticsError')
    except statistics.StatisticsError:
        pass
print("TestMedianHigh::test_empty_data: ok")
"###);
    assert_output(&out, r###"TestMedianHigh::test_empty_data: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_median_high__test_no_args.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_median_high__test_no_args() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_median_high__test_no_args"
# subject = "cpython.test_statistics.TestMedianHigh.test_no_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestMedianHigh::test_no_args
"""Auto-ported test: TestMedianHigh::test_no_args (CPython 3.12 oracle)."""


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
self_func = statistics.median
self_func = statistics.median_high

try:
    self_func()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestMedianHigh::test_no_args: ok")
"###);
    assert_output(&out, r###"TestMedianHigh::test_no_args: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_median_low__test_empty_data.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_median_low__test_empty_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_median_low__test_empty_data"
# subject = "cpython.test_statistics.TestMedianLow.test_empty_data"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestMedianLow::test_empty_data
"""Auto-ported test: TestMedianLow::test_empty_data (CPython 3.12 oracle)."""


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
self_func = statistics.median
self_func = statistics.median_low
for empty in ([], (), iter([])):

    try:
        self_func(empty)
        raise AssertionError('expected statistics.StatisticsError')
    except statistics.StatisticsError:
        pass
print("TestMedianLow::test_empty_data: ok")
"###);
    assert_output(&out, r###"TestMedianLow::test_empty_data: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_median_low__test_no_args.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_median_low__test_no_args() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_median_low__test_no_args"
# subject = "cpython.test_statistics.TestMedianLow.test_no_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestMedianLow::test_no_args
"""Auto-ported test: TestMedianLow::test_no_args (CPython 3.12 oracle)."""


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
self_func = statistics.median
self_func = statistics.median_low

try:
    self_func()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestMedianLow::test_no_args: ok")
"###);
    assert_output(&out, r###"TestMedianLow::test_no_args: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_mode__test_empty_data.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_mode__test_empty_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_mode__test_empty_data"
# subject = "cpython.test_statistics.TestMode.test_empty_data"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestMode::test_empty_data
"""Auto-ported test: TestMode::test_empty_data (CPython 3.12 oracle)."""


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
self_func = statistics.mode
for empty in ([], (), iter([])):

    try:
        self_func(empty)
        raise AssertionError('expected statistics.StatisticsError')
    except statistics.StatisticsError:
        pass
print("TestMode::test_empty_data: ok")
"###);
    assert_output(&out, r###"TestMode::test_empty_data: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_multi_mode__test_basics.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_multi_mode__test_basics() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_multi_mode__test_basics"
# subject = "cpython.test_statistics.TestMultiMode.test_basics"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestMultiMode::test_basics
"""Auto-ported test: TestMultiMode::test_basics (CPython 3.12 oracle)."""


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
multimode = statistics.multimode

assert multimode('aabbbbbbbbcc') == ['b']

assert multimode('aabbbbccddddeeffffgg') == ['b', 'd', 'f']

assert multimode('') == []
print("TestMultiMode::test_basics: ok")
"###);
    assert_output(&out, r###"TestMultiMode::test_basics: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_normal_dist_c__test_copy.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_normal_dist_c__test_copy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_normal_dist_c__test_copy"
# subject = "cpython.test_statistics.TestNormalDistC.test_copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestNormalDistC::test_copy
"""Auto-ported test: TestNormalDistC::test_copy (CPython 3.12 oracle)."""


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
module = c_statistics
sys.modules['statistics'] = module
nd = module.NormalDist(37.5, 5.625)
nd1 = copy.copy(nd)

assert nd == nd1
nd2 = copy.deepcopy(nd)

assert nd == nd2
print("TestNormalDistC::test_copy: ok")
"###);
    assert_output(&out, r###"TestNormalDistC::test_copy: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_normal_dist_c__test_hashability.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_normal_dist_c__test_hashability() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_normal_dist_c__test_hashability"
# subject = "cpython.test_statistics.TestNormalDistC.test_hashability"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestNormalDistC::test_hashability
"""Auto-ported test: TestNormalDistC::test_hashability (CPython 3.12 oracle)."""


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
module = c_statistics
sys.modules['statistics'] = module
ND = module.NormalDist
s = {ND(100, 15), ND(100.0, 15.0), ND(100, 10), ND(95, 15), ND(100, 15)}

assert len(s) == 3
print("TestNormalDistC::test_hashability: ok")
"###);
    assert_output(&out, r###"TestNormalDistC::test_hashability: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_normal_dist_c__test_pickle.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_normal_dist_c__test_pickle() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_normal_dist_c__test_pickle"
# subject = "cpython.test_statistics.TestNormalDistC.test_pickle"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestNormalDistC::test_pickle
"""Auto-ported test: TestNormalDistC::test_pickle (CPython 3.12 oracle)."""


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
module = c_statistics
sys.modules['statistics'] = module
nd = module.NormalDist(37.5, 5.625)
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    pickled = pickle.loads(pickle.dumps(nd, protocol=proto))

    assert nd == pickled
print("TestNormalDistC::test_pickle: ok")
"###);
    assert_output(&out, r###"TestNormalDistC::test_pickle: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_normal_dist_c__test_properties.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_normal_dist_c__test_properties() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_normal_dist_c__test_properties"
# subject = "cpython.test_statistics.TestNormalDistC.test_properties"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestNormalDistC::test_properties
"""Auto-ported test: TestNormalDistC::test_properties (CPython 3.12 oracle)."""


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
module = c_statistics
sys.modules['statistics'] = module
X = module.NormalDist(100, 15)

assert X.mean == 100

assert X.median == 100

assert X.mode == 100

assert X.stdev == 15

assert X.variance == 225
print("TestNormalDistC::test_properties: ok")
"###);
    assert_output(&out, r###"TestNormalDistC::test_properties: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_normal_dist_python__test_copy.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_normal_dist_python__test_copy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_normal_dist_python__test_copy"
# subject = "cpython.test_statistics.TestNormalDistPython.test_copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestNormalDistPython::test_copy
"""Auto-ported test: TestNormalDistPython::test_copy (CPython 3.12 oracle)."""


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
module = py_statistics
sys.modules['statistics'] = module
nd = module.NormalDist(37.5, 5.625)
nd1 = copy.copy(nd)

assert nd == nd1
nd2 = copy.deepcopy(nd)

assert nd == nd2
print("TestNormalDistPython::test_copy: ok")
"###);
    assert_output(&out, r###"TestNormalDistPython::test_copy: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_normal_dist_python__test_hashability.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_normal_dist_python__test_hashability() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_normal_dist_python__test_hashability"
# subject = "cpython.test_statistics.TestNormalDistPython.test_hashability"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestNormalDistPython::test_hashability
"""Auto-ported test: TestNormalDistPython::test_hashability (CPython 3.12 oracle)."""


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
module = py_statistics
sys.modules['statistics'] = module
ND = module.NormalDist
s = {ND(100, 15), ND(100.0, 15.0), ND(100, 10), ND(95, 15), ND(100, 15)}

assert len(s) == 3
print("TestNormalDistPython::test_hashability: ok")
"###);
    assert_output(&out, r###"TestNormalDistPython::test_hashability: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_normal_dist_python__test_pickle.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_normal_dist_python__test_pickle() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_normal_dist_python__test_pickle"
# subject = "cpython.test_statistics.TestNormalDistPython.test_pickle"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestNormalDistPython::test_pickle
"""Auto-ported test: TestNormalDistPython::test_pickle (CPython 3.12 oracle)."""


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
module = py_statistics
sys.modules['statistics'] = module
nd = module.NormalDist(37.5, 5.625)
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    pickled = pickle.loads(pickle.dumps(nd, protocol=proto))

    assert nd == pickled
print("TestNormalDistPython::test_pickle: ok")
"###);
    assert_output(&out, r###"TestNormalDistPython::test_pickle: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_normal_dist_python__test_properties.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_normal_dist_python__test_properties() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_normal_dist_python__test_properties"
# subject = "cpython.test_statistics.TestNormalDistPython.test_properties"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestNormalDistPython::test_properties
"""Auto-ported test: TestNormalDistPython::test_properties (CPython 3.12 oracle)."""


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
module = py_statistics
sys.modules['statistics'] = module
X = module.NormalDist(100, 15)

assert X.mean == 100

assert X.median == 100

assert X.mode == 100

assert X.stdev == 15

assert X.variance == 225
print("TestNormalDistPython::test_properties: ok")
"###);
    assert_output(&out, r###"TestNormalDistPython::test_properties: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_numeric_test_case__test_numerictestcase_is_testcase.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_numeric_test_case__test_numerictestcase_is_testcase() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_numeric_test_case__test_numerictestcase_is_testcase"
# subject = "cpython.test_statistics.TestNumericTestCase.test_numerictestcase_is_testcase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestNumericTestCase::test_numerictestcase_is_testcase
"""Auto-ported test: TestNumericTestCase::test_numerictestcase_is_testcase (CPython 3.12 oracle)."""


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

assert issubclass(NumericTestCase, unittest.TestCase)
print("TestNumericTestCase::test_numerictestcase_is_testcase: ok")
"###);
    assert_output(&out, r###"TestNumericTestCase::test_numerictestcase_is_testcase: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_p_stdev__test_empty_data.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_p_stdev__test_empty_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_p_stdev__test_empty_data"
# subject = "cpython.test_statistics.TestPStdev.test_empty_data"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestPStdev::test_empty_data
"""Auto-ported test: TestPStdev::test_empty_data (CPython 3.12 oracle)."""


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
rel = 1e-12
self_func = statistics.pstdev
for empty in ([], (), iter([])):

    try:
        self_func(empty)
        raise AssertionError('expected statistics.StatisticsError')
    except statistics.StatisticsError:
        pass
print("TestPStdev::test_empty_data: ok")
"###);
    assert_output(&out, r###"TestPStdev::test_empty_data: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_p_variance__test_empty_data.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_p_variance__test_empty_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_p_variance__test_empty_data"
# subject = "cpython.test_statistics.TestPVariance.test_empty_data"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestPVariance::test_empty_data
"""Auto-ported test: TestPVariance::test_empty_data (CPython 3.12 oracle)."""


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
rel = 1e-12
self_func = statistics.pvariance
for empty in ([], (), iter([])):

    try:
        self_func(empty)
        raise AssertionError('expected statistics.StatisticsError')
    except statistics.StatisticsError:
        pass
print("TestPVariance::test_empty_data: ok")
"###);
    assert_output(&out, r###"TestPVariance::test_empty_data: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_quantiles__test_equal_inputs.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_quantiles__test_equal_inputs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_quantiles__test_equal_inputs"
# subject = "cpython.test_statistics.TestQuantiles.test_equal_inputs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestQuantiles::test_equal_inputs
"""Auto-ported test: TestQuantiles::test_equal_inputs (CPython 3.12 oracle)."""


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
for n in range(2, 10):
    data = [10.0] * n

    assert quantiles(data) == [10.0, 10.0, 10.0]

    assert quantiles(data, method='inclusive') == [10.0, 10.0, 10.0]
print("TestQuantiles::test_equal_inputs: ok")
"###);
    assert_output(&out, r###"TestQuantiles::test_equal_inputs: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_quantiles__test_equal_sized_groups.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_quantiles__test_equal_sized_groups() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_quantiles__test_equal_sized_groups"
# subject = "cpython.test_statistics.TestQuantiles.test_equal_sized_groups"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestQuantiles::test_equal_sized_groups
"""Auto-ported test: TestQuantiles::test_equal_sized_groups (CPython 3.12 oracle)."""


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
total = 10000
data = [random.expovariate(0.2) for i in range(total)]
while len(set(data)) != total:
    data.append(random.expovariate(0.2))
data.sort()
for n in (1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10000):
    group_size = total // n

    assert [bisect.bisect(data, q) for q in quantiles(data, n=n)] == list(range(group_size, total, group_size))
for n in (13, 19, 59, 109, 211, 571, 1019, 1907, 5261, 9769):
    group_sizes = {total // n, total // n + 1}
    pos = [bisect.bisect(data, q) for q in quantiles(data, n=n)]
    sizes = {q - p for p, q in zip(pos, pos[1:])}

    assert sizes <= group_sizes
print("TestQuantiles::test_equal_sized_groups: ok")
"###);
    assert_output(&out, r###"TestQuantiles::test_equal_sized_groups: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_stdev__test_empty_data.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_stdev__test_empty_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_stdev__test_empty_data"
# subject = "cpython.test_statistics.TestStdev.test_empty_data"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestStdev::test_empty_data
"""Auto-ported test: TestStdev::test_empty_data (CPython 3.12 oracle)."""


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
rel = 1e-12
self_func = statistics.stdev
for empty in ([], (), iter([])):

    try:
        self_func(empty)
        raise AssertionError('expected statistics.StatisticsError')
    except statistics.StatisticsError:
        pass
print("TestStdev::test_empty_data: ok")
"###);
    assert_output(&out, r###"TestStdev::test_empty_data: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_stdev__test_single_value.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_stdev__test_single_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_stdev__test_single_value"
# subject = "cpython.test_statistics.TestStdev.test_single_value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestStdev::test_single_value
"""Auto-ported test: TestStdev::test_single_value (CPython 3.12 oracle)."""


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
rel = 1e-12
self_func = statistics.stdev
for x in (81, 203.74, 390000000000000.0, Fraction(5, 21), Decimal('35.719')):

    try:
        self_func([x])
        raise AssertionError('expected statistics.StatisticsError')
    except statistics.StatisticsError:
        pass
print("TestStdev::test_single_value: ok")
"###);
    assert_output(&out, r###"TestStdev::test_single_value: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_variance__test_empty_data.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_variance__test_empty_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_variance__test_empty_data"
# subject = "cpython.test_statistics.TestVariance.test_empty_data"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestVariance::test_empty_data
"""Auto-ported test: TestVariance::test_empty_data (CPython 3.12 oracle)."""


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
rel = 1e-12
self_func = statistics.variance
for empty in ([], (), iter([])):

    try:
        self_func(empty)
        raise AssertionError('expected statistics.StatisticsError')
    except statistics.StatisticsError:
        pass
print("TestVariance::test_empty_data: ok")
"###);
    assert_output(&out, r###"TestVariance::test_empty_data: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/test_variance__test_single_value.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_test_variance__test_single_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_variance__test_single_value"
# subject = "cpython.test_statistics.TestVariance.test_single_value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_statistics.py::TestVariance::test_single_value
"""Auto-ported test: TestVariance::test_single_value (CPython 3.12 oracle)."""


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
rel = 1e-12
self_func = statistics.variance
for x in (35, 24.7, 8200000000000000.0, Fraction(19, 30), Decimal('4.2084')):

    try:
        self_func([x])
        raise AssertionError('expected statistics.StatisticsError')
    except statistics.StatisticsError:
        pass
print("TestVariance::test_single_value: ok")
"###);
    assert_output(&out, r###"TestVariance::test_single_value: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/statistics/variance_is_stdev_squared.py`.
#[test]
fn test_gen_behavior_std_libs_statistics_variance_is_stdev_squared() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "variance_is_stdev_squared"
# subject = "statistics.variance"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.variance: sample variance equals the square of the sample stdev for the same data, with the exact CPython value 4.571428571428571"""
import statistics

_data = [2, 4, 4, 4, 5, 5, 7, 9]
_s = statistics.stdev(_data)
_v = statistics.variance(_data)
# variance == stdev ** 2.
assert abs(_v - _s ** 2) < 1e-10, (_v, _s)
assert abs(_v - 4.571428571428571) < 1e-10, _v
# pvariance for 1..5 is exactly 2.0.
assert abs(statistics.pvariance([1, 2, 3, 4, 5]) - 2.0) < 1e-10

print("variance_is_stdev_squared OK")
"###);
    assert_output(&out, r###"variance_is_stdev_squared OK
"###);
}
