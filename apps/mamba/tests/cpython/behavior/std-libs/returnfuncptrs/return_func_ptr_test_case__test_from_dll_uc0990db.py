# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "returnfuncptrs"
# dimension = "behavior"
# case = "return_func_ptr_test_case__test_from_dll_uc0990db"
# subject = "cpython.test_returnfuncptrs.ReturnFuncPtrTestCase.test_from_dll"
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
strchr = CFUNCTYPE(c_char_p, c_char_p, c_char)(('my_strchr', dll))
assert strchr(b'abcdef', b'b')
assert strchr(b'abcdef', b'x') == None
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

print("ReturnFuncPtrTestCase::test_from_dll: ok")
