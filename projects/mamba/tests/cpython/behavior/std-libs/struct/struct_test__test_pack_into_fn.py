# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_pack_into_fn"
# subject = "cpython.test_struct.StructTest.test_pack_into_fn"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_pack_into_fn
"""Auto-ported test: StructTest::test_pack_into_fn (CPython 3.12 oracle)."""


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
test_string = b'Reykjavik rocks, eow!'
writable_buf = array.array('b', b' ' * 100)
fmt = '21s'
pack_into = lambda *args: struct.pack_into(fmt, *args)
pack_into(writable_buf, 0, test_string)
from_buf = writable_buf.tobytes()[:len(test_string)]

assert from_buf == test_string
pack_into(writable_buf, 10, test_string)
from_buf = writable_buf.tobytes()[:len(test_string) + 10]

assert from_buf == test_string[:10] + test_string
small_buf = array.array('b', b' ' * 10)

try:
    pack_into(small_buf, 0, test_string)
    raise AssertionError('expected (ValueError, struct.error)')
except (ValueError, struct.error):
    pass

try:
    pack_into(small_buf, 2, test_string)
    raise AssertionError('expected (ValueError, struct.error)')
except (ValueError, struct.error):
    pass
print("StructTest::test_pack_into_fn: ok")
