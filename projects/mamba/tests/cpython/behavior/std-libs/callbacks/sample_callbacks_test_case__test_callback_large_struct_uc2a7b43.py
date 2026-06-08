# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "callbacks"
# dimension = "behavior"
# case = "sample_callbacks_test_case__test_callback_large_struct_uc2a7b43"
# subject = "cpython.test_callbacks.SampleCallbacksTestCase.test_callback_large_struct"
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

class Check:
    pass

class X(Structure):
    _fields_ = [('first', c_ulong), ('second', c_ulong), ('third', c_ulong)]

def callback(check, s):
    check.first = s.first
    check.second = s.second
    check.third = s.third
    s.first = s.second = s.third = 195948557
check = Check()
s = X()
s.first = 3735928559
s.second = 3405691582
s.third = 195894762
CALLBACK = CFUNCTYPE(None, X)
dll = CDLL(_ctypes_test.__file__)
func = dll._testfunc_cbk_large_struct
func.argtypes = (X, CALLBACK)
func.restype = None
func(s, CALLBACK(functools.partial(callback, check)))
assert check.first == s.first
assert check.second == s.second
assert check.third == s.third
assert check.first == 3735928559
assert check.second == 3405691582
assert check.third == 195894762
assert s.first == check.first
assert s.second == check.second
assert s.third == check.third

print("SampleCallbacksTestCase::test_callback_large_struct: ok")
