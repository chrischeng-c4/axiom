# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sizes"
# dimension = "behavior"
# case = "sizes_test_case__test_time_t_uc533d46"
# subject = "cpython.test_sizes.SizesTestCase.test_time_t"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_sizes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
assert sizeof(c_time_t) == SIZEOF_TIME_T

print("SizesTestCase::test_time_t: ok")
