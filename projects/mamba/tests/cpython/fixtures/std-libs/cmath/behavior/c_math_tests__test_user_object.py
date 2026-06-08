# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "c_math_tests__test_user_object"
# subject = "cpython.test_cmath.CMathTests.test_user_object"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmath.py::CMathTests::test_user_object
"""Auto-ported test: CMathTests::test_user_object (CPython 3.12 oracle)."""


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
self_test_values = open(test_file, encoding='utf-8')
cx_arg = 4.419414439 + 1.497100113j
flt_arg = -6.131677725
non_complexes = ['not complex', 1, 5, 2.0, None, object(), NotImplemented]

class MyComplex:

    def __init__(self, value):
        self.value = value

    def __complex__(self):
        return self.value

class SomeException(Exception):
    pass

class MyComplexException:

    def __complex__(self):
        raise SomeException

class NeitherComplexNorFloat(object):
    pass

class Index:

    def __int__(self):
        return 2

    def __index__(self):
        return 2

class MyInt:

    def __int__(self):
        return 2

class FloatAndComplex:

    def __float__(self):
        return flt_arg

    def __complex__(self):
        return cx_arg

class JustFloat:

    def __float__(self):
        return flt_arg
for f in test_functions:

    assert f(MyComplex(cx_arg)) == f(cx_arg)

    assert f(FloatAndComplex()) == f(cx_arg)

    assert f(JustFloat()) == f(flt_arg)

    assert f(Index()) == f(int(Index()))

    try:
        f(NeitherComplexNorFloat())
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        f(MyInt())
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    for bad_complex in non_complexes:

        try:
            f(MyComplex(bad_complex))
            raise AssertionError('expected TypeError')
        except TypeError:
            pass

    try:
        f(MyComplexException())
        raise AssertionError('expected SomeException')
    except SomeException:
        pass
print("CMathTests::test_user_object: ok")
