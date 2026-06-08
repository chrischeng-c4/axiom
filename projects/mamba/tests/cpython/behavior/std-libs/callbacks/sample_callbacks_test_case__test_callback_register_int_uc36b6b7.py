# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "callbacks"
# dimension = "behavior"
# case = "sample_callbacks_test_case__test_callback_register_int_uc36b6b7"
# subject = "cpython.test_callbacks.SampleCallbacksTestCase.test_callback_register_int"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_callbacks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import functools
from ctypes import *
from _ctypes import CTYPES_MAX_ARGCOUNT
import _ctypes_test
dll = CDLL(_ctypes_test.__file__)
CALLBACK = CFUNCTYPE(c_int, c_int, c_int, c_int, c_int, c_int)
func = dll._testfunc_cbk_reg_int
func.argtypes = (c_int, c_int, c_int, c_int, c_int, CALLBACK)
func.restype = c_int

def callback(a, b, c, d, e):
    return a + b + c + d + e
result = func(2, 3, 4, 5, 6, CALLBACK(callback))
assert result == callback(2 * 2, 3 * 3, 4 * 4, 5 * 5, 6 * 6)

print("SampleCallbacksTestCase::test_callback_register_int: ok")
