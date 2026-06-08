# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_new_features"
# subject = "cpython.test_struct.StructTest.test_new_features"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_new_features
"""Auto-ported test: StructTest::test_new_features (CPython 3.12 oracle)."""


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
tests = [('c', b'a', b'a', b'a', 0), ('xc', b'a', b'\x00a', b'\x00a', 0), ('cx', b'a', b'a\x00', b'a\x00', 0), ('s', b'a', b'a', b'a', 0), ('0s', b'helloworld', b'', b'', 1), ('1s', b'helloworld', b'h', b'h', 1), ('9s', b'helloworld', b'helloworl', b'helloworl', 1), ('10s', b'helloworld', b'helloworld', b'helloworld', 0), ('11s', b'helloworld', b'helloworld\x00', b'helloworld\x00', 1), ('20s', b'helloworld', b'helloworld' + 10 * b'\x00', b'helloworld' + 10 * b'\x00', 1), ('0p', b'helloworld', b'', b'', 1), ('1p', b'helloworld', b'\x00', b'\x00', 1), ('2p', b'helloworld', b'\x01h', b'\x01h', 1), ('10p', b'helloworld', b'\thelloworl', b'\thelloworl', 1), ('11p', b'helloworld', b'\nhelloworld', b'\nhelloworld', 0), ('12p', b'helloworld', b'\nhelloworld\x00', b'\nhelloworld\x00', 1), ('20p', b'helloworld', b'\nhelloworld' + 9 * b'\x00', b'\nhelloworld' + 9 * b'\x00', 1), ('b', 7, b'\x07', b'\x07', 0), ('b', -7, b'\xf9', b'\xf9', 0), ('B', 7, b'\x07', b'\x07', 0), ('B', 249, b'\xf9', b'\xf9', 0), ('h', 700, b'\x02\xbc', b'\xbc\x02', 0), ('h', -700, b'\xfdD', b'D\xfd', 0), ('H', 700, b'\x02\xbc', b'\xbc\x02', 0), ('H', 65536 - 700, b'\xfdD', b'D\xfd', 0), ('i', 70000000, b'\x04,\x1d\x80', b'\x80\x1d,\x04', 0), ('i', -70000000, b'\xfb\xd3\xe2\x80', b'\x80\xe2\xd3\xfb', 0), ('I', 70000000, b'\x04,\x1d\x80', b'\x80\x1d,\x04', 0), ('I', 4294967296 - 70000000, b'\xfb\xd3\xe2\x80', b'\x80\xe2\xd3\xfb', 0), ('l', 70000000, b'\x04,\x1d\x80', b'\x80\x1d,\x04', 0), ('l', -70000000, b'\xfb\xd3\xe2\x80', b'\x80\xe2\xd3\xfb', 0), ('L', 70000000, b'\x04,\x1d\x80', b'\x80\x1d,\x04', 0), ('L', 4294967296 - 70000000, b'\xfb\xd3\xe2\x80', b'\x80\xe2\xd3\xfb', 0), ('f', 2.0, b'@\x00\x00\x00', b'\x00\x00\x00@', 0), ('d', 2.0, b'@\x00\x00\x00\x00\x00\x00\x00', b'\x00\x00\x00\x00\x00\x00\x00@', 0), ('f', -2.0, b'\xc0\x00\x00\x00', b'\x00\x00\x00\xc0', 0), ('d', -2.0, b'\xc0\x00\x00\x00\x00\x00\x00\x00', b'\x00\x00\x00\x00\x00\x00\x00\xc0', 0), ('?', 0, b'\x00', b'\x00', 0), ('?', 3, b'\x01', b'\x01', 1), ('?', True, b'\x01', b'\x01', 0), ('?', [], b'\x00', b'\x00', 1), ('?', (1,), b'\x01', b'\x01', 1)]
for fmt, arg, big, lil, asy in tests:
    for xfmt, exp in [('>' + fmt, big), ('!' + fmt, big), ('<' + fmt, lil), ('=' + fmt, ISBIGENDIAN and big or lil)]:
        res = struct.pack(xfmt, arg)

        assert res == exp

        assert struct.calcsize(xfmt) == len(res)
        rev = struct.unpack(xfmt, res)[0]
        if rev != arg:

            assert asy
print("StructTest::test_new_features: ok")
