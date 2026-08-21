# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keeprefs"
# dimension = "behavior"
# case = "simple_test_case__test_cint_uc554f17"
# subject = "cpython.test_keeprefs.SimpleTestCase.test_cint"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_keeprefs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
x = c_int()
assert x._objects == None
x.value = 42
assert x._objects == None
x = c_int(99)
assert x._objects == None

print("SimpleTestCase::test_cint: ok")
