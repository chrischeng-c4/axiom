# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "format_functions_test_case__test_getformat"
# subject = "cpython.test_float.FormatFunctionsTestCase.test_getformat"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_float.py::FormatFunctionsTestCase::test_getformat
"""Auto-ported test: FormatFunctionsTestCase::test_getformat (CPython 3.12 oracle)."""


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

assert float.__getformat__('double') in ['unknown', 'IEEE, big-endian', 'IEEE, little-endian']

assert float.__getformat__('float') in ['unknown', 'IEEE, big-endian', 'IEEE, little-endian']

try:
    float.__getformat__('chicken')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float.__getformat__(1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("FormatFunctionsTestCase::test_getformat: ok")
