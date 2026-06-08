# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pointers"
# dimension = "behavior"
# case = "pointers_test_case__test_pointer_crash_uc17e973"
# subject = "cpython.test_pointers.PointersTestCase.test_pointer_crash"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_pointers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test

class A(POINTER(c_ulong)):
    pass
POINTER(c_ulong)(c_ulong(22))
try:
    A(c_ulong(33))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("PointersTestCase::test_pointer_crash: ok")
