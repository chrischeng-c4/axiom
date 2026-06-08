# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "transactions"
# dimension = "behavior"
# case = "isolation_level_post_init__test_isolation_level_exclusive"
# subject = "cpython.test_transactions.IsolationLevelPostInit.test_isolation_level_exclusive"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_transactions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sqlite3 as sqlite
from contextlib import contextmanager
QUERY = 'insert into t values(1)'
self_cx = sqlite.connect(':memory:')
self_cx.execute('create table t(t)')
self_traced = []
self_cx.set_trace_callback(lambda stmt: self_traced.append(stmt))
self_cx.isolation_level = 'EXCLUSIVE'
with self_cx:
    self_cx.execute(QUERY)
assert self_traced == ['BEGIN EXCLUSIVE', QUERY, 'COMMIT']

print("IsolationLevelPostInit::test_isolation_level_exclusive: ok")
