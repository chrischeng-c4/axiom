# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "internals"
# dimension = "behavior"
# case = "objects_test_case__test_ptr_struct_ucd549a0"
# subject = "cpython.test_internals.ObjectsTestCase.test_ptr_struct"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_internals.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
from sys import getrefcount as grc

def assertSame(a, b):
    assert id(a) == id(b)

class X(Structure):
    _fields_ = [('data', POINTER(c_int))]
A = c_int * 4
a = A(11, 22, 33, 44)
assert a._objects == None
x = X()
x.data = a

print("ObjectsTestCase::test_ptr_struct: ok")
