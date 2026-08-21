# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "varsize_struct"
# dimension = "behavior"
# case = "var_size_test__test_resize_uc1d6774"
# subject = "cpython.test_varsize_struct.VarSizeTest.test_resize"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_varsize_struct.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *

class X(Structure):
    _fields_ = [('item', c_int), ('array', c_int * 1)]
assert sizeof(X) == sizeof(c_int) * 2
x = X()
x.item = 42
x.array[0] = 100
assert sizeof(x) == sizeof(c_int) * 2
new_size = sizeof(X) + sizeof(c_int) * 1
resize(x, new_size)
assert sizeof(x) == new_size
assert (x.item, x.array[0]) == (42, 100)
new_size = sizeof(X) + sizeof(c_int) * 9
resize(x, new_size)
assert sizeof(x) == new_size
assert (x.item, x.array[0]) == (42, 100)
new_size = sizeof(X) + sizeof(c_int) * 1
resize(x, new_size)
assert sizeof(x) == new_size
assert (x.item, x.array[0]) == (42, 100)

print("VarSizeTest::test_resize: ok")
