# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct_fields"
# dimension = "behavior"
# case = "struct_fields_test_case__test___set___uc689ea4"
# subject = "cpython.test_struct_fields.StructFieldsTestCase.test___set__"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_struct_fields.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sys
from ctypes import *

class MyCStruct(Structure):
    _fields_ = (('field', c_int),)
try:
    MyCStruct.field.__set__('wrong type self', 42)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

class MyCUnion(Union):
    _fields_ = (('field', c_int),)
try:
    MyCUnion.field.__set__('wrong type self', 42)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("StructFieldsTestCase::test___set__: ok")
