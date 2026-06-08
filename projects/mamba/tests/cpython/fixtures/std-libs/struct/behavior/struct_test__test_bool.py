# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_bool"
# subject = "cpython.test_struct.StructTest.test_bool"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_bool
"""Auto-ported test: StructTest::test_bool (CPython 3.12 oracle)."""


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
class ExplodingBool(object):

    def __bool__(self):
        raise OSError
for prefix in tuple('<>!=') + ('',):
    false = ((), [], [], '', 0)
    true = ([1], 'test', 5, -1, 4294967295 + 1, 4294967295 / 2)
    falseFormat = prefix + '?' * len(false)
    packedFalse = struct.pack(falseFormat, *false)
    unpackedFalse = struct.unpack(falseFormat, packedFalse)
    trueFormat = prefix + '?' * len(true)
    packedTrue = struct.pack(trueFormat, *true)
    unpackedTrue = struct.unpack(trueFormat, packedTrue)

    assert len(true) == len(unpackedTrue)

    assert len(false) == len(unpackedFalse)
    for t in unpackedFalse:

        assert not t
    for t in unpackedTrue:

        assert t
    packed = struct.pack(prefix + '?', 1)

    assert len(packed) == struct.calcsize(prefix + '?')
    if len(packed) != 1:

        assert not prefix
    try:
        struct.pack(prefix + '?', ExplodingBool())
    except OSError:
        pass
    else:

        raise AssertionError('Expected OSError: struct.pack(%r, ExplodingBool())' % (prefix + '?'))
for c in [b'\x01', b'\x7f', b'\xff', b'\x0f', b'\xf0']:

    assert struct.unpack('>?', c)[0]

    assert struct.unpack('<?', c)[0]

    assert struct.unpack('=?', c)[0]

    assert struct.unpack('@?', c)[0]
print("StructTest::test_bool: ok")
