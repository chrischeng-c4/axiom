# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "factory"
# dimension = "behavior"
# case = "row_factory_tests__test_custom_factory_uc805152"
# subject = "cpython.test_factory.RowFactoryTests.test_custom_factory"
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
self_con.row_factory = lambda cur, row: list(row)
row = self_con.execute('select 1, 2').fetchone()
assert isinstance(row, list)

print("RowFactoryTests::test_custom_factory: ok")
