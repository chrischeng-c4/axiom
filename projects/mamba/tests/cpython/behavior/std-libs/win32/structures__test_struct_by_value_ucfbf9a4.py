# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "win32"
# dimension = "behavior"
# case = "structures__test_struct_by_value_ucfbf9a4"
# subject = "cpython.test_win32.Structures.test_struct_by_value"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_win32.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test

class POINT(Structure):
    _fields_ = [('x', c_long), ('y', c_long)]

class RECT(Structure):
    _fields_ = [('left', c_long), ('top', c_long), ('right', c_long), ('bottom', c_long)]
dll = CDLL(_ctypes_test.__file__)
pt = POINT(15, 25)
left = c_long.in_dll(dll, 'left')
top = c_long.in_dll(dll, 'top')
right = c_long.in_dll(dll, 'right')
bottom = c_long.in_dll(dll, 'bottom')
rect = RECT(left, top, right, bottom)
PointInRect = dll.PointInRect
PointInRect.argtypes = [POINTER(RECT), POINT]
assert 1 == PointInRect(byref(rect), pt)
ReturnRect = dll.ReturnRect
ReturnRect.argtypes = [c_int, RECT, POINTER(RECT), POINT, RECT, POINTER(RECT), POINT, RECT]
ReturnRect.restype = RECT
for i in range(4):
    ret = ReturnRect(i, rect, pointer(rect), pt, rect, byref(rect), pt, rect)
    assert ret.left == left.value
    assert ret.right == right.value
    assert ret.top == top.value
    assert ret.bottom == bottom.value
from ctypes import _pointer_type_cache
del _pointer_type_cache[RECT]

print("Structures::test_struct_by_value: ok")
