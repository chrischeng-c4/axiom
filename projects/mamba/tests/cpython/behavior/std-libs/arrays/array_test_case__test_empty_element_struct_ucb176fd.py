# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "arrays"
# dimension = "behavior"
# case = "array_test_case__test_empty_element_struct_ucb176fd"
# subject = "cpython.test_arrays.ArrayTestCase.test_empty_element_struct"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_arrays.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sys
from ctypes import *

class EmptyStruct(Structure):
    _fields_ = []
obj = (EmptyStruct * 2)()
assert sizeof(obj) == 0

print("ArrayTestCase::test_empty_element_struct: ok")
