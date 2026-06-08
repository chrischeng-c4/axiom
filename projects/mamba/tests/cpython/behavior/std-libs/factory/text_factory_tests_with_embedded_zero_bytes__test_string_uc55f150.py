# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "factory"
# dimension = "behavior"
# case = "text_factory_tests_with_embedded_zero_bytes__test_string_uc55f150"
# subject = "cpython.test_factory.TextFactoryTestsWithEmbeddedZeroBytes.test_string"
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
self_con.execute('create table test (value text)')
self_con.execute('insert into test (value) values (?)', ('a\x00b',))
row = self_con.execute('select value from test').fetchone()
assert type(row[0]) is str
assert row[0] == 'a\x00b'

print("TextFactoryTestsWithEmbeddedZeroBytes::test_string: ok")
