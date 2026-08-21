# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "refcounts"
# dimension = "behavior"
# case = "another_leak__test_callback_uc9b5c91"
# subject = "cpython.test_refcounts.AnotherLeak.test_callback"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_refcounts.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import ctypes
import gc
import _ctypes_test
import sys
proto = ctypes.CFUNCTYPE(ctypes.c_int, ctypes.c_int, ctypes.c_int)

def func(a, b):
    return a * b * 2
f = proto(func)
a = sys.getrefcount(ctypes.c_int)
f(1, 2)
assert sys.getrefcount(ctypes.c_int) == a

print("AnotherLeak::test_callback: ok")
