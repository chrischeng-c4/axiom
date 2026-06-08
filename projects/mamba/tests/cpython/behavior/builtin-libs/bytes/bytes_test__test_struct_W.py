# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_struct_W"
# subject = "cpython.test_bytes.BytesTest.test_struct_W"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_bytes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sys
from ctypes import *

class X(Structure):
    _fields_ = [('a', c_wchar * 3)]
x = X('abc')
try:
    X(b'abc')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
assert x.a == 'abc'
assert type(x.a) == str

print("BytesTest::test_struct_W: ok")
