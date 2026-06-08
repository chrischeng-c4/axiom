# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getargs"
# dimension = "behavior"
# case = "string__test_case__test_C"
# subject = "cpython.test_getargs.String_TestCase.test_C"
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
from _testcapi import getargs_C
try:
    getargs_C('abc')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
assert getargs_C('a') == 97
assert getargs_C('€') == 8364
assert getargs_C('🐍') == 128013
try:
    getargs_C(b'a')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_C(bytearray(b'a'))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_C(memoryview(b'a'))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_C(97)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_C(None)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("String_TestCase::test_C: ok")
