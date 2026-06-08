# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "memfunctions"
# dimension = "behavior"
# case = "mem_functions_test__test_memmove_ucdebe06"
# subject = "cpython.test_memfunctions.MemFunctionsTest.test_memmove"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_memfunctions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sys
from ctypes import *
a = create_string_buffer(1000000)
p = b'Hello, World'
result = memmove(a, p, len(p))
assert a.value == b'Hello, World'
assert string_at(result) == b'Hello, World'
assert string_at(result, 5) == b'Hello'
assert string_at(result, 16) == b'Hello, World\x00\x00\x00\x00'
assert string_at(result, 0) == b''

print("MemFunctionsTest::test_memmove: ok")
