# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_boundary_error_message"
# subject = "cpython.test_struct.StructTest.test_boundary_error_message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_boundary_error_message
"""Auto-ported test: StructTest::test_boundary_error_message (CPython 3.12 oracle)."""


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
regex1 = 'pack_into requires a buffer of at least 6 bytes for packing 1 bytes at offset 5 \\(actual buffer size is 1\\)'
try:
    struct.pack_into('b', bytearray(1), 5, 1)
    raise AssertionError('expected struct.error')
except struct.error as _aR_e:
    import re as _re_aR
    assert _re_aR.search(regex1, str(_aR_e))
regex2 = 'unpack_from requires a buffer of at least 6 bytes for unpacking 1 bytes at offset 5 \\(actual buffer size is 1\\)'
try:
    struct.unpack_from('b', bytearray(1), 5)
    raise AssertionError('expected struct.error')
except struct.error as _aR_e:
    import re as _re_aR
    assert _re_aR.search(regex2, str(_aR_e))
print("StructTest::test_boundary_error_message: ok")
