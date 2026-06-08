# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "c_math_tests__test_abs"
# subject = "cpython.test_cmath.CMathTests.test_abs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmath.py::CMathTests::test_abs
"""Auto-ported test: CMathTests::test_abs (CPython 3.12 oracle)."""


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
for z in complex_zeros:

    assert abs(z) == 0.0
for z in complex_infinities:

    assert abs(z) == INF

assert abs(complex(NAN, -INF)) == INF

assert math.isnan(abs(complex(NAN, -2.3)))

assert math.isnan(abs(complex(NAN, -0.0)))

assert math.isnan(abs(complex(NAN, 0.0)))

assert math.isnan(abs(complex(NAN, 2.3)))

assert abs(complex(NAN, INF)) == INF

assert abs(complex(-INF, NAN)) == INF

assert math.isnan(abs(complex(-2.3, NAN)))

assert math.isnan(abs(complex(-0.0, NAN)))

assert math.isnan(abs(complex(0.0, NAN)))

assert math.isnan(abs(complex(2.3, NAN)))

assert abs(complex(INF, NAN)) == INF

assert math.isnan(abs(complex(NAN, NAN)))
print("CMathTests::test_abs: ok")
