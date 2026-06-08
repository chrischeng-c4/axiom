# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "unpack_iterator_test__test_half_float"
# subject = "cpython.test_struct.UnpackIteratorTest.test_half_float"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_struct.py::UnpackIteratorTest::test_half_float
"""Auto-ported test: UnpackIteratorTest::test_half_float (CPython 3.12 oracle)."""


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
format_bits_float__cleanRoundtrip_list = [(b'\x00<', 1.0), (b'\x00\xc0', -2.0), (b'\xff{', 65504.0), (b'\x00\x04', 2 ** (-14)), (b'\x01\x00', 2 ** (-24)), (b'\x00\x00', 0.0), (b'\x00\x80', -0.0), (b'\x00|', float('+inf')), (b'\x00\xfc', float('-inf')), (b'U5', 0.333251953125)]
for le_bits, f in format_bits_float__cleanRoundtrip_list:
    be_bits = le_bits[::-1]

    assert f == struct.unpack('<e', le_bits)[0]

    assert le_bits == struct.pack('<e', f)

    assert f == struct.unpack('>e', be_bits)[0]

    assert be_bits == struct.pack('>e', f)
    if sys.byteorder == 'little':

        assert f == struct.unpack('e', le_bits)[0]

        assert le_bits == struct.pack('e', f)
    else:

        assert f == struct.unpack('e', be_bits)[0]

        assert be_bits == struct.pack('e', f)
format_bits__nan_list = [('<e', b'\x01\xfc'), ('<e', b'\x00\xfe'), ('<e', b'\xff\xff'), ('<e', b'\x01|'), ('<e', b'\x00~'), ('<e', b'\xff\x7f')]
for formatcode, bits in format_bits__nan_list:

    assert math.isnan(struct.unpack('<e', bits)[0])

    assert math.isnan(struct.unpack('>e', bits[::-1])[0])
packed = struct.pack('<e', math.nan)

assert packed[1] & 126 == 126
packed = struct.pack('<e', -math.nan)

assert packed[1] & 126 == 126
format_bits_float__rounding_list = [('>e', b'\x00\x01', 2.0 ** (-25) + 2.0 ** (-35)), ('>e', b'\x00\x00', 2.0 ** (-25)), ('>e', b'\x00\x00', 2.0 ** (-26)), ('>e', b'\x03\xff', 2.0 ** (-14) - 2.0 ** (-24)), ('>e', b'\x03\xff', 2.0 ** (-14) - 2.0 ** (-25) - 2.0 ** (-65)), ('>e', b'\x04\x00', 2.0 ** (-14) - 2.0 ** (-25)), ('>e', b'\x04\x00', 2.0 ** (-14)), ('>e', b'<\x01', 1.0 + 2.0 ** (-11) + 2.0 ** (-16)), ('>e', b'<\x00', 1.0 + 2.0 ** (-11)), ('>e', b'<\x00', 1.0 + 2.0 ** (-12)), ('>e', b'{\xff', 65504), ('>e', b'{\xff', 65519), ('>e', b'\x80\x01', -2.0 ** (-25) - 2.0 ** (-35)), ('>e', b'\x80\x00', -2.0 ** (-25)), ('>e', b'\x80\x00', -2.0 ** (-26)), ('>e', b'\xbc\x01', -1.0 - 2.0 ** (-11) - 2.0 ** (-16)), ('>e', b'\xbc\x00', -1.0 - 2.0 ** (-11)), ('>e', b'\xbc\x00', -1.0 - 2.0 ** (-12)), ('>e', b'\xfb\xff', -65519)]
for formatcode, bits, f in format_bits_float__rounding_list:

    assert bits == struct.pack(formatcode, f)
format_bits_float__roundingError_list = [('>e', 65520.0), ('>e', 65536.0), ('>e', 1e+300), ('>e', -65520.0), ('>e', -65536.0), ('>e', -1e+300), ('<e', 65520.0), ('<e', 65536.0), ('<e', 1e+300), ('<e', -65520.0), ('<e', -65536.0), ('<e', -1e+300)]
for formatcode, f in format_bits_float__roundingError_list:

    try:
        struct.pack(formatcode, f)
        raise AssertionError('expected OverflowError')
    except OverflowError:
        pass
format_bits_float__doubleRoundingError_list = [('>e', b'g\xff', 137405399039 * 2 ** (-26))]
for formatcode, bits, f in format_bits_float__doubleRoundingError_list:

    assert bits == struct.pack(formatcode, f)
print("UnpackIteratorTest::test_half_float: ok")
