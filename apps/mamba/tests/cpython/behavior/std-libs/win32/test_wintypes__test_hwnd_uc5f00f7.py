# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "win32"
# dimension = "behavior"
# case = "test_wintypes__test_hwnd_uc5f00f7"
# subject = "cpython.test_win32.TestWintypes.test_HWND"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_win32.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test
from ctypes import wintypes
assert sizeof(wintypes.HWND) == sizeof(c_void_p)

print("TestWintypes::test_HWND: ok")
