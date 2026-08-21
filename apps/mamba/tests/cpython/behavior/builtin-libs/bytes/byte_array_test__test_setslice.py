# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_setslice"
# subject = "cpython.test_bytes.ByteArrayTest.test_setslice"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_setslice
"""Auto-ported test: ByteArrayTest::test_setslice (CPython 3.12 oracle)."""


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
b = bytearray(range(10))

assert list(b) == list(range(10))
b[0:5] = bytearray([1, 1, 1, 1, 1])

assert b == bytearray([1, 1, 1, 1, 1, 5, 6, 7, 8, 9])
del b[0:-5]

assert b == bytearray([5, 6, 7, 8, 9])
b[0:0] = bytearray([0, 1, 2, 3, 4])

assert b == bytearray(range(10))
b[-7:-3] = bytearray([100, 101])

assert b == bytearray([0, 1, 2, 100, 101, 7, 8, 9])
b[3:5] = [3, 4, 5, 6]

assert b == bytearray(range(10))
b[3:0] = [42, 42, 42]

assert b == bytearray([0, 1, 2, 42, 42, 42, 3, 4, 5, 6, 7, 8, 9])
b[3:] = b'foo'

assert b == bytearray([0, 1, 2, 102, 111, 111])
b[:3] = memoryview(b'foo')

assert b == bytearray([102, 111, 111, 102, 111, 111])
b[3:4] = []

assert b == bytearray([102, 111, 111, 111, 111])
for elem in [5, -5, 0, int(1e+21), 'str', 2.3, ['a', 'b'], [b'a', b'b'], [[]]]:
    try:
        b[3:4] = elem
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
for elem in [[254, 255, 256], [-256, 9000]]:
    try:
        b[3:4] = elem
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
print("ByteArrayTest::test_setslice: ok")
