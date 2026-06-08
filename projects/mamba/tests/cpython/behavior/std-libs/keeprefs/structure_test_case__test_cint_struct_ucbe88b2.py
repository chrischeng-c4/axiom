# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keeprefs"
# dimension = "behavior"
# case = "structure_test_case__test_cint_struct_ucbe88b2"
# subject = "cpython.test_keeprefs.StructureTestCase.test_cint_struct"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_keeprefs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *

class X(Structure):
    _fields_ = [('a', c_int), ('b', c_int)]
x = X()
assert x._objects == None
x.a = 42
x.b = 99
assert x._objects == None

print("StructureTestCase::test_cint_struct: ok")
