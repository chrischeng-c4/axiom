# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_struct_types_immutable"
# subject = "cpython.test_struct.StructTest.test__struct_types_immutable"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test__struct_types_immutable
"""Auto-ported test: StructTest::test__struct_types_immutable (CPython 3.12 oracle)."""


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
def check_sizeof(format_str, number_of_codes):
    totalsize = support.calcobjsize('2n3P')
    totalsize += struct.calcsize('P3n0P') * (number_of_codes + 1)
    support.check_sizeof(self, struct.Struct(format_str), totalsize)
Struct = struct.Struct
unpack_iterator = type(struct.iter_unpack('b', b'x'))
for cls in (Struct, unpack_iterator):
    try:
        cls.x = 1
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("StructTest::test__struct_types_immutable: ok")
