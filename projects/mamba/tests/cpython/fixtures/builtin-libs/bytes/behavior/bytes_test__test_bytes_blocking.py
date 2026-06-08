# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_bytes_blocking"
# subject = "cpython.test_bytes.BytesTest.test_bytes_blocking"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_bytes_blocking
"""Auto-ported test: BytesTest::test_bytes_blocking (CPython 3.12 oracle)."""


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
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected

class IterationBlocked(list):
    __bytes__ = None
i = [0, 1, 2, 3]

assert bytes(i) == b'\x00\x01\x02\x03'

try:
    bytes(IterationBlocked(i))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class IntBlocked(int):
    __bytes__ = None

assert bytes(3) == b'\x00\x00\x00'

try:
    bytes(IntBlocked(3))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class BytesSubclassBlocked(bytes):
    __bytes__ = None

assert bytes(b'ab') == b'ab'

try:
    bytes(BytesSubclassBlocked(b'ab'))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class BufferBlocked(bytearray):
    __bytes__ = None
ba, bb = (bytearray(b'ab'), BufferBlocked(b'ab'))

assert bytes(ba) == b'ab'

try:
    bytes(bb)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("BytesTest::test_bytes_blocking: ok")
