# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "arrays"
# dimension = "behavior"
# case = "array_test_case__test_subclass_uceb3b0f"
# subject = "cpython.test_arrays.ArrayTestCase.test_subclass"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_arrays.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sys
from ctypes import *

class T(Array):
    _type_ = c_int
    _length_ = 13

class U(T):
    pass

class V(U):
    pass

class W(V):
    pass

class X(T):
    _type_ = c_short

class Y(T):
    _length_ = 187
for c in [T, U, V, W]:
    assert c._type_ == c_int
    assert c._length_ == 13
    assert c()._type_ == c_int
    assert c()._length_ == 13
assert X._type_ == c_short
assert X._length_ == 13
assert X()._type_ == c_short
assert X()._length_ == 13
assert Y._type_ == c_int
assert Y._length_ == 187
assert Y()._type_ == c_int
assert Y()._length_ == 187

print("ArrayTestCase::test_subclass: ok")
