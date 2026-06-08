# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sizes"
# dimension = "behavior"
# case = "sizes_test_case__test_16_ucf0c823"
# subject = "cpython.test_sizes.SizesTestCase.test_16"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_sizes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
assert 2 == sizeof(c_int16)
assert 2 == sizeof(c_uint16)

print("SizesTestCase::test_16: ok")
