# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_contains"
# subject = "cpython.test_bytes.BytesTest.test_contains"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_contains
"""Auto-ported test: BytesTest::test_contains (CPython 3.12 oracle)."""


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
b = type2test(b'abc')

assert ord('a') in b

assert int(ord('a')) in b

assert 200 not in b

try:
    (lambda: 300 in b)()
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    (lambda: -1 in b)()
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    (lambda: sys.maxsize + 1 in b)()
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    (lambda: None in b)()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    (lambda: float(ord('a')) in b)()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    (lambda: 'a' in b)()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
for f in (bytes, bytearray):

    assert f(b'') in b

    assert f(b'a') in b

    assert f(b'b') in b

    assert f(b'c') in b

    assert f(b'ab') in b

    assert f(b'bc') in b

    assert f(b'abc') in b

    assert f(b'ac') not in b

    assert f(b'd') not in b

    assert f(b'dab') not in b

    assert f(b'abd') not in b
print("BytesTest::test_contains: ok")
