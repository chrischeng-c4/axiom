# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "factory"
# dimension = "behavior"
# case = "text_factory_tests__test_unicode_uc2eee66"
# subject = "cpython.test_factory.TextFactoryTests.test_unicode"
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
austria = 'Österreich'
row = self_con.execute('select ?', (austria,)).fetchone()
assert type(row[0]) == str

print("TextFactoryTests::test_unicode: ok")
