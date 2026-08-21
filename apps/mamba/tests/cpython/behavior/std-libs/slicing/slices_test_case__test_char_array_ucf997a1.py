# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "slicing"
# dimension = "behavior"
# case = "slices_test_case__test_char_array_ucf997a1"
# subject = "cpython.test_slicing.SlicesTestCase.test_char_array"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_slicing.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test
s = b'abcdefghijklmnopqrstuvwxyz\x00'
p = (c_char * 27)(*s)
assert p[:] == s
assert p[:] == s
assert p[::-1] == s[::-1]
assert p[5::-2] == s[5::-2]
assert p[2:5:-3] == s[2:5:-3]

print("SlicesTestCase::test_char_array: ok")
