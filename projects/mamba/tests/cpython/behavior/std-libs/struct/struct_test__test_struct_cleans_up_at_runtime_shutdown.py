# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_test__test_struct_cleans_up_at_runtime_shutdown"
# subject = "cpython.test_struct.StructTest.test_struct_cleans_up_at_runtime_shutdown"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_struct.py::StructTest::test_struct_cleans_up_at_runtime_shutdown
"""Auto-ported test: StructTest::test_struct_cleans_up_at_runtime_shutdown (CPython 3.12 oracle)."""


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
code = "if 1:\n            import struct\n\n            class C:\n                def __init__(self):\n                    self.pack = struct.pack\n                def __del__(self):\n                    self.pack('I', -42)\n\n            struct.x = C()\n            "
rc, stdout, stderr = assert_python_ok('-c', code)

assert rc == 0

assert stdout.rstrip() == b''

assert b'Exception ignored in:' in stderr

assert b'C.__del__' in stderr
print("StructTest::test_struct_cleans_up_at_runtime_shutdown: ok")
