# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strings"
# dimension = "behavior"
# case = "string_array_test_case__test_uc8b30f8"
# subject = "cpython.test_strings.StringArrayTestCase.test"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_strings.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
BUF = c_char * 4
buf = BUF(b'a', b'b', b'c')
assert buf.value == b'abc'
assert buf.raw == b'abc\x00'
buf.value = b'ABCD'
assert buf.value == b'ABCD'
assert buf.raw == b'ABCD'
buf.value = b'x'
assert buf.value == b'x'
assert buf.raw == b'x\x00CD'
buf[1] = b'Z'
assert buf.value == b'xZCD'
assert buf.raw == b'xZCD'
try:
    setattr(buf, 'value', b'aaaaaaaa')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    setattr(buf, 'value', 42)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("StringArrayTestCase::test: ok")
