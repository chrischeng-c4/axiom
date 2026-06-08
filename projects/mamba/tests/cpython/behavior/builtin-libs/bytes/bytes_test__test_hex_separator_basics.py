# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_hex_separator_basics"
# subject = "cpython.test_bytes.BytesTest.test_hex_separator_basics"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_hex_separator_basics
"""Auto-ported test: BytesTest::test_hex_separator_basics (CPython 3.12 oracle)."""


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
three_bytes = type2test(b'\xb9\x01\xef')

assert three_bytes.hex() == 'b901ef'
try:
    three_bytes.hex('')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    three_bytes.hex('xx')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert three_bytes.hex(':', 0) == 'b901ef'
try:
    three_bytes.hex(None, 0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    three_bytes.hex('ÿ')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    three_bytes.hex(b'\xff')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    three_bytes.hex(b'\x80')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    three_bytes.hex(chr(256))
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert three_bytes.hex(':', 0) == 'b901ef'

assert three_bytes.hex(b'\x00') == 'b9\x0001\x00ef'

assert three_bytes.hex('\x00') == 'b9\x0001\x00ef'

assert three_bytes.hex(b'\x7f') == 'b9\x7f01\x7fef'

assert three_bytes.hex('\x7f') == 'b9\x7f01\x7fef'

assert three_bytes.hex(':', 3) == 'b901ef'

assert three_bytes.hex(':', 4) == 'b901ef'

assert three_bytes.hex(':', -4) == 'b901ef'

assert three_bytes.hex(':') == 'b9:01:ef'

assert three_bytes.hex(b'$') == 'b9$01$ef'

assert three_bytes.hex(':', 1) == 'b9:01:ef'

assert three_bytes.hex(':', -1) == 'b9:01:ef'

assert three_bytes.hex(':', 2) == 'b9:01ef'

assert three_bytes.hex(':', 1) == 'b9:01:ef'

assert three_bytes.hex('*', -2) == 'b901*ef'
value = b'{s\x05\x00\x00\x00worldi\x02\x00\x00\x00s\x05\x00\x00\x00helloi\x01\x00\x00\x000'

assert value.hex('.', 8) == '7b7305000000776f.726c646902000000.730500000068656c.6c6f690100000030'
print("BytesTest::test_hex_separator_basics: ok")
