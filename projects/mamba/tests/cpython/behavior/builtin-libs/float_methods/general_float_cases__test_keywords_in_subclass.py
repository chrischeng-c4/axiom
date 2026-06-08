# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "general_float_cases__test_keywords_in_subclass"
# subject = "cpython.test_float.GeneralFloatCases.test_keywords_in_subclass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_float.py::GeneralFloatCases::test_keywords_in_subclass
"""Auto-ported test: GeneralFloatCases::test_keywords_in_subclass (CPython 3.12 oracle)."""


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
class subclass(float):
    pass
u = subclass(2.5)

assert type(u) is subclass

assert float(u) == 2.5
try:
    subclass(x=0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class subclass_with_init(float):

    def __init__(self, arg, newarg=None):
        self.newarg = newarg
u = subclass_with_init(2.5, newarg=3)

assert type(u) is subclass_with_init

assert float(u) == 2.5

assert u.newarg == 3

class subclass_with_new(float):

    def __new__(cls, arg, newarg=None):
        self = super().__new__(cls, arg)
        self.newarg = newarg
        return self
u = subclass_with_new(2.5, newarg=3)

assert type(u) is subclass_with_new

assert float(u) == 2.5

assert u.newarg == 3
print("GeneralFloatCases::test_keywords_in_subclass: ok")
