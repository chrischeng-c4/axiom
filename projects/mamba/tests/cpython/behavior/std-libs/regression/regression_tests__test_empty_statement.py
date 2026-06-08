# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_empty_statement"
# subject = "cpython.test_regression.RegressionTests.test_empty_statement"
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
'\n        pysqlite used to segfault with SQLite versions 3.5.x. These return NULL\n        for "no-operation" statements\n        '
self_con.execute('')

print("RegressionTests::test_empty_statement: ok")
