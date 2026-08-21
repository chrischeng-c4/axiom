# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structures"
# dimension = "behavior"
# case = "structure_test_case__test_huge_field_name"
# subject = "cpython.test_structures.StructureTestCase.test_huge_field_name"
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
formats = {'c': c_char, 'b': c_byte, 'B': c_ubyte, 'h': c_short, 'H': c_ushort, 'i': c_int, 'I': c_uint, 'l': c_long, 'L': c_ulong, 'q': c_longlong, 'Q': c_ulonglong, 'f': c_float, 'd': c_double}

def get_except(func, *args):
    try:
        func(*args)
    except Exception as detail:
        return (detail.__class__, str(detail))

def _test_issue18060(Vector):
    if sys.platform == 'win32':
        libm = CDLL(find_library('msvcrt.dll'))
    else:
        libm = CDLL(find_library('m'))
    libm.atan2.argtypes = [Vector]
    libm.atan2.restype = c_double
    arg = Vector(y=0.0, x=-1.0)
    assert abs(libm.atan2(arg) - 3.141592653589793) < 1e-07

def create_class(length):

    class S(Structure):
        _fields_ = [('x' * length, c_int)]
for length in [10 ** i for i in range(0, 8)]:
    try:
        create_class(length)
    except MemoryError:
        pass

print("StructureTestCase::test_huge_field_name: ok")
