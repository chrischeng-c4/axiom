# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "internals"
# dimension = "behavior"
# case = "objects_test_case__test_xxx_uc4bbbca"
# subject = "cpython.test_internals.ObjectsTestCase.test_xxx"
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
    _fields_ = [('a', c_char_p), ('b', c_char_p)]

class Y(Structure):
    _fields_ = [('x', X), ('y', X)]
s1 = b'Hello, World'
s2 = b'Hallo, Welt'
x = X()
x.a = s1
x.b = s2
assert x._objects == {'0': s1, '1': s2}
y = Y()
y.x = x
assert y._objects == {'0': {'0': s1, '1': s2}}

print("ObjectsTestCase::test_xxx: ok")
