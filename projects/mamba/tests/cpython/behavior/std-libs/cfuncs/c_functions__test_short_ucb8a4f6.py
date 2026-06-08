# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cfuncs"
# dimension = "behavior"
# case = "c_functions__test_short_ucb8a4f6"
# subject = "cpython.test_cfuncs.CFunctions.test_short"
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
_dll.tf_h.restype = c_short
_dll.tf_h.argtypes = (c_short,)
assert _dll.tf_h(-32766) == -10922
assert S() == -32766

print("CFunctions::test_short: ok")
