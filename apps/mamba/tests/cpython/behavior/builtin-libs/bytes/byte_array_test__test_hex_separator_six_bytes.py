# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_hex_separator_six_bytes"
# subject = "cpython.test_bytes.ByteArrayTest.test_hex_separator_six_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_hex_separator_six_bytes
"""Auto-ported test: ByteArrayTest::test_hex_separator_six_bytes (CPython 3.12 oracle)."""


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
six_bytes = type2test((x * 3 for x in range(1, 7)))

assert six_bytes.hex() == '0306090c0f12'

assert six_bytes.hex('.', 1) == '03.06.09.0c.0f.12'

assert six_bytes.hex(' ', 2) == '0306 090c 0f12'

assert six_bytes.hex('-', 3) == '030609-0c0f12'

assert six_bytes.hex(':', 4) == '0306:090c0f12'

assert six_bytes.hex(':', 5) == '03:06090c0f12'

assert six_bytes.hex(':', 6) == '0306090c0f12'

assert six_bytes.hex(':', 95) == '0306090c0f12'

assert six_bytes.hex('_', -3) == '030609_0c0f12'

assert six_bytes.hex(':', -4) == '0306090c:0f12'

assert six_bytes.hex(b'@', -5) == '0306090c0f@12'

assert six_bytes.hex(':', -6) == '0306090c0f12'

assert six_bytes.hex(' ', -95) == '0306090c0f12'
print("ByteArrayTest::test_hex_separator_six_bytes: ok")
