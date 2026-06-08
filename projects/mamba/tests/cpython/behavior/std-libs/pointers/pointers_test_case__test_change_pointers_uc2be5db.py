# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pointers"
# dimension = "behavior"
# case = "pointers_test_case__test_change_pointers_uc2be5db"
# subject = "cpython.test_pointers.PointersTestCase.test_change_pointers"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_pointers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test
dll = CDLL(_ctypes_test.__file__)
func = dll._testfunc_p_p
i = c_int(87654)
func.restype = POINTER(c_int)
func.argtypes = (POINTER(c_int),)
res = func(pointer(i))
assert res[0] == 87654
assert res.contents.value == 87654
res[0] = 54345
assert i.value == 54345
x = c_int(12321)
res.contents = x
assert i.value == 54345
x.value = -99
assert res.contents.value == -99

print("PointersTestCase::test_change_pointers: ok")
