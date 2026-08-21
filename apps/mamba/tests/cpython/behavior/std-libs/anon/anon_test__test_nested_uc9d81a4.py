# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "anon"
# dimension = "behavior"
# case = "anon_test__test_nested_uc9d81a4"
# subject = "cpython.test_anon.AnonTest.test_nested"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_anon.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *

class ANON_S(Structure):
    _fields_ = [('a', c_int)]

class ANON_U(Union):
    _fields_ = [('_', ANON_S), ('b', c_int)]
    _anonymous_ = ['_']

class Y(Structure):
    _fields_ = [('x', c_int), ('_', ANON_U), ('y', c_int)]
    _anonymous_ = ['_']
assert Y.x.offset == 0
assert Y.a.offset == sizeof(c_int)
assert Y.b.offset == sizeof(c_int)
assert Y._.offset == sizeof(c_int)
assert Y.y.offset == sizeof(c_int) * 2

print("AnonTest::test_nested: ok")
