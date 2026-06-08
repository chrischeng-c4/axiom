# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "hex_float_test_case__test_invalid_inputs"
# subject = "cpython.test_float.HexFloatTestCase.test_invalid_inputs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_float.py::HexFloatTestCase::test_invalid_inputs
"""Auto-ported test: HexFloatTestCase::test_invalid_inputs (CPython 3.12 oracle)."""


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
MAX = fromHex('0x.fffffffffffff8p+1024')
MIN = fromHex('0x1p-1022')
TINY = fromHex('0x0.0000000000001p-1022')
EPS = fromHex('0x0.0000000000001p0')
invalid_inputs = ['infi', '-Infinit', '++inf', '-+Inf', '--nan', '+-NaN', 'snan', 'NaNs', 'nna', 'an', 'nf', 'nfinity', 'inity', 'iinity', '0xnan', '', ' ', 'x1.0p0', '0xX1.0p0', '+ 0x1.0p0', '- 0x1.0p0', '0 x1.0p0', '0x 1.0p0', '0x1 2.0p0', '+0x1 .0p0', '0x1. 0p0', '-0x1.0 1p0', '-0x1.0 p0', '+0x1.0p +0', '0x1.0p -0', '0x1.0p 0', '+0x1.0p+ 0', '-0x1.0p- 0', '++0x1.0p-0', '--0x1.0p0', '+-0x1.0p+0', '-+0x1.0p0', '0x1.0p++0', '+0x1.0p+-0', '-0x1.0p-+0', '0x1.0p--0', '0x1.0.p0', '0x.p0', '0x1,p0', '0x1pa', '0x1p０', '０x1p0', '0x１p0', '0x1.０p0', '0x1p0 \n 0x2p0', '0x1p0\x00 0x1p0']
for x in invalid_inputs:
    try:
        result = fromHex(x)
    except ValueError:
        pass
    else:

        raise AssertionError('Expected float.fromhex(%r) to raise ValueError; got %r instead' % (x, result))
print("HexFloatTestCase::test_invalid_inputs: ok")
