# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_rfind"
# subject = "cpython.test_bytes.ByteArrayTest.test_rfind"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_rfind
"""Auto-ported test: ByteArrayTest::test_rfind (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'mississippi')
i = 105
w = 119

assert b.rfind(b'ss') == 5

assert b.rfind(b'w') == -1

assert b.rfind(b'mississippian') == -1

assert b.rfind(i) == 10

assert b.rfind(w) == -1

assert b.rfind(b'ss', 3) == 5

assert b.rfind(b'ss', 0, 6) == 2

assert b.rfind(i, 1, 3) == 1

assert b.rfind(i, 3, 9) == 7

assert b.rfind(w, 1, 3) == -1
print("ByteArrayTest::test_rfind: ok")
