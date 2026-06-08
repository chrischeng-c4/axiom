# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbapi"
# dimension = "behavior"
# case = "sqlite_on_conflict_tests__test_on_conflict_abort_raises_without_transactions"
# subject = "cpython.test_dbapi.SqliteOnConflictTests.test_on_conflict_abort_raises_without_transactions"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_dbapi.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_dbapi
_suite = unittest.defaultTestLoader.loadTestsFromName("SqliteOnConflictTests.test_on_conflict_abort_raises_without_transactions", test_dbapi)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SqliteOnConflictTests.test_on_conflict_abort_raises_without_transactions did not pass"
print("SqliteOnConflictTests::test_on_conflict_abort_raises_without_transactions: ok")
