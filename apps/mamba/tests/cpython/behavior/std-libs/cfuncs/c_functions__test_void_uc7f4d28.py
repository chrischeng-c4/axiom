# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cfuncs"
# dimension = "behavior"
# case = "c_functions__test_void_uc7f4d28"
# subject = "cpython.test_cfuncs.CFunctions.test_void"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_cfuncs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test
_dll = CDLL(_ctypes_test.__file__)

def S():
    return c_longlong.in_dll(_dll, 'last_tf_arg_s').value

def U():
    return c_ulonglong.in_dll(_dll, 'last_tf_arg_u').value
_dll.tv_i.restype = None
_dll.tv_i.argtypes = (c_int,)
assert _dll.tv_i(42) == None
assert S() == 42
assert _dll.tv_i(-42) == None
assert S() == -42

print("CFunctions::test_void: ok")
