# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "transactions"
# dimension = "behavior"
# case = "transactional_d_d_l__test_ddl_does_not_autostart_transaction"
# subject = "cpython.test_transactions.TransactionalDDL.test_ddl_does_not_autostart_transaction"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_transactions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sqlite3 as sqlite
from contextlib import contextmanager
self_con = sqlite.connect(':memory:')
self_con.execute('create table test(i)')
self_con.rollback()
result = self_con.execute('select * from test').fetchall()
assert result == []

print("TransactionalDDL::test_ddl_does_not_autostart_transaction: ok")
