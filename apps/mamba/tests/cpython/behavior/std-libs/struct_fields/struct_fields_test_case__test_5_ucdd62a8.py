# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct_fields"
# dimension = "behavior"
# case = "struct_fields_test_case__test_5_ucdd62a8"
# subject = "cpython.test_struct_fields.StructFieldsTestCase.test_5"
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
    _fields_ = (('char', c_char * 5),)
x = X(b'#' * 5)
x.char = b'a\x00b\x00'
assert bytes(x) == b'a\x00###'

print("StructFieldsTestCase::test_5: ok")
