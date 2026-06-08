# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cfuncs"
# dimension = "behavior"
# case = "c_functions__test_callwithresult_ucf611cf"
# subject = "cpython.test_cfuncs.CFunctions.test_callwithresult"
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

def process_result(result):
    return result * 2
_dll.tf_i.restype = process_result
_dll.tf_i.argtypes = (c_int,)
assert _dll.tf_i(42) == 28
assert S() == 42
assert _dll.tf_i(-42) == -28
assert S() == -42

print("CFunctions::test_callwithresult: ok")
