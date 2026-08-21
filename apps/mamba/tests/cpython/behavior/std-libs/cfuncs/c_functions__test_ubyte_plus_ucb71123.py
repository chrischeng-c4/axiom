# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cfuncs"
# dimension = "behavior"
# case = "c_functions__test_ubyte_plus_ucb71123"
# subject = "cpython.test_cfuncs.CFunctions.test_ubyte_plus"
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
_dll.tf_bB.restype = c_ubyte
_dll.tf_bB.argtypes = (c_byte, c_ubyte)
assert _dll.tf_bB(0, 255) == 85
assert U() == 255

print("CFunctions::test_ubyte_plus: ok")
