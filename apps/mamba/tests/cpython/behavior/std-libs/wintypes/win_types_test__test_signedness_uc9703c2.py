# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wintypes"
# dimension = "behavior"
# case = "win_types_test__test_signedness_uc9703c2"
# subject = "cpython.test_wintypes.WinTypesTest.test_signedness"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_wintypes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_wintypes
_suite = unittest.defaultTestLoader.loadTestsFromName("WinTypesTest.test_signedness", test_wintypes)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython WinTypesTest.test_signedness did not pass"
print("WinTypesTest::test_signedness: ok")
