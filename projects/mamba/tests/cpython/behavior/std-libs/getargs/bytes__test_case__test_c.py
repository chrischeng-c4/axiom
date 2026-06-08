# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getargs"
# dimension = "behavior"
# case = "bytes__test_case__test_c"
# subject = "cpython.test_getargs.Bytes_TestCase.test_c"
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
from _testcapi import getargs_c
try:
    getargs_c(b'abc')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
assert getargs_c(b'a') == 97
assert getargs_c(bytearray(b'a')) == 97
try:
    getargs_c(memoryview(b'a'))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_c('s')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_c(97)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_c(None)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("Bytes_TestCase::test_c: ok")
