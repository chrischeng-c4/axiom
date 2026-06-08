# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structures"
# dimension = "behavior"
# case = "pointer_member_test_case__test"
# subject = "cpython.test_structures.PointerMemberTestCase.test"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_structures.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import platform
from platform import architecture as _architecture
import struct
import sys
from ctypes import CDLL, Array, Structure, Union, POINTER, sizeof, byref, alignment, c_void_p, c_char, c_wchar, c_byte, c_ubyte, c_uint8, c_uint16, c_uint32, c_short, c_ushort, c_int, c_uint, c_long, c_ulong, c_longlong, c_ulonglong, c_float, c_double
from ctypes.util import find_library
from struct import calcsize
import _ctypes_test
from collections import namedtuple

class S(Structure):
    _fields_ = [('array', POINTER(c_int))]
s = S()
s.array = (c_int * 3)(1, 2, 3)
items = [s.array[i] for i in range(3)]
assert items == [1, 2, 3]
s.array[0] = 42
items = [s.array[i] for i in range(3)]
assert items == [42, 2, 3]
s.array[0] = 1
items = [s.array[i] for i in range(3)]
assert items == [1, 2, 3]

print("PointerMemberTestCase::test: ok")
