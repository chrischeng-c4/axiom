# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "transactions"
# dimension = "behavior"
# case = "rollback_tests__test_no_duplicate_rows_after_rollback_new_query"
# subject = "cpython.test_transactions.RollbackTests.test_no_duplicate_rows_after_rollback_new_query"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_transactions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sqlite3 as sqlite
from contextlib import contextmanager

def _check_rows():
    for i, row in enumerate(self_res):
        assert row[0] == i
self_con = sqlite.connect(':memory:')
self_cur1 = self_con.cursor()
self_cur2 = self_con.cursor()
with self_con:
    self_con.execute('create table t(c)')
    self_con.executemany('insert into t values(?)', [(0,), (1,), (2,)])
self_cur1.execute('begin transaction')
select = 'select c from t'
self_cur1.execute(select)
self_con.rollback()
self_res = self_cur2.execute(select)
self_cur1.execute('select c from t where c = 1')
_check_rows()

print("RollbackTests::test_no_duplicate_rows_after_rollback_new_query: ok")
