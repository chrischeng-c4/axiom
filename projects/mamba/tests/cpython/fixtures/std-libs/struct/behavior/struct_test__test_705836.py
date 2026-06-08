# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_705836"
# subject = "cpython.test_struct.StructTest.test_705836"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_705836
"""Auto-ported test: StructTest::test_705836 (CPython 3.12 oracle)."""


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
for base in range(1, 33):
    delta = 0.5
    while base - delta / 2.0 != base:
        delta /= 2.0
    smaller = base - delta
    packed = struct.pack('<f', smaller)
    unpacked = struct.unpack('<f', packed)[0]

    assert base == unpacked
    bigpacked = struct.pack('>f', smaller)

    assert bigpacked == string_reverse(packed)
    unpacked = struct.unpack('>f', bigpacked)[0]

    assert base == unpacked
big = (1 << 24) - 1
big = math.ldexp(big, 127 - 23)
packed = struct.pack('>f', big)
unpacked = struct.unpack('>f', packed)[0]

assert big == unpacked
big = (1 << 25) - 1
big = math.ldexp(big, 127 - 24)

try:
    struct.pack('>f', big)
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass
print("StructTest::test_705836: ok")
