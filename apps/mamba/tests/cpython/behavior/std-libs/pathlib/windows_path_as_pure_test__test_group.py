# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "windows_path_as_pure_test__test_group"
# subject = "cpython.test_pathlib.WindowsPathAsPureTest.test_group"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pathlib
_suite = unittest.defaultTestLoader.loadTestsFromName("WindowsPathAsPureTest.test_group", test_pathlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython WindowsPathAsPureTest.test_group did not pass"
print("WindowsPathAsPureTest::test_group: ok")
