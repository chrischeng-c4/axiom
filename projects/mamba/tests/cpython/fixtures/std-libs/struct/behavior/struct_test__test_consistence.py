# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_consistence"
# subject = "cpython.test_struct.StructTest.test_consistence"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_consistence
"""Auto-ported test: StructTest::test_consistence (CPython 3.12 oracle)."""


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

try:
    struct.calcsize('Z')
    raise AssertionError('expected struct.error')
except struct.error:
    pass
sz = struct.calcsize('i')

assert sz * 3 == struct.calcsize('iii')
fmt = 'cbxxxxxxhhhhiillffd?'
fmt3 = '3c3b18x12h6i6l6f3d3?'
sz = struct.calcsize(fmt)
sz3 = struct.calcsize(fmt3)

assert sz * 3 == sz3

try:
    struct.pack('iii', 3)
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.pack('i', 3, 3, 3)
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.pack('i', 'foo')
    raise AssertionError('expected (TypeError, struct.error)')
except (TypeError, struct.error):
    pass

try:
    struct.pack('P', 'foo')
    raise AssertionError('expected (TypeError, struct.error)')
except (TypeError, struct.error):
    pass

try:
    struct.unpack('d', b'flap')
    raise AssertionError('expected struct.error')
except struct.error:
    pass
s = struct.pack('ii', 1, 2)

try:
    struct.unpack('iii', s)
    raise AssertionError('expected struct.error')
except struct.error:
    pass

try:
    struct.unpack('i', s)
    raise AssertionError('expected struct.error')
except struct.error:
    pass
print("StructTest::test_consistence: ok")
