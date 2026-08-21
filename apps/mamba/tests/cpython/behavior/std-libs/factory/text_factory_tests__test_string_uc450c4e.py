# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "factory"
# dimension = "behavior"
# case = "text_factory_tests__test_string_uc450c4e"
# subject = "cpython.test_factory.TextFactoryTests.test_string"
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
self_con.text_factory = bytes
austria = 'Österreich'
row = self_con.execute('select ?', (austria,)).fetchone()
assert type(row[0]) == bytes
assert row[0] == austria.encode('utf-8')

print("TextFactoryTests::test_string: ok")
