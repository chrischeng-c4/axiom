# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "arrays"
# dimension = "behavior"
# case = "array_test_case__test_step_overflow_ucaf2a21"
# subject = "cpython.test_arrays.ArrayTestCase.test_step_overflow"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_arrays.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sys
from ctypes import *
a = (c_int * 5)()
a[3::sys.maxsize] = (1,)
assert a[3::sys.maxsize] == [1]
a = (c_char * 5)()
a[3::sys.maxsize] = b'A'
assert a[3::sys.maxsize] == b'A'
a = (c_wchar * 5)()
a[3::sys.maxsize] = u'X'
assert a[3::sys.maxsize] == u'X'

print("ArrayTestCase::test_step_overflow: ok")
