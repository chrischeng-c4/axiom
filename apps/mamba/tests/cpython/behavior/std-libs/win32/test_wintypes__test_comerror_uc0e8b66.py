# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "win32"
# dimension = "behavior"
# case = "test_wintypes__test_comerror_uc0e8b66"
# subject = "cpython.test_win32.TestWintypes.test_COMError"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_win32.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_win32
_suite = unittest.defaultTestLoader.loadTestsFromName("TestWintypes.test_COMError", test_win32)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestWintypes.test_COMError did not pass"
print("TestWintypes::test_COMError: ok")
