# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_fromhex"
# subject = "cpython.test_bytes.BytesTest.test_fromhex"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_fromhex
"""Auto-ported test: BytesTest::test_fromhex (CPython 3.12 oracle)."""


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
    type2test.fromhex()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    type2test.fromhex(1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert type2test.fromhex('') == type2test()
b = bytearray([26, 43, 48])

assert type2test.fromhex('1a2B30') == b

assert type2test.fromhex('  1A 2B  30   ') == b

assert type2test.fromhex(' 1A\n2B\t30\x0b') == b
for c in '\t\n\x0b\x0c\r ':

    assert type2test.fromhex(c) == type2test()
for c in '\x1c\x1d\x1e\x1f\x85\xa0\u2000\u2002\u2028':

    try:
        type2test.fromhex(c)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

assert type2test.fromhex('0000') == b'\x00\x00'

try:
    type2test.fromhex(b'1B')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    type2test.fromhex('a')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test.fromhex('rt')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test.fromhex('1a b cd')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test.fromhex('\x00')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test.fromhex('12   \x00   34')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
for data, pos in (('12 x4 56', 3), ('12 3x 56', 4), ('12 xy 56', 3), ('12 3ÿ 56', 4)):
    try:
        type2test.fromhex(data)
        raise AssertionError('expected ValueError')
    except ValueError as _aR_e:
        import types as _types_aR
        cm = _types_aR.SimpleNamespace(exception=_aR_e)

    assert 'at position %s' % pos in str(cm.exception)
print("BytesTest::test_fromhex: ok")
