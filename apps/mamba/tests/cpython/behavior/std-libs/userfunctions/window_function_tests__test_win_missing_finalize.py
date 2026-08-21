# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "userfunctions"
# dimension = "behavior"
# case = "window_function_tests__test_win_missing_finalize"
# subject = "cpython.test_userfunctions.WindowFunctionTests.test_win_missing_finalize"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_userfunctions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_userfunctions
_suite = unittest.defaultTestLoader.loadTestsFromName("WindowFunctionTests.test_win_missing_finalize", test_userfunctions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython WindowFunctionTests.test_win_missing_finalize did not pass"
print("WindowFunctionTests::test_win_missing_finalize: ok")
