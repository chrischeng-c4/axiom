# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_calcsize"
# subject = "cpython.test_struct.StructTest.test_calcsize"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_calcsize
"""Auto-ported test: StructTest::test_calcsize (CPython 3.12 oracle)."""


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
expected_size = {'b': 1, 'B': 1, 'h': 2, 'H': 2, 'i': 4, 'I': 4, 'l': 4, 'L': 4, 'q': 8, 'Q': 8}
for code, byteorder in iter_integer_formats(('=', '<', '>', '!')):
    format = byteorder + code
    size = struct.calcsize(format)

    assert size == expected_size[code]
native_pairs = ('bB', 'hH', 'iI', 'lL', 'nN', 'qQ')
for format_pair in native_pairs:
    for byteorder in ('', '@'):
        signed_size = struct.calcsize(byteorder + format_pair[0])
        unsigned_size = struct.calcsize(byteorder + format_pair[1])

        assert signed_size == unsigned_size

assert struct.calcsize('b') == 1

assert 2 <= struct.calcsize('h')

assert 4 <= struct.calcsize('l')

assert struct.calcsize('h') <= struct.calcsize('i')

assert struct.calcsize('i') <= struct.calcsize('l')

assert 8 <= struct.calcsize('q')

assert struct.calcsize('l') <= struct.calcsize('q')

assert struct.calcsize('n') >= struct.calcsize('i')

assert struct.calcsize('n') >= struct.calcsize('P')
print("StructTest::test_calcsize: ok")
