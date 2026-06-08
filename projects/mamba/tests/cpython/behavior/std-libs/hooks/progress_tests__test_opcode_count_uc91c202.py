# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hooks"
# dimension = "behavior"
# case = "progress_tests__test_opcode_count_uc91c202"
# subject = "cpython.test_hooks.ProgressTests.test_opcode_count"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_hooks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import contextlib
import sqlite3 as sqlite
'\n        Test that the opcode argument is respected.\n        '
con = sqlite.connect(':memory:')
progress_calls = []

def progress():
    progress_calls.append(None)
    return 0
con.set_progress_handler(progress, 1)
curs = con.cursor()
curs.execute('\n            create table foo (a, b)\n            ')
first_count = len(progress_calls)
progress_calls = []
con.set_progress_handler(progress, 2)
curs.execute('\n            create table bar (a, b)\n            ')
second_count = len(progress_calls)
assert first_count >= second_count

print("ProgressTests::test_opcode_count: ok")
