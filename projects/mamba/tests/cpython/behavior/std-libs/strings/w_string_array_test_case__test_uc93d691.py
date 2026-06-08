# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strings"
# dimension = "behavior"
# case = "w_string_array_test_case__test_uc93d691"
# subject = "cpython.test_strings.WStringArrayTestCase.test"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_strings.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
BUF = c_wchar * 4
buf = BUF('a', 'b', 'c')
assert buf.value == 'abc'
buf.value = 'ABCD'
assert buf.value == 'ABCD'
buf.value = 'x'
assert buf.value == 'x'
buf[1] = 'Z'
assert buf.value == 'xZCD'

print("WStringArrayTestCase::test: ok")
