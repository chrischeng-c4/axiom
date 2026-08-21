# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct_fields"
# dimension = "behavior"
# case = "struct_fields_test_case__test_6_ucd43183"
# subject = "cpython.test_struct_fields.StructFieldsTestCase.test_6"
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
    _fields_ = [('x', c_int)]
CField = type(X.x)
try:
    CField()
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("StructFieldsTestCase::test_6: ok")
