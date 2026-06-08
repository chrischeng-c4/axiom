# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_boundary_error_message_with_negative_offset"
# subject = "cpython.test_struct.StructTest.test_boundary_error_message_with_negative_offset"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_boundary_error_message_with_negative_offset
"""Auto-ported test: StructTest::test_boundary_error_message_with_negative_offset (CPython 3.12 oracle)."""


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
byte_list = bytearray(10)
try:
    struct.pack_into('<I', byte_list, -2, 123)
    raise AssertionError('expected struct.error')
except struct.error as _aR_e:
    import re as _re_aR
    assert _re_aR.search('no space to pack 4 bytes at offset -2', str(_aR_e))
try:
    struct.pack_into('<B', byte_list, -11, 123)
    raise AssertionError('expected struct.error')
except struct.error as _aR_e:
    import re as _re_aR
    assert _re_aR.search('offset -11 out of range for 10-byte buffer', str(_aR_e))
try:
    struct.unpack_from('<I', byte_list, -2)
    raise AssertionError('expected struct.error')
except struct.error as _aR_e:
    import re as _re_aR
    assert _re_aR.search('not enough data to unpack 4 bytes at offset -2', str(_aR_e))
try:
    struct.unpack_from('<B', byte_list, -11)
    raise AssertionError('expected struct.error')
except struct.error as _aR_e:
    import re as _re_aR
    assert _re_aR.search('offset -11 out of range for 10-byte buffer', str(_aR_e))
print("StructTest::test_boundary_error_message_with_negative_offset: ok")
