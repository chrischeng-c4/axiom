# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_p_code"
# subject = "cpython.test_struct.StructTest.test_p_code"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_p_code
"""Auto-ported test: StructTest::test_p_code (CPython 3.12 oracle)."""


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
for code, input, expected, expectedback in [('0p', b'abc', b'', b''), ('p', b'abc', b'\x00', b''), ('1p', b'abc', b'\x00', b''), ('2p', b'abc', b'\x01a', b'a'), ('3p', b'abc', b'\x02ab', b'ab'), ('4p', b'abc', b'\x03abc', b'abc'), ('5p', b'abc', b'\x03abc\x00', b'abc'), ('6p', b'abc', b'\x03abc\x00\x00', b'abc'), ('1000p', b'x' * 1000, b'\xff' + b'x' * 999, b'x' * 255)]:
    got = struct.pack(code, input)

    assert got == expected
    got, = struct.unpack(code, got)

    assert got == expectedback
print("StructTest::test_p_code: ok")
