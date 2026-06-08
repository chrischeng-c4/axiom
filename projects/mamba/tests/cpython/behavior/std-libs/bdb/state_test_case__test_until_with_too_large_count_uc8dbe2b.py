# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "state_test_case__test_until_with_too_large_count_uc8dbe2b"
# subject = "cpython.test_bdb.StateTestCase.test_until_with_too_large_count"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bdb.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_bdb
_suite = unittest.defaultTestLoader.loadTestsFromName("StateTestCase.test_until_with_too_large_count", test_bdb)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StateTestCase.test_until_with_too_large_count did not pass"
print("StateTestCase::test_until_with_too_large_count: ok")
