# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strings"
# dimension = "behavior"
# case = "string_array_test_case__test_c_buffer_raw_uc0fc0b0"
# subject = "cpython.test_strings.StringArrayTestCase.test_c_buffer_raw"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_strings.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
buf = c_buffer(32)
buf.raw = memoryview(b'Hello, World')
assert buf.value == b'Hello, World'
try:
    setattr(buf, 'value', memoryview(b'abc'))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    setattr(buf, 'raw', memoryview(b'x' * 100))
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass

print("StringArrayTestCase::test_c_buffer_raw: ok")
