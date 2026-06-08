# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getargs"
# dimension = "behavior"
# case = "string__test_case__test_s"
# subject = "cpython.test_getargs.String_TestCase.test_s"
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
from _testcapi import getargs_s
assert getargs_s('abcé') == b'abc\xc3\xa9'
try:
    getargs_s('nul:\x00')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    getargs_s(b'bytes')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_s(bytearray(b'bytearray'))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_s(memoryview(b'memoryview'))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_s(None)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("String_TestCase::test_s: ok")
