# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hooks"
# dimension = "behavior"
# case = "progress_tests__test_error_in_progress_handler_uc0e5fbb"
# subject = "cpython.test_hooks.ProgressTests.test_error_in_progress_handler"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_hooks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_hooks
_suite = unittest.defaultTestLoader.loadTestsFromName("ProgressTests.test_error_in_progress_handler", test_hooks)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ProgressTests.test_error_in_progress_handler did not pass"
print("ProgressTests::test_error_in_progress_handler: ok")
