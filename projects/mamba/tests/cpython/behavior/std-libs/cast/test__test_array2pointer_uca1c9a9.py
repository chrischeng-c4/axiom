# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cast"
# dimension = "behavior"
# case = "test__test_array2pointer_uca1c9a9"
# subject = "cpython.test_cast.Test.test_array2pointer"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_cast.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import sys
array = (c_int * 3)(42, 17, 2)
ptr = cast(array, POINTER(c_int))
assert [ptr[i] for i in range(3)] == [42, 17, 2]
if 2 * sizeof(c_short) == sizeof(c_int):
    ptr = cast(array, POINTER(c_short))
    if sys.byteorder == 'little':
        assert [ptr[i] for i in range(6)] == [42, 0, 17, 0, 2, 0]
    else:
        assert [ptr[i] for i in range(6)] == [0, 42, 0, 17, 0, 2]

print("Test::test_array2pointer: ok")
