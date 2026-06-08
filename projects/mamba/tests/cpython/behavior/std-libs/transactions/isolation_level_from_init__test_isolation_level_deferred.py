# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "transactions"
# dimension = "behavior"
# case = "isolation_level_from_init__test_isolation_level_deferred"
# subject = "cpython.test_transactions.IsolationLevelFromInit.test_isolation_level_deferred"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_transactions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_transactions
_suite = unittest.defaultTestLoader.loadTestsFromName("IsolationLevelFromInit.test_isolation_level_deferred", test_transactions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython IsolationLevelFromInit.test_isolation_level_deferred did not pass"
print("IsolationLevelFromInit::test_isolation_level_deferred: ok")
