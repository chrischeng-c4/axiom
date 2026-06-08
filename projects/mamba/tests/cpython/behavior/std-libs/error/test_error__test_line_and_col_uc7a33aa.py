# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "error"
# dimension = "behavior"
# case = "test_error__test_line_and_col_uc7a33aa"
# subject = "cpython.test_error.TestError.test_line_and_col"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_error.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_tomllib import test_error
_suite = unittest.defaultTestLoader.loadTestsFromName("TestError.test_line_and_col", test_error)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestError.test_line_and_col did not pass"
print("TestError::test_line_and_col: ok")
