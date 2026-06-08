# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "factory"
# dimension = "behavior"
# case = "row_factory_tests__test_sqlite_row_slice_uca5b54c"
# subject = "cpython.test_factory.RowFactoryTests.test_sqlite_row_slice"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_factory.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sqlite3 as sqlite
from collections.abc import Sequence
self_con = sqlite.connect(':memory:')
self_con.row_factory = sqlite.Row
row = self_con.execute('select 1, 2, 3, 4').fetchone()
assert row[0:0] == ()
assert row[0:1] == (1,)
assert row[1:3] == (2, 3)
assert row[3:1] == ()
assert row[1:] == (2, 3, 4)
assert row[:3] == (1, 2, 3)
assert row[-2:-1] == (3,)
assert row[-2:] == (3, 4)
assert row[0:4:2] == (1, 3)
assert row[3:0:-2] == (4, 2)

print("RowFactoryTests::test_sqlite_row_slice: ok")
