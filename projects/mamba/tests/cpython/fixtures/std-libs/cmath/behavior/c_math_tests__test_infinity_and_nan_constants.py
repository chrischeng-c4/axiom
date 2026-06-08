# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "c_math_tests__test_infinity_and_nan_constants"
# subject = "cpython.test_cmath.CMathTests.test_infinity_and_nan_constants"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmath.py::CMathTests::test_infinity_and_nan_constants
"""Auto-ported test: CMathTests::test_infinity_and_nan_constants (CPython 3.12 oracle)."""


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

assert cmath.inf.real == math.inf

assert cmath.inf.imag == 0.0

assert cmath.infj.real == 0.0

assert cmath.infj.imag == math.inf

assert math.isnan(cmath.nan.real)

assert cmath.nan.imag == 0.0

assert cmath.nanj.real == 0.0

assert math.isnan(cmath.nanj.imag)

assert math.copysign(1.0, cmath.nan.real) == 1.0

assert math.copysign(1.0, cmath.nan.imag) == 1.0

assert math.copysign(1.0, cmath.nanj.real) == 1.0

assert math.copysign(1.0, cmath.nanj.imag) == 1.0

assert repr(cmath.inf) == 'inf'

assert repr(cmath.infj) == 'infj'

assert repr(cmath.nan) == 'nan'

assert repr(cmath.nanj) == 'nanj'
print("CMathTests::test_infinity_and_nan_constants: ok")
