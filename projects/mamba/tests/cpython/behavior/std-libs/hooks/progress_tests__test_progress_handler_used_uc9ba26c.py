# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hooks"
# dimension = "behavior"
# case = "progress_tests__test_progress_handler_used_uc9ba26c"
# subject = "cpython.test_hooks.ProgressTests.test_progress_handler_used"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_hooks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import contextlib
import sqlite3 as sqlite
'\n        Test that the progress handler is invoked once it is set.\n        '
con = sqlite.connect(':memory:')
progress_calls = []

def progress():
    progress_calls.append(None)
    return 0
con.set_progress_handler(progress, 1)
con.execute('\n            create table foo(a, b)\n            ')
assert progress_calls

print("ProgressTests::test_progress_handler_used: ok")
