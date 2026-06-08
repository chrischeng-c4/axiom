# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "factory"
# dimension = "behavior"
# case = "text_factory_tests__test_custom_ucfbfd23"
# subject = "cpython.test_factory.TextFactoryTests.test_custom"
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
self_con.text_factory = lambda x: str(x, 'utf-8', 'ignore')
austria = 'Österreich'
row = self_con.execute('select ?', (austria,)).fetchone()
assert type(row[0]) == str
assert row[0].endswith('reich')

print("TextFactoryTests::test_custom: ok")
