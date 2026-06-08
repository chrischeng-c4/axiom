# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_auto_commit"
# subject = "cpython.test_regression.RegressionTests.test_auto_commit"
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
'\n        Verifies that creating a connection in autocommit mode works.\n        2.5.3 introduced a regression so that these could no longer\n        be created.\n        '
con = sqlite.connect(':memory:', isolation_level=None)

print("RegressionTests::test_auto_commit: ok")
