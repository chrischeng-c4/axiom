# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "format_test_case__test_issue35560"
# subject = "cpython.test_float.FormatTestCase.test_issue35560"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_float.py::FormatTestCase::test_issue35560
"""Auto-ported test: FormatTestCase::test_issue35560 (CPython 3.12 oracle)."""


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

assert format(123.0, '00') == '123.0'

assert format(123.34, '00f') == '123.340000'

assert format(123.34, '00e') == '1.233400e+02'

assert format(123.34, '00g') == '123.34'

assert format(123.34, '00.10f') == '123.3400000000'

assert format(123.34, '00.10e') == '1.2334000000e+02'

assert format(123.34, '00.10g') == '123.34'

assert format(123.34, '01f') == '123.340000'

assert format(-123.0, '00') == '-123.0'

assert format(-123.34, '00f') == '-123.340000'

assert format(-123.34, '00e') == '-1.233400e+02'

assert format(-123.34, '00g') == '-123.34'

assert format(-123.34, '00.10f') == '-123.3400000000'

assert format(-123.34, '00.10f') == '-123.3400000000'

assert format(-123.34, '00.10e') == '-1.2334000000e+02'

assert format(-123.34, '00.10g') == '-123.34'
print("FormatTestCase::test_issue35560: ok")
