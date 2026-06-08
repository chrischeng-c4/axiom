# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getargs"
# dimension = "behavior"
# case = "tuple__test_case__test_tuple"
# subject = "cpython.test_getargs.Tuple_TestCase.test_tuple"
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
from _testcapi import getargs_tuple
ret = getargs_tuple(1, (2, 3))
assert ret == (1, 2, 3)

class seq:

    def __len__(self):
        return 2

    def __getitem__(self, n):
        raise ValueError
try:
    getargs_tuple(1, seq())
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("Tuple_TestCase::test_tuple: ok")
