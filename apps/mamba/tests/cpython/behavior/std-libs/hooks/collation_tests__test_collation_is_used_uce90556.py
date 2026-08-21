# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hooks"
# dimension = "behavior"
# case = "collation_tests__test_collation_is_used_uce90556"
# subject = "cpython.test_hooks.CollationTests.test_collation_is_used"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_hooks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_hooks
_suite = unittest.defaultTestLoader.loadTestsFromName("CollationTests.test_collation_is_used", test_hooks)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CollationTests.test_collation_is_used did not pass"
print("CollationTests::test_collation_is_used: ok")
