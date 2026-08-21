# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_statement_reset"
# subject = "cpython.test_regression.RegressionTests.test_statement_reset"
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
con = sqlite.connect(':memory:', cached_statements=5)
cursors = [con.cursor() for x in range(5)]
cursors[0].execute('create table test(x)')
for i in range(10):
    cursors[0].executemany('insert into test(x) values (?)', [(x,) for x in range(10)])
for i in range(5):
    cursors[i].execute(' ' * i + 'select x from test')
con.rollback()

print("RegressionTests::test_statement_reset: ok")
