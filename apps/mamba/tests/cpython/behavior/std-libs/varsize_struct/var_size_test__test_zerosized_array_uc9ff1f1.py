# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "varsize_struct"
# dimension = "behavior"
# case = "var_size_test__test_zerosized_array_uc9ff1f1"
# subject = "cpython.test_varsize_struct.VarSizeTest.test_zerosized_array"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_varsize_struct.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
array = (c_int * 0)()
try:
    array.__setitem__(0, None)
    raise AssertionError('assertRaises: no raise')
except IndexError:
    pass
try:
    array.__getitem__(0)
    raise AssertionError('assertRaises: no raise')
except IndexError:
    pass
try:
    array.__setitem__(1, None)
    raise AssertionError('assertRaises: no raise')
except IndexError:
    pass
try:
    array.__getitem__(1)
    raise AssertionError('assertRaises: no raise')
except IndexError:
    pass
try:
    array.__setitem__(-1, None)
    raise AssertionError('assertRaises: no raise')
except IndexError:
    pass
try:
    array.__getitem__(-1)
    raise AssertionError('assertRaises: no raise')
except IndexError:
    pass

print("VarSizeTest::test_zerosized_array: ok")
