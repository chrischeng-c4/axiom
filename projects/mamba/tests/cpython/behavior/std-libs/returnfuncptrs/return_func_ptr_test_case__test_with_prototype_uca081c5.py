# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "returnfuncptrs"
# dimension = "behavior"
# case = "return_func_ptr_test_case__test_with_prototype_uca081c5"
# subject = "cpython.test_returnfuncptrs.ReturnFuncPtrTestCase.test_with_prototype"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_returnfuncptrs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test
dll = CDLL(_ctypes_test.__file__)
get_strchr = dll.get_strchr
get_strchr.restype = CFUNCTYPE(c_char_p, c_char_p, c_char)
strchr = get_strchr()
assert strchr(b'abcdef', b'b') == b'bcdef'
assert strchr(b'abcdef', b'x') == None
assert strchr(b'abcdef', 98) == b'bcdef'
assert strchr(b'abcdef', 107) == None
try:
    strchr(b'abcdef', 3.0)
    raise AssertionError('assertRaises: no raise')
except ArgumentError:
    pass
try:
    strchr(b'abcdef')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("ReturnFuncPtrTestCase::test_with_prototype: ok")
