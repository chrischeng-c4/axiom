# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_surrogates"
# subject = "cpython.test_regression.RegressionTests.test_surrogates"
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
con = sqlite.connect(':memory:')
try:
    con("select '\ud8ff'")
    raise AssertionError('assertRaises: no raise')
except UnicodeEncodeError:
    pass
try:
    con("select '\udcff'")
    raise AssertionError('assertRaises: no raise')
except UnicodeEncodeError:
    pass
cur = con.cursor()
try:
    cur.execute("select '\ud8ff'")
    raise AssertionError('assertRaises: no raise')
except UnicodeEncodeError:
    pass
try:
    cur.execute("select '\udcff'")
    raise AssertionError('assertRaises: no raise')
except UnicodeEncodeError:
    pass

print("RegressionTests::test_surrogates: ok")
