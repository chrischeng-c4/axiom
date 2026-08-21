# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "arrays"
# dimension = "behavior"
# case = "array_test_case__test_empty_element_array_uc5df13b"
# subject = "cpython.test_arrays.ArrayTestCase.test_empty_element_array"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_arrays.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sys
from ctypes import *

class EmptyArray(Array):
    _type_ = c_int
    _length_ = 0
obj = (EmptyArray * 2)()
assert sizeof(obj) == 0

print("ArrayTestCase::test_empty_element_array: ok")
