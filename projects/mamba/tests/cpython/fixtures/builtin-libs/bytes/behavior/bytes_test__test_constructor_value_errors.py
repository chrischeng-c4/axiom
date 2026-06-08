# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_constructor_value_errors"
# subject = "cpython.test_bytes.BytesTest.test_constructor_value_errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_constructor_value_errors
"""Auto-ported test: BytesTest::test_constructor_value_errors (CPython 3.12 oracle)."""


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

try:
    type2test([-1])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([-sys.maxsize])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([-sys.maxsize - 1])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([-sys.maxsize - 2])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([-10 ** 100])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([256])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([257])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([sys.maxsize])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([sys.maxsize + 1])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([10 ** 100])
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("BytesTest::test_constructor_value_errors: ok")
