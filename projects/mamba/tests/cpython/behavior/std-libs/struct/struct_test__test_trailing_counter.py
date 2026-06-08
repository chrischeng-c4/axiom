# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_trailing_counter"
# subject = "cpython.test_struct.StructTest.test_trailing_counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_trailing_counter
"""Auto-ported test: StructTest::test_trailing_counter (CPython 3.12 oracle)."""


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
store = array.array('b', b' ' * 100)

try:
    struct.pack('12345')
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.unpack('12345', b'')
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.pack_into('12345', store, 0)
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.unpack_from('12345', store, 0)
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.pack('c12345', 'x')
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.unpack('c12345', b'x')
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.pack_into('c12345', store, 0, 'x')
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.unpack_from('c12345', store, 0)
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.pack('14s42', 'spam and eggs')
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.unpack('14s42', b'spam and eggs')
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.pack_into('14s42', store, 0, 'spam and eggs')
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.unpack_from('14s42', store, 0)
    raise AssertionError('expected struct.error')
except struct.error:
    pass
print("StructTest::test_trailing_counter: ok")
