# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keeprefs"
# dimension = "behavior"
# case = "structure_test_case__test_struct_struct_uc101fef"
# subject = "cpython.test_keeprefs.StructureTestCase.test_struct_struct"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_keeprefs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *

class POINT(Structure):
    _fields_ = [('x', c_int), ('y', c_int)]

class RECT(Structure):
    _fields_ = [('ul', POINT), ('lr', POINT)]
r = RECT()
r.ul.x = 0
r.ul.y = 1
r.lr.x = 2
r.lr.y = 3
assert r._objects == None
r = RECT()
pt = POINT(1, 2)
r.ul = pt
assert r._objects == {'0': {}}
r.ul.x = 22
r.ul.y = 44
assert r._objects == {'0': {}}
r.lr = POINT()
assert r._objects == {'0': {}, '1': {}}

print("StructureTestCase::test_struct_struct: ok")
