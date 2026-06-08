# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "is_close_tests__test_reject_complex_tolerances"
# subject = "cpython.test_cmath.IsCloseTests.test_reject_complex_tolerances"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmath.py::IsCloseTests::test_reject_complex_tolerances
"""Auto-ported test: IsCloseTests::test_reject_complex_tolerances (CPython 3.12 oracle)."""


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
isclose = cmath.isclose
try:
    isclose(1j, 1j, rel_tol=1j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    isclose(1j, 1j, abs_tol=1j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    isclose(1j, 1j, rel_tol=1j, abs_tol=1j)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("IsCloseTests::test_reject_complex_tolerances: ok")
