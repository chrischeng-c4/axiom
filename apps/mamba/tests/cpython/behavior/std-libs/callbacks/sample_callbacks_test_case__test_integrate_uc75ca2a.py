# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "callbacks"
# dimension = "behavior"
# case = "sample_callbacks_test_case__test_integrate_uc75ca2a"
# subject = "cpython.test_callbacks.SampleCallbacksTestCase.test_integrate"
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
CALLBACK = CFUNCTYPE(c_double, c_double)
integrate = dll.integrate
integrate.argtypes = (c_double, c_double, CALLBACK, c_long)
integrate.restype = c_double

def func(x):
    return x ** 2
result = integrate(0.0, 1.0, CALLBACK(func), 10)
diff = abs(result - 1.0 / 3.0)
assert diff < 0.01

print("SampleCallbacksTestCase::test_integrate: ok")
