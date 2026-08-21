# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct_fields"
# dimension = "behavior"
# case = "struct_fields_test_case__test_4_ucbd24ab"
# subject = "cpython.test_struct_fields.StructFieldsTestCase.test_4"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_struct_fields.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sys
from ctypes import *

class X(Structure):
    pass

class Y(X):
    pass
try:
    setattr(X, '_fields_', [])
    raise AssertionError('assertRaises: no raise')
except AttributeError:
    pass
Y._fields_ = []
try:
    setattr(X, '_fields_', [])
    raise AssertionError('assertRaises: no raise')
except AttributeError:
    pass

print("StructFieldsTestCase::test_4: ok")
