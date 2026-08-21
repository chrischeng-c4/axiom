# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_statement_finalization_on_close_db"
# subject = "cpython.test_regression.RegressionTests.test_statement_finalization_on_close_db"
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
con = sqlite.connect(':memory:')
cursors = []
for i in range(105):
    cur = con.cursor()
    cursors.append(cur)
    cur.execute('select 1 x union select ' + str(i))
con.close()

print("RegressionTests::test_statement_finalization_on_close_db: ok")
