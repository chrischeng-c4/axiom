# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_commit_cursor_reset"
# subject = "cpython.test_regression.RegressionTests.test_commit_cursor_reset"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_regression.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_regression
_suite = unittest.defaultTestLoader.loadTestsFromName("RegressionTests.test_commit_cursor_reset", test_regression)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RegressionTests.test_commit_cursor_reset did not pass"
print("RegressionTests::test_commit_cursor_reset: ok")
