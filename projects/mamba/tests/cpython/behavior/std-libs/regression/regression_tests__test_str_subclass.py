# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_str_subclass"
# subject = "cpython.test_regression.RegressionTests.test_str_subclass"
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
"\n        The Python 3.0 port of the module didn't cope with values of subclasses of str.\n        "

class MyStr(str):
    pass
self_con.execute('select ?', (MyStr('abc'),))

print("RegressionTests::test_str_subclass: ok")
