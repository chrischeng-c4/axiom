# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_workaround_for_buggy_sqlite_transfer_bindings"
# subject = "cpython.test_regression.RegressionTests.test_workaround_for_buggy_sqlite_transfer_bindings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_regression.py"
# status = "filled"
# ///
import datetime
import sqlite3 as sqlite
import weakref
import functools
self_con = sqlite.connect(':memory:')
'\n        pysqlite would crash with older SQLite versions unless\n        a workaround is implemented.\n        '
self_con.execute('create table foo(bar)')
self_con.execute('drop table foo')
self_con.execute('create table foo(bar)')

print("RegressionTests::test_workaround_for_buggy_sqlite_transfer_bindings: ok")
