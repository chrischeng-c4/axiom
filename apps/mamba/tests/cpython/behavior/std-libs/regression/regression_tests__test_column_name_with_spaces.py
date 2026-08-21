# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_column_name_with_spaces"
# subject = "cpython.test_regression.RegressionTests.test_column_name_with_spaces"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_regression.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import datetime
import sqlite3 as sqlite
import weakref
import functools
self_con = sqlite.connect(':memory:')
cur = self_con.cursor()
cur.execute('select 1 as "foo bar [datetime]"')
assert cur.description[0][0] == 'foo bar [datetime]'
cur.execute('select 1 as "foo baz"')
assert cur.description[0][0] == 'foo baz'

print("RegressionTests::test_column_name_with_spaces: ok")
