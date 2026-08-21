# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cast"
# dimension = "behavior"
# case = "test__test_address2pointer_ucf4ceaf"
# subject = "cpython.test_cast.Test.test_address2pointer"
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
address = addressof(array)
ptr = cast(c_void_p(address), POINTER(c_int))
assert [ptr[i] for i in range(3)] == [42, 17, 2]
ptr = cast(address, POINTER(c_int))
assert [ptr[i] for i in range(3)] == [42, 17, 2]

print("Test::test_address2pointer: ok")
