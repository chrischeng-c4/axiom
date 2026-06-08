# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strings"
# dimension = "behavior"
# case = "string_array_test_case__test_param_2_ucc29434"
# subject = "cpython.test_strings.StringArrayTestCase.test_param_2"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_strings.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
BUF = c_char * 4
buf = BUF()

print("StringArrayTestCase::test_param_2: ok")
