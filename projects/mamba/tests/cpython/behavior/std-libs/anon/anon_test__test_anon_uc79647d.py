# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "anon"
# dimension = "behavior"
# case = "anon_test__test_anon_uc79647d"
# subject = "cpython.test_anon.AnonTest.test_anon"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_anon.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *

class ANON(Union):
    _fields_ = [('a', c_int), ('b', c_int)]

class Y(Structure):
    _fields_ = [('x', c_int), ('_', ANON), ('y', c_int)]
    _anonymous_ = ['_']
assert Y.a.offset == sizeof(c_int)
assert Y.b.offset == sizeof(c_int)
assert ANON.a.offset == 0
assert ANON.b.offset == 0

print("AnonTest::test_anon: ok")
