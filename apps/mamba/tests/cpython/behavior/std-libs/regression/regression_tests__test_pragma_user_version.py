# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_pragma_user_version"
# subject = "cpython.test_regression.RegressionTests.test_pragma_user_version"
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
cur = self_con.cursor()
cur.execute('pragma user_version')

print("RegressionTests::test_pragma_user_version: ok")
