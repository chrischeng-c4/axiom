# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getargs"
# dimension = "behavior"
# case = "bytes__test_case__test_y"
# subject = "cpython.test_getargs.Bytes_TestCase.test_y"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_getargs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import math
import string
import sys
from _testcapi import getargs_keywords, getargs_keyword_only
from _testcapi import UCHAR_MAX, USHRT_MAX, UINT_MAX, ULONG_MAX, INT_MAX, INT_MIN, LONG_MIN, LONG_MAX, PY_SSIZE_T_MIN, PY_SSIZE_T_MAX, SHRT_MIN, SHRT_MAX, FLT_MIN, FLT_MAX, DBL_MIN, DBL_MAX
from _testcapi import getargs_y
try:
    getargs_y('abcé')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
assert getargs_y(b'bytes') == b'bytes'
try:
    getargs_y(b'nul:\x00')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    getargs_y(bytearray(b'bytearray'))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_y(memoryview(b'memoryview'))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_y(None)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("Bytes_TestCase::test_y: ok")
