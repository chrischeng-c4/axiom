# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regression"
# dimension = "behavior"
# case = "regression_tests__test_bpo37347"
# subject = "cpython.test_regression.RegressionTests.test_bpo37347"
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

class Printer:

    def log(self, *args):
        return sqlite.SQLITE_OK
for method in [self_con.set_trace_callback, functools.partial(self_con.set_progress_handler, n=1), self_con.set_authorizer]:
    printer_instance = Printer()
    method(printer_instance.log)
    method(printer_instance.log)
    self_con.execute('select 1')
    method(None)

print("RegressionTests::test_bpo37347: ok")
