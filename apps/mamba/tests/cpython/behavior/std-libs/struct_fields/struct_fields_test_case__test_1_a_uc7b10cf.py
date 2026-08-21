# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct_fields"
# dimension = "behavior"
# case = "struct_fields_test_case__test_1_a_uc7b10cf"
# subject = "cpython.test_struct_fields.StructFieldsTestCase.test_1_A"
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
assert sizeof(X) == 0
X._fields_ = []
try:
    setattr(X, '_fields_', [])
    raise AssertionError('assertRaises: no raise')
except AttributeError:
    pass

print("StructFieldsTestCase::test_1_A: ok")
