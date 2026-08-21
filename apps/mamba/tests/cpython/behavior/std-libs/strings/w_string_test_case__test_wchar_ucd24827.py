# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strings"
# dimension = "behavior"
# case = "w_string_test_case__test_wchar_ucd24827"
# subject = "cpython.test_strings.WStringTestCase.test_wchar"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_strings.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
c_wchar('x')
repr(byref(c_wchar('x')))
c_wchar('x')

print("WStringTestCase::test_wchar: ok")
