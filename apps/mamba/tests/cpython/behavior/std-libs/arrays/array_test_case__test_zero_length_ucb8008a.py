# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "arrays"
# dimension = "behavior"
# case = "array_test_case__test_zero_length_ucb8008a"
# subject = "cpython.test_arrays.ArrayTestCase.test_zero_length"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_arrays.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sys
from ctypes import *

class T(Array):
    _type_ = c_int
    _length_ = 0

print("ArrayTestCase::test_zero_length: ok")
