# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbapi"
# dimension = "behavior"
# case = "multiprocess_tests__test_ctx_mgr_rollback_if_commit_failed"
# subject = "cpython.test_dbapi.MultiprocessTests.test_ctx_mgr_rollback_if_commit_failed"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_dbapi.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_dbapi
_suite = unittest.defaultTestLoader.loadTestsFromName("MultiprocessTests.test_ctx_mgr_rollback_if_commit_failed", test_dbapi)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MultiprocessTests.test_ctx_mgr_rollback_if_commit_failed did not pass"
print("MultiprocessTests::test_ctx_mgr_rollback_if_commit_failed: ok")
