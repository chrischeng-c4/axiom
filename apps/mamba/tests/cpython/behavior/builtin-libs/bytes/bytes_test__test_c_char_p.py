# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_c_char_p"
# subject = "cpython.test_bytes.BytesTest.test_c_char_p"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_bytes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sys
from ctypes import *
c_char_p(b'foo bar')
try:
    c_char_p('foo bar')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("BytesTest::test_c_char_p: ok")
