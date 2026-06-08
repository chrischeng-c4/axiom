# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "general_float_cases__test_floatasratio"
# subject = "cpython.test_float.GeneralFloatCases.test_floatasratio"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_float.py::GeneralFloatCases::test_floatasratio
"""Auto-ported test: GeneralFloatCases::test_floatasratio (CPython 3.12 oracle)."""


import fractions
import operator
import os
import random
import sys
import struct
import time
import unittest
from test import support
from test.support.testcase import FloatsAreIdenticalMixin
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS
from math import isinf, isnan, copysign, ldexp
import math


try:
    import _testcapi
except ImportError:
    _testcapi = None

INF = float('inf')

NAN = float('nan')

test_dir = os.path.dirname(__file__) or os.curdir

format_testfile = os.path.join(test_dir, 'formatfloat_testcases.txt')

class FloatSubclass(float):
    pass

class OtherFloatSubclass(float):
    pass

BE_DOUBLE_INF = b'\x7f\xf0\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_INF = bytes(reversed(BE_DOUBLE_INF))

BE_DOUBLE_NAN = b'\x7f\xf8\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_NAN = bytes(reversed(BE_DOUBLE_NAN))

BE_FLOAT_INF = b'\x7f\x80\x00\x00'

LE_FLOAT_INF = bytes(reversed(BE_FLOAT_INF))

BE_FLOAT_NAN = b'\x7f\xc0\x00\x00'

LE_FLOAT_NAN = bytes(reversed(BE_FLOAT_NAN))

fromHex = float.fromhex

toHex = float.hex


# --- test body ---
for f, ratio in [(0.875, (7, 8)), (-0.875, (-7, 8)), (0.0, (0, 1)), (11.5, (23, 2))]:

    assert f.as_integer_ratio() == ratio
for i in range(10000):
    f = random.random()
    f *= 10 ** random.randint(-100, 100)
    n, d = f.as_integer_ratio()

    assert float(n).__truediv__(d) == f
R = fractions.Fraction

assert R(0, 1) == R(*float(0.0).as_integer_ratio())

assert R(5, 2) == R(*float(2.5).as_integer_ratio())

assert R(1, 2) == R(*float(0.5).as_integer_ratio())

assert R(4728779608739021, 2251799813685248) == R(*float(2.1).as_integer_ratio())

assert R(-4728779608739021, 2251799813685248) == R(*float(-2.1).as_integer_ratio())

assert R(-2100, 1) == R(*float(-2100.0).as_integer_ratio())

try:
    float('inf').as_integer_ratio()
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass

try:
    float('-inf').as_integer_ratio()
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass

try:
    float('nan').as_integer_ratio()
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("GeneralFloatCases::test_floatasratio: ok")
