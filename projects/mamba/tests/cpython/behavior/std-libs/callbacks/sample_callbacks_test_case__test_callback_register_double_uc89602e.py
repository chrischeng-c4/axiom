# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "callbacks"
# dimension = "behavior"
# case = "sample_callbacks_test_case__test_callback_register_double_uc89602e"
# subject = "cpython.test_callbacks.SampleCallbacksTestCase.test_callback_register_double"
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
CALLBACK = CFUNCTYPE(c_double, c_double, c_double, c_double, c_double, c_double)
func = dll._testfunc_cbk_reg_double
func.argtypes = (c_double, c_double, c_double, c_double, c_double, CALLBACK)
func.restype = c_double

def callback(a, b, c, d, e):
    return a + b + c + d + e
result = func(1.1, 2.2, 3.3, 4.4, 5.5, CALLBACK(callback))
assert result == callback(1.1 * 1.1, 2.2 * 2.2, 3.3 * 3.3, 4.4 * 4.4, 5.5 * 5.5)

print("SampleCallbacksTestCase::test_callback_register_double: ok")
