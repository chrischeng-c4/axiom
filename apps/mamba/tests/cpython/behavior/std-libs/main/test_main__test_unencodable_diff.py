# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "main"
# dimension = "behavior"
# case = "test_main__test_unencodable_diff"
# subject = "cpython.test_main.TestMain.test_unencodable_diff"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/test_main.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_lib2to3 import test_main
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMain.test_unencodable_diff", test_main)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMain.test_unencodable_diff did not pass"
print("TestMain::test_unencodable_diff: ok")
