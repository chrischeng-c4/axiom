# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "arrays"
# dimension = "behavior"
# case = "array_test_case__test_cache_uc02ccab"
# subject = "cpython.test_arrays.ArrayTestCase.test_cache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_arrays.py"
# status = "filled"
# ///
import sys
from ctypes import *

class my_int(c_int):
    pass
t1 = my_int * 1
t2 = my_int * 1
assert t1 is t2

print("ArrayTestCase::test_cache: ok")
