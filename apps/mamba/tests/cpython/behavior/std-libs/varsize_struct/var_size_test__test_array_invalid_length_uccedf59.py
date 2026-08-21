# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "varsize_struct"
# dimension = "behavior"
# case = "var_size_test__test_array_invalid_length_uccedf59"
# subject = "cpython.test_varsize_struct.VarSizeTest.test_array_invalid_length"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_varsize_struct.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
try:
    (lambda: c_int * -1)()
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    (lambda: c_int * -3)()
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass

print("VarSizeTest::test_array_invalid_length: ok")
