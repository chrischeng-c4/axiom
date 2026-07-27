use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/builtin-libs/complex/complex_test__test_abs.py`.
#[test]
fn test_gen_behavior_builtin_libs_complex_complex_test__test_abs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_abs"
# subject = "cpython.test_complex.ComplexTest.test_abs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_complex.py::ComplexTest::test_abs
"""Auto-ported test: ComplexTest::test_abs (CPython 3.12 oracle)."""


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
nums = [complex(x / 3.0, y / 7.0) for x in range(-9, 9) for y in range(-9, 9)]
for num in nums:

    assert abs((num.real ** 2 + num.imag ** 2) ** 0.5 - abs(num)) < 1e-07
print("ComplexTest::test_abs: ok")
"###);
    assert_output(&out, r###"ComplexTest::test_abs: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/complex/complex_test__test_add.py`.
#[test]
fn test_gen_behavior_builtin_libs_complex_complex_test__test_add() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_add"
# subject = "cpython.test_complex.ComplexTest.test_add"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_complex.py::ComplexTest::test_add
"""Auto-ported test: ComplexTest::test_add (CPython 3.12 oracle)."""


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

assert 1j + int(+1) == complex(+1, 1)

assert 1j + int(-1) == complex(-1, 1)

try:
    operator.add(1j, 10 ** 1000)
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass

try:
    operator.add(1j, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.add(None, 1j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ComplexTest::test_add: ok")
"###);
    assert_output(&out, r###"ComplexTest::test_add: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/complex/complex_test__test_boolcontext.py`.
#[test]
fn test_gen_behavior_builtin_libs_complex_complex_test__test_boolcontext() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_boolcontext"
# subject = "cpython.test_complex.ComplexTest.test_boolcontext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_complex.py::ComplexTest::test_boolcontext
"""Auto-ported test: ComplexTest::test_boolcontext (CPython 3.12 oracle)."""


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
for i in range(100):

    assert complex(random() + 1e-06, random() + 1e-06)

assert not complex(0.0, 0.0)

assert 1j
print("ComplexTest::test_boolcontext: ok")
"###);
    assert_output(&out, r###"ComplexTest::test_boolcontext: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/complex/complex_test__test_conjugate.py`.
#[test]
fn test_gen_behavior_builtin_libs_complex_complex_test__test_conjugate() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_conjugate"
# subject = "cpython.test_complex.ComplexTest.test_conjugate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_complex.py::ComplexTest::test_conjugate
"""Auto-ported test: ComplexTest::test_conjugate (CPython 3.12 oracle)."""


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
assertClose(complex(5.3, 9.8).conjugate(), 5.3 - 9.8j)
print("ComplexTest::test_conjugate: ok")
"###);
    assert_output(&out, r###"ComplexTest::test_conjugate: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/complex/complex_test__test_divmod.py`.
#[test]
fn test_gen_behavior_builtin_libs_complex_complex_test__test_divmod() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_divmod"
# subject = "cpython.test_complex.ComplexTest.test_divmod"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_complex.py::ComplexTest::test_divmod
"""Auto-ported test: ComplexTest::test_divmod (CPython 3.12 oracle)."""


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

try:
    divmod(1 + 1j, 1 + 0j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    divmod(1 + 1j, 1.0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    divmod(1 + 1j, 1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    divmod(1.0, 1 + 0j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    divmod(1, 1 + 0j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ComplexTest::test_divmod: ok")
"###);
    assert_output(&out, r###"ComplexTest::test_divmod: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/complex/complex_test__test_divmod_zero_division.py`.
#[test]
fn test_gen_behavior_builtin_libs_complex_complex_test__test_divmod_zero_division() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_divmod_zero_division"
# subject = "cpython.test.test_complex.ComplexTest.test_divmod_zero_division"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_complex.py::ComplexTest::test_divmod_zero_division
"""Auto-ported test: ComplexTest::test_divmod_zero_division (CPython 3.12 oracle)."""


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
for a, b in ZERO_DIVISION:

    try:
        divmod(a, b)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("ComplexTest::test_divmod_zero_division: ok")
"###);
    assert_output(&out, r###"ComplexTest::test_divmod_zero_division: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/complex/complex_test__test_neg.py`.
#[test]
fn test_gen_behavior_builtin_libs_complex_complex_test__test_neg() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_neg"
# subject = "cpython.test_complex.ComplexTest.test_neg"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_complex.py::ComplexTest::test_neg
"""Auto-ported test: ComplexTest::test_neg (CPython 3.12 oracle)."""


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

assert -(1 + 6j) == -1 - 6j
print("ComplexTest::test_neg: ok")
"###);
    assert_output(&out, r###"ComplexTest::test_neg: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/complex/complex_test__test_overflow.py`.
#[test]
fn test_gen_behavior_builtin_libs_complex_complex_test__test_overflow() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_overflow"
# subject = "cpython.test_complex.ComplexTest.test_overflow"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_complex.py::ComplexTest::test_overflow
"""Auto-ported test: ComplexTest::test_overflow (CPython 3.12 oracle)."""


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

assert complex('1e500') == complex(INF, 0.0)

assert complex('-1e500j') == complex(0.0, -INF)

assert complex('-1e500+1.8e308j') == complex(-INF, INF)
print("ComplexTest::test_overflow: ok")
"###);
    assert_output(&out, r###"ComplexTest::test_overflow: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/complex/complex_test__test_pow_with_small_integer_exponents.py`.
#[test]
fn test_gen_behavior_builtin_libs_complex_complex_test__test_pow_with_small_integer_exponents() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_pow_with_small_integer_exponents"
# subject = "cpython.test_complex.ComplexTest.test_pow_with_small_integer_exponents"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_complex.py::ComplexTest::test_pow_with_small_integer_exponents
"""Auto-ported test: ComplexTest::test_pow_with_small_integer_exponents (CPython 3.12 oracle)."""


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
values = [complex(5.0, 12.0), complex(5e+100, 1.2e+101), complex(-4.0, INF), complex(INF, 0.0)]
exponents = [-19, -5, -3, -2, -1, 0, 1, 2, 3, 5, 19]
for value in values:
    for exponent in exponents:
        try:
            int_pow = value ** exponent
        except OverflowError:
            int_pow = 'overflow'
        try:
            float_pow = value ** float(exponent)
        except OverflowError:
            float_pow = 'overflow'
        try:
            complex_pow = value ** complex(exponent)
        except OverflowError:
            complex_pow = 'overflow'

        assert str(float_pow) == str(int_pow)

        assert str(complex_pow) == str(int_pow)
print("ComplexTest::test_pow_with_small_integer_exponents: ok")
"###);
    assert_output(&out, r###"ComplexTest::test_pow_with_small_integer_exponents: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/complex/complex_test__test_sub.py`.
#[test]
fn test_gen_behavior_builtin_libs_complex_complex_test__test_sub() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_sub"
# subject = "cpython.test_complex.ComplexTest.test_sub"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_complex.py::ComplexTest::test_sub
"""Auto-ported test: ComplexTest::test_sub (CPython 3.12 oracle)."""


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

assert 1j - int(+1) == complex(-1, 1)

assert 1j - int(-1) == complex(1, 1)

try:
    operator.sub(1j, 10 ** 1000)
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass

try:
    operator.sub(1j, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.sub(None, 1j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ComplexTest::test_sub: ok")
"###);
    assert_output(&out, r###"ComplexTest::test_sub: ok
"###);
}
