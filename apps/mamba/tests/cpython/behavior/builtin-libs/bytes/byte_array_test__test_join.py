# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_join"
# subject = "cpython.test_bytes.ByteArrayTest.test_join"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_join
"""Auto-ported test: ByteArrayTest::test_join (CPython 3.12 oracle)."""


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

assert type2test(b'').join([]) == b''

assert type2test(b'').join([b'']) == b''
for lst in [[b'abc'], [b'a', b'bc'], [b'ab', b'c'], [b'a', b'b', b'c']]:
    lst = list(map(type2test, lst))

    assert type2test(b'').join(lst) == b'abc'

    assert type2test(b'').join(tuple(lst)) == b'abc'

    assert type2test(b'').join(iter(lst)) == b'abc'
dot_join = type2test(b'.:').join

assert dot_join([b'ab', b'cd']) == b'ab.:cd'

assert dot_join([memoryview(b'ab'), b'cd']) == b'ab.:cd'

assert dot_join([b'ab', memoryview(b'cd')]) == b'ab.:cd'

assert dot_join([bytearray(b'ab'), b'cd']) == b'ab.:cd'

assert dot_join([b'ab', bytearray(b'cd')]) == b'ab.:cd'
seq = [b'abc'] * 100000
expected = b'abc' + b'.:abc' * 99999

assert dot_join(seq) == expected
seq = [b'abc'] * 100000
expected = b'abc' * 100000

assert type2test(b'').join(seq) == expected

try:
    type2test(b' ').join(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    dot_join([bytearray(b'ab'), 'cd', b'ef'])
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    dot_join([memoryview(b'ab'), 'cd', b'ef'])
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ByteArrayTest::test_join: ok")
