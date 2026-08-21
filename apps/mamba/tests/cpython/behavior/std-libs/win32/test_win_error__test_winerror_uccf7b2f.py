# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "win32"
# dimension = "behavior"
# case = "test_win_error__test_winerror_uccf7b2f"
# subject = "cpython.test_win32.TestWinError.test_winerror"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_win32.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_win32
_suite = unittest.defaultTestLoader.loadTestsFromName("TestWinError.test_winerror", test_win32)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestWinError.test_winerror did not pass"
print("TestWinError::test_winerror: ok")
