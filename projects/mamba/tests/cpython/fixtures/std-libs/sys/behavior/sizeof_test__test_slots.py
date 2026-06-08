# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sizeof_test__test_slots"
# subject = "cpython.test_sys.SizeofTest.test_slots"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::SizeofTest::test_slots
"""Auto-ported test: SizeofTest::test_slots (CPython 3.12 oracle)."""


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

def check_slots(obj, base, extra):
    expected = sys.getsizeof(base) + struct.calcsize(extra)
    if gc.is_tracked(obj) and (not gc.is_tracked(base)):
        expected += self_gc_headsize

    assert sys.getsizeof(obj) == expected
self_P = struct.calcsize('P')
self_longdigit = sys.int_info.sizeof_digit
import _testinternalcapi
self_gc_headsize = _testinternalcapi.SIZEOF_PYGC_HEAD
check = check_slots

class BA(bytearray):
    __slots__ = ('a', 'b', 'c')
check(BA(), bytearray(), '3P')

class D(dict):
    __slots__ = ('a', 'b', 'c')
check(D(x=[]), {'x': []}, '3P')

class L(list):
    __slots__ = ('a', 'b', 'c')
check(L(), [], '3P')

class S(set):
    __slots__ = ('a', 'b', 'c')
check(S(), set(), '3P')

class FS(frozenset):
    __slots__ = ('a', 'b', 'c')
check(FS(), frozenset(), '3P')
from collections import OrderedDict

class OD(OrderedDict):
    __slots__ = ('a', 'b', 'c')
check(OD(x=[]), OrderedDict(x=[]), '3P')
print("SizeofTest::test_slots: ok")
