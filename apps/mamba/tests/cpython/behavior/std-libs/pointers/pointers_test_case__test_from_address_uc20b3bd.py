# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pointers"
# dimension = "behavior"
# case = "pointers_test_case__test_from_address_uc20b3bd"
# subject = "cpython.test_pointers.PointersTestCase.test_from_address"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_pointers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test
from array import array
a = array('i', [100, 200, 300, 400, 500])
addr = a.buffer_info()[0]
p = POINTER(POINTER(c_int))

print("PointersTestCase::test_from_address: ok")
