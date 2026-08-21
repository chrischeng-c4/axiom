# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keeprefs"
# dimension = "behavior"
# case = "structure_test_case__test_ccharp_struct_uc477213"
# subject = "cpython.test_keeprefs.StructureTestCase.test_ccharp_struct"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_keeprefs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *

class X(Structure):
    _fields_ = [('a', c_char_p), ('b', c_char_p)]
x = X()
assert x._objects == None
x.a = b'spam'
x.b = b'foo'
assert x._objects == {'0': b'spam', '1': b'foo'}

print("StructureTestCase::test_ccharp_struct: ok")
