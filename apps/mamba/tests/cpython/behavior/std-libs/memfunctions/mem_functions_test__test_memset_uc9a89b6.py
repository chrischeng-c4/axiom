# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "memfunctions"
# dimension = "behavior"
# case = "mem_functions_test__test_memset_uc9a89b6"
# subject = "cpython.test_memfunctions.MemFunctionsTest.test_memset"
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
result = memset(a, ord('x'), 16)
assert a.value == b'xxxxxxxxxxxxxxxx'
assert string_at(result) == b'xxxxxxxxxxxxxxxx'
assert string_at(a) == b'xxxxxxxxxxxxxxxx'
assert string_at(a, 20) == b'xxxxxxxxxxxxxxxx\x00\x00\x00\x00'

print("MemFunctionsTest::test_memset: ok")
