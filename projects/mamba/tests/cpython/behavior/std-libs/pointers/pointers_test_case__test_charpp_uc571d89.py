# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pointers"
# dimension = "behavior"
# case = "pointers_test_case__test_charpp_uc571d89"
# subject = "cpython.test_pointers.PointersTestCase.test_charpp"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_pointers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test
'Test that a character pointer-to-pointer is correctly passed'
dll = CDLL(_ctypes_test.__file__)
func = dll._testfunc_c_p_p
func.restype = c_char_p
argv = (c_char_p * 2)()
argc = c_int(2)
argv[0] = b'hello'
argv[1] = b'world'
result = func(byref(argc), argv)
assert result == b'world'

print("PointersTestCase::test_charpp: ok")
