# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "makedir_tests__test_win32_mkdir_700"
# subject = "cpython.test_os.MakedirTests.test_win32_mkdir_700"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_os
_suite = unittest.defaultTestLoader.loadTestsFromName("MakedirTests.test_win32_mkdir_700", test_os)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MakedirTests.test_win32_mkdir_700 did not pass"
print("MakedirTests::test_win32_mkdir_700: ok")
