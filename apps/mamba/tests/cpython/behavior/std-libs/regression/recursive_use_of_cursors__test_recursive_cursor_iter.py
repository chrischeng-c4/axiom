# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "recursive_use_of_cursors__test_recursive_cursor_iter"
# subject = "cpython.test_regression.RecursiveUseOfCursors.test_recursive_cursor_iter"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_regression.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_regression
_suite = unittest.defaultTestLoader.loadTestsFromName("RecursiveUseOfCursors.test_recursive_cursor_iter", test_regression)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RecursiveUseOfCursors.test_recursive_cursor_iter did not pass"
print("RecursiveUseOfCursors::test_recursive_cursor_iter: ok")
