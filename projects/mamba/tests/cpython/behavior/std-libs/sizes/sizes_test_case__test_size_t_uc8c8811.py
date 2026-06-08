# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sizes"
# dimension = "behavior"
# case = "sizes_test_case__test_size_t_uc8c8811"
# subject = "cpython.test_sizes.SizesTestCase.test_size_t"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_sizes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
assert sizeof(c_void_p) == sizeof(c_size_t)

print("SizesTestCase::test_size_t: ok")
