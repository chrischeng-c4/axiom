# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "arrays"
# dimension = "behavior"
# case = "array_test_case__test_classcache_uc419901"
# subject = "cpython.test_arrays.ArrayTestCase.test_classcache"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_arrays.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sys
from ctypes import *
assert ARRAY(c_int, 3) is not ARRAY(c_int, 4)
assert ARRAY(c_int, 3) is ARRAY(c_int, 3)

print("ArrayTestCase::test_classcache: ok")
