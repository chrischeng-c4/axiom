# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getargs"
# dimension = "behavior"
# case = "object__test_case__test_Y"
# subject = "cpython.test_getargs.Object_TestCase.test_Y"
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
from _testcapi import getargs_Y
obj = bytearray(b'bytearray')
assert getargs_Y(obj) is obj
try:
    getargs_Y(b'bytes')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_Y('str')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_Y(None)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_Y(memoryview(obj))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("Object_TestCase::test_Y: ok")
