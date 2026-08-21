# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_register_adapter"
# subject = "cpython.test_regression.RegressionTests.test_register_adapter"
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
'\n        See issue 3312.\n        '
try:
    sqlite.register_adapter({}, None)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("RegressionTests::test_register_adapter: ok")
