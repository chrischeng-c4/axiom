# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "memfunctions"
# dimension = "behavior"
# case = "mem_functions_test__test_cast_uc949049"
# subject = "cpython.test_memfunctions.MemFunctionsTest.test_cast"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_memfunctions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sys
from ctypes import *
a = (c_ubyte * 32)(*map(ord, 'abcdef'))
assert cast(a, c_char_p).value == b'abcdef'
assert cast(a, POINTER(c_byte))[:7] == [97, 98, 99, 100, 101, 102, 0]
assert cast(a, POINTER(c_byte))[:7] == [97, 98, 99, 100, 101, 102, 0]
assert cast(a, POINTER(c_byte))[6:-1:-1] == [0, 102, 101, 100, 99, 98, 97]
assert cast(a, POINTER(c_byte))[:7:2] == [97, 99, 101, 0]
assert cast(a, POINTER(c_byte))[:7:7] == [97]

print("MemFunctionsTest::test_cast: ok")
