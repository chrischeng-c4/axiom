# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hooks"
# dimension = "behavior"
# case = "progress_tests__test_clear_handler_uc680621"
# subject = "cpython.test_hooks.ProgressTests.test_clear_handler"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_hooks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_hooks
_suite = unittest.defaultTestLoader.loadTestsFromName("ProgressTests.test_clear_handler", test_hooks)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ProgressTests.test_clear_handler did not pass"
print("ProgressTests::test_clear_handler: ok")
