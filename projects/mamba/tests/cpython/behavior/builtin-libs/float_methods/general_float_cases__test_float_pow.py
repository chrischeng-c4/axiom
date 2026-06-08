# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "general_float_cases__test_float_pow"
# subject = "cpython.test_float.GeneralFloatCases.test_float_pow"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_float.py::GeneralFloatCases::test_float_pow
"""Auto-ported test: GeneralFloatCases::test_float_pow (CPython 3.12 oracle)."""


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
def assertEqualAndEqualSign(a, b):

    assert (a, copysign(1.0, a)) == (b, copysign(1.0, b))
for pow_op in (pow, operator.pow):

    assert isnan(pow_op(-INF, NAN))

    assert isnan(pow_op(-2.0, NAN))

    assert isnan(pow_op(-1.0, NAN))

    assert isnan(pow_op(-0.5, NAN))

    assert isnan(pow_op(-0.0, NAN))

    assert isnan(pow_op(0.0, NAN))

    assert isnan(pow_op(0.5, NAN))

    assert isnan(pow_op(2.0, NAN))

    assert isnan(pow_op(INF, NAN))

    assert isnan(pow_op(NAN, NAN))

    assert isnan(pow_op(NAN, -INF))

    assert isnan(pow_op(NAN, -2.0))

    assert isnan(pow_op(NAN, -1.0))

    assert isnan(pow_op(NAN, -0.5))

    assert isnan(pow_op(NAN, 0.5))

    assert isnan(pow_op(NAN, 1.0))

    assert isnan(pow_op(NAN, 2.0))

    assert isnan(pow_op(NAN, INF))

    try:
        pow_op(-0.0, -1.0)
        raise AssertionError('expected ZeroDivisionError')
    except ZeroDivisionError:
        pass

    try:
        pow_op(0.0, -1.0)
        raise AssertionError('expected ZeroDivisionError')
    except ZeroDivisionError:
        pass

    try:
        pow_op(-0.0, -2.0)
        raise AssertionError('expected ZeroDivisionError')
    except ZeroDivisionError:
        pass

    try:
        pow_op(-0.0, -0.5)
        raise AssertionError('expected ZeroDivisionError')
    except ZeroDivisionError:
        pass

    try:
        pow_op(0.0, -2.0)
        raise AssertionError('expected ZeroDivisionError')
    except ZeroDivisionError:
        pass

    try:
        pow_op(0.0, -0.5)
        raise AssertionError('expected ZeroDivisionError')
    except ZeroDivisionError:
        pass
    assertEqualAndEqualSign(pow_op(-0.0, 1.0), -0.0)
    assertEqualAndEqualSign(pow_op(0.0, 1.0), 0.0)
    assertEqualAndEqualSign(pow_op(-0.0, 0.5), 0.0)
    assertEqualAndEqualSign(pow_op(-0.0, 2.0), 0.0)
    assertEqualAndEqualSign(pow_op(0.0, 0.5), 0.0)
    assertEqualAndEqualSign(pow_op(0.0, 2.0), 0.0)
    assertEqualAndEqualSign(pow_op(-1.0, -INF), 1.0)
    assertEqualAndEqualSign(pow_op(-1.0, INF), 1.0)
    assertEqualAndEqualSign(pow_op(1.0, -INF), 1.0)
    assertEqualAndEqualSign(pow_op(1.0, -2.0), 1.0)
    assertEqualAndEqualSign(pow_op(1.0, -1.0), 1.0)
    assertEqualAndEqualSign(pow_op(1.0, -0.5), 1.0)
    assertEqualAndEqualSign(pow_op(1.0, -0.0), 1.0)
    assertEqualAndEqualSign(pow_op(1.0, 0.0), 1.0)
    assertEqualAndEqualSign(pow_op(1.0, 0.5), 1.0)
    assertEqualAndEqualSign(pow_op(1.0, 1.0), 1.0)
    assertEqualAndEqualSign(pow_op(1.0, 2.0), 1.0)
    assertEqualAndEqualSign(pow_op(1.0, INF), 1.0)
    assertEqualAndEqualSign(pow_op(1.0, NAN), 1.0)
    assertEqualAndEqualSign(pow_op(-INF, 0.0), 1.0)
    assertEqualAndEqualSign(pow_op(-2.0, 0.0), 1.0)
    assertEqualAndEqualSign(pow_op(-1.0, 0.0), 1.0)
    assertEqualAndEqualSign(pow_op(-0.5, 0.0), 1.0)
    assertEqualAndEqualSign(pow_op(-0.0, 0.0), 1.0)
    assertEqualAndEqualSign(pow_op(0.0, 0.0), 1.0)
    assertEqualAndEqualSign(pow_op(0.5, 0.0), 1.0)
    assertEqualAndEqualSign(pow_op(1.0, 0.0), 1.0)
    assertEqualAndEqualSign(pow_op(2.0, 0.0), 1.0)
    assertEqualAndEqualSign(pow_op(INF, 0.0), 1.0)
    assertEqualAndEqualSign(pow_op(NAN, 0.0), 1.0)
    assertEqualAndEqualSign(pow_op(-INF, -0.0), 1.0)
    assertEqualAndEqualSign(pow_op(-2.0, -0.0), 1.0)
    assertEqualAndEqualSign(pow_op(-1.0, -0.0), 1.0)
    assertEqualAndEqualSign(pow_op(-0.5, -0.0), 1.0)
    assertEqualAndEqualSign(pow_op(-0.0, -0.0), 1.0)
    assertEqualAndEqualSign(pow_op(0.0, -0.0), 1.0)
    assertEqualAndEqualSign(pow_op(0.5, -0.0), 1.0)
    assertEqualAndEqualSign(pow_op(1.0, -0.0), 1.0)
    assertEqualAndEqualSign(pow_op(2.0, -0.0), 1.0)
    assertEqualAndEqualSign(pow_op(INF, -0.0), 1.0)
    assertEqualAndEqualSign(pow_op(NAN, -0.0), 1.0)

    assert type(pow_op(-2.0, -0.5)) == complex

    assert type(pow_op(-2.0, 0.5)) == complex

    assert type(pow_op(-1.0, -0.5)) == complex

    assert type(pow_op(-1.0, 0.5)) == complex

    assert type(pow_op(-0.5, -0.5)) == complex

    assert type(pow_op(-0.5, 0.5)) == complex
    assertEqualAndEqualSign(pow_op(-0.5, -INF), INF)
    assertEqualAndEqualSign(pow_op(-0.0, -INF), INF)
    assertEqualAndEqualSign(pow_op(0.0, -INF), INF)
    assertEqualAndEqualSign(pow_op(0.5, -INF), INF)
    assertEqualAndEqualSign(pow_op(-INF, -INF), 0.0)
    assertEqualAndEqualSign(pow_op(-2.0, -INF), 0.0)
    assertEqualAndEqualSign(pow_op(2.0, -INF), 0.0)
    assertEqualAndEqualSign(pow_op(INF, -INF), 0.0)
    assertEqualAndEqualSign(pow_op(-0.5, INF), 0.0)
    assertEqualAndEqualSign(pow_op(-0.0, INF), 0.0)
    assertEqualAndEqualSign(pow_op(0.0, INF), 0.0)
    assertEqualAndEqualSign(pow_op(0.5, INF), 0.0)
    assertEqualAndEqualSign(pow_op(-INF, INF), INF)
    assertEqualAndEqualSign(pow_op(-2.0, INF), INF)
    assertEqualAndEqualSign(pow_op(2.0, INF), INF)
    assertEqualAndEqualSign(pow_op(INF, INF), INF)
    assertEqualAndEqualSign(pow_op(-INF, -1.0), -0.0)
    assertEqualAndEqualSign(pow_op(-INF, -0.5), 0.0)
    assertEqualAndEqualSign(pow_op(-INF, -2.0), 0.0)
    assertEqualAndEqualSign(pow_op(-INF, 1.0), -INF)
    assertEqualAndEqualSign(pow_op(-INF, 0.5), INF)
    assertEqualAndEqualSign(pow_op(-INF, 2.0), INF)
    assertEqualAndEqualSign(pow_op(INF, 0.5), INF)
    assertEqualAndEqualSign(pow_op(INF, 1.0), INF)
    assertEqualAndEqualSign(pow_op(INF, 2.0), INF)
    assertEqualAndEqualSign(pow_op(INF, -2.0), 0.0)
    assertEqualAndEqualSign(pow_op(INF, -1.0), 0.0)
    assertEqualAndEqualSign(pow_op(INF, -0.5), 0.0)
    assertEqualAndEqualSign(pow_op(-2.0, -2.0), 0.25)
    assertEqualAndEqualSign(pow_op(-2.0, -1.0), -0.5)
    assertEqualAndEqualSign(pow_op(-2.0, -0.0), 1.0)
    assertEqualAndEqualSign(pow_op(-2.0, 0.0), 1.0)
    assertEqualAndEqualSign(pow_op(-2.0, 1.0), -2.0)
    assertEqualAndEqualSign(pow_op(-2.0, 2.0), 4.0)
    assertEqualAndEqualSign(pow_op(-1.0, -2.0), 1.0)
    assertEqualAndEqualSign(pow_op(-1.0, -1.0), -1.0)
    assertEqualAndEqualSign(pow_op(-1.0, -0.0), 1.0)
    assertEqualAndEqualSign(pow_op(-1.0, 0.0), 1.0)
    assertEqualAndEqualSign(pow_op(-1.0, 1.0), -1.0)
    assertEqualAndEqualSign(pow_op(-1.0, 2.0), 1.0)
    assertEqualAndEqualSign(pow_op(2.0, -2.0), 0.25)
    assertEqualAndEqualSign(pow_op(2.0, -1.0), 0.5)
    assertEqualAndEqualSign(pow_op(2.0, -0.0), 1.0)
    assertEqualAndEqualSign(pow_op(2.0, 0.0), 1.0)
    assertEqualAndEqualSign(pow_op(2.0, 1.0), 2.0)
    assertEqualAndEqualSign(pow_op(2.0, 2.0), 4.0)
    assertEqualAndEqualSign(pow_op(1.0, -1e+100), 1.0)
    assertEqualAndEqualSign(pow_op(1.0, 1e+100), 1.0)
    assertEqualAndEqualSign(pow_op(-1.0, -1e+100), 1.0)
    assertEqualAndEqualSign(pow_op(-1.0, 1e+100), 1.0)
    assertEqualAndEqualSign(pow_op(-2.0, -2000.0), 0.0)

    assert type(pow_op(-2.0, -2000.5)) == complex
    assertEqualAndEqualSign(pow_op(-2.0, -2001.0), -0.0)
    assertEqualAndEqualSign(pow_op(2.0, -2000.0), 0.0)
    assertEqualAndEqualSign(pow_op(2.0, -2000.5), 0.0)
    assertEqualAndEqualSign(pow_op(2.0, -2001.0), 0.0)
    assertEqualAndEqualSign(pow_op(-0.5, 2000.0), 0.0)

    assert type(pow_op(-0.5, 2000.5)) == complex
    assertEqualAndEqualSign(pow_op(-0.5, 2001.0), -0.0)
    assertEqualAndEqualSign(pow_op(0.5, 2000.0), 0.0)
    assertEqualAndEqualSign(pow_op(0.5, 2000.5), 0.0)
    assertEqualAndEqualSign(pow_op(0.5, 2001.0), 0.0)
print("GeneralFloatCases::test_float_pow: ok")
