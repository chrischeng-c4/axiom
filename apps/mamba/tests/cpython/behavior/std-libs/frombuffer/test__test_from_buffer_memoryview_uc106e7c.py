# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "frombuffer"
# dimension = "behavior"
# case = "test__test_from_buffer_memoryview_uc106e7c"
# subject = "cpython.test_frombuffer.Test.test_from_buffer_memoryview"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_frombuffer.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import array
import gc
a = [c_char.from_buffer(memoryview(bytearray(b'a')))]
a.append(a)
del a
gc.collect()

print("Test::test_from_buffer_memoryview: ok")
