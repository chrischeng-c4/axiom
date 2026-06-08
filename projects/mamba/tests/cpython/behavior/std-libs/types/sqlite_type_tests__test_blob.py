# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "sqlite_type_tests__test_blob"
# subject = "cpython.test_types.SqliteTypeTests.test_blob"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import datetime
import sqlite3 as sqlite
import sys
self_con = sqlite.connect(':memory:')
self_cur = self_con.cursor()
self_cur.execute('create table test(i integer, s varchar, f number, b blob)')
sample = b'Guglhupf'
val = memoryview(sample)
self_cur.execute('insert into test(b) values (?)', (val,))
self_cur.execute('select b from test')
row = self_cur.fetchone()
assert row[0] == sample

print("SqliteTypeTests::test_blob: ok")
