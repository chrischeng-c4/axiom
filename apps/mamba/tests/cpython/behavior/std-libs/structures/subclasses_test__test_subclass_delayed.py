# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structures"
# dimension = "behavior"
# case = "subclasses_test__test_subclass_delayed"
# subject = "cpython.test_structures.SubclassesTest.test_subclass_delayed"
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

class X(Structure):
    pass
assert sizeof(X) == 0
X._fields_ = [('a', c_int)]

class Y(X):
    pass
assert sizeof(Y) == sizeof(X)
Y._fields_ = [('b', c_int)]

class Z(X):
    pass
assert sizeof(X) == sizeof(c_int)
assert sizeof(Y) == sizeof(c_int) * 2
assert sizeof(Z) == sizeof(c_int)
assert X._fields_ == [('a', c_int)]
assert Y._fields_ == [('b', c_int)]
assert Z._fields_ == [('a', c_int)]

print("SubclassesTest::test_subclass_delayed: ok")
