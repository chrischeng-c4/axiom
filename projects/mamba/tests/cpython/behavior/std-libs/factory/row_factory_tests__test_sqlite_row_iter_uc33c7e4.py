# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "factory"
# dimension = "behavior"
# case = "row_factory_tests__test_sqlite_row_iter_uc33c7e4"
# subject = "cpython.test_factory.RowFactoryTests.test_sqlite_row_iter"
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
row = self_con.execute('select 1 as a, 2 as b').fetchone()
items = [col for col in row]
assert items == [1, 2]
items = [col for col in row]
assert items == [1, 2]

print("RowFactoryTests::test_sqlite_row_iter: ok")
