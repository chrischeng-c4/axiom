# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_custom"
# subject = "cpython.test_bytes.BytesTest.test_custom"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_custom
"""Auto-ported test: BytesTest::test_custom (CPython 3.12 oracle)."""


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

assert bytes(BytesSubclass(b'abc')) == b'abc'

assert BytesSubclass(OtherBytesSubclass(b'abc')) == BytesSubclass(b'abc')

assert bytes(WithBytes(b'abc')) == b'abc'

assert BytesSubclass(WithBytes(b'abc')) == BytesSubclass(b'abc')

class NoBytes:
    pass

try:
    bytes(NoBytes())
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    bytes(WithBytes('abc'))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    bytes(WithBytes(None))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class IndexWithBytes:

    def __bytes__(self):
        return b'a'

    def __index__(self):
        return 42

assert bytes(IndexWithBytes()) == b'a'

class StrWithBytes(str):

    def __new__(cls, value):
        self = str.__new__(cls, '€')
        self.value = value
        return self

    def __bytes__(self):
        return self.value

assert bytes(StrWithBytes(b'abc')) == b'abc'

assert bytes(StrWithBytes(b'abc'), 'iso8859-15') == b'\xa4'

assert bytes(StrWithBytes(BytesSubclass(b'abc'))) == b'abc'

assert BytesSubclass(StrWithBytes(b'abc')) == BytesSubclass(b'abc')

assert BytesSubclass(StrWithBytes(b'abc'), 'iso8859-15') == BytesSubclass(b'\xa4')

assert BytesSubclass(StrWithBytes(BytesSubclass(b'abc'))) == BytesSubclass(b'abc')

assert BytesSubclass(StrWithBytes(OtherBytesSubclass(b'abc'))) == BytesSubclass(b'abc')
assertTypedEqual(bytes(WithBytes(BytesSubclass(b'abc'))), BytesSubclass(b'abc'))
assertTypedEqual(BytesSubclass(WithBytes(BytesSubclass(b'abc'))), BytesSubclass(b'abc'))
assertTypedEqual(BytesSubclass(WithBytes(OtherBytesSubclass(b'abc'))), BytesSubclass(b'abc'))

class BytesWithBytes(bytes):

    def __new__(cls, value):
        self = bytes.__new__(cls, b'\xa4')
        self.value = value
        return self

    def __bytes__(self):
        return self.value
assertTypedEqual(bytes(BytesWithBytes(b'abc')), b'abc')
assertTypedEqual(BytesSubclass(BytesWithBytes(b'abc')), BytesSubclass(b'abc'))
assertTypedEqual(bytes(BytesWithBytes(BytesSubclass(b'abc'))), BytesSubclass(b'abc'))
assertTypedEqual(BytesSubclass(BytesWithBytes(BytesSubclass(b'abc'))), BytesSubclass(b'abc'))
assertTypedEqual(BytesSubclass(BytesWithBytes(OtherBytesSubclass(b'abc'))), BytesSubclass(b'abc'))
print("BytesTest::test_custom: ok")
