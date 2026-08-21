# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_windows_path_test__test_parts"
# subject = "cpython.test_pathlib.PureWindowsPathTest.test_parts"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pathlib
_suite = unittest.defaultTestLoader.loadTestsFromName("PureWindowsPathTest.test_parts", test_pathlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PureWindowsPathTest.test_parts did not pass"
print("PureWindowsPathTest::test_parts: ok")
