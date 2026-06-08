# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cfuncs"
# dimension = "behavior"
# case = "c_functions__test_ulong_ucb560be"
# subject = "cpython.test_cfuncs.CFunctions.test_ulong"
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
_dll.tf_L.restype = c_ulong
_dll.tf_L.argtypes = (c_ulong,)
assert _dll.tf_L(4294967295) == 1431655765
assert U() == 4294967295

print("CFunctions::test_ulong: ok")
