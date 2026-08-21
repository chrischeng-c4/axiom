# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "transactions"
# dimension = "behavior"
# case = "transactional_d_d_l__test_immediate_transactional_ddl"
# subject = "cpython.test_transactions.TransactionalDDL.test_immediate_transactional_ddl"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_transactions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_transactions
_suite = unittest.defaultTestLoader.loadTestsFromName("TransactionalDDL.test_immediate_transactional_ddl", test_transactions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TransactionalDDL.test_immediate_transactional_ddl did not pass"
print("TransactionalDDL::test_immediate_transactional_ddl: ok")
