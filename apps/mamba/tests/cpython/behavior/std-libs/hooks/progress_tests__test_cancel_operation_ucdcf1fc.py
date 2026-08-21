# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hooks"
# dimension = "behavior"
# case = "progress_tests__test_cancel_operation_ucdcf1fc"
# subject = "cpython.test_hooks.ProgressTests.test_cancel_operation"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_hooks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import contextlib
import sqlite3 as sqlite
'\n        Test that returning a non-zero value stops the operation in progress.\n        '
con = sqlite.connect(':memory:')

def progress():
    return 1
con.set_progress_handler(progress, 1)
curs = con.cursor()
try:
    curs.execute('create table bar (a, b)')
    raise AssertionError('assertRaises: no raise')
except sqlite.OperationalError:
    pass

print("ProgressTests::test_cancel_operation: ok")
