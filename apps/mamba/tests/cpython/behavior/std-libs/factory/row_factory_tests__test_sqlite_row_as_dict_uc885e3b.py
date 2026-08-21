# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "factory"
# dimension = "behavior"
# case = "row_factory_tests__test_sqlite_row_as_dict_uc885e3b"
# subject = "cpython.test_factory.RowFactoryTests.test_sqlite_row_as_dict"
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
d = dict(row)
assert d['a'] == row['a']
assert d['b'] == row['b']

print("RowFactoryTests::test_sqlite_row_as_dict: ok")
