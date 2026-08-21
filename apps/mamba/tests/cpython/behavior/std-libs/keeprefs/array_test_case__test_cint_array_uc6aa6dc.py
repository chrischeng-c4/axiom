# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keeprefs"
# dimension = "behavior"
# case = "array_test_case__test_cint_array_uc6aa6dc"
# subject = "cpython.test_keeprefs.ArrayTestCase.test_cint_array"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_keeprefs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
INTARR = c_int * 3
ia = INTARR()
assert ia._objects == None
ia[0] = 1
ia[1] = 2
ia[2] = 3
assert ia._objects == None

class X(Structure):
    _fields_ = [('x', c_int), ('a', INTARR)]
x = X()
x.x = 1000
x.a[0] = 42
x.a[1] = 96
assert x._objects == None
x.a = ia
assert x._objects == {'1': {}}

print("ArrayTestCase::test_cint_array: ok")
