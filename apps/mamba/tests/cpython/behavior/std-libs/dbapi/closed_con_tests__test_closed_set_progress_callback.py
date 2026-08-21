# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbapi"
# dimension = "behavior"
# case = "closed_con_tests__test_closed_set_progress_callback"
# subject = "cpython.test_dbapi.ClosedConTests.test_closed_set_progress_callback"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_dbapi.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_dbapi
_suite = unittest.defaultTestLoader.loadTestsFromName("ClosedConTests.test_closed_set_progress_callback", test_dbapi)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ClosedConTests.test_closed_set_progress_callback did not pass"
print("ClosedConTests::test_closed_set_progress_callback: ok")
