# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "internals"
# dimension = "behavior"
# case = "objects_test_case__test_simple_struct_uccf5019"
# subject = "cpython.test_internals.ObjectsTestCase.test_simple_struct"
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
    _fields_ = [('a', c_int), ('b', c_int)]
a = 421234
b = 421235
x = X()
assert x._objects == None
x.a = a
x.b = b
assert x._objects == None

print("ObjectsTestCase::test_simple_struct: ok")
