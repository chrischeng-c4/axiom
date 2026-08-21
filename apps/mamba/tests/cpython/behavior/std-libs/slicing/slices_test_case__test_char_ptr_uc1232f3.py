# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "slicing"
# dimension = "behavior"
# case = "slices_test_case__test_char_ptr_uc1232f3"
# subject = "cpython.test_slicing.SlicesTestCase.test_char_ptr"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_slicing.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test
s = b'abcdefghijklmnopqrstuvwxyz'
dll = CDLL(_ctypes_test.__file__)
dll.my_strdup.restype = POINTER(c_char)
dll.my_free.restype = None
res = dll.my_strdup(s)
assert res[:len(s)] == s
assert res[:3] == s[:3]
assert res[:len(s)] == s
assert res[len(s) - 1:-1:-1] == s[::-1]
assert res[len(s) - 1:5:-7] == s[:5:-7]
assert res[0:-1:-1] == s[0::-1]
import operator
try:
    operator.getitem(res, slice(None, None, None))
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    operator.getitem(res, slice(0, None, None))
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    operator.getitem(res, slice(None, 5, -1))
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    operator.getitem(res, slice(-5, None, None))
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    operator.setitem(res, slice(0, 5), 'abcde')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
dll.my_free(res)
dll.my_strdup.restype = POINTER(c_byte)
res = dll.my_strdup(s)
assert res[:len(s)] == list(range(ord('a'), ord('z') + 1))
assert res[:len(s)] == list(range(ord('a'), ord('z') + 1))
dll.my_free(res)

print("SlicesTestCase::test_char_ptr: ok")
