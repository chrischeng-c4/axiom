# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "incomplete"
# dimension = "behavior"
# case = "my_test_case__test_incomplete_example_uc09acf8"
# subject = "cpython.test_incomplete.MyTestCase.test_incomplete_example"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_incomplete.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
lpcell = POINTER('cell')

class cell(Structure):
    _fields_ = [('name', c_char_p), ('next', lpcell)]
SetPointerType(lpcell, cell)
c1 = cell()
c1.name = b'foo'
c2 = cell()
c2.name = b'bar'
c1.next = pointer(c2)
c2.next = pointer(c1)
p = c1
result = []
for i in range(8):
    result.append(p.name)
    p = p.next[0]
assert result == [b'foo', b'bar'] * 4
from ctypes import _pointer_type_cache
del _pointer_type_cache[cell]

print("MyTestCase::test_incomplete_example: ok")
