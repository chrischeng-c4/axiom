# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pointers"
# dimension = "behavior"
# case = "pointers_test_case__test_other_uc84c307"
# subject = "cpython.test_pointers.PointersTestCase.test_other"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_pointers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test

class Table(Structure):
    _fields_ = [('a', c_int), ('b', c_int), ('c', c_int)]
pt = pointer(Table(1, 2, 3))
assert pt.contents.a == 1
assert pt.contents.b == 2
assert pt.contents.c == 3
pt.contents.c = 33
from ctypes import _pointer_type_cache
del _pointer_type_cache[Table]

print("PointersTestCase::test_other: ok")
