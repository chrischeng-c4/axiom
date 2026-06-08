# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "slicing"
# dimension = "behavior"
# case = "slices_test_case__test_char_ptr_with_free_ucebec10"
# subject = "cpython.test_slicing.SlicesTestCase.test_char_ptr_with_free"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_slicing.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test
dll = CDLL(_ctypes_test.__file__)
s = b'abcdefghijklmnopqrstuvwxyz'

class allocated_c_char_p(c_char_p):
    pass
dll.my_free.restype = None

def errcheck(result, func, args):
    retval = result.value
    dll.my_free(result)
    return retval
dll.my_strdup.restype = allocated_c_char_p
dll.my_strdup.errcheck = errcheck
try:
    res = dll.my_strdup(s)
    assert res == s
finally:
    del dll.my_strdup.errcheck

print("SlicesTestCase::test_char_ptr_with_free: ok")
