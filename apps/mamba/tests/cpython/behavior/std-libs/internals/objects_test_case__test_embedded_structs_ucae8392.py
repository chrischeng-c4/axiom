# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "internals"
# dimension = "behavior"
# case = "objects_test_case__test_embedded_structs_ucae8392"
# subject = "cpython.test_internals.ObjectsTestCase.test_embedded_structs"
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

class Y(Structure):
    _fields_ = [('x', X), ('y', X)]
y = Y()
assert y._objects == None
x1, x2 = (X(), X())
y.x, y.y = (x1, x2)
assert y._objects == {'0': {}, '1': {}}
x1.a, x2.b = (42, 93)
assert y._objects == {'0': {}, '1': {}}

print("ObjectsTestCase::test_embedded_structs: ok")
