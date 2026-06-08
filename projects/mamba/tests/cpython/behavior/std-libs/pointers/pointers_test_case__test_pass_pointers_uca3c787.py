# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pointers"
# dimension = "behavior"
# case = "pointers_test_case__test_pass_pointers_uca3c787"
# subject = "cpython.test_pointers.PointersTestCase.test_pass_pointers"
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
if sizeof(c_longlong) == sizeof(c_void_p):
    func.restype = c_longlong
else:
    func.restype = c_long
i = c_int(12345678)
address = func(byref(i))
assert c_int.from_address(address).value == 12345678
func.restype = POINTER(c_int)
res = func(pointer(i))
assert res.contents.value == 12345678
assert res[0] == 12345678

print("PointersTestCase::test_pass_pointers: ok")
