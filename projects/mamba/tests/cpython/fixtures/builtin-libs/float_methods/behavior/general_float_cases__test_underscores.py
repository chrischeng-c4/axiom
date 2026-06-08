# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "general_float_cases__test_underscores"
# subject = "cpython.test_float.GeneralFloatCases.test_underscores"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_float.py::GeneralFloatCases::test_underscores
"""Auto-ported test: GeneralFloatCases::test_underscores (CPython 3.12 oracle)."""


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
for lit in VALID_UNDERSCORE_LITERALS:
    if not any((ch in lit for ch in 'jJxXoObB')):

        assert float(lit) == eval(lit)

        assert float(lit) == float(lit.replace('_', ''))
for lit in INVALID_UNDERSCORE_LITERALS:
    if lit in ('0_7', '09_99'):
        continue
    if not any((ch in lit for ch in 'jJxXoObB')):

        try:
            float(lit)
            raise AssertionError('expected ValueError')
        except ValueError:
            pass

try:
    float('_NaN')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('Na_N')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('IN_F')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('-_INF')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('-INF_')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float(b'0_.\xff9')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("GeneralFloatCases::test_underscores: ok")
