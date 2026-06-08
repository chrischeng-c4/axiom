# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_unpack_from"
# subject = "cpython.test_struct.StructTest.test_unpack_from"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_unpack_from
"""Auto-ported test: StructTest::test_unpack_from (CPython 3.12 oracle)."""


from collections import abc
import array
import gc
import math
import operator
import unittest
import struct
import sys
import weakref
from test import support
from test.support import import_helper
from test.support.script_helper import assert_python_ok


ISBIGENDIAN = sys.byteorder == 'big'

integer_codes = ('b', 'B', 'h', 'H', 'i', 'I', 'l', 'L', 'q', 'Q', 'n', 'N')

byteorders = ('', '@', '=', '<', '>', '!')

def iter_integer_formats(byteorders=byteorders):
    for code in integer_codes:
        for byteorder in byteorders:
            if byteorder not in ('', '@') and code in ('n', 'N'):
                continue
            yield (code, byteorder)

def string_reverse(s):
    return s[::-1]

def bigendian_to_native(value):
    if ISBIGENDIAN:
        return value
    else:
        return string_reverse(value)


# --- test body ---
test_string = b'abcd01234'
fmt = '4s'
s = struct.Struct(fmt)
for cls in (bytes, bytearray):
    data = cls(test_string)

    assert s.unpack_from(data) == (b'abcd',)

    assert s.unpack_from(data, 2) == (b'cd01',)

    assert s.unpack_from(data, 4) == (b'0123',)
    for i in range(6):

        assert s.unpack_from(data, i) == (data[i:i + 4],)
    for i in range(6, len(test_string) + 1):

        try:
            s.unpack_from(data, i)
            raise AssertionError('expected struct.error')
        except struct.error:
            pass
for cls in (bytes, bytearray):
    data = cls(test_string)

    assert struct.unpack_from(fmt, data) == (b'abcd',)

    assert struct.unpack_from(fmt, data, 2) == (b'cd01',)

    assert struct.unpack_from(fmt, data, 4) == (b'0123',)
    for i in range(6):

        assert struct.unpack_from(fmt, data, i) == (data[i:i + 4],)
    for i in range(6, len(test_string) + 1):

        try:
            struct.unpack_from(fmt, data, i)
            raise AssertionError('expected struct.error')
        except struct.error:
            pass

assert s.unpack_from(buffer=test_string, offset=2) == (b'cd01',)
print("StructTest::test_unpack_from: ok")
