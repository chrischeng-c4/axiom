# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getargs"
# dimension = "behavior"
# case = "string__test_case__test_es_hash"
# subject = "cpython.test_getargs.String_TestCase.test_es_hash"
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
from _testcapi import getargs_es_hash
assert getargs_es_hash('abcé') == b'abc\xc3\xa9'
assert getargs_es_hash('abcé', 'latin1') == b'abc\xe9'
try:
    getargs_es_hash('abcé', 'ascii')
    raise AssertionError('assertRaises: no raise')
except UnicodeEncodeError:
    pass
try:
    getargs_es_hash('abcé', 'spam')
    raise AssertionError('assertRaises: no raise')
except LookupError:
    pass
try:
    getargs_es_hash(b'bytes', 'latin1')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_es_hash(bytearray(b'bytearray'), 'latin1')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_es_hash(memoryview(b'memoryview'), 'latin1')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    getargs_es_hash(None, 'latin1')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
assert getargs_es_hash('nul:\x00', 'latin1') == b'nul:\x00'
buf = bytearray(b'x' * 8)
assert getargs_es_hash('abcé', 'latin1', buf) == b'abc\xe9'
assert buf == bytearray(b'abc\xe9\x00xxx')
buf = bytearray(b'x' * 5)
assert getargs_es_hash('abcé', 'latin1', buf) == b'abc\xe9'
assert buf == bytearray(b'abc\xe9\x00')
buf = bytearray(b'x' * 4)
try:
    getargs_es_hash('abcé', 'latin1', buf)
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
assert buf == bytearray(b'x' * 4)
buf = bytearray()
try:
    getargs_es_hash('abcé', 'latin1', buf)
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass

print("String_TestCase::test_es_hash: ok")
