# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "unpack_iterator_test__test_arbitrary_buffer"
# subject = "cpython.test_struct.UnpackIteratorTest.test_arbitrary_buffer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::UnpackIteratorTest::test_arbitrary_buffer
"""Auto-ported test: UnpackIteratorTest::test_arbitrary_buffer (CPython 3.12 oracle)."""


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
s = struct.Struct('>IB')
b = bytes(range(1, 11))
it = s.iter_unpack(memoryview(b))

assert next(it) == (16909060, 5)

assert next(it) == (101124105, 10)

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass
print("UnpackIteratorTest::test_arbitrary_buffer: ok")
