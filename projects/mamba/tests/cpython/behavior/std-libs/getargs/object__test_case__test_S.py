# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getargs"
# dimension = "behavior"
# case = "object__test_case__test_S"
# subject = "cpython.test_getargs.Object_TestCase.test_S"
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
from _testcapi import getargs_S
obj = b'bytes'
assert getargs_S(obj) is obj
try:
    getargs_S(bytearray(b'bytearray'))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_S('str')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_S(None)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_S(memoryview(obj))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("Object_TestCase::test_S: ok")
