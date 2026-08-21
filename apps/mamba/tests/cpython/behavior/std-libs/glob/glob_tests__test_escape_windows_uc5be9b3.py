# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "glob_tests__test_escape_windows_uc5be9b3"
# subject = "cpython.test_glob.GlobTests.test_escape_windows"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_glob
_suite = unittest.defaultTestLoader.loadTestsFromName("GlobTests.test_escape_windows", test_glob)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GlobTests.test_escape_windows did not pass"
print("GlobTests::test_escape_windows: ok")
