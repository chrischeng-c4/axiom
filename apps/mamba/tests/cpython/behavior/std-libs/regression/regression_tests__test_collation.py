# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_collation"
# subject = "cpython.test_regression.RegressionTests.test_collation"
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

def collation_cb(a, b):
    return 1
try:
    self_con.create_collation('\udc80', collation_cb)
    raise AssertionError('assertRaises: no raise')
except UnicodeEncodeError:
    pass

print("RegressionTests::test_collation: ok")
