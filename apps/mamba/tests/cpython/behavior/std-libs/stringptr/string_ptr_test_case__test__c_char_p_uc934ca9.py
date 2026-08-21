# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringptr"
# dimension = "behavior"
# case = "string_ptr_test_case__test__c_char_p_uc934ca9"
# subject = "cpython.test_stringptr.StringPtrTestCase.test__c_char_p"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_stringptr.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test

class X(Structure):
    _fields_ = [('str', c_char_p)]
x = X()
assert x.str == None
x.str = b'Hello, World'
assert x.str == b'Hello, World'
b = c_buffer(b'Hello, World')
try:
    setattr(x, b'str', b)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("StringPtrTestCase::test__c_char_p: ok")
