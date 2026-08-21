# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "transactions"
# dimension = "behavior"
# case = "transaction_tests__test_replace_starts_transaction"
# subject = "cpython.test_transactions.TransactionTests.test_replace_starts_transaction"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_transactions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_transactions
_suite = unittest.defaultTestLoader.loadTestsFromName("TransactionTests.test_replace_starts_transaction", test_transactions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TransactionTests.test_replace_starts_transaction did not pass"
print("TransactionTests::test_replace_starts_transaction: ok")
