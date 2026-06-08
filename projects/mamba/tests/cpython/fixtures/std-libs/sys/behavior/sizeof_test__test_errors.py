# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sizeof_test__test_errors"
# subject = "cpython.test_sys.SizeofTest.test_errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::SizeofTest::test_errors
"""Auto-ported test: SizeofTest::test_errors (CPython 3.12 oracle)."""


import builtins
import codecs
import gc
import io
import locale
import operator
import os
import random
import struct
import subprocess
import sys
import sysconfig
import test.support
from test import support
from test.support import os_helper
from test.support.script_helper import assert_python_ok, assert_python_failure
from test.support import threading_helper
from test.support import import_helper
import textwrap
import unittest
import warnings


try:
    from test.support import interpreters
except ImportError:
    interpreters = None

def requires_subinterpreters(func):
    deco = unittest.skipIf(interpreters is None, 'Test requires subinterpreters')
    return deco(func)

DICT_KEY_STRUCT_FORMAT = 'n2BI2n'


# --- test body ---
check_sizeof = test.support.check_sizeof
self_P = struct.calcsize('P')
self_longdigit = sys.int_info.sizeof_digit
import _testinternalcapi
self_gc_headsize = _testinternalcapi.SIZEOF_PYGC_HEAD

class BadSizeof:

    def __sizeof__(self):
        raise ValueError

try:
    sys.getsizeof(BadSizeof())
    raise AssertionError('expected ValueError')
except ValueError:
    pass

class InvalidSizeof:

    def __sizeof__(self):
        return None

try:
    sys.getsizeof(InvalidSizeof())
    raise AssertionError('expected TypeError')
except TypeError:
    pass
sentinel = ['sentinel']

assert sys.getsizeof(InvalidSizeof(), sentinel) is sentinel

class FloatSizeof:

    def __sizeof__(self):
        return 4.5

try:
    sys.getsizeof(FloatSizeof())
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert sys.getsizeof(FloatSizeof(), sentinel) is sentinel

class OverflowSizeof(int):

    def __sizeof__(self):
        return int(self)

assert sys.getsizeof(OverflowSizeof(sys.maxsize)) == sys.maxsize + self_gc_headsize * 2
try:
    sys.getsizeof(OverflowSizeof(sys.maxsize + 1))
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass
try:
    sys.getsizeof(OverflowSizeof(-1))
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    sys.getsizeof(OverflowSizeof(-sys.maxsize - 1))
    raise AssertionError('expected (ValueError, OverflowError)')
except (ValueError, OverflowError):
    pass
print("SizeofTest::test_errors: ok")
