# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_transitiveness"
# subject = "cpython.test_struct.StructTest.test_transitiveness"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_transitiveness
"""Auto-ported test: StructTest::test_transitiveness (CPython 3.12 oracle)."""


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
c = b'a'
b = 1
h = 255
i = 65535
l = 65536
f = 3.1415
d = 3.1415
t = True
for prefix in ('', '@', '<', '>', '=', '!'):
    for format in ('xcbhilfd?', 'xcBHILfd?'):
        format = prefix + format
        s = struct.pack(format, c, b, h, i, l, f, d, t)
        cp, bp, hp, ip, lp, fp, dp, tp = struct.unpack(format, s)

        assert cp == c

        assert bp == b

        assert hp == h

        assert ip == i

        assert lp == l

        assert int(100 * fp) == int(100 * f)

        assert int(100 * dp) == int(100 * d)

        assert tp == t
print("StructTest::test_transitiveness: ok")
