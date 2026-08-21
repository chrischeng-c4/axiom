# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sizes"
# dimension = "behavior"
# case = "sizes_test_case__test_8_uc027758"
# subject = "cpython.test_sizes.SizesTestCase.test_8"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_sizes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
assert 1 == sizeof(c_int8)
assert 1 == sizeof(c_uint8)

print("SizesTestCase::test_8: ok")
