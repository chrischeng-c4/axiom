# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "buffers"
# dimension = "behavior"
# case = "string_buffer_test_case__test_buffer_interface_ucc894df"
# subject = "cpython.test_buffers.StringBufferTestCase.test_buffer_interface"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_buffers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
assert len(bytearray(create_string_buffer(0))) == 0
assert len(bytearray(create_string_buffer(1))) == 1

print("StringBufferTestCase::test_buffer_interface: ok")
