# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_pragma_autocommit"
# subject = "cpython.test_regression.RegressionTests.test_pragma_autocommit"
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
'\n        Verifies that running a PRAGMA statement that does an autocommit does\n        work. This did not work in 2.5.3/2.5.4.\n        '
cur = self_con.cursor()
cur.execute('create table foo(bar)')
cur.execute('insert into foo(bar) values (5)')
cur.execute('pragma page_size')
row = cur.fetchone()

print("RegressionTests::test_pragma_autocommit: ok")
