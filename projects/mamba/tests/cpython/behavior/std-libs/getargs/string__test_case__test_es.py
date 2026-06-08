# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getargs"
# dimension = "behavior"
# case = "string__test_case__test_es"
# subject = "cpython.test_getargs.String_TestCase.test_es"
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
from _testcapi import getargs_es
assert getargs_es('abcé') == b'abc\xc3\xa9'
assert getargs_es('abcé', 'latin1') == b'abc\xe9'
try:
    getargs_es('abcé', 'ascii')
    raise AssertionError('assertRaises: no raise')
except UnicodeEncodeError:
    pass
try:
    getargs_es('abcé', 'spam')
    raise AssertionError('assertRaises: no raise')
except LookupError:
    pass
try:
    getargs_es(b'bytes', 'latin1')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_es(bytearray(b'bytearray'), 'latin1')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_es(memoryview(b'memoryview'), 'latin1')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_es(None, 'latin1')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_es('nul:\x00', 'latin1')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("String_TestCase::test_es: ok")
