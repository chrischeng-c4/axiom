# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keeprefs"
# dimension = "behavior"
# case = "pointer_to_structure__test_uc3a0fb2"
# subject = "cpython.test_keeprefs.PointerToStructure.test"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_keeprefs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *

class POINT(Structure):
    _fields_ = [('x', c_int), ('y', c_int)]

class RECT(Structure):
    _fields_ = [('a', POINTER(POINT)), ('b', POINTER(POINT))]
r = RECT()
p1 = POINT(1, 2)
r.a = pointer(p1)
r.b = pointer(p1)
r.a[0].x = 42
r.a[0].y = 99
from ctypes import _pointer_type_cache
del _pointer_type_cache[POINT]

print("PointerToStructure::test: ok")
