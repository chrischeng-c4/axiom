# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "buffers"
# dimension = "behavior"
# case = "string_buffer_test_case__test_buffer_uca8cf3e"
# subject = "cpython.test_buffers.StringBufferTestCase.test_buffer"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_buffers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
b = create_string_buffer(32)
assert len(b) == 32
assert sizeof(b) == 32 * sizeof(c_char)
assert type(b[0]) is bytes
b = create_string_buffer(b'abc')
assert len(b) == 4
assert sizeof(b) == 4 * sizeof(c_char)
assert type(b[0]) is bytes
assert b[0] == b'a'
assert b[:] == b'abc\x00'
assert b[:] == b'abc\x00'
assert b[::-1] == b'\x00cba'
assert b[::2] == b'ac'
assert b[::5] == b'a'
try:
    create_string_buffer('abc')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("StringBufferTestCase::test_buffer: ok")
