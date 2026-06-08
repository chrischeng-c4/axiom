# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "sqlite_type_tests__test_string"
# subject = "cpython.test_types.SqliteTypeTests.test_string"
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
self_cur.execute('insert into test(s) values (?)', ('Österreich',))
self_cur.execute('select s from test')
row = self_cur.fetchone()
assert row[0] == 'Österreich'

print("SqliteTypeTests::test_string: ok")
