# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_connection_call"
# subject = "cpython.test_regression.RegressionTests.test_connection_call"
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
'\n        Call a connection with a non-string SQL request: check error handling\n        of the statement constructor.\n        '
try:
    self_con(b'select 1')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("RegressionTests::test_connection_call: ok")
