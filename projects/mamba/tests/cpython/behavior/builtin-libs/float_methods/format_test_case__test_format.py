# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "format_test_case__test_format"
# subject = "cpython.test_float.FormatTestCase.test_format"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_float.py::FormatTestCase::test_format
"""Auto-ported test: FormatTestCase::test_format (CPython 3.12 oracle)."""


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

assert format(0.0, 'f') == '0.000000'

assert format(0.0, '') == '0.0'

assert format(0.01, '') == '0.01'

assert format(0.01, 'g') == '0.01'
x = 100 / 7.0

assert format(x, '') == str(x)

assert format(x, '-') == str(x)

assert format(x, '>') == str(x)

assert format(x, '2') == str(x)

assert format(1.0, 'f') == '1.000000'

assert format(-1.0, 'f') == '-1.000000'

assert format(1.0, ' f') == ' 1.000000'

assert format(-1.0, ' f') == '-1.000000'

assert format(1.0, '+f') == '+1.000000'

assert format(-1.0, '+f') == '-1.000000'

assert format(-1.0, '%') == '-100.000000%'

try:
    format(3.0, 's')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
for format_spec in 'sbcdoxX':

    try:
        format(0.0, format_spec)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        format(1.0, format_spec)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        format(-1.0, format_spec)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        format(1e+100, format_spec)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        format(-1e+100, format_spec)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        format(1e-100, format_spec)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        format(-1e-100, format_spec)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

assert format(NAN, 'f') == 'nan'

assert format(NAN, 'F') == 'NAN'

assert format(INF, 'f') == 'inf'

assert format(INF, 'F') == 'INF'
print("FormatTestCase::test_format: ok")
