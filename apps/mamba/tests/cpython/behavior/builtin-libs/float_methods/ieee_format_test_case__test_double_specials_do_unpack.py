# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "ieee_format_test_case__test_double_specials_do_unpack"
# subject = "cpython.test_float.IEEEFormatTestCase.test_double_specials_do_unpack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_float.py::IEEEFormatTestCase::test_double_specials_do_unpack
"""Auto-ported test: IEEEFormatTestCase::test_double_specials_do_unpack (CPython 3.12 oracle)."""


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
for fmt, data in [('>d', BE_DOUBLE_INF), ('>d', BE_DOUBLE_NAN), ('<d', LE_DOUBLE_INF), ('<d', LE_DOUBLE_NAN)]:
    struct.unpack(fmt, data)
print("IEEEFormatTestCase::test_double_specials_do_unpack: ok")
