# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structures"
# dimension = "behavior"
# case = "structure_test_case__test_38368"
# subject = "cpython.test_structures.StructureTestCase.test_38368"
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

class U(Union):
    _fields_ = [('f1', c_uint8 * 16), ('f2', c_uint16 * 8), ('f3', c_uint32 * 4)]
u = U()
u.f3[0] = 19088743
u.f3[1] = 2309737967
u.f3[2] = 1985229328
u.f3[3] = 4275878552
f1 = [u.f1[i] for i in range(16)]
f2 = [u.f2[i] for i in range(8)]
if sys.byteorder == 'little':
    assert f1 == [103, 69, 35, 1, 239, 205, 171, 137, 16, 50, 84, 118, 152, 186, 220, 254]
    assert f2 == [17767, 291, 52719, 35243, 12816, 30292, 47768, 65244]

print("StructureTestCase::test_38368: ok")
