# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "transactions"
# dimension = "behavior"
# case = "autocommit_attribute__test_autocommit_enabled"
# subject = "cpython.test_transactions.AutocommitAttribute.test_autocommit_enabled"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_transactions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_transactions
_suite = unittest.defaultTestLoader.loadTestsFromName("AutocommitAttribute.test_autocommit_enabled", test_transactions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AutocommitAttribute.test_autocommit_enabled did not pass"
print("AutocommitAttribute::test_autocommit_enabled: ok")
