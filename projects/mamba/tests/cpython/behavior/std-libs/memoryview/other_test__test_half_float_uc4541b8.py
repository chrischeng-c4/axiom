# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "memoryview"
# dimension = "behavior"
# case = "other_test__test_half_float_uc4541b8"
# subject = "cpython.test_memoryview.OtherTest.test_half_float"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_memoryview.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sys
import gc
import weakref
import array
import io
import copy
import pickle
import struct
half_data = struct.pack('eee', 0.0, -1.5, 1.5)
float_data = struct.pack('fff', 0.0, -1.5, 1.5)
half_view = memoryview(half_data).cast('e')
float_view = memoryview(float_data).cast('f')
assert half_view.nbytes * 2 == float_view.nbytes
assert half_view.tolist() == float_view.tolist()

print("OtherTest::test_half_float: ok")
