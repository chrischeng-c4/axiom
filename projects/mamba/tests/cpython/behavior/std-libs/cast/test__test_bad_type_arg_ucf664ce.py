# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cast"
# dimension = "behavior"
# case = "test__test_bad_type_arg_ucf664ce"
# subject = "cpython.test_cast.Test.test_bad_type_arg"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_cast.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import sys
array_type = c_byte * sizeof(c_int)
array = array_type()
try:
    cast(array, None)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    cast(array, array_type)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

class Struct(Structure):
    _fields_ = [('a', c_int)]
try:
    cast(array, Struct)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

class MyUnion(Union):
    _fields_ = [('a', c_int)]
try:
    cast(array, MyUnion)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("Test::test_bad_type_arg: ok")
