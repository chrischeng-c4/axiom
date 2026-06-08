# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "memoryview"
# dimension = "behavior"
# case = "other_test__test_memoryview_hex_ucf8e9a6"
# subject = "cpython.test_memoryview.OtherTest.test_memoryview_hex"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_memoryview.py"
# status = "filled"
# ///
import sys
import gc
import weakref
import array
import io
import copy
import pickle
import struct
x = b'0' * 200000
m1 = memoryview(x)
m2 = m1[::-1]
assert m2.hex() == '30' * 200000

print("OtherTest::test_memoryview_hex: ok")
